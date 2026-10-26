//! Шаблоны текстов файлов для машин мира: заметки, списки дел, история команд,
//! забытые пароли, рабочие документы, письма, системные файлы, журналы.
//! Тексты по-русски; вымышленные люди и данные. Только сборка строк.

use super::machine::{fmt_date_ru, fmt_syslog, fmt_time};
use super::names::Person;
use crate::rng::Rng;

/// Личная заметка сотрудника.
pub fn notes_txt(rng: &mut Rng, org_name: &str, person: &Person) -> String {
    let lines = [
        "Не забыть продлить пропуск до конца месяца.",
        "Созвон по проекту перенесли на утро.",
        "Запросить доступ к общему диску у администратора.",
        "Купить кофе в отдел, закончился.",
        "Отчёт сдать до пятницы, не тянуть.",
        "Пароль от почты меняли недавно — записан в другом файле.",
        "Поговорить с начальником про отпуск.",
    ];
    let mut out = format!("Заметки — {} ({})\n\n", person.first, org_name);
    let n = rng.range(2, 4);
    for _ in 0..n {
        out.push_str("- ");
        out.push_str(rng.pick(&lines));
        out.push('\n');
    }
    out
}

/// Список дел.
pub fn todo_txt(rng: &mut Rng) -> String {
    let items = [
        "обновить документацию",
        "ответить на письма",
        "проверить резервную копию",
        "перезапустить службу на сервере",
        "согласовать смету",
        "закрыть старые заявки",
        "подготовить выгрузку для бухгалтерии",
    ];
    let mut out = String::from("TODO\n");
    let n = rng.range(3, 5);
    for _ in 0..n {
        out.push_str("[ ] ");
        out.push_str(rng.pick(&items));
        out.push('\n');
    }
    out
}

/// История командной оболочки. Иногда в ней светится пароль или адрес
/// другой машины — ниточка к соседям по сети.
pub fn bash_history(rng: &mut Rng, leak_password: Option<&str>, leak_ip: Option<&str>) -> String {
    let mut lines: Vec<String> = vec![
        "ls -la".into(),
        "cd /var/log".into(),
        "tail -n 40 auth.log".into(),
        "df -h".into(),
        "ps aux".into(),
        "cat notes.txt".into(),
    ];
    if let Some(ip) = leak_ip {
        lines.push(format!("ssh admin@{}", ip));
        lines.push(format!("ping {}", ip));
    }
    if let Some(pw) = leak_password {
        lines.push(format!("mysql -u root -p{}", pw));
    }
    lines.push("history | tail".into());
    rng.shuffle(&mut lines);
    let mut out = lines.join("\n");
    out.push('\n');
    out
}

/// Файл с паролями у неосторожного пользователя.
pub fn passwords_txt(entries: &[(String, String)]) -> String {
    let mut out = String::from("мои пароли (не показывать никому!)\n\n");
    for (what, pw) in entries {
        out.push_str(&format!("{}: {}\n", what, pw));
    }
    out
}

/// Рабочий договор (вымышленный).
pub fn contract_txt(rng: &mut Rng, org_name: &str, counterparty: &str) -> String {
    let sum = rng.range(50, 900) * 1000;
    format!(
        "ДОГОВОР №{:04}\n\nСтороны: {} и {}.\nПредмет: поставка услуг по проекту.\nСумма: {} у.е.\nСрок: до конца квартала.\n\nПодписи сторон.\n",
        rng.range(100, 9999),
        org_name,
        counterparty,
        sum
    )
}

/// База пациентов больницы (вымышленные записи).
pub fn patients_csv(rng: &mut Rng, people: &[&Person]) -> String {
    let diag = ["наблюдение", "терапия", "реабилитация", "обследование", "профилактика"];
    let mut out = String::from("id,фамилия,имя,возраст,палата,назначение\n");
    for (i, pr) in people.iter().enumerate() {
        out.push_str(&format!(
            "{},{},{},{},{},{}\n",
            1000 + i,
            pr.last,
            pr.first,
            rng.range(19, 86),
            rng.range(1, 40),
            rng.pick(&diag)
        ));
    }
    out
}

/// Журнал транзакций банка (вымышленные суммы).
pub fn transactions_log(rng: &mut Rng, base_time: u64) -> String {
    let mut out = String::from("время;счёт;операция;сумма\n");
    let n = rng.range(6, 12);
    let mut t = base_time;
    for _ in 0..n {
        let op = *rng.pick(&["зачисление", "списание", "перевод", "оплата"]);
        out.push_str(&format!(
            "{};{:010};{};{}\n",
            fmt_time(t),
            rng.range(1, 9_999_999),
            op,
            rng.range(100, 500_000)
        ));
        t += rng.range(600, 7200) as u64;
    }
    out
}

/// График смен.
pub fn shift_schedule(rng: &mut Rng, people: &[&Person]) -> String {
    let mut out = String::from("График смен на неделю\n\n");
    let days = ["Пн", "Вт", "Ср", "Чт", "Пт", "Сб", "Вс"];
    for d in days {
        let who = rng.pick(people);
        out.push_str(&format!("{}: {} {}\n", d, who.first, who.last));
    }
    out
}

/// Настройки камеры.
pub fn camera_config_ini(rng: &mut Rng, label: &str) -> String {
    format!(
        "[camera]\nname={}\nresolution={}\nfps={}\nnight_mode={}\nrtsp_port=554\nrecord=true\n",
        label,
        rng.pick(&["1280x720", "1920x1080", "640x480"]),
        rng.pick(&["15", "25", "30"]),
        rng.pick(&["on", "off"])
    )
}

/// Типовой `/etc/ssh/sshd_config`.
pub fn sshd_config(permit_root: bool) -> String {
    format!(
        "Port 22\nProtocol 2\nPermitRootLogin {}\nPasswordAuthentication yes\nPubkeyAuthentication yes\nX11Forwarding no\nClientAliveInterval 300\n",
        if permit_root { "yes" } else { "no" }
    )
}

/// Одна строка входа для `auth.log`.
pub fn auth_line(time: u64, host: &str, ok: bool, user: &str, ip: &str) -> String {
    if ok {
        format!(
            "{} {} sshd[{}]: Accepted password for {} from {} port {} ssh2",
            fmt_syslog(time),
            host,
            1000 + (time % 50000),
            user,
            ip,
            40000 + (time % 20000)
        )
    } else {
        format!(
            "{} {} sshd[{}]: Failed password for {} from {} port {} ssh2",
            fmt_syslog(time),
            host,
            1000 + (time % 50000),
            user,
            ip,
            40000 + (time % 20000)
        )
    }
}

/// `/etc/motd` рабочей машины организации.
pub fn org_motd(org_name: &str) -> String {
    format!(
        "===============================================\n {}\n Доступ только для сотрудников. Все действия\n записываются в журнал.\n===============================================\n",
        org_name
    )
}

/// Приветствие в письме провайдера о подключении.
pub fn isp_welcome(handle: &str, ip: &str, date: u64) -> String {
    format!(
        "Здравствуйте, {}!\n\nВаше подключение активно с {}.\nТариф: «Домашний», скорость до 100 Мбит/с.\nВаш внешний адрес: {}\nСервер имён и почта — на нашей стороне.\n\nПо вопросам — отвечайте на это письмо.\nПровайдер «Меридиан»\n",
        handle,
        fmt_date_ru(date),
        ip
    )
}
