//! Таблицы служб: порты, версии, баннеры, имена демонов; наборы служб по видам
//! машин; таблица уязвимых версий. Только данные — логика в `services`.

use super::machine::MachineKind;

/// Шаблон службы. `banner` — строка приветствия, `{v}` заменяется версией.
pub struct ServiceTemplate {
    pub name: &'static str,
    pub port: u16,
    /// Версии от старых к новым.
    pub versions: &'static [&'static str],
    pub banner: &'static str,
    pub proc_name: &'static str,
    pub mem_mb: u32,
}

pub const SERVICES: &[ServiceTemplate] = &[
    ServiceTemplate {
        name: "ftp",
        port: 21,
        versions: &["vsftpd 2.3.4", "vsftpd 3.0.2", "ProFTPD 1.3.5e", "vsftpd 3.0.5", "FerryFTP 2.1.0"],
        banner: "220 ({v})",
        proc_name: "vsftpd",
        mem_mb: 6,
    },
    ServiceTemplate {
        name: "ssh",
        port: 22,
        versions: &[
            "OpenSSH_6.6.1p1", "OpenSSH_7.2p2", "OpenSSH_7.4", "OpenSSH_7.9p1", "OpenSSH_8.2p1",
            "OpenSSH_8.9p1", "OpenSSH_9.3p1", "OpenSSH_9.6p1", "OpenSSH_9.8p1",
        ],
        banner: "SSH-2.0-{v}",
        proc_name: "sshd",
        mem_mb: 9,
    },
    ServiceTemplate {
        name: "telnet",
        port: 23,
        versions: &["telnetd 0.17", "BusyBox telnetd 1.22", "BusyBox telnetd 1.31"],
        banner: "\u{ff}\u{fd}\u{18}\u{ff}\u{fd} login: ",
        proc_name: "telnetd",
        mem_mb: 2,
    },
    ServiceTemplate {
        name: "smtp",
        port: 25,
        versions: &["Postfix 2.11.3", "Exim 4.86", "Postfix 3.4.14", "Postfix 3.8.1", "Lumen MTA 1.9"],
        banner: "220 mail ESMTP {v}",
        proc_name: "smtpd",
        mem_mb: 24,
    },
    ServiceTemplate {
        name: "dns",
        port: 53,
        versions: &["BIND 9.9.5", "BIND 9.11.4", "Unbound 1.9.0", "BIND 9.18.12", "Knot DNS 3.3.4"],
        banner: "{v}",
        proc_name: "named",
        mem_mb: 32,
    },
    ServiceTemplate {
        name: "http",
        port: 80,
        versions: &[
            "Apache/2.2.22", "Apache/2.4.18", "nginx/1.10.3", "nginx/1.18.0", "Apache/2.4.57",
            "nginx/1.24.0", "Caddy/2.7.6", "lighttpd/1.4.73",
        ],
        banner: "HTTP/1.1 200 OK\r\nServer: {v}",
        proc_name: "httpd",
        mem_mb: 40,
    },
    ServiceTemplate {
        name: "imap",
        port: 143,
        versions: &["Dovecot 2.2.9", "Dovecot 2.2.27", "Dovecot 2.3.16", "Courier-IMAP 5.1.0", "Dovecot 2.3.21"],
        banner: "* OK [CAPABILITY IMAP4rev1] {v} ready.",
        proc_name: "imapd",
        mem_mb: 18,
    },
    ServiceTemplate {
        name: "https",
        port: 443,
        versions: &["Apache/2.4.18", "nginx/1.14.2", "nginx/1.18.0", "Apache/2.4.57", "nginx/1.24.0", "Caddy/2.7.6"],
        banner: "HTTP/1.1 200 OK\r\nServer: {v}",
        proc_name: "httpd",
        mem_mb: 44,
    },
    ServiceTemplate {
        name: "smb",
        port: 445,
        versions: &["Samba 3.6.25", "Samba 4.3.11", "Samba 4.9.5", "Samba 4.15.13", "Samba 4.19.4", "Windows SMB 3.1.1"],
        banner: "{v}",
        proc_name: "smbd",
        mem_mb: 20,
    },
    ServiceTemplate {
        name: "mqtt",
        port: 1883,
        versions: &["Mosquitto 1.4.10", "Mosquitto 1.6.9", "Mosquitto 2.0.11", "Mosquitto 2.0.18", "EMQX 5.3.0"],
        banner: "{v}",
        proc_name: "mosquitto",
        mem_mb: 5,
    },
    ServiceTemplate {
        name: "hl7",
        port: 2575,
        versions: &["MedLink HL7 Listener 1.2", "MedLink HL7 Listener 2.0", "Vitalis Bridge 3.1"],
        banner: "MSH|^~\\&|{v}",
        proc_name: "hl7d",
        mem_mb: 12,
    },
    ServiceTemplate {
        name: "mysql",
        port: 3306,
        versions: &["MySQL 5.5.62", "MySQL 5.7.33", "MariaDB 10.3.39", "MySQL 8.0.36", "MariaDB 11.2.3"],
        banner: "{v}",
        proc_name: "mysqld",
        mem_mb: 180,
    },
    ServiceTemplate {
        name: "rdp",
        port: 3389,
        versions: &["Microsoft Terminal Services 6.1", "Microsoft Terminal Services 10.0", "xrdp 0.9.19"],
        banner: "{v}",
        proc_name: "rdpd",
        mem_mb: 30,
    },
    ServiceTemplate {
        name: "badge",
        port: 4370,
        versions: &["GateKeeper Access 2.4", "GateKeeper Access 3.0", "Portal-Pass 1.7"],
        banner: "GK/{v}",
        proc_name: "gatekeeper",
        mem_mb: 10,
    },
    ServiceTemplate {
        name: "postgresql",
        port: 5432,
        versions: &["PostgreSQL 9.4.26", "PostgreSQL 10.23", "PostgreSQL 12.17", "PostgreSQL 15.5", "PostgreSQL 16.2"],
        banner: "{v}",
        proc_name: "postgres",
        mem_mb: 160,
    },
    ServiceTemplate {
        name: "vnc",
        port: 5900,
        versions: &["RealVNC 4.1.3", "TightVNC 1.3.10", "TigerVNC 1.9.0", "RealVNC 6.11.0", "TigerVNC 1.13.1"],
        banner: "RFB 003.008 ({v})",
        proc_name: "vncserver",
        mem_mb: 22,
    },
    ServiceTemplate {
        name: "ledger",
        port: 7700,
        versions: &["LedgerLink Core 4.2", "LedgerLink Core 5.0", "Argentum TXN 2.8"],
        banner: "LL {v} READY",
        proc_name: "ledgerd",
        mem_mb: 96,
    },
    ServiceTemplate {
        name: "atm-link",
        port: 7701,
        versions: &["LedgerLink ATM Agent 1.3", "LedgerLink ATM Agent 2.1", "Argentum Cash Node 1.0"],
        banner: "ATM {v}",
        proc_name: "atmagent",
        mem_mb: 14,
    },
    ServiceTemplate {
        name: "monitor",
        port: 8443,
        versions: &["SentinelView 3.2", "SentinelView 4.0", "Argus Watch 2.5"],
        banner: "HTTP/1.1 401 Unauthorized\r\nServer: {v}",
        proc_name: "sentinel",
        mem_mb: 120,
    },
];

/// Шаблон службы по имени.
pub fn template(name: &str) -> Option<&'static ServiceTemplate> {
    SERVICES.iter().find(|t| t.name == name)
}

/// Набор служб для вида машины: обязательные, ровно одна из `one_of`
/// и необязательные с вероятностью.
pub struct KindServices {
    pub always: &'static [&'static str],
    pub one_of: &'static [&'static str],
    pub maybe: &'static [(&'static str, f64)],
}

pub fn kind_services(kind: MachineKind) -> KindServices {
    match kind {
        MachineKind::Player => KindServices { always: &["ssh"], one_of: &[], maybe: &[] },
        MachineKind::HomePc => KindServices {
            always: &["ssh"],
            one_of: &[],
            maybe: &[("smb", 0.45), ("vnc", 0.25), ("telnet", 0.1), ("rdp", 0.2)],
        },
        MachineKind::Router => KindServices {
            always: &["ssh", "http"],
            one_of: &[],
            maybe: &[("telnet", 0.45), ("dns", 0.3)],
        },
        MachineKind::Gateway => KindServices {
            always: &["ssh", "http"],
            one_of: &[],
            maybe: &[("https", 0.5), ("ftp", 0.15), ("telnet", 0.1)],
        },
        MachineKind::Server => KindServices {
            always: &["ssh"],
            one_of: &[],
            maybe: &[("http", 0.5), ("ftp", 0.3), ("smb", 0.3), ("telnet", 0.1)],
        },
        MachineKind::Workstation => KindServices {
            always: &["ssh"],
            one_of: &[],
            maybe: &[("rdp", 0.55), ("smb", 0.5), ("vnc", 0.2)],
        },
        MachineKind::Database => KindServices {
            always: &["ssh"],
            one_of: &["mysql", "postgresql"],
            maybe: &[("smb", 0.15)],
        },
        MachineKind::FileServer => KindServices {
            always: &["ssh", "smb"],
            one_of: &[],
            maybe: &[("ftp", 0.6), ("http", 0.2)],
        },
        MachineKind::Mail => KindServices {
            always: &["ssh", "smtp", "imap"],
            one_of: &[],
            maybe: &[("http", 0.4)],
        },
        MachineKind::Web => KindServices {
            always: &["ssh", "http"],
            one_of: &[],
            maybe: &[("https", 0.7), ("ftp", 0.3)],
        },
        MachineKind::Camera => KindServices {
            always: &["http", "mqtt"],
            one_of: &[],
            maybe: &[("telnet", 0.6), ("ssh", 0.3)],
        },
        MachineKind::Badge => KindServices {
            always: &["ssh", "badge"],
            one_of: &[],
            maybe: &[("mqtt", 0.5), ("http", 0.4)],
        },
        MachineKind::Atm => KindServices {
            always: &["atm-link"],
            one_of: &[],
            maybe: &[("telnet", 0.3), ("rdp", 0.5), ("ssh", 0.3)],
        },
        MachineKind::MedicalDevice => KindServices {
            always: &["mqtt", "hl7"],
            one_of: &[],
            maybe: &[("telnet", 0.5), ("http", 0.4), ("ssh", 0.3)],
        },
        MachineKind::Dns => KindServices { always: &["ssh", "dns"], one_of: &[], maybe: &[] },
        MachineKind::Isp => KindServices {
            always: &["ssh", "http"],
            one_of: &[],
            maybe: &[("https", 0.6), ("ftp", 0.2)],
        },
        MachineKind::Honeypot => KindServices {
            always: &["ssh", "ftp", "telnet", "http", "mysql"],
            one_of: &[],
            maybe: &[("smb", 0.8), ("vnc", 0.5)],
        },
    }
}

/// Уязвимые версии: (служба, начало версии, название, код эффекта, доступ, область).
/// Доступ: creds — логин; shell — прямой вход; read:/путь — утечка файла; root — права;
/// info — только сведения. Область: remote — по сети; local — на машине (повышение прав).
pub const VULNS: &[(&str, &str, &str, &str, &str, &str)] = &[
    ("ssh", "OpenSSH_6.", "Устаревшая версия ssh", "weak_ssh", "creds", "remote"),
    ("ssh", "OpenSSH_7.2", "Устаревшая версия ssh", "weak_ssh", "creds", "remote"),
    ("ftp", "vsftpd 2.", "Бэкдор в ftp-сервере", "weak_ftp", "shell", "remote"),
    ("ftp", "ProFTPD 1.3.5", "Бэкдор в ftp-сервере", "weak_ftp", "shell", "remote"),
    ("telnet", "", "Открытый протокол без шифрования", "plain_protocol", "shell", "remote"),
    ("smtp", "Exim 4.86", "Устаревший почтовый сервер", "weak_mail", "read:/var/mail/root", "remote"),
    ("imap", "Dovecot 2.2.9", "Устаревший почтовый сервер", "weak_mail", "read:/var/mail/root", "remote"),
    ("http", "Apache/2.2", "Чтение файлов через веб-сервер", "weak_web", "read:/etc/passwd", "remote"),
    ("http", "nginx/1.10", "Чтение файлов через веб-сервер", "weak_web", "read:/etc/passwd", "remote"),
    ("mysql", "MySQL 5.5", "Утечка учёток из базы", "weak_db", "creds", "remote"),
    ("postgresql", "PostgreSQL 9.", "Утечка учёток из базы", "weak_db", "creds", "remote"),
    ("smb", "Samba 3.", "Выполнение команд через общие папки", "weak_share", "shell", "remote"),
    ("rdp", "Microsoft Terminal Services 6.", "Устаревший удалённый рабочий стол", "weak_rdp", "shell", "remote"),
    ("vnc", "RealVNC 4.", "Устаревший VNC", "weak_vnc", "shell", "remote"),
    ("mqtt", "Mosquitto 1.4", "Открытый брокер сообщений", "weak_mqtt", "info", "remote"),
    ("dns", "BIND 9.9", "Устаревший сервер имён", "weak_dns", "info", "remote"),
    ("hl7", "MedLink HL7 Listener 1.", "Устаревший медицинский шлюз", "weak_medical", "shell", "remote"),
    ("ledger", "LedgerLink Core 4.", "Утечка проводок из банковского ядра", "weak_ledger", "read:/var/ledger/transactions.log", "remote"),
    ("atm-link", "LedgerLink ATM Agent 1.", "Устаревший агент банкомата", "weak_atm", "shell", "remote"),
    ("badge", "GateKeeper Access 2.", "Устаревшая пропускная система", "weak_badge", "shell", "remote"),
    ("monitor", "SentinelView 3.", "Доступ к панели мониторинга", "weak_monitor", "read:/var/log/alerts.log", "remote"),
    // локальные — для повышения прав (privesc): совпадают с ОС машины
    ("kernel", "Debian 10", "Уязвимое ядро (повышение прав)", "local_root", "root", "local"),
    ("kernel", "Debian 11", "Уязвимое ядро (повышение прав)", "local_root", "root", "local"),
    ("kernel", "Ubuntu 18.04", "Уязвимое ядро (повышение прав)", "local_root", "root", "local"),
    ("kernel", "Ubuntu 20.04", "Уязвимое ядро (повышение прав)", "local_root", "root", "local"),
    ("kernel", "CentOS 7", "Уязвимое ядро (повышение прав)", "local_root", "root", "local"),
    ("kernel", "Windows Server 2012", "Уязвимое ядро (повышение прав)", "local_root", "root", "local"),
    ("kernel", "Windows 7", "Уязвимое ядро (повышение прав)", "local_root", "root", "local"),

    // Additional payload mechanics, available in generated game worlds.
    ("http", "Apache/2.2", "Переполнение размера HTTP-записи", "payload:integer_overflow", "read:/etc/passwd", "remote"),
    ("kernel", "Debian 10", "Знаковый индекс таблицы ядра", "payload:signedness", "root", "local"),
    ("smtp", "Exim 4.86", "Разная длина символов и байтов письма", "payload:length_confusion", "read:/var/mail/root", "remote"),
    ("kernel", "Debian 11", "Ссылка ядра после освобождения", "payload:use_after_free", "root", "local"),
    ("kernel", "Ubuntu 20.04", "Повторное освобождение объекта ядра", "payload:double_free", "root", "local"),
    ("kernel", "CentOS 7", "Путаница типов объектов ядра", "payload:type_confusion", "root", "local"),
    ("ledger", "LedgerLink Core 4.", "Доступ к чужой записи реестра", "payload:object_reference", "read:/var/ledger/transactions.log", "remote"),
    ("badge", "GateKeeper Access 2.", "Массовое присваивание роли пропуска", "payload:mass_assignment", "shell", "remote"),
    ("vnc", "RealVNC 4.", "Путаница получателя игрового токена", "payload:token_confusion", "shell", "remote"),
    ("http", "nginx/1.10", "Разные границы HTTP-запроса", "payload:request_smuggling", "read:/etc/passwd", "remote"),
    ("dns", "BIND 9.9", "Подмена записи общего кеша", "payload:cache_poisoning", "info", "remote"),
    ("imap", "Dovecot 2.2.9", "Внедрение служебного заголовка", "payload:header_injection", "read:/var/mail/root", "remote"),
    ("hl7", "MedLink HL7 Listener 1.", "Восстановление небезопасного объекта", "payload:unsafe_deserialization", "shell", "remote"),
    ("mqtt", "Mosquitto 1.4", "Изменение наследуемого флага брокера", "payload:prototype_pollution", "info", "remote"),
    ("http", "Apache/2.2", "Вычисление шаблона в служебном контексте", "payload:template_injection", "read:/etc/passwd", "remote"),
    ("monitor", "SentinelView 3.", "Внешняя сущность отчёта мониторинга", "payload:xml_entity", "read:/var/log/alerts.log", "remote"),
    ("http", "nginx/1.10", "Перебор состояний фильтра запроса", "payload:regex_backtracking", "info", "remote"),
    ("smb", "Samba 3.", "Перекрытие фрагментов сообщения", "payload:fragment_overlap", "shell", "remote"),
    ("ssh", "OpenSSH_7.2", "Предсказуемый генератор токенов", "payload:weak_rng", "creds", "remote"),
    ("kernel", "Windows Server 2012", "Усечение маски прав ядра", "payload:permission_mask", "root", "local"),
];