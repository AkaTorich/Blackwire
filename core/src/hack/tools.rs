//! Инструменты на захваченных машинах: перехватчик ввода (кейлоггер) и ботнет.
//! Кейлоггер со временем ловит чужой логин; ботнет из захваченных узлов тихо
//! приносит ₵. Оба оставляют след-имплант, который может найти защита.

use serde::{Deserialize, Serialize};

use crate::net::machine::{FoundCred, LogKind, MachineId};
use crate::world::World;

/// Перехватчик ввода на машине.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Keylogger {
    pub machine: MachineId,
    /// Игровое время установки.
    pub planted: u64,
    /// Пойманный логин (когда появится).
    pub captured: Option<FoundCred>,
}

/// Через сколько секунд кейлоггер ловит логин.
const CATCH_AFTER: u64 = 90;
/// Доход ботнета: ₵ за узел в минуту.
const BOT_PER_NODE_MIN: f64 = 1.0;

/// Поставить кейлоггер на захваченную машину.
pub fn plant_keylog(world: &mut World, id: MachineId) -> Result<(), String> {
    if !world.has_machine(id) {
        return Err("нет такой машины".to_string());
    }
    if !world.player.owned.contains(&id) {
        return Err("нужен доступ к машине — сначала взломай её".to_string());
    }
    if world.keyloggers.iter().any(|k| k.machine == id) {
        return Err("перехватчик уже стоит на этой машине".to_string());
    }
    let now = world.now();
    let _ = world.machine_mut(id).fs.write_text("/tmp/.klog", "input capture running\n", "root");
    world.keyloggers.push(Keylogger { machine: id, planted: now, captured: None });
    world.log(id, LogKind::Warn, "установлен перехватчик ввода");
    Ok(())
}

/// Шаг кейлоггеров: ловят логин спустя время.
pub fn tick_keyloggers(world: &mut World, _dt: u64) {
    let now = world.now();
    let ready: Vec<usize> = world
        .keyloggers
        .iter()
        .enumerate()
        .filter(|(_, k)| k.captured.is_none() && now.saturating_sub(k.planted) >= CATCH_AFTER)
        .map(|(i, _)| i)
        .collect();
    for i in ready {
        let mid = world.keyloggers[i].machine;
        if !world.has_machine(mid) || !world.machine(mid).up {
            continue;
        }
        let cred = {
            let users = world.machine(mid).users.list();
            users.iter().find(|u| !u.is_admin).or_else(|| users.first()).map(|u| (u.name.clone(), u.password.clone()))
        };
        if let Some((user, pw)) = cred {
            let host = world.machine(mid).hostname.clone();
            let c = FoundCred { machine: mid, user, password: pw, note: format!("перехват ({})", host) };
            world.add_found_cred(c.clone());
            world.keyloggers[i].captured = Some(c);
        }
    }
}

/// Захватить машину в ботнет.
pub fn botnet_add(world: &mut World, id: MachineId) -> Result<(), String> {
    if !world.has_machine(id) {
        return Err("нет такой машины".to_string());
    }
    if !world.player.owned.contains(&id) {
        return Err("нужен доступ к машине — сначала взломай её".to_string());
    }
    if world.botnet.contains(&id) {
        return Err("машина уже в сети".to_string());
    }
    let _ = world.machine_mut(id).fs.write_text("/tmp/.bot", "node enlisted\n", "root");
    world.botnet.push(id);
    world.log(id, LogKind::Warn, "узел подключён к сети");
    Ok(())
}

/// Убрать машину из ботнета.
pub fn botnet_remove(world: &mut World, id: MachineId) -> bool {
    let before = world.botnet.len();
    world.botnet.retain(|m| *m != id);
    world.botnet.len() != before
}

/// Шаг ботнета: пассивный доход от живых узлов.
pub fn tick_botnet(world: &mut World, dt: u64) {
    if world.botnet.is_empty() {
        return;
    }
    let n = world.botnet.iter().filter(|id| world.has_machine(**id) && world.machine(**id).up).count();
    if n == 0 {
        return;
    }
    world.botnet_acc += n as f64 * (BOT_PER_NODE_MIN / 60.0) * dt as f64;
    let whole = world.botnet_acc.floor();
    if whole >= 1.0 {
        world.botnet_acc -= whole;
        world.player.money += whole as i64;
        world.botnet_mined += whole as i64;
    }
}

/// Живые узлы ботнета.
pub fn botnet_size(world: &World) -> usize {
    world.botnet.iter().filter(|id| world.has_machine(**id) && world.machine(**id).up).count()
}
