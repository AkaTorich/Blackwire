//! Машина сети: адреса, железо, файловая система, пользователи, процессы,
//! службы, журнал и почта. Всё состояние сериализуется в JSON вместе с миром.

use serde::{Deserialize, Serialize};

use super::services::Service;

pub type MachineId = u32;

/// Начало игрового времени: 2031-03-14 03:00 UTC, секунды от 1970 года.
/// Журналы и письма мира датируются относительно этой точки (в прошлое).
pub const WORLD_EPOCH: u64 = 1_931_223_600;

/// Вид машины. По нему выбираются службы, файлы и рисунок на карте.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum MachineKind {
    Player,
    HomePc,
    Router,
    Gateway,
    Server,
    Workstation,
    Database,
    FileServer,
    Mail,
    Web,
    Camera,
    Badge,
    Atm,
    MedicalDevice,
    Dns,
    Isp,
    Honeypot,
}

impl MachineKind {
    /// Человеческое название вида — для карты и `neofetch`.
    pub fn title(self) -> &'static str {
        match self {
            MachineKind::Player => "компьютер игрока",
            MachineKind::HomePc => "домашний компьютер",
            MachineKind::Router => "маршрутизатор",
            MachineKind::Gateway => "шлюз",
            MachineKind::Server => "сервер",
            MachineKind::Workstation => "рабочая станция",
            MachineKind::Database => "база данных",
            MachineKind::FileServer => "файловый сервер",
            MachineKind::Mail => "почтовый сервер",
            MachineKind::Web => "веб-сервер",
            MachineKind::Camera => "камера",
            MachineKind::Badge => "пропускная система",
            MachineKind::Atm => "банкомат",
            MachineKind::MedicalDevice => "медицинский прибор",
            MachineKind::Dns => "сервер имён",
            MachineKind::Isp => "узел провайдера",
            MachineKind::Honeypot => "сервер",
        }
    }

    /// Машина без человека за клавиатурой: прибор, камера, банкомат.
    pub fn is_appliance(self) -> bool {
        matches!(
            self,
            MachineKind::Camera | MachineKind::Badge | MachineKind::Atm | MachineKind::MedicalDevice
        )
    }
}

/// Вид записи журнала машины.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LogKind {
    Login,
    LoginFailed,
    Logout,
    Exec,
    FileChange,
    Service,
    Net,
    Warn,
}

/// Одна строка журнала машины.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct LogEntry {
    /// Игровое время, секунды от 1970 года.
    pub time: u64,
    pub kind: LogKind,
    pub text: String,
}

/// Папка почтового ящика.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MailFolder {
    Inbox,
    Sent,
}

/// Письмо в ящике машины. `to` — адрес вида `user@domain`: по нему
/// приложение «Почта» показывает письма нужного пользователя.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct MailMessage {
    pub id: u32,
    pub from: String,
    pub to: String,
    pub subject: String,
    pub body: String,
    pub time: u64,
    pub read: bool,
    pub folder: MailFolder,
}

/// Найденный логин: машина, пользователь, пароль и откуда он добыт.
/// Хранится у игрока (`Player::found_creds`) — подбор, эксплойты, утечки.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct FoundCred {
    pub machine: MachineId,
    pub user: String,
    pub password: String,
    pub note: String,
}

/// Машина сети.
///
/// Адреса: у шлюза и у выставленных наружу серверов `ip` — публичный, а `lan_ip` —
/// адрес внутри подсети организации. У внутренней машины другого адреса нет,
/// поэтому её `ip` совпадает с `lan_ip`. Подсети организаций не пересекаются,
/// так что все `ip` в мире различны.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Machine {
    pub id: MachineId,
    pub hostname: String,
    pub ip: String,
    pub lan_ip: Option<String>,
    pub domain: Option<String>,
    pub os: String,
    pub cpu: String,
    pub ram_mb: u32,
    pub disk_mb: u32,
    pub fs: crate::vfs::Vfs,
    pub users: crate::users::UserDb,
    pub procs: crate::procs::ProcTable,
    pub services: Vec<Service>,
    pub logs: Vec<LogEntry>,
    pub mail: Vec<MailMessage>,
    pub kind: MachineKind,
    pub org: Option<u32>,
    pub router: Option<MachineId>,
    pub up: bool,
    /// Место на карте, 0..1.
    pub x: f32,
    pub y: f32,
}

impl Machine {
    /// Машина видна из внешней сети: у неё есть собственный публичный адрес.
    pub fn is_public(&self) -> bool {
        match &self.lan_ip {
            Some(lan) => lan != &self.ip,
            None => true,
        }
    }

    /// Служба на порту, если есть.
    pub fn service(&self, port: u16) -> Option<&Service> {
        self.services.iter().find(|s| s.port == port)
    }

    /// Запущенная служба по имени (`ssh`, `http`, …).
    pub fn service_named(&self, name: &str) -> Option<&Service> {
        self.services.iter().find(|s| s.name == name && s.running)
    }

    /// Веб-страница по пути: сперва https, затем http.
    pub fn page(&self, path: &str) -> Option<&super::services::WebPage> {
        let want = normalize_path(path);
        for port in [443u16, 80, 8080, 8443] {
            if let Some(s) = self.service(port) {
                if let Some(p) = s.pages.iter().find(|p| normalize_path(&p.path) == want) {
                    return Some(p);
                }
            }
        }
        None
    }

    /// Добавить запись в журнал.
    pub fn log(&mut self, time: u64, kind: LogKind, text: impl Into<String>) {
        self.logs.push(LogEntry { time, kind, text: text.into() });
    }

    /// Положить письмо в ящик; номер назначается по порядку.
    pub fn push_mail(&mut self, mut msg: MailMessage) -> u32 {
        let id = self.mail.iter().map(|m| m.id).max().unwrap_or(0) + 1;
        msg.id = id;
        self.mail.push(msg);
        id
    }
}

/// Привести путь страницы к одному виду: ведущая косая черта, без хвостовой.
pub fn normalize_path(path: &str) -> String {
    let p = path.split(['?', '#']).next().unwrap_or("");
    let mut s = if p.starts_with('/') { p.to_string() } else { format!("/{}", p) };
    while s.len() > 1 && s.ends_with('/') {
        s.pop();
    }
    s
}

/// Разложить секунды от 1970 года на (год, месяц, день, час, минута, секунда).
pub fn civil(secs: u64) -> (i64, u32, u32, u32, u32, u32) {
    let days = (secs / 86_400) as i64;
    let rem = secs % 86_400;
    // Алгоритм Говарда Хиннанта: дни → григорианская дата.
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let year = if m <= 2 { y + 1 } else { y };
    (year, m, d, (rem / 3600) as u32, ((rem % 3600) / 60) as u32, (rem % 60) as u32)
}

/// Время в виде `2031-03-12 09:14`.
pub fn fmt_time(secs: u64) -> String {
    let (y, m, d, hh, mm, _) = civil(secs);
    format!("{:04}-{:02}-{:02} {:02}:{:02}", y, m, d, hh, mm)
}

/// Время в духе syslog: `Mar 12 09:14:02`.
pub fn fmt_syslog(secs: u64) -> String {
    const MONTHS: [&str; 12] = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    let (_, m, d, hh, mm, ss) = civil(secs);
    format!("{} {:2} {:02}:{:02}:{:02}", MONTHS[(m - 1) as usize], d, hh, mm, ss)
}

/// Дата по-русски: `12 марта 2031`.
pub fn fmt_date_ru(secs: u64) -> String {
    const MONTHS: [&str; 12] = [
        "января", "февраля", "марта", "апреля", "мая", "июня", "июля", "августа", "сентября",
        "октября", "ноября", "декабря",
    ];
    let (y, m, d, _, _, _) = civil(secs);
    format!("{} {} {}", d, MONTHS[(m - 1) as usize], y)
}
