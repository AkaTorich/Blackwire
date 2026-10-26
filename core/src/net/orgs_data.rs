//! Таблицы шаблонов организаций: имена (вымышленные, звучные, разного
//! происхождения), домены, описания и состав машин по видам. Только данные.

use super::machine::MachineKind as K;
use super::orgs::{Care, NameVariant, OrgKind, OrgTemplate, Role};

/// Короткая запись роли: вид, основа имени, сколько (min..=max), наружу ли.
const fn role(kind: K, host: &'static str, min: u8, max: u8, exposed: bool) -> Role {
    Role { kind, host, min, max, exposed }
}

const fn nv(
    name: &'static str,
    tag: &'static str,
    domain: &'static str,
    description: &'static str,
) -> NameVariant {
    NameVariant { name, tag, domain, description }
}

// --- Корпорации --------------------------------------------------------------

const CORP_ENERGY: &[NameVariant] = &[
    nv("Хельветика Энерго", "helvet", "helvetica-energo.net",
       "Энергетический концерн: сети, подстанции, добыча."),
    nv("Нордваль Пауэр", "nordval", "nordval.net",
       "Энергетика северных регионов, гидростанции и сети."),
    nv("Торнгат Энергетикс", "torngat", "torngat-en.net",
       "Тепловые и ветровые станции, продажа мощности."),
];
const CORP_BIOTECH: &[NameVariant] = &[
    nv("Авиценна Биолаб", "avicenna", "avicenna-bio.net",
       "Биотехнологии: ферменты, вакцины, генетика."),
    nv("Мидгард Фарма", "midgard", "midgard-pharma.net",
       "Фармацевтика и лабораторные исследования."),
    nv("Кёльвин Биотех", "kelvin", "kelvin-biotech.net",
       "Клеточные культуры и биореакторы."),
];
const CORP_LOGISTICS: &[NameVariant] = &[
    nv("Сильфида Логистик", "silfida", "silfida-log.net",
       "Логистика и грузоперевозки, складские сети."),
    nv("Караван Транзит", "karavan", "karavan-transit.net",
       "Международные перевозки и таможенные склады."),
    nv("Орлан Карго", "orlan", "orlan-cargo.net",
       "Авиационная и морская логистика."),
];
// Пул имён для ИИ-корпораций вошёл в «энергетику» и «логистику» выше; отдельный
// шаблон ИИ-стартапа убран, чтобы мир оставался в пределах 60–120 машин.

const CORP_ROLES: &[Role] = &[
    role(K::Workstation, "ws-{name}", 1, 1, false),
    role(K::FileServer, "files", 1, 1, false),
    role(K::Database, "db", 1, 1, false),
    role(K::Mail, "mail", 1, 1, true),
    role(K::Web, "web", 1, 1, true),
    role(K::Camera, "cam", 1, 1, false),
    role(K::Badge, "badge", 0, 1, false),
];

// --- Банки -------------------------------------------------------------------

const BANK_VARIANTS: &[NameVariant] = &[
    nv("Банк Аргентум", "argentum", "argentum-bank.net",
       "Коммерческий банк: вклады, кредиты, карты."),
    nv("Деметра Кредит", "demetra", "demetra-credit.net",
       "Кредитная организация и расчётный центр."),
    nv("Остзее Банк", "ostzee", "ostzee-bank.net",
       "Банк с сетью банкоматов и отделений."),
    nv("Сольвей Финанс", "solvey", "solvey-finance.net",
       "Платежи, переводы, обслуживание предприятий."),
];
const BANK_ROLES: &[Role] = &[
    role(K::Workstation, "ws-{name}", 1, 1, false),
    role(K::Database, "txdb", 1, 1, false),
    role(K::Server, "ledger", 1, 1, false),
    role(K::Web, "web", 1, 1, true),
    role(K::Mail, "mail", 1, 1, true),
    role(K::Atm, "atm", 1, 2, false),
    role(K::Badge, "badge", 1, 1, false),
];

// --- Больницы ----------------------------------------------------------------

const HOSP_VARIANTS: &[NameVariant] = &[
    nv("Клиника Гиппократа", "gippokrat", "gippokrat-clinic.net",
       "Многопрофильная клиника и диагностический центр."),
    nv("Медцентр Авалон", "avalon", "avalon-med.net",
       "Больница с отделениями хирургии и терапии."),
    nv("Госпиталь Святой Бригитты", "brigitta", "brigitta-hosp.net",
       "Городская больница и станция скорой помощи."),
];
const HOSP_ROLES: &[Role] = &[
    role(K::Workstation, "ws-{name}", 1, 1, false),
    role(K::Database, "patients", 1, 1, false),
    role(K::FileServer, "files", 1, 1, false),
    role(K::MedicalDevice, "dev", 1, 2, false),
    role(K::Web, "web", 1, 1, true),
    role(K::Mail, "mail", 1, 1, true),
];

// --- Центр безопасности (SOC) ------------------------------------------------

const SOC_VARIANTS: &[NameVariant] = &[
    nv("Центр «Сентинель»", "sentinel", "sentinel-soc.net",
       "Центр мониторинга безопасности: следит за сетями клиентов."),
    nv("Аргус Секьюрити", "argus", "argus-sec.net",
       "Служба наблюдения и реагирования на инциденты."),
];
const SOC_ROLES: &[Role] = &[
    role(K::Server, "monitor", 1, 1, false),
    role(K::Database, "events", 1, 1, false),
    role(K::Workstation, "ws-{name}", 1, 1, false),
    role(K::Web, "portal", 1, 1, true),
];

// --- Киберполиция ------------------------------------------------------------

const POLICE_VARIANTS: &[NameVariant] = &[
    nv("Управление киберполиции", "cyberpol", "cyberpol.gov.net",
       "Государственное управление по киберпреступлениям."),
];
const POLICE_ROLES: &[Role] = &[
    role(K::Server, "case", 1, 1, false),
    role(K::Database, "registry", 1, 1, false),
    role(K::Workstation, "ws-{name}", 1, 1, false),
    role(K::Web, "portal", 1, 1, true),
];

// --- Провайдер ---------------------------------------------------------------

const ISP_VARIANTS: &[NameVariant] = &[
    nv("Провайдер «Меридиан»", "meridian", "meridian.net",
       "Интернет-провайдер: доступ, имена, биллинг."),
];
const ISP_ROLES: &[Role] = &[
    role(K::Dns, "dns", 1, 1, true),
    role(K::Mail, "mail", 1, 1, true),
    role(K::Server, "billing", 1, 1, false),
    role(K::Web, "status", 1, 1, true),
    role(K::Isp, "core", 1, 2, false),
];

// --- Университет -------------------------------------------------------------

const UNI_VARIANTS: &[NameVariant] = &[
    nv("Университет Теслы", "tesla-u", "tesla-univ.net",
       "Технический университет: факультеты, лаборатории, библиотека."),
];
const UNI_ROLES: &[Role] = &[
    role(K::Web, "www", 1, 1, true),
    role(K::Mail, "mail", 1, 1, true),
    role(K::FileServer, "lib", 1, 1, false),
    role(K::Database, "students", 1, 1, false),
    role(K::Workstation, "lab-{name}", 1, 2, false),
    role(K::Server, "grid", 1, 1, false),
];

// --- Хостинг -----------------------------------------------------------------

const HOST_VARIANTS: &[NameVariant] = &[
    nv("Хостинг «Грифон»", "grifon", "grifon-host.net",
       "Хостинг-провайдер: много маленьких сайтов на общих серверах."),
];
const HOST_ROLES: &[Role] = &[
    role(K::Web, "node", 1, 2, true),
    role(K::Server, "panel", 1, 1, true),
    role(K::Database, "sql", 1, 1, false),
    role(K::FileServer, "backup", 1, 1, false),
];

// --- Правительство -----------------------------------------------------------

const GOV_VARIANTS: &[NameVariant] = &[
    nv("Городская управа", "uprava", "uprava.gov.net",
       "Муниципальное учреждение: услуги, реестры, документы."),
];
const GOV_ROLES: &[Role] = &[
    role(K::Web, "portal", 1, 1, true),
    role(K::Database, "registry", 1, 1, false),
    role(K::Mail, "mail", 1, 1, true),
    role(K::Workstation, "ws-{name}", 1, 1, false),
    role(K::FileServer, "archive", 1, 1, false),
];

// --- СМИ ---------------------------------------------------------------------

const MEDIA_VARIANTS: &[NameVariant] = &[
    nv("Вестник «Гермес»", "germes", "germes-news.net",
       "Информационное издание: новости, расследования."),
];
const MEDIA_ROLES: &[Role] = &[
    role(K::Web, "www", 1, 1, true),
    role(K::Mail, "mail", 1, 1, true),
    role(K::FileServer, "cms", 1, 1, false),
    role(K::Workstation, "ws-{name}", 1, 1, false),
];

// --- Магазин -----------------------------------------------------------------

const SHOP_VARIANTS: &[NameVariant] = &[
    nv("Магазин «Агора»", "agora", "agora-shop.net",
       "Интернет-магазин: товары, заказы, оплата."),
];
const SHOP_ROLES: &[Role] = &[
    role(K::Web, "shop", 1, 1, true),
    role(K::Database, "orders", 1, 1, false),
    role(K::Mail, "mail", 1, 1, true),
    role(K::Workstation, "ws-{name}", 1, 1, false),
];

// --- Домашние компьютеры -----------------------------------------------------

const HOME_VARIANTS: &[NameVariant] = &[
    nv("Марат", "marat", "marat.home.net", "Домашний компьютер."),
    nv("Лейла", "leyla", "leyla.home.net", "Домашний компьютер."),
    nv("Эйнар", "einar", "einar.home.net", "Домашний компьютер."),
    nv("Сольвейг", "solveyg", "solveyg.home.net", "Домашний компьютер."),
    nv("Тэмуджин", "temuchin", "temuchin.home.net", "Домашний компьютер."),
    nv("Ядвига", "yadviga", "yadviga.home.net", "Домашний компьютер."),
    nv("Касым", "kasym", "kasym.home.net", "Домашний компьютер."),
    nv("Нанна", "nanna", "nanna.home.net", "Домашний компьютер."),
    nv("Рашид", "rashid", "rashid.home.net", "Домашний компьютер."),
    nv("Бьёрн", "bjorn", "bjorn.home.net", "Домашний компьютер."),
    nv("Зухра", "zuhra", "zuhra.home.net", "Домашний компьютер."),
    nv("Освальд", "osvald", "osvald.home.net", "Домашний компьютер."),
];
const HOME_ROLES: &[Role] = &[role(K::HomePc, "pc", 1, 1, true)];

// --- Приманка (honeypot) -----------------------------------------------------

const HONEY_VARIANTS: &[NameVariant] = &[
    nv("Файловый архив «Либер»", "liber", "liber-archive.net",
       "Открытый файловый сервер — слишком открытый."),
];
const HONEY_ROLES: &[Role] = &[role(K::Honeypot, "srv", 1, 1, true)];

// --- Скрытая сеть hello-world ------------------------------------------------

const HW_VARIANTS: &[NameVariant] = &[
    nv("hello-world", "hw", "hw",
       "Скрытая сеть группы hello-world."),
];
const HW_ROLES: &[Role] = &[
    role(K::Server, "board.hw", 1, 1, false),
    role(K::FileServer, "drop.hw", 1, 1, false),
    role(K::Server, "relay.hw", 1, 1, false),
];

/// Все шаблоны организаций в порядке размещения в мире. Провайдер идёт первым,
/// чтобы его DNS и почта были готовы раньше домашних адресов.
pub const TEMPLATES: &[OrgTemplate] = &[
    OrgTemplate { kind: OrgKind::Isp, variants: ISP_VARIANTS, care: Care::Normal,
        gateway_host: Some("gw"), roles: ISP_ROLES, count: (1, 1) },
    OrgTemplate { kind: OrgKind::Corporation, variants: CORP_ENERGY, care: Care::Normal,
        gateway_host: None, roles: CORP_ROLES, count: (1, 1) },
    OrgTemplate { kind: OrgKind::Corporation, variants: CORP_BIOTECH, care: Care::Strict,
        gateway_host: None, roles: CORP_ROLES, count: (1, 1) },
    OrgTemplate { kind: OrgKind::Corporation, variants: CORP_LOGISTICS, care: Care::Sloppy,
        gateway_host: None, roles: CORP_ROLES, count: (1, 1) },
    OrgTemplate { kind: OrgKind::Bank, variants: BANK_VARIANTS, care: Care::Strict,
        gateway_host: None, roles: BANK_ROLES, count: (2, 2) },
    OrgTemplate { kind: OrgKind::Hospital, variants: HOSP_VARIANTS, care: Care::Sloppy,
        gateway_host: None, roles: HOSP_ROLES, count: (2, 2) },
    OrgTemplate { kind: OrgKind::SecurityCenter, variants: SOC_VARIANTS, care: Care::Strict,
        gateway_host: None, roles: SOC_ROLES, count: (1, 1) },
    OrgTemplate { kind: OrgKind::CyberPolice, variants: POLICE_VARIANTS, care: Care::Strict,
        gateway_host: None, roles: POLICE_ROLES, count: (1, 1) },
    OrgTemplate { kind: OrgKind::University, variants: UNI_VARIANTS, care: Care::Normal,
        gateway_host: None, roles: UNI_ROLES, count: (1, 1) },
    OrgTemplate { kind: OrgKind::Hosting, variants: HOST_VARIANTS, care: Care::Normal,
        gateway_host: None, roles: HOST_ROLES, count: (1, 1) },
    OrgTemplate { kind: OrgKind::Government, variants: GOV_VARIANTS, care: Care::Normal,
        gateway_host: None, roles: GOV_ROLES, count: (1, 1) },
    OrgTemplate { kind: OrgKind::Media, variants: MEDIA_VARIANTS, care: Care::Normal,
        gateway_host: None, roles: MEDIA_ROLES, count: (1, 1) },
    OrgTemplate { kind: OrgKind::Shop, variants: SHOP_VARIANTS, care: Care::Sloppy,
        gateway_host: None, roles: SHOP_ROLES, count: (1, 1) },
    OrgTemplate { kind: OrgKind::Home, variants: HOME_VARIANTS, care: Care::Sloppy,
        gateway_host: None, roles: HOME_ROLES, count: (4, 6) },
    OrgTemplate { kind: OrgKind::Honeypot, variants: HONEY_VARIANTS, care: Care::Sloppy,
        gateway_host: None, roles: HONEY_ROLES, count: (1, 1) },
    OrgTemplate { kind: OrgKind::Underground, variants: HW_VARIANTS, care: Care::Strict,
        gateway_host: Some("gate.hw"), roles: HW_ROLES, count: (1, 1) },
];
