//! Сборка одной машины: железо, учётные записи, службы, процессы, файлы по
//! роли, журнал и рабочая почта. Веб-страницы навешивает генератор отдельно.

use super::files_data as files;
use super::machine::{LogKind, Machine, MachineId, MachineKind};
use super::mail;
use super::names::{self, Person};
use super::orgs::Care;
use super::services::{self, Service};
use crate::procs::ProcTable;
use crate::rng::Rng;
use crate::users::UserDb;
use crate::vfs::Vfs;

/// Что нужно знать для сборки машины.
pub struct MachineSpec<'a> {
    pub id: MachineId,
    pub hostname: String,
    pub ip: String,
    pub lan_ip: Option<String>,
    pub domain: Option<String>,
    pub kind: MachineKind,
    pub org: Option<u32>,
    pub router: Option<MachineId>,
    pub org_name: String,
    pub care: Care,
    pub x: f32,
    pub y: f32,
    /// Владельцы учётных записей (кроме root).
    pub people: Vec<&'a Person>,
    /// Адреса соседей по подсети — для журнала входов и зацепок.
    pub peer_ips: Vec<String>,
    pub epoch: u64,
}

const DAY: u64 = 86_400;

/// Железо по виду машины: (os, cpu, ram_mb, disk_mb).
fn hardware(kind: MachineKind, rng: &mut Rng) -> (String, String, u32, u32) {
    let linux = ["Debian 11", "Debian 12", "Ubuntu 20.04", "Ubuntu 22.04", "AlmaLinux 9", "Arch Linux"];
    let server_cpu = ["Xeon E5-2650", "Xeon Silver 4210", "EPYC 7302", "Xeon Gold 6230"];
    let desk_cpu = ["Core i5-8400", "Core i7-9700", "Ryzen 5 3600", "Core i3-10100"];
    match kind {
        MachineKind::Player | MachineKind::HomePc => (
            (*rng.pick(&["Debian 12", "Ubuntu 22.04", "Linux Mint 21"])).to_string(),
            (*rng.pick(&desk_cpu)).to_string(),
            *rng.pick(&[4096u32, 8192, 16384]),
            *rng.pick(&[256_000u32, 512_000, 1_000_000]),
        ),
        MachineKind::Router => ("RouterOS 7".to_string(), "MIPS 74Kc".to_string(), 256, 512),
        MachineKind::Camera => ("BusyBox Linux".to_string(), "ARM Cortex-A7".to_string(), 128, 256),
        MachineKind::Badge => ("Embedded Linux".to_string(), "ARM Cortex-A9".to_string(), 256, 512),
        MachineKind::Atm => ("Windows Embedded 7".to_string(), "Atom D2550".to_string(), 2048, 64_000),
        MachineKind::MedicalDevice => ("Embedded Linux".to_string(), "ARM Cortex-A53".to_string(), 512, 4096),
        MachineKind::Workstation => (
            (*rng.pick(&linux)).to_string(),
            (*rng.pick(&desk_cpu)).to_string(),
            *rng.pick(&[8192u32, 16384]),
            *rng.pick(&[256_000u32, 512_000]),
        ),
        MachineKind::Database => (
            (*rng.pick(&linux)).to_string(),
            (*rng.pick(&server_cpu)).to_string(),
            *rng.pick(&[32768u32, 65536]),
            *rng.pick(&[2_000_000u32, 4_000_000]),
        ),
        _ => (
            (*rng.pick(&linux)).to_string(),
            (*rng.pick(&server_cpu)).to_string(),
            *rng.pick(&[8192u32, 16384, 32768]),
            *rng.pick(&[1_000_000u32, 2_000_000]),
        ),
    }
}

/// Пароль root по аккуратности организации.
fn root_password(care: Care, tag: &str, year: i64, rng: &mut Rng) -> String {
    if rng.chance(care.weak_root_password()) {
        names::weak_password(rng, tag, year)
    } else {
        names::strong_password(rng)
    }
}

/// Пароль обычного пользователя.
fn user_password(care: Care, login: &str, year: i64, rng: &mut Rng) -> String {
    if rng.chance(care.weak_user_password()) {
        names::weak_password(rng, login, year)
    } else {
        names::strong_password(rng)
    }
}

/// Собрать машину по описанию.
pub fn build(spec: MachineSpec, rng: &mut Rng) -> Machine {
    let MachineSpec {
        id, hostname, ip, lan_ip, domain, kind, org, router, org_name, care, x, y, people, peer_ips, epoch,
    } = spec;

    let (os, cpu, ram_mb, disk_mb) = hardware(kind, rng);
    let year = 2029 + rng.range(0, 3);

    // Учётные записи.
    let owner_login = people.first().map(|p| p.login).unwrap_or("root");
    let mut users = UserDb::new_root(&root_password(care, "root", year, rng));
    let mut fs = Vfs::new_unix(owner_login);
    for (i, person) in people.iter().enumerate() {
        let is_admin = i == 0 && matches!(kind, MachineKind::Server | MachineKind::Database | MachineKind::FileServer | MachineKind::Mail | MachineKind::Web | MachineKind::Gateway | MachineKind::Dns | MachineKind::Isp);
        let pw = user_password(care, person.login, year, rng);
        users.add(person.login, &pw, is_admin);
        fs.add_user_home(person.login);
    }

    // Службы и процессы.
    let services = services::services_for(kind, care.age(), rng);
    let mut procs = ProcTable::new();
    procs.set_clock(epoch);
    for s in &services {
        if s.running {
            procs.spawn_daemon(s.proc_name(), "root", s.mem_mb());
        }
    }

    let mut m = Machine {
        id,
        hostname: hostname.clone(),
        ip,
        lan_ip,
        domain: domain.clone(),
        os,
        cpu,
        ram_mb,
        disk_mb,
        fs,
        users,
        procs,
        services,
        logs: Vec::new(),
        mail: Vec::new(),
        kind,
        org,
        router,
        up: true,
        x,
        y,
    };

    populate_files(&mut m, &people, &org_name, care, &peer_ips, epoch, rng);
    populate_logs(&mut m, &people, &peer_ips, epoch, rng);
    populate_mail(&mut m, &people, domain.as_deref(), &org_name, &peer_ips, care, epoch, rng);

    m
}

/// Файлы по роли и домашние файлы людей.
fn populate_files(
    m: &mut Machine,
    people: &[&Person],
    org_name: &str,
    care: Care,
    peer_ips: &[String],
    epoch: u64,
    rng: &mut Rng,
) {
    let has_ssh = m.service_named("ssh").is_some();

    // Домашние файлы сотрудников.
    for (i, person) in people.iter().enumerate() {
        let home = format!("/home/{}", person.login);
        let _ = m.fs.write_text(&format!("{}/notes.txt", home), &files::notes_txt(rng, org_name, person), person.login);
        let _ = m.fs.write_text(&format!("{}/todo.txt", home), &files::todo_txt(rng), person.login);

        // В историю иногда просачивается пароль или адрес соседа.
        let leak_ip = if rng.chance(0.3) && !peer_ips.is_empty() {
            Some(rng.pick(peer_ips).as_str())
        } else {
            None
        };
        let own_pw = m.users.get(person.login).map(|u| u.password.clone());
        let leak_pw = if rng.chance(0.2) { own_pw.as_deref() } else { None };
        let hist = files::bash_history(rng, leak_pw, leak_ip);
        let _ = m.fs.write_text(&format!("{}/.bash_history", home), &hist, person.login);

        // Забытый файл с паролями у неосторожных.
        if rng.chance(care.passwords_file()) {
            let mut entries = vec![("почта".to_string(), m.users.get(person.login).map(|u| u.password.clone()).unwrap_or_default())];
            if !peer_ips.is_empty() {
                entries.push((format!("сервер {}", rng.pick(peer_ips)), names::strong_password(rng)));
            }
            let _ = m.fs.write_text(&format!("{}/passwords.txt", home), &files::passwords_txt(&entries), person.login);
        }

        // Папка почты.
        let _ = m.fs.mkdir_all(&format!("{}/mail", home), person.login);

        // Рабочие документы для первого сотрудника.
        if i == 0 {
            let _ = m.fs.write_text(
                &format!("{}/contract_01.txt", home),
                &files::contract_txt(rng, org_name, "ООО «Контрагент»"),
                person.login,
            );
        }
    }

    // Документы по виду машины.
    match m.kind {
        MachineKind::Database if org_name.contains("Клиник") || org_name.contains("центр") || org_name.contains("Госпиталь") || org_name.contains("Медцентр") => {
            let sample: Vec<&Person> = names::PEOPLE.iter().take(8).collect();
            let _ = m.fs.write_text("/var/data/patients.csv", &files::patients_csv(rng, &sample), "root");
        }
        MachineKind::Database => {
            let _ = m.fs.write_text("/var/data/transactions.log", &files::transactions_log(rng, epoch - 2 * DAY), "root");
            // шифрованный секрет, ключ — в заметках администратора
            let key = names::strong_password(rng);
            let secret = format!("Резервный ключ-пароль: {}\nДоступ к копии: backup@{}\n", names::strong_password(rng), m.hostname);
            let _ = m.fs.write_text("/root/secret.enc", &crate::hack::cipher::encrypt(&secret, &key), "root");
            let _ = m.fs.write_text("/root/notes.txt", &format!("secret.enc — расшифровать ключом: {}\n", key), "root");
        }
        MachineKind::Camera => {
            let _ = m.fs.write_text("/etc/camera_config.ini", &files::camera_config_ini(rng, &m.hostname), "root");
        }
        MachineKind::MedicalDevice => {
            let _ = m.fs.write_text("/etc/device.ini", &format!("[device]\nname={}\nmode=monitor\nmqtt_topic=vitals/{}\n", m.hostname, m.id), "root");
        }
        MachineKind::FileServer => {
            let sample: Vec<&Person> = people.iter().copied().collect();
            if !sample.is_empty() {
                let _ = m.fs.write_text("/srv/share/shift_schedule.txt", &files::shift_schedule(rng, &sample), "root");
            }
        }
        _ => {}
    }

    // Файлы администратора на серверах с ssh.
    if has_ssh && matches!(m.kind, MachineKind::Server | MachineKind::Database | MachineKind::FileServer | MachineKind::Mail | MachineKind::Web | MachineKind::Gateway | MachineKind::Dns | MachineKind::Isp | MachineKind::Honeypot) {
        let permit_root = matches!(care, Care::Sloppy) || rng.chance(0.3);
        let _ = m.fs.write_text("/etc/ssh/sshd_config", &files::sshd_config(permit_root), "root");
    }

    // Приветствие для рабочих машин организации.
    if m.org.is_some() && m.domain.is_some() {
        let _ = m.fs.write_text("/etc/motd", &files::org_motd(org_name), "root");
    }
}

/// Начальные записи журнала: входы сотрудников в рабочее время, перезапуски служб.
fn populate_logs(m: &mut Machine, people: &[&Person], peer_ips: &[String], epoch: u64, rng: &mut Rng) {
    let host = m.hostname.clone();
    let mut auth_lines: Vec<String> = Vec::new();

    // Перезапуски служб вчера.
    for s in m.services.clone() {
        if s.running && rng.chance(0.4) {
            let t = epoch - DAY - rng.range(0, 6 * 3600) as i64 as u64;
            m.log(t, LogKind::Service, format!("служба {} перезапущена", s.name));
        }
    }

    // Входы сотрудников в рабочие часы последних дней.
    let logins: Vec<&str> = if people.is_empty() { vec!["root"] } else { people.iter().map(|p| p.login).collect() };
    let n = rng.range(3, 7);
    for _ in 0..n {
        let day = rng.range(1, 6) as u64;
        let hour = rng.range(9, 18) as u64;
        let t = epoch - day * DAY + hour * 3600;
        let user = *rng.pick(&logins);
        let ip = if !peer_ips.is_empty() && rng.chance(0.6) {
            rng.pick(peer_ips).clone()
        } else {
            "127.0.0.1".to_string()
        };
        let ok = rng.chance(0.85);
        m.log(t, if ok { LogKind::Login } else { LogKind::LoginFailed }, format!("{} {} с {}", if ok { "вход" } else { "неудачный вход" }, user, ip));
        auth_lines.push(files::auth_line(t, &host, ok, user, &ip));
    }

    // Файл /var/log/auth.log из тех же записей — ниточки между машинами.
    if m.service_named("ssh").is_some() {
        auth_lines.sort();
        let _ = m.fs.write_text("/var/log/auth.log", &format!("{}\n", auth_lines.join("\n")), "root");
    }
}

/// Рабочая почта сотрудников.
fn populate_mail(
    m: &mut Machine,
    people: &[&Person],
    domain: Option<&str>,
    org_name: &str,
    peer_ips: &[String],
    _care: Care,
    epoch: u64,
    rng: &mut Rng,
) {
    let domain = match domain {
        Some(d) => d,
        None => return,
    };
    if m.kind.is_appliance() {
        return;
    }
    let people_owned: Vec<&Person> = people.to_vec();
    for (i, person) in people_owned.iter().enumerate() {
        let colleague = people_owned.get((i + 1) % people_owned.len().max(1)).copied();
        let colleague = if colleague.map(|c| c.login) == Some(person.login) { None } else { colleague };
        let leak_ip = if rng.chance(0.3) && !peer_ips.is_empty() { Some(rng.pick(peer_ips).as_str()) } else { None };
        let leak_pw = if rng.chance(0.25) { Some(names::strong_password(rng)) } else { None };
        let letters = mail::employee_mail(rng, person, org_name, domain, colleague, leak_ip, leak_pw.as_deref(), epoch);
        for l in letters {
            m.push_mail(l);
        }
    }
}

/// Приклеить готовый список служб (используется редко; оставлено для расширения).
pub fn attach_services(m: &mut Machine, extra: Vec<Service>) {
    for s in extra {
        if m.service(s.port).is_none() {
            m.services.push(s);
        }
    }
    m.services.sort_by_key(|s| s.port);
}
