//! Своя база: аренда релеев, плата за аренду, возврат за неуплату.

use blackwire_core::base;
use blackwire_core::world::World;

fn world() -> World {
    World::new(7, "ghost", "secret")
}

#[test]
fn rent_creates_owned_relay() {
    let mut w = world();
    w.player.money = 2000;
    let m0 = w.machines.len();
    let r = base::rent(&mut w).expect("аренда");
    assert_eq!(w.player.money, 2000 - base::rent_price(), "₵ списаны");
    assert_eq!(w.machines.len(), m0 + 1, "появилась машина");
    assert!(w.player.owned.contains(&r.machine), "релей принадлежит игроку");
    assert_eq!(w.find_by_ip(&r.ip), Some(r.machine), "у релея публичный адрес");
    // можно войти root'ом с выданным паролем
    assert!(w.machine(r.machine).users.check_password("root", &r.password));
    assert!(w.machine(r.machine).service_named("ssh").is_some(), "на релее есть ssh");
}

#[test]
fn rent_fails_without_money() {
    let mut w = world();
    w.player.money = 100;
    assert!(base::rent(&mut w).is_err());
}

#[test]
fn rent_upkeep_charges_over_time() {
    let mut w = world();
    w.player.money = 2000;
    base::rent(&mut w).unwrap();
    let after_rent = w.player.money;
    base::tick(&mut w, 90_000); // прошло больше суток игрового времени
    assert!(w.player.money < after_rent, "за аренду понемногу списывают");
}

#[test]
fn relay_reclaimed_when_broke() {
    let mut w = world();
    w.player.money = 600;
    base::rent(&mut w).unwrap();
    assert_eq!(w.base.rented.len(), 1);
    w.player.money = 0;
    base::tick(&mut w, 200_000); // накопился долг
    assert!(w.base.rented.is_empty(), "релей забрали за неуплату");
    assert!(w.base.pending.is_some());
}

#[test]
fn release_gives_up_relay() {
    let mut w = world();
    w.player.money = 2000;
    let r = base::rent(&mut w).unwrap();
    base::release(&mut w, &r.ip).expect("отказ");
    assert!(w.base.rented.is_empty());
    assert!(!w.player.owned.contains(&r.machine));
    assert!(w.find_by_ip(&r.ip).is_none(), "машина убрана из сети");
}
