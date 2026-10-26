//! Достижения: открываются по состоянию мира, не повторяются, переживают сохранение.

use blackwire_core::achievements;
use blackwire_core::world::World;

fn world() -> World {
    World::new(7, "ghost", "secret")
}

#[test]
fn fresh_world_has_none() {
    let w = world();
    let (have, total) = achievements::count(&w);
    assert_eq!(have, 0, "в начале достижений нет");
    assert!(total >= 10, "достижений в каталоге достаточно");
}

#[test]
fn first_blood_unlocks_on_owning_a_machine() {
    let mut w = world();
    w.player.owned.push(5);
    achievements::check(&mut w);
    assert!(w.achievements.unlocked.contains("first_blood"));
    let ev = achievements::take_pending(&mut w).expect("пришло уведомление");
    assert_eq!(ev.name, "Первая кровь");
}

#[test]
fn does_not_fire_twice() {
    let mut w = world();
    w.player.owned.push(5);
    achievements::check(&mut w);
    let _ = achievements::take_pending(&mut w);
    achievements::check(&mut w); // второй проход
    assert!(achievements::take_pending(&mut w).is_none(), "повторно не уведомляет");
    let (have, _) = achievements::count(&w);
    assert_eq!(have, 1, "открыто ровно одно");
}

#[test]
fn money_and_debt_unlock() {
    let mut w = world();
    w.player.money = 3500;
    w.finance.cleared = true;
    achievements::check(&mut w);
    assert!(w.achievements.unlocked.contains("rich"));
    assert!(w.achievements.unlocked.contains("debtfree"));
}

#[test]
fn survives_save_roundtrip() {
    let mut w = world();
    w.player.owned.push(5);
    achievements::check(&mut w);
    let json = w.to_json();
    let w2 = World::from_json(&json).expect("загрузка");
    assert!(w2.achievements.unlocked.contains("first_blood"), "открытое сохраняется");
}

#[test]
fn ticks_through_world() {
    let mut w = world();
    w.player.scanned = vec![1, 2, 3, 4, 5];
    w.tick(1000); // один игровой такт прогоняет achievements::check
    assert!(w.achievements.unlocked.contains("recon"), "разведчик открылся в такте мира");
}
