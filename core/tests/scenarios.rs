//! Свои сценарии: запуск, проверка условий, награда, сохранение.

use blackwire_core::net::machine::{MailFolder, MailMessage};
use blackwire_core::scenarios::{self, Scenario, ScenarioGoal};
use blackwire_core::world::World;

fn world() -> World {
    World::new(7, "ghost", "secret")
}

#[test]
fn money_goal_completes_and_pays() {
    let mut w = world();
    w.player.money = 500;
    scenarios::start(&mut w, Scenario {
        name: "Касса".into(),
        brief: "Накопить на дорогу.".into(),
        goal: ScenarioGoal::Money { amount: 400 },
        reward: 200,
    });
    w.tick(1000);
    assert!(w.scenarios.done, "цель по деньгам достигнута");
    assert_eq!(w.player.money, 700, "награда начислена один раз");
    let ev = scenarios::take_pending(&mut w).expect("уведомление");
    assert!(ev.text.contains("Касса"));
    // повторно не платит
    w.tick(1000);
    assert_eq!(w.player.money, 700);
}

#[test]
fn own_host_goal() {
    let mut w = world();
    let target = w.machines.iter().find(|m| m.id != 0).map(|m| (m.id, m.hostname.clone())).unwrap();
    scenarios::start(&mut w, Scenario {
        name: "Захват".into(),
        brief: "Взять узел.".into(),
        goal: ScenarioGoal::OwnHost { host: target.1.clone() },
        reward: 300,
    });
    w.tick(1000);
    assert!(!w.scenarios.done, "пока узел не захвачен");
    w.player.owned.push(target.0);
    w.tick(1000);
    assert!(w.scenarios.done, "узел захвачен — сценарий пройден");
}

#[test]
fn deliver_goal_checks_sent_mail() {
    let mut w = world();
    scenarios::start(&mut w, Scenario {
        name: "Передача".into(),
        brief: "Слить улику.".into(),
        goal: ScenarioGoal::Deliver { needle: "ПАРОЛЬ-42".into() },
        reward: 150,
    });
    w.tick(1000);
    assert!(!w.scenarios.done);
    let now = w.now();
    w.machines[0].push_mail(MailMessage {
        id: 0,
        from: "ghost@home.net".into(),
        to: "drop@hw".into(),
        subject: "вот".into(),
        body: "держи: ПАРОЛЬ-42 внутри".into(),
        time: now,
        read: true,
        folder: MailFolder::Sent,
    });
    w.tick(1000);
    assert!(w.scenarios.done, "строка ушла в «Отправленных»");
}

#[test]
fn targets_lists_public_hosts() {
    let w = world();
    let t = scenarios::targets(&w);
    assert!(!t.is_empty(), "есть публичные узлы для конструктора");
    assert!(t.windows(2).all(|p| p[0] <= p[1]), "отсортировано");
}

#[test]
fn survives_save_roundtrip() {
    let mut w = world();
    scenarios::start(&mut w, Scenario {
        name: "Касса".into(),
        brief: "x".into(),
        goal: ScenarioGoal::Money { amount: 999999 },
        reward: 10,
    });
    let w2 = World::from_json(&w.to_json()).expect("загрузка");
    assert!(w2.scenarios.active.is_some(), "активный сценарий сохраняется");
}
