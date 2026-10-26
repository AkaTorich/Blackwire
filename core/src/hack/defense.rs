//! Защита цели: файрволы и ловушки-приманки. Файрвол закрывает узел, пока его
//! не обойдёшь; приманка резко поднимает подозрение и подстёгивает слежку.

use crate::net::machine::{LogKind, MachineId, MachineKind};
use crate::net::orgs::OrgKind;
use crate::session::Session;
use crate::world::World;

use super::tracks;

/// Есть ли на узле файрвол (выводится из вида организации — защищённые ставят).
pub fn has_firewall(world: &World, id: MachineId) -> bool {
    match world.org_of(id).map(|o| o.kind) {
        Some(OrgKind::SecurityCenter) | Some(OrgKind::CyberPolice) => true,
        Some(OrgKind::Bank) | Some(OrgKind::Government) => matches!(world.machine(id).kind, MachineKind::Gateway | MachineKind::Database | MachineKind::Server),
        _ => false,
    }
}

/// Закрыт ли узел файрволом прямо сейчас (не обойдён).
pub fn firewall_up(world: &World, id: MachineId) -> bool {
    has_firewall(world, id) && !world.fw_down.contains(&id)
}

/// Попытка обойти файрвол. Шумно (у наблюдаемых растёт подозрение). Успех —
/// узел открыт навсегда (запоминается).
pub fn bypass(world: &mut World, session: &Session, id: MachineId) -> Result<bool, String> {
    if !has_firewall(world, id) {
        return Err("на этом узле нет файрвола".to_string());
    }
    if world.fw_down.contains(&id) {
        return Ok(true);
    }
    let src = tracks::source_ip(world, session);
    world.log(id, LogKind::Warn, &format!("попытка обхода файрвола с {}", src));
    if tracks::is_monitored(world, id) {
        tracks::attack_heat(world, session, id);
    }
    let ok = world.rng.chance(0.6);
    if ok {
        world.fw_down.insert(id);
    }
    Ok(ok)
}

/// Приманка ли это (honeypot).
pub fn is_trap(world: &World, id: MachineId) -> bool {
    matches!(world.machine(id).kind, MachineKind::Honeypot) || world.org_of(id).map(|o| matches!(o.kind, OrgKind::Honeypot)).unwrap_or(false)
}

/// Сработала ловушка: резкий скачок подозрения и подстёгивание расследования.
pub fn spring_trap(world: &mut World, id: MachineId) {
    tracks::add_heat(world, 40.0);
    world.investigation.progress = (world.investigation.progress + 30.0).min(100.0);
    world.player.flags.insert("trapped".to_string(), "1".to_string());
    world.log(id, LogKind::Warn, "сработала ловушка-приманка");
}
