//! Живой мир: сеть не стоит на месте. Раз в несколько игровых секунд случается
//! мелкое событие — сотрудник входит на машину, перезапускается служба, узел
//! перезагружается, появляется городская новость. Днём (рабочие часы)
//! активность выше. Благодаря этому журналы меняются со временем, разведка
//! видит свежие следы, а мир ощущается населённым. Всё — модель, случайность
//! из зерна.

use serde::{Deserialize, Serialize};

use crate::net::machine::{LogKind, MachineId, MachineKind};
use crate::world::{GameClock, World};

/// Строка ленты заметных публичных событий (для команды `news`).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct FeedItem {
    pub time: u64,
    pub text: String,
}

/// Состояние живого мира.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct LifeState {
    /// Лента последних городских событий (новые в конце).
    pub feed: Vec<FeedItem>,
}

const NEWS: &[&str] = &[
    "Город вводит единый пропуск: всё в одной карте — и всё под присмотром.",
    "Банк «Сольвей» отчитался о рекордной прибыли на фоне волны изъятий.",
    "Сбой в больничной сети: приём по записи задерживается.",
    "Киберполиция обещает «навести порядок в сети к концу квартала».",
    "Жители окраин жалуются на отключения связи по ночам.",
    "Корпорации подписали договор об обмене данными о клиентах.",
    "Университет открыл кафедру цифровой безопасности — набор закрыт.",
    "Неизвестные оставили на городском сайте одно слово. Его быстро стёрли.",
    "Тарифы провайдера снова выросли; качество, как обычно, обещают позже.",
    "Центр «Сентинель» расширяет сеть наблюдения за трафиком.",
];

/// Кандидаты для события: включённые машины (не игрок).
fn up_machines(world: &World) -> Vec<MachineId> {
    world.machines.iter().filter(|m| m.id != 0 && m.up).map(|m| m.id).collect()
}

fn add_feed(world: &mut World, text: String) {
    let time = world.now();
    world.life.feed.push(FeedItem { time, text });
    let len = world.life.feed.len();
    if len > 24 {
        world.life.feed.drain(0..len - 24);
    }
}

/// Не дать журналу машины расти без предела.
fn cap_logs(world: &mut World, id: MachineId) {
    let logs = &mut world.machine_mut(id).logs;
    if logs.len() > 200 {
        let cut = logs.len() - 200;
        logs.drain(0..cut);
    }
}

/// Шаг живого мира. Зовётся из `World::tick`.
pub fn tick(world: &mut World, dt: u64) {
    let hour = GameClock::civil(world.now()).3;
    let work = (8..20).contains(&hour);
    let chance = if work { 0.06 } else { 0.015 } * dt as f64;
    if !world.rng.chance(chance) {
        return;
    }
    let now = world.now();
    let roll = world.rng.range(0, 100);
    if roll < 55 {
        // вход сотрудника
        let cands = up_machines(world);
        if let Some(&id) = pick(world, &cands) {
            let admin=world.rng.chance(0.15);
            let user = world.machine(id).users.list().iter().find(|u| u.is_admin==admin).map(|u| u.name.clone());
            if let Some(user) = user {
                let from = world.machine(id).lan_ip.clone().unwrap_or_else(|| "127.0.0.1".to_string());
                world.plugins.npc_logins.entry(id).or_default().insert(user.clone(),now+600);
                world.machine_mut(id).log(now, LogKind::Login, format!("вход сотрудника {} с {}", user, from));
                cap_logs(world, id);
            }
        }
    } else if roll < 82 {
        // перезапуск службы
        let cands = up_machines(world);
        if let Some(&id) = pick(world, &cands) {
            let svc = world.machine(id).services.iter().find(|s| s.running).map(|s| s.name.clone());
            if let Some(svc) = svc {
                world.machine_mut(id).log(now, LogKind::Service, format!("служба {} перезапущена", svc));
                cap_logs(world, id);
            }
        }
    } else if roll < 93 {
        // городская новость
        let n = world.rng.range(0, NEWS.len() as i64) as usize;
        add_feed(world, NEWS[n].to_string());
    } else {
        // перезагрузка узла (не игрок, не шлюз)
        let cands: Vec<MachineId> = world
            .machines
            .iter()
            .filter(|m| m.id != 0 && m.up && !matches!(m.kind, MachineKind::Gateway | MachineKind::Router | MachineKind::Dns))
            .map(|m| m.id)
            .collect();
        if let Some(&id) = pick(world, &cands) {
            let host = world.machine(id).hostname.clone();
            let secs = world.rng.range(30, 120) as u64;
            world.shutdown_machine(id, secs);
            add_feed(world, format!("узел {} ушёл на перезагрузку", host));
        }
    }
}

/// Случайный элемент (через свой генератор мира).
fn pick<'a>(world: &mut World, xs: &'a [MachineId]) -> Option<&'a MachineId> {
    if xs.is_empty() {
        return None;
    }
    let i = world.rng.range(0, xs.len() as i64) as usize;
    xs.get(i)
}

/// Лента последних событий (для команды `news`), новые сверху.
pub fn feed(world: &World) -> Vec<FeedItem> {
    let mut v = world.life.feed.clone();
    v.reverse();
    v
}
