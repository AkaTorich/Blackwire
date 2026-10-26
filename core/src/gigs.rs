//! Подработки — случайные оплачиваемые заказы (не сюжет). Всегда открыто
//! несколько мелких дел: прислать файл скупщику или получить доступ к машине.
//! Выполнил — заплатили, и на доске появляется новое. Это ровный доход после
//! сюжета и просто занятие. Цели берутся из мира, случайность — из зерна.

use serde::{Deserialize, Serialize};

use crate::net::machine::{MachineId, MailFolder, MailMessage};
use crate::world::World;

/// Адрес скупщика, куда шлют добычу по заказу.
const BROKER: &str = "broker@hw";
/// Сколько заказов держать открытыми.
const OPEN: usize = 3;

/// Что требует заказ.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum GigGoal {
    /// Прислать содержимое файла скупщику.
    Deliver { target: MachineId, path: String, needle: String },
    /// Получить доступ к машине.
    Own { target: MachineId },
}

/// Один заказ.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Gig {
    pub id: u32,
    pub title: String,
    pub objective: String,
    pub reward: i64,
    pub goal: GigGoal,
}

/// Состояние подработок (в мире).
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct GigState {
    pub next_id: u32,
    pub gigs: Vec<Gig>,
    pub earned: i64,
    pub done: u32,
    #[serde(skip)]
    pub pending: Option<GigEvent>,
}

#[derive(Clone, Debug)]
pub struct GigEvent {
    pub title: String,
    pub text: String,
}

pub fn take_pending(world: &mut World) -> Option<GigEvent> {
    world.gigs.pending.take()
}

const TITLES: &[&str] = &["Заказ скупщика", "Серый заказ", "Тихое дело", "Мелкая работа", "Заказ без имени"];

fn is_candidate(world: &World, id: MachineId) -> bool {
    let m = world.machine(id);
    id != 0 && m.up && m.is_public() && !crate::hack::remote_vulns(m).is_empty()
}

fn already_target(state: &GigState, id: MachineId) -> bool {
    state.gigs.iter().any(|g| match &g.goal {
        GigGoal::Deliver { target, .. } | GigGoal::Own { target } => *target == id,
    })
}

/// Читаемый файл машины и характерная строка из него.
fn pick_file(world: &World, id: MachineId) -> Option<(String, String)> {
    let fs = &world.machine(id).fs;
    for (root, pat) in [("/home", "*.txt"), ("/root", "*.txt"), ("/var/log", "auth.log"), ("/var", "*.log"), ("/etc", "motd")] {
        for p in fs.find(root, pat) {
            if let Ok(t) = fs.read_text(&p) {
                if let Some(n) = needle_of(&t) {
                    return Some((p, n));
                }
            }
        }
    }
    None
}

fn needle_of(text: &str) -> Option<String> {
    text.lines()
        .map(|l| l.trim())
        .filter(|l| l.len() >= 12 && l.len() <= 120)
        .max_by_key(|l| l.len())
        .or_else(|| text.lines().map(|l| l.trim()).find(|l| !l.is_empty() && l.len() <= 120))
        .map(|s| s.to_string())
}

fn make_gig(world: &mut World) -> Option<Gig> {
    // кандидаты: публичные уязвимые машины, ещё не занятые заказом
    let own_kind = world.gigs.next_id % 2 == 0;
    let cands: Vec<MachineId> = world
        .machines
        .iter()
        .map(|m| m.id)
        .filter(|id| is_candidate(world, *id) && !already_target(&world.gigs, *id))
        .filter(|id| !own_kind || !world.player.owned.contains(id))
        .collect();
    if cands.is_empty() {
        return None;
    }
    let target = cands[world.rng.range(0, cands.len() as i64) as usize];
    let host = world.machine(target).hostname.clone();
    let ip = world.machine(target).ip.clone();
    let id = world.gigs.next_id + 1;
    world.gigs.next_id = id;
    let title = TITLES[world.rng.range(0, TITLES.len() as i64) as usize].to_string();

    if own_kind {
        let reward = 150 + world.rng.range(0, 80);
        Some(Gig {
            id,
            title,
            objective: format!("Получить доступ к {} ({}) — {} ₵", host, ip, reward),
            reward,
            goal: GigGoal::Own { target },
        })
    } else if let Some((path, needle)) = pick_file(world, target) {
        let reward = 100 + world.rng.range(0, 70);
        Some(Gig {
            id,
            title,
            objective: format!("Прислать {} с {} ({}) на {} — {} ₵", path, host, ip, BROKER, reward),
            reward,
            goal: GigGoal::Deliver { target, path, needle },
        })
    } else {
        // файла нет — пусть будет заказ на доступ
        let reward = 150 + world.rng.range(0, 80);
        Some(Gig {
            id,
            title,
            objective: format!("Получить доступ к {} ({}) — {} ₵", host, ip, reward),
            reward,
            goal: GigGoal::Own { target },
        })
    }
}

/// Пополнить доску до нужного числа заказов.
pub fn ensure(world: &mut World) {
    if world.orgs.is_empty() {
        return;
    }
    let mut guard = 0;
    while world.gigs.gigs.len() < OPEN && guard < 20 {
        guard += 1;
        match make_gig(world) {
            Some(g) => world.gigs.gigs.push(g),
            None => break,
        }
    }
}

fn goal_met(world: &World, goal: &GigGoal) -> bool {
    match goal {
        GigGoal::Own { target } => world.player.owned.contains(target),
        GigGoal::Deliver { needle, .. } => world.machines[0]
            .mail
            .iter()
            .any(|m| m.folder == MailFolder::Sent && m.to.contains(BROKER) && m.body.contains(needle.as_str())),
    }
}

/// Проверка выполнения. Зовётся из `World::tick`.
pub fn check(world: &mut World) {
    ensure(world);
    let done: Vec<usize> = world.gigs.gigs.iter().enumerate().filter(|(_, g)| goal_met(world, &g.goal)).map(|(i, _)| i).collect();
    if done.is_empty() {
        return;
    }
    // снимаем выполненные (с конца, чтобы индексы не сползали)
    let mut paid = 0;
    let mut last_title = String::new();
    for i in done.into_iter().rev() {
        let g = world.gigs.gigs.remove(i);
        world.player.money += g.reward;
        world.gigs.earned += g.reward;
        world.gigs.done += 1;
        paid += g.reward;
        last_title = g.title;
    }
    // записка от скупщика
    let to = format!("{}@home.net", world.player.handle);
    let now = world.now();
    world.machines[0].push_mail(MailMessage {
        id: 0,
        from: BROKER.to_string(),
        to,
        subject: "Оплата заказа".to_string(),
        body: format!("Работа принята. Перевод: {} ₵. Новые заказы — на доске (команда gigs).", paid),
        time: now,
        read: false,
        folder: MailFolder::Inbox,
    });
    world.gigs.pending = Some(GigEvent { title: "Скупщик".to_string(), text: format!("Заказ «{}» оплачен (+{} ₵).", last_title, paid) });
    ensure(world);
}

pub fn list(world: &World) -> &[Gig] {
    &world.gigs.gigs
}
