//! Классификация слабых мест машины: по версии её служб (удалённые) и по ОС
//! (локальные — для повышения прав). Опирается на таблицу `net::services::VULNS`.

use serde::{Deserialize, Serialize};

use crate::net::machine::Machine;
use crate::net::services;

/// Что даёт слабое место.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum VulnAccess {
    /// Узнать логин (имя и пароль).
    Creds,
    /// Прямой вход — оболочка на машине.
    Shell,
    /// Права root.
    Root,
    /// Утечка файла по пути.
    Read(String),
    /// Только сведения.
    Info,
}

/// Найденное слабое место.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct VulnInfo {
    pub service: String,
    pub port: u16,
    pub version: String,
    pub name: String,
    pub access: VulnAccess,
    /// true — локальное (нужен уже полученный доступ), для повышения прав.
    pub local: bool,
}

/// Строка доступа из таблицы → `VulnAccess`.
pub fn access_from_str(s: &str) -> VulnAccess {
    if let Some(p) = s.strip_prefix("read:") {
        return VulnAccess::Read(p.to_string());
    }
    match s {
        "creds" => VulnAccess::Creds,
        "shell" => VulnAccess::Shell,
        "root" => VulnAccess::Root,
        "info" => VulnAccess::Info,
        _ => VulnAccess::Info,
    }
}

/// Все слабые места машины: удалённые и локальные.
pub fn scan_vulns(m: &Machine) -> Vec<VulnInfo> {
    let mut out = remote_vulns(m);
    out.extend(local_vulns(m));
    out
}

/// Удалённые слабые места — по запущенным службам и их версиям.
pub fn remote_vulns(m: &Machine) -> Vec<VulnInfo> {
    let mut out = Vec::new();
    for s in &m.services {
        if !s.running {
            continue;
        }
        for v in services::find_vulns(s) {
            if v.scope != "remote" {
                continue;
            }
            out.push(VulnInfo {
                service: s.name.clone(),
                port: s.port,
                version: s.version.clone(),
                name: v.name,
                access: access_from_str(&v.access),
                local: false,
            });
        }
    }
    out
}

/// Локальные слабые места — по ОС (уязвимое ядро). Если по ОС ничего не
/// совпало, но машина заброшена (есть удалённая уязвимость) — всё равно
/// считаем повышение прав возможным.
pub fn local_vulns(m: &Machine) -> Vec<VulnInfo> {
    let mut out = Vec::new();
    for v in services::known_vulns() {
        if v.scope == "local" && v.service == "kernel" && m.os.starts_with(&v.version_prefix) {
            out.push(VulnInfo {
                service: "kernel".to_string(),
                port: 0,
                version: m.os.clone(),
                name: v.name,
                access: access_from_str(&v.access),
                local: true,
            });
        }
    }
    if out.is_empty() && !remote_vulns(m).is_empty() {
        out.push(VulnInfo {
            service: "kernel".to_string(),
            port: 0,
            version: m.os.clone(),
            name: "Уязвимое ядро (повышение прав)".to_string(),
            access: VulnAccess::Root,
            local: true,
        });
    }
    out
}
