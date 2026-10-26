//! Компьютер игрока (`machines[0]`) и его домашний маршрутизатор.

use super::machine::{LogKind, Machine, MachineId, MachineKind};
use super::names;
use super::player_data as pd;
use super::services;
use crate::procs::ProcTable;
use crate::rng::Rng;
use crate::users::UserDb;
use crate::vfs::Vfs;

const DAY: u64 = 86_400;

/// Известный узел для `/etc/hosts`.
pub struct Known {
    pub host: String,
    pub ip: String,
}

/// Собрать компьютер игрока.
pub fn build_player(
    id: MachineId,
    handle: &str,
    password: &str,
    ip: String,
    lan_ip: String,
    router: Option<MachineId>,
    known: &[Known],
    x: f32,
    y: f32,
    epoch: u64,
    rng: &mut Rng,
) -> Machine {
    let hostname = format!("{}-pc", handle);
    let home = format!("/home/{}", handle);

    let mut users = UserDb::new_root(password);
    users.add(handle, password, true);

    let mut fs = Vfs::new_unix(handle);
    fs.add_user_home(handle);

    // Домашние файлы.
    let readme = pd::readme(handle) + "\nСвои эксплоиты: probe адрес → research адрес id → exploit new ~/scripts/my_exploit.adr адрес id.\nПодсказки: research -h 3 адрес id. Готовые эксплоиты: market, затем market buy exp_<механика>.\n";
    let _ = fs.write_text(&format!("{}/readme.txt", home), &readme, handle);
    let _ = fs.mkdir_all(&format!("{}/scripts", home), handle);
    let _ = fs.write_text(&format!("{}/scripts/hello.adr", home), pd::script_hello(), handle);
    let _ = fs.write_text(&format!("{}/scripts/scan.adr", home), pd::script_scan(), handle);
    let _ = fs.write_text(&format!("{}/scripts/bruteforce_demo.adr", home), &pd::script_bruteforce(&home), handle);
    let _ = fs.write_text(&format!("{}/scripts/backup.adr", home), &pd::script_backup(&home), handle);
    let _ = fs.set_mode(&format!("{}/scripts/hello.adr", home), 0o755);
    let _ = fs.set_mode(&format!("{}/scripts/scan.adr", home), 0o755);
    let _ = fs.set_mode(&format!("{}/scripts/bruteforce_demo.adr", home), 0o755);
    let _ = fs.set_mode(&format!("{}/scripts/backup.adr", home), 0o755);
    let _ = fs.mkdir_all(&format!("{}/mail", home), handle);

    // Словарь паролей: ровно те же слова, из которых берутся слабые пароли мира.
    let wordlist = names::wordlist().join("\n");
    let _ = fs.write_text(&format!("{}/wordlist.txt", home), &format!("{}\n", wordlist), handle);

    // /etc/hosts с парой известных узлов.
    let mut hosts = String::from("127.0.0.1\tlocalhost\n");
    hosts.push_str(&format!("{}\t{}\n", ip, hostname));
    for k in known {
        hosts.push_str(&format!("{}\t{}\n", k.ip, k.host));
    }
    let _ = fs.write_text("/etc/hosts", &hosts, "root");
    let _ = fs.write_text("/etc/motd", pd::motd(), "root");

    // Службы и процессы.
    let svc = services::services_for(MachineKind::Player, services::Age::Fresh, rng);
    let mut procs = ProcTable::new();
    procs.set_clock(epoch);
    for s in &svc {
        if s.running {
            procs.spawn_daemon(s.proc_name(), "root", s.mem_mb());
        }
    }

    let mut m = Machine {

        device_type: None, custom_vulns: Vec::new(),
        id,
        hostname,
        ip,
        lan_ip: Some(lan_ip),
        domain: Some("home.net".to_string()),
        os: "Debian 12".to_string(),
        cpu: "Core i5-8400".to_string(),
        ram_mb: 8192,
        disk_mb: 512_000,
        fs,
        users,
        procs,
        services: svc,
        logs: Vec::new(),
        mail: Vec::new(),
        kind: MachineKind::Player,
        org: None,
        router,
        up: true,
        x,
        y,
    };

    m.log(epoch - DAY, LogKind::Login, format!("вход {} с 127.0.0.1", handle));
    m.log(epoch - 2 * 3600, LogKind::Service, "система загружена".to_string());
    m
}

/// Собрать домашний маршрутизатор игрока.
pub fn build_router(
    id: MachineId,
    ip: String,
    lan_ip: String,
    weak: bool,
    x: f32,
    y: f32,
    epoch: u64,
    rng: &mut Rng,
) -> Machine {
    let hostname = "home-router".to_string();
    let password = if weak { (*rng.pick(names::COMMON_PASSWORDS)).to_string() } else { names::strong_password(rng) };
    let mut users = UserDb::new_root(&password);
    users.add("admin", &password, true);

    let mut fs = Vfs::new_unix("admin");
    let _ = fs.write_text("/etc/config/network", "wan=dhcp\nlan=192.168.0.1/24\ndhcp=on\n", "root");

    let svc = services::services_for(MachineKind::Router, services::Age::Old, rng);
    let mut procs = ProcTable::new();
    procs.set_clock(epoch);
    for s in &svc {
        if s.running {
            procs.spawn_daemon(s.proc_name(), "root", s.mem_mb());
        }
    }

    let mut m = Machine {

        device_type: None, custom_vulns: Vec::new(),
        id,
        hostname,
        ip,
        lan_ip: Some(lan_ip),
        domain: None,
        os: "RouterOS 7".to_string(),
        cpu: "MIPS 74Kc".to_string(),
        ram_mb: 256,
        disk_mb: 512,
        fs,
        users,
        procs,
        services: svc,
        logs: Vec::new(),
        mail: Vec::new(),
        kind: MachineKind::Router,
        org: None,
        router: None,
        up: true,
        x,
        y,
    };
    m.log(epoch - 5 * DAY, LogKind::Service, "маршрутизатор перезагружен".to_string());
    m
}
