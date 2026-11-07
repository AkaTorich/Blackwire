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
    pub name_en: &'static str,
    pub desc_en: &'static str,
}

impl Ach {
    /// Название на текущем языке.
    pub fn name(&self) -> &'static str {
        tr!(self.name, self.name_en)
    }
    /// Описание на текущем языке.
    pub fn desc(&self) -> &'static str {
        tr!(self.desc, self.desc_en)
    }
}

pub const ACHIEVEMENTS: &[Ach] = &[
    Ach { id: "first_blood", name: "Первая кровь", desc: "получить доступ к чужой машине", name_en: "First Blood", desc_en: "gain access to someone else's machine" },
    Ach { id: "recon", name: "Разведчик", desc: "просканировать пять узлов", name_en: "Scout", desc_en: "scan five hosts" },
    Ach { id: "cracker", name: "Взломщик", desc: "добыть чужой логин", name_en: "Cracker", desc_en: "obtain someone else's login" },
    Ach { id: "toolsmith", name: "С инструментом", desc: "купить три скрипта в маркете", name_en: "Toolsmith", desc_en: "buy three scripts on the market" },
    Ach { id: "spy", name: "Шпион", desc: "перехватить логин кейлоггером", name_en: "Spy", desc_en: "capture a login with a keylogger" },
    Ach { id: "botmaster", name: "Хозяин сети", desc: "собрать ботнет из трёх узлов", name_en: "Botmaster", desc_en: "build a botnet of three hosts" },
    Ach { id: "landlord", name: "Со своей базой", desc: "арендовать прокси-релей", name_en: "Landlord", desc_en: "rent a proxy relay" },
    Ach { id: "gigworker", name: "Подёнщик", desc: "выполнить три подработки", name_en: "Gig Worker", desc_en: "complete three gigs" },
    Ach { id: "wanted", name: "В розыске", desc: "довести расследование до пеленга", name_en: "Wanted", desc_en: "let an investigation reach the trace stage" },
    Ach { id: "survivor", name: "Ушёл живым", desc: "пережить рейд", name_en: "Survivor", desc_en: "survive a raid" },
    Ach { id: "defender", name: "Начеку", desc: "вычистить вражескую закладку", name_en: "On Guard", desc_en: "remove a hostile implant" },
    Ach { id: "rich", name: "При деньгах", desc: "накопить 3000 ₵", name_en: "Well Off", desc_en: "save up 3000 ₵" },
    Ach { id: "debtfree", name: "Свободен от долга", desc: "закрыть кредит", name_en: "Debt Free", desc_en: "pay off the loan" },
    Ach { id: "legend", name: "Один из них", desc: "пройти путь до конца", name_en: "One of Them", desc_en: "walk the path to the end" },
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
            world.achievements.pending = Some(AchEvent { name: a.name().to_string() });
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
