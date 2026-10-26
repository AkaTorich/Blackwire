//! Живой мир: сеть со временем подаёт признаки жизни (входы, службы, новости,
//! перезагрузки), а лента событий ограничена.

use blackwire_core::life;
use blackwire_core::world::World;

fn work_hours_world() -> World {
    let mut w = World::new(7, "ghost", "secret");
    // перевести часы на рабочее время (~09:00), когда активность выше
    w.clock.epoch_secs += 6 * 3600;
    w
}

#[test]
fn world_shows_signs_of_life() {
    let mut w = work_hours_world();
    let logs0: usize = w.machines.iter().map(|m| m.logs.len()).sum();
    let feed0 = w.life.feed.len();
    for _ in 0..4000 {
        life::tick(&mut w, 1);
    }
    let logs1: usize = w.machines.iter().map(|m| m.logs.len()).sum();
    let feed1 = w.life.feed.len();
    assert!(feed1 > feed0 || logs1 > logs0, "за время что-то произошло в сети");
}

#[test]
fn feed_is_capped() {
    let mut w = work_hours_world();
    for _ in 0..6000 {
        life::tick(&mut w, 1);
    }
    assert!(w.life.feed.len() <= 24, "лента не растёт без предела");
}

#[test]
fn quiet_at_start() {
    let w = World::new(7, "ghost", "secret");
    // сразу после создания лента пуста
    assert!(life::feed(&w).is_empty());
}
