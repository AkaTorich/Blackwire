//! Службы на портах: описание, веб-страницы, таблица уязвимых версий.
//! Данные (порты, версии, баннеры, наборы по видам машин) — в `services_data`.

use serde::{Deserialize, Serialize};

use super::machine::MachineKind;
use super::services_data::{self, ServiceTemplate};
use crate::rng::Rng;

/// Веб-страница http-службы. `path` начинается с `/`, `html` — простой
/// семантичный HTML без стилей: вид задаёт браузер игры.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct WebPage {
    pub path: String,
    pub title: String,
    pub html: String,
}

/// Служба на порту.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Service {
    pub port: u16,
    pub name: String,
    pub version: String,
    pub running: bool,
    pub banner: String,
    pub pages: Vec<WebPage>,
}

impl Service {
    /// Служба отдаёт веб-страницы.
    pub fn is_web(&self) -> bool {
        self.name == "http" || self.name == "https"
    }

    /// Имя процесса-демона для таблицы процессов.
    pub fn proc_name(&self) -> &'static str {
        services_data::template(&self.name).map(|t| t.proc_name).unwrap_or("daemon")
    }

    /// Память демона в мегабайтах.
    pub fn mem_mb(&self) -> u32 {
        services_data::template(&self.name).map(|t| t.mem_mb).unwrap_or(8)
    }
}

/// Известная уязвимость: служба и начало строки версии, которые её выдают.
/// `access` — что даёт («creds», «shell», «read:/путь», «root», «info»),
/// `scope` — удалённая («remote») или локальная для повышения прав («local»).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Vuln {
    pub service: String,
    pub version_prefix: String,
    pub name: String,
    pub effect: String,
    pub access: String,
    pub scope: String,
}

/// Таблица известных уязвимостей.
pub fn known_vulns() -> Vec<Vuln> {
    services_data::VULNS
        .iter()
        .map(|(service, prefix, name, effect, access, scope)| Vuln {
            service: service.to_string(),
            version_prefix: prefix.to_string(),
            name: name.to_string(),
            effect: effect.to_string(),
            access: access.to_string(),
            scope: scope.to_string(),
        })
        .collect()
}

/// Уязвимости, подходящие к службе (по имени и началу версии).
pub fn find_vulns(service: &Service) -> Vec<Vuln> {
    known_vulns()
        .into_iter()
        .filter(|v| v.service == service.name && service.version.starts_with(v.version_prefix.as_str()))
        .collect()
}

/// Насколько старое железо и софт: доля «старых» версий при выборе.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Age {
    Old,
    Mixed,
    Fresh,
}

/// Собрать службу по имени из таблицы, выбрав версию по «возрасту» машины.
pub fn make_service(name: &str, age: Age, rng: &mut Rng) -> Option<Service> {
    let t: &ServiceTemplate = services_data::template(name)?;
    let n = t.versions.len() as i64;
    let half = (n / 2).max(1);
    let idx = match age {
        Age::Old => rng.range(0, half),
        Age::Fresh => rng.range(n - half, n),
        Age::Mixed => rng.range(0, n),
    } as usize;
    let version = t.versions[idx.min(t.versions.len() - 1)].to_string();
    let banner = t.banner.replace("{v}", &version);
    Some(Service {
        port: t.port,
        name: t.name.to_string(),
        version,
        running: true,
        banner,
        pages: Vec::new(),
    })
}

/// Набор служб для машины данного вида. Веб-страницы добавляются позже (`web`).
pub fn services_for(kind: MachineKind, age: Age, rng: &mut Rng) -> Vec<Service> {
    let set = services_data::kind_services(kind);
    let mut out: Vec<Service> = Vec::new();
    let add = |name: &str, rng: &mut Rng, out: &mut Vec<Service>| {
        if out.iter().any(|s| s.name == name) {
            return;
        }
        if let Some(s) = make_service(name, age, rng) {
            out.push(s);
        }
    };
    for name in set.always {
        add(name, rng, &mut out);
    }
    if !set.one_of.is_empty() {
        let name = *rng.pick(set.one_of);
        add(name, rng, &mut out);
    }
    for (name, p) in set.maybe {
        if rng.chance(*p) {
            add(name, rng, &mut out);
        }
    }
    out.sort_by_key(|s| s.port);
    out
}
