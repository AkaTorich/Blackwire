//! Разрешение имён игровой сети: по hostname, домену организации, `www.домен`
//! и `mail.домен`; особое имя `home.net` ведёт на почтовый сервер провайдера.
//! Обратный поиск `reverse` отдаёт hostname по адресу.

use super::machine::{Machine, MachineId, MachineKind};
use super::orgs::{Org, OrgKind};

/// Публичный адрес машины (то, что видно снаружи).
fn public_ip(m: &Machine) -> String {
    m.ip.clone()
}

/// Найти машину организации данного вида, выставленную наружу.
fn exposed_member<'a>(machines: &'a [Machine], org: &Org, kind: MachineKind) -> Option<&'a Machine> {
    org.members
        .iter()
        .filter_map(|id| machines.get(*id as usize))
        .find(|m| m.kind == kind && m.is_public())
}

fn gateway_of<'a>(machines: &'a [Machine], org: &Org) -> Option<&'a Machine> {
    machines.get(org.gateway as usize)
}

/// Разрешить имя в публичный ip.
pub fn resolve(machines: &[Machine], orgs: &[Org], name: &str) -> Option<String> {
    let mut n = name.trim().to_lowercase();
    while n.ends_with('.') {
        n.pop();
    }
    if n.is_empty() {
        return None;
    }

    // Точное совпадение по hostname.
    if let Some(m) = machines.iter().find(|m| m.hostname.to_lowercase() == n) {
        return Some(public_ip(m));
    }

    // Особое имя домашней почты: сервер провайдера.
    if n == "home.net" || n == "mail.home.net" || n == "www.home.net" {
        if let Some(isp) = orgs.iter().find(|o| o.kind == OrgKind::Isp) {
            if let Some(mail) = exposed_member(machines, isp, MachineKind::Mail) {
                return Some(public_ip(mail));
            }
            if let Some(gw) = gateway_of(machines, isp) {
                return Some(public_ip(gw));
            }
        }
    }

    // Домены организаций.
    for org in orgs {
        let d = org.domain.to_lowercase();
        if d.is_empty() {
            continue;
        }
        if n == d || n == format!("www.{}", d) {
            if let Some(web) = exposed_member(machines, org, MachineKind::Web) {
                return Some(public_ip(web));
            }
            if let Some(gw) = gateway_of(machines, org) {
                return Some(public_ip(gw));
            }
        }
        if n == format!("mail.{}", d) {
            if let Some(mail) = exposed_member(machines, org, MachineKind::Mail) {
                return Some(public_ip(mail));
            }
            if let Some(gw) = gateway_of(machines, org) {
                return Some(public_ip(gw));
            }
        }
    }

    None
}

/// Обратный поиск: hostname по адресу (публичному или локальному).
pub fn reverse(machines: &[Machine], ip: &str) -> Option<String> {
    let ip = ip.trim();
    machines
        .iter()
        .find(|m| m.ip == ip || m.lan_ip.as_deref() == Some(ip))
        .map(|m| m.hostname.clone())
}

/// Идентификатор машины по адресу — для удобства генератора и мира.
pub fn machine_by_ip(machines: &[Machine], ip: &str) -> Option<MachineId> {
    machines
        .iter()
        .find(|m| m.ip == ip || m.lan_ip.as_deref() == Some(ip))
        .map(|m| m.id)
}
