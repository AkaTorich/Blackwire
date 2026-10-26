//! Организации: вид, домен, шлюз, подсеть, члены. Шаблоны (имена, домены,
//! состав машин) — в `orgs_data`; здесь типы и доступ к шаблонам.

use serde::{Deserialize, Serialize};

use super::machine::{MachineId, MachineKind};

/// Вид организации.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum OrgKind {
    Corporation,
    Bank,
    Hospital,
    SecurityCenter,
    CyberPolice,
    Isp,
    University,
    Hosting,
    Government,
    Media,
    Shop,
    Home,
    /// Открытый файловый сервер-приманка.
    Honeypot,
    /// Скрытая сеть группы hello-world.
    Underground,
}

impl OrgKind {
    pub fn title(self) -> &'static str {
        match self {
            OrgKind::Corporation => "корпорация",
            OrgKind::Bank => "банк",
            OrgKind::Hospital => "больница",
            OrgKind::SecurityCenter => "центр безопасности",
            OrgKind::CyberPolice => "киберполиция",
            OrgKind::Isp => "провайдер",
            OrgKind::University => "университет",
            OrgKind::Hosting => "хостинг",
            OrgKind::Government => "государственное учреждение",
            OrgKind::Media => "СМИ",
            OrgKind::Shop => "магазин",
            OrgKind::Home => "домашняя сеть",
            OrgKind::Honeypot => "сервер-приманка",
            OrgKind::Underground => "скрытая сеть",
        }
    }
}

/// Организация мира.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Org {
    pub id: u32,
    pub name: String,
    pub kind: OrgKind,
    pub domain: String,
    pub gateway: MachineId,
    /// Начало адресов подсети: `"10.14.0."` или `"192.168.7."`.
    pub subnet: String,
    /// Все машины организации, включая шлюз.
    pub members: Vec<MachineId>,
    pub description: String,
}

/// Аккуратность организации: от неё зависят слабые пароли, старые версии служб
/// и забытые файлы с паролями.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Care {
    Sloppy,
    Normal,
    Strict,
}

impl Care {
    /// Вероятность слабого пароля у обычного пользователя.
    pub fn weak_user_password(self) -> f64 {
        match self {
            Care::Sloppy => 0.7,
            Care::Normal => 0.35,
            Care::Strict => 0.08,
        }
    }
    /// Вероятность слабого пароля root.
    pub fn weak_root_password(self) -> f64 {
        match self {
            Care::Sloppy => 0.5,
            Care::Normal => 0.15,
            Care::Strict => 0.0,
        }
    }
    /// Вероятность файла `passwords.txt` в домашней папке.
    pub fn passwords_file(self) -> f64 {
        match self {
            Care::Sloppy => 0.6,
            Care::Normal => 0.25,
            Care::Strict => 0.05,
        }
    }
    /// Возраст софта для выбора версий служб.
    pub fn age(self) -> super::services::Age {
        match self {
            Care::Sloppy => super::services::Age::Old,
            Care::Normal => super::services::Age::Mixed,
            Care::Strict => super::services::Age::Fresh,
        }
    }
}

/// Вариант имени организации одного шаблона: выбирается по зерну.
pub struct NameVariant {
    pub name: &'static str,
    /// Короткая латинская метка — основа имён машин (`korund-db`).
    pub tag: &'static str,
    pub domain: &'static str,
    pub description: &'static str,
}

/// Роль машины в составе организации.
pub struct Role {
    pub kind: MachineKind,
    /// Основа имени машины: `"db"` → `tag-db`; с точкой — полное имя (`"board.hw"`);
    /// `"{name}"` — по имени сотрудника (`tag-ws-marat`).
    pub host: &'static str,
    pub min: u8,
    pub max: u8,
    /// Машина выставлена наружу: получает публичный адрес.
    pub exposed: bool,
}

/// Шаблон организации.
pub struct OrgTemplate {
    pub kind: OrgKind,
    pub variants: &'static [NameVariant],
    pub care: Care,
    /// Полное имя шлюза, если не `tag-gw`.
    pub gateway_host: Option<&'static str>,
    pub roles: &'static [Role],
    /// Сколько организаций создать из шаблона (разные варианты имени).
    pub count: (u8, u8),
}

/// Все шаблоны организаций в порядке размещения в мире.
pub fn templates() -> &'static [OrgTemplate] {
    super::orgs_data::TEMPLATES
}
