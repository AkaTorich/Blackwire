//! Концовка: условие победы (долг закрыт + цепочка пройдена), финал и то, что
//! игра продолжается после победы. Плюс устойчивость экономики (логин — разовый
//! доход).

use blackwire_core::ending;
use blackwire_core::missions::MissionStatus;
use blackwire_core::net::machine::FoundCred;
use blackwire_core::world::World;

fn world() -> World {
    World::new(7, "ghost", "secret")
}

fn win_conditions(w: &mut World) {
    for m in &mut w.missions.missions {
        m.status = MissionStatus::Done;
    }
    w.missions.active = w.missions.missions.len();
    w.finance.cleared = true;
}

#[test]
fn not_won_at_start() {
    let w = world();
    assert!(!w.ending.won);
    let s = ending::summary(&w);
    assert!(!s.won);
    assert!(!s.debt_cleared);
    assert_eq!(s.missions_total, 7);
    assert!(s.text.is_empty(), "до победы заключительных слов нет");
}

#[test]
fn needs_both_conditions() {
    // только задания
    let mut a = world();
    for m in &mut a.missions.missions {
        m.status = MissionStatus::Done;
    }
    ending::check(&mut a);
    assert!(!a.ending.won, "без закрытого долга победы нет");

    // только долг
    let mut b = world();
    b.finance.cleared = true;
    ending::check(&mut b);
    assert!(!b.ending.won, "без пройденной цепочки победы нет");
}

#[test]
fn wins_when_both_met() {
    let mut w = world();
    let mail_before = w.machines[0].mail.len();
    win_conditions(&mut w);
    ending::check(&mut w);
    assert!(w.ending.won, "сошлись оба условия — победа");
    assert_eq!(w.player.flags.get("won").map(|s| s.as_str()), Some("1"));
    assert!(w.ending.pending.is_some(), "есть событие финала");
    assert!(w.machines[0].mail.len() > mail_before, "пришло последнее письмо");
    let s = ending::summary(&w);
    assert!(s.won && !s.text.is_empty(), "в сводке есть заключительные слова");
}

#[test]
fn game_continues_after_win() {
    let mut w = world();
    win_conditions(&mut w);
    ending::check(&mut w);
    let _ = ending::take_pending(&mut w);
    // мир живёт дальше: тики идут, повторная проверка не роняет и не дублирует
    for _ in 0..5 {
        w.tick(1000);
    }
    assert!(w.ending.won);
    assert!(w.ending.pending.is_none(), "событие финала не повторяется");
    // игрок по-прежнему может действовать — деньги на месте
    w.player.money += 100;
    assert!(w.player.money >= 100);
}

#[test]
fn found_cred_is_one_time_income() {
    let mut w = world();
    let c = FoundCred { machine: 5, user: "root".into(), password: "x".into(), note: "".into() };
    assert!(w.add_found_cred(c.clone()), "первый раз — новый логин");
    assert!(!w.add_found_cred(c.clone()), "тот же логин повторно не добывается");
    assert_eq!(w.player.found_creds.len(), 1, "дубль не копится — сбыт не ферма");
}
