//! Достижения: за ключевые дела игрок получает отметку. Проверяются по
//! состоянию мира раз в такт; открытое остаётся открытым.

use serde::{Deserialize, Serialize};

use crate::world::World;

/// Достижение.
#[derive(Clone, Copy, Debug)]
pub struct Ach {
    pub id: &'static str,
    pub name: &'static str,
    pub desc: &'static str,
}

pub const ACHIEVEMENTS: &[Ach] = &[
    Ach { id: "first_blood", name: "Первая кровь", desc: "получить доступ к чужой машине" },
    Ach { id: "recon", name: "Разведчик", desc: "просканировать пять узлов" },
    Ach { id: "cracker", name: "Взломщик", desc: "добыть чужой логин" },
    Ach { id: "toolsmith", name: "С инструментом", desc: "купить три скрипта в маркете" },
    Ach { id: "spy", name: "Шпион", desc: "перехватить логин кейлоггером" },
    Ach { id: "botmaster", name: "Хозяин сети", desc: "собрать ботнет из трёх узлов" },
    Ach { id: "landlord", name: "Со своей базой", desc: "арендовать прокси-релей" },
    Ach { id: "gigworker", name: "Подёнщик", desc: "выполнить три подработки" },
    Ach { id: "wanted", name: "В розыске", desc: "довести расследование до пеленга" },
    Ach { id: "survivor", name: "Ушёл живым", desc: "пережить рейд" },
    Ach { id: "defender", name: "Начеку", desc: "вычистить вражескую закладку" },
    Ach { id: "rich", name: "При деньгах", desc: "накопить 3000 ₵" },
    Ach { id: "debtfree", name: "Свободен от долга", desc: "закрыть кредит" },
    Ach { id: "legend", name: "Один из них", desc: "пройти путь до конца" },
];

/// Состояние достижений (в мире).
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct AchState {
    pub unlocked: std::collections::BTreeSet<String>,
    #[serde(skip)]
    pub pending: Option<AchEvent>,
}

#[derive(Clone, Debug)]
pub struct AchEvent {
    pub name: String,
}

pub fn take_pending(world: &mut World) -> Option<AchEvent> {
    world.achievements.pending.take()
}

fn met(world: &World, id: &str) -> bool {
    use crate::hack::watch::InvStage;
    match id {
        "first_blood" => !world.player.owned.is_empty(),
        "recon" => world.player.scanned.len() >= 5,
        "cracker" => !world.player.harvested.is_empty(),
        "toolsmith" => world.market.bought.len() >= 3,
        "spy" => world.keyloggers.iter().any(|k| k.captured.is_some()),
        "botmaster" => crate::hack::tools::botnet_size(world) >= 3,
        "landlord" => !world.base.rented.is_empty(),
        "gigworker" => world.gigs.done >= 3,
        "wanted" => world.player.traced || matches!(world.investigation.stage, InvStage::Tracing),
        "survivor" => world.investigation.raids >= 1,
        "defender" => world.player.flags.contains_key("cleaned"),
        "rich" => world.player.money >= 3000,
        "debtfree" => world.finance.cleared,
        "legend" => world.ending.won,
        _ => false,
    }
}

/// Проверить достижения. Зовётся из `World::tick`.
pub fn check(world: &mut World) {
    for a in ACHIEVEMENTS {
        if !world.achievements.unlocked.contains(a.id) && met(world, a.id) {
            world.achievements.unlocked.insert(a.id.to_string());
            world.achievements.pending = Some(AchEvent { name: a.name.to_string() });
        }
    }
}

/// Список достижений с отметкой «открыто».
pub fn rows(world: &World) -> Vec<(&'static Ach, bool)> {
    ACHIEVEMENTS.iter().map(|a| (a, world.achievements.unlocked.contains(a.id))).collect()
}

pub fn count(world: &World) -> (usize, usize) {
    (world.achievements.unlocked.len(), ACHIEVEMENTS.len()+crate::plugins::catalog::entries(world,"achievements").len())
}
