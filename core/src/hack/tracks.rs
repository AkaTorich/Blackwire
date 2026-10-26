//! Следы и подозрение. Действие над машиной оставляет в её журнале запись с
//! адресом, с которого оно пришло (текущий hop). Через прокси-цепочку цель
//! видит адрес прокси, а не игрока. Наблюдаемые цели (центры безопасности,
//! киберполиция, машины со службой мониторинга, приманки) поднимают подозрение
//! (`heat`). Чистка журналов убирает улики; `heat` медленно спадает со временем.

use serde::{Deserialize, Serialize};

use crate::net::machine::{MachineId, MachineKind};
use crate::net::orgs::OrgKind;
use crate::session::Session;
use crate::world::World;

/// Итог чистки журналов на машине.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ClearReport {
    pub machine: MachineId,
    pub host: String,
    pub removed: usize,
    pub denied: bool,
}

/// Адрес, который цель видит у источника (машина текущего hop'а).
pub fn source_ip(world: &World, session: &Session) -> String {
    world.machine(session.machine_id()).ip.clone()
}

/// Под наблюдением ли цель: центр безопасности, киберполиция, служба
/// мониторинга или приманка.
pub fn is_monitored(world: &World, target: MachineId) -> bool {
    let m = world.machine(target);
    if m.service_named("monitor").is_some() {
        return true;
    }
    if matches!(m.kind, MachineKind::Honeypot) {
        return true;
    }
    if let Some(org) = world.org_of(target) {
        if matches!(org.kind, OrgKind::SecurityCenter | OrgKind::CyberPolice) {
            return true;
        }
    }
    false
}

/// Поднять подозрение за действие над наблюдаемой целью. Возвращает прирост.
pub fn attack_heat(world: &mut World, session: &Session, target: MachineId) -> f32 {
    // через прокси (цепочка длиннее своей машины) реальный адрес скрыт — тише
    let proxied = session.hops.len() > 1;
    let base = if proxied { 3.0 } else { 14.0 };
    let mult = match world.org_of(target) {
        Some(o) if matches!(o.kind, OrgKind::CyberPolice) => 1.6,
        Some(o) if matches!(o.kind, OrgKind::SecurityCenter) => 1.3,
        _ => 1.0,
    };
    // «Туннель» из магазина — следы вдвое тише
    let vpn = if world.player.upgrades.contains("vpn") { 0.5 } else { 1.0 };
    let amount = base * mult * vpn;
    add_heat(world, amount);
    amount
}

/// Прибавить к подозрению (0..100). На 100 — пеленг: `traced`, сброс к 60.
pub fn add_heat(world: &mut World, amount: f32) {
    let p = &mut world.player;
    p.heat = (p.heat + amount).clamp(0.0, 100.0);
    if p.heat > p.heat_peak {
        p.heat_peak = p.heat;
    }
    if p.heat >= 100.0 && !p.traced {
        p.traced = true;
        p.heat = 60.0;
    }
}

pub fn heat(world: &World) -> f32 {
    world.player.heat
}

/// Подозрение медленно спадает со временем (зовётся из `World::tick`).
pub fn decay(world: &mut World, dt_secs: u64) {
    if world.player.heat > 0.0 {
        // «Чистильщик» из магазина ускоряет спад
        let rate = if world.player.upgrades.contains("scrubber") { 0.15 } else { 0.05 };
        world.player.heat = (world.player.heat - rate * dt_secs as f32).max(0.0);
    }
}

/// Машины и число записей, где засветился реальный адрес игрока.
pub fn evidence(world: &World) -> Vec<(MachineId, usize)> {
    let real = world.machines[0].ip.clone();
    let mut out = Vec::new();
    for m in &world.machines {
        if m.id == 0 {
            continue;
        }
        let n = m.logs.iter().filter(|e| e.text.contains(&real)).count();
        if n > 0 {
            out.push((m.id, n));
        }
    }
    out
}

/// Убрать следы на машине текущего hop'а. Нужны права (root или владелец).
/// Без шаблона убирает след игрока — его реальный адрес и адреса hop'ов.
pub fn clear_logs(world: &mut World, session: &Session, pattern: Option<&str>) -> Result<ClearReport, String> {
    let mid = session.machine_id();
    let user = session.user().to_string();
    let host = world.machine(mid).hostname.clone();
    if !world.machine(mid).users.is_admin(&user) {
        return Ok(ClearReport { machine: mid, host, removed: 0, denied: true });
    }
    // что убираем — считаем до изменяемого заимствования машины
    let needles: Vec<String> = match pattern {
        Some(p) => vec![p.to_string()],
        None => {
            let mut v = vec![world.machines[0].ip.clone()];
            for h in &session.hops {
                v.push(world.machine(h.machine).ip.clone());
            }
            v
        }
    };
    let m = world.machine_mut(mid);
    let before = m.logs.len();
    m.logs.retain(|e| !needles.iter().any(|n| !n.is_empty() && e.text.contains(n)));
    let mut removed = before - m.logs.len();
    // почистить файлы журналов
    for path in ["/var/log/auth.log", "/var/log/system.log", "/var/log/syslog"] {
        if let Ok(text) = m.fs.read_text(path) {
            let kept: Vec<&str> = text.lines().filter(|l| !needles.iter().any(|n| !n.is_empty() && l.contains(n))).collect();
            let cut = text.lines().count() - kept.len();
            if cut > 0 {
                let _ = m.fs.write_text(path, &format!("{}\n", kept.join("\n")), "root");
                removed += cut;
            }
        }
    }
    Ok(ClearReport { machine: mid, host, removed, denied: false })
}
