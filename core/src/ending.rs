//! Концовка. Победа наступает, когда сошлись две вещи: пройдена вся цепочка
//! заданий hello-world и закрыт кредит. Тогда группа присылает последнее письмо,
//! а оболочка показывает финал со сводкой пути. После победы игра не кончается —
//! можно остаться в системе.

use serde::{Deserialize, Serialize};

use crate::net::machine::{MailFolder, MailMessage};
use crate::world::{GameClock, World, START_EPOCH};

/// Состояние концовки (в мире).
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct EndingState {
    pub won: bool,
    /// Игровое время победы.
    pub at: u64,
    #[serde(skip)]
    pub pending: Option<EndingEvent>,
}

#[derive(Clone, Debug)]
pub struct EndingEvent {
    pub title: String,
    pub text: String,
}

/// Сводка пути — для окна финала и команды `ending`.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EndingSummary {
    pub won: bool,
    pub missions_done: usize,
    pub missions_total: usize,
    pub debt_cleared: bool,
    pub money: i64,
    pub money_earned: i64,
    pub debt_paid: i64,
    pub machines_owned: usize,
    pub scanned: usize,
    pub upgrades: usize,
    pub peak_heat: f32,
    pub raids: u32,
    pub time_played: u64,
    pub at_text: String,
    /// Заключительные слова группы (пусто, пока не победа).
    pub text: String,
}

const FINALE: &str = "Ты закрыл их бумажку с печатью — тот самый долг, из-за которого у тебя вынесли полдома. \
И прошёл все двери, которые мы тебе показали, вплоть до тех, кто следит за всеми.\n\n\
Мы были тенью. Нас считали выдумкой. Теперь нас двое тысяч, и ты — один из них. \
Город не стал добрее, но у его изгоев стало одним другом больше.\n\n\
Оставайся в системе сколько хочешь — она твоя. А когда снова понадобится рука в тени, ты знаешь, где нас найти.\n\n\
hello-world";

const FINALE_EN: &str = "You closed their paper with a stamp — the very debt they gutted half your home over. \
And you went through every door we showed you, all the way to those who watch everyone.\n\n\
We were a shadow. People thought we were a fiction. Now there are two thousand of us, and you are one of them. \
The city hasn't grown kinder, but its outcasts have one more friend.\n\n\
Stay in the system as long as you like — it's yours. And when a hand in the shadows is needed again, you know where to find us.\n\n\
hello-world";

/// Заключительное письмо на текущем языке.
fn finale() -> &'static str {
    tr!(FINALE, FINALE_EN)
}

pub fn take_pending(world: &mut World) -> Option<EndingEvent> {
    world.ending.pending.take()
}

/// Пройдена ли вся цепочка заданий.
fn chain_complete(world: &World) -> bool {
    !world.missions.missions.is_empty()
        && world.missions.missions.iter().all(|m| matches!(m.status, crate::missions::MissionStatus::Done))
}

/// Проверить условие победы. Зовётся из `World::tick`.
pub fn check(world: &mut World) {
    if world.ending.won {
        return;
    }
    if chain_complete(world) && world.finance.cleared {
        world.ending.won = true;
        world.ending.at = world.now();
        world.player.flags.insert("won".to_string(), "1".to_string());
        let to = format!("{}@home.net", world.player.handle);
        let now = world.now();
        world.machines[0].push_mail(MailMessage {
            id: 0,
            from: "null@hw".to_string(),
            to,
            subject: tr!("Добро пожаловать домой", "Welcome home").to_string(),
            body: finale().to_string(),
            time: now,
            read: false,
            folder: MailFolder::Inbox,
        });
        world.ending.pending = Some(EndingEvent {
            title: "hello-world".to_string(),
            text: tr!("Долг закрыт, все задания пройдены. Ты — один из нас.", "Debt cleared, every mission done. You are one of us.").to_string(),
        });
    }
}

/// Собрать сводку пути.
pub fn summary(world: &World) -> EndingSummary {
    let won = world.ending.won;
    EndingSummary {
        won,
        missions_done: world.missions.missions.iter().filter(|m| matches!(m.status, crate::missions::MissionStatus::Done)).count(),
        missions_total: world.missions.missions.len(),
        debt_cleared: world.finance.cleared,
        money: world.player.money,
        money_earned: world.missions.money_earned,
        debt_paid: world.finance.paid_total,
        machines_owned: world.player.owned.len(),
        scanned: world.player.scanned.len(),
        upgrades: world.player.upgrades.len(),
        peak_heat: world.player.heat_peak,
        raids: world.investigation.raids,
        time_played: world.now().saturating_sub(START_EPOCH),
        at_text: if won { GameClock::format_time(world.ending.at) } else { String::new() },
        text: if won { finale().to_string() } else { String::new() },
    }
}
