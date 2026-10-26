//! Задания hello-world: построение цепочки, проверка выполнения по состоянию
//! мира, награда и ответ группы. Над моделью игровой сети.

use blackwire_core::missions::{self, Goal, MissionStatus};
use blackwire_core::net::machine::{MailFolder, MailMessage};
use blackwire_core::world::World;

fn world() -> World {
    let mut w = World::new(7, "ghost", "secret");
    missions::check(&mut w); // строит цепочку
    w
}

fn push_sent(w: &mut World, to: &str, body: &str) {
    let now = w.now();
    w.machines[0].mail.push(MailMessage {
        id: 999,
        from: "ghost@home.net".to_string(),
        to: to.to_string(),
        subject: "добыча".to_string(),
        body: body.to_string(),
        time: now,
        read: true,
        folder: MailFolder::Sent,
    });
}

#[test]
fn chain_builds() {
    let w = world();
    assert_eq!(w.missions.missions.len(), 7, "в цепочке семь заданий");
    assert_eq!(w.missions.active, 0);
    assert_eq!(w.missions.missions[0].status, MissionStatus::Active);
    assert_eq!(w.missions.missions[1].status, MissionStatus::Locked);
    // первое задание пришло письмом от группы
    assert!(w.machines[0].mail.iter().any(|m| m.from == "null@hw" && m.subject.contains("Стук")), "брифинг m1 в почте");
}

#[test]
fn targets_are_real_machines() {
    let w = world();
    for m in &w.missions.missions {
        match &m.goal {
            Goal::ReadDeliver { target, .. } | Goal::Own { target } | Goal::Plant { target, .. } => {
                assert!(w.has_machine(*target), "цель задания — настоящая машина: {}", m.title);
            }
            Goal::CoverTracks { .. } => {}
        }
    }
}

#[test]
fn m1_completes_on_delivery() {
    let mut w = world();
    let (to, needle) = match &w.missions.missions[0].goal {
        Goal::ReadDeliver { to, needle, .. } => (to.clone(), needle.clone()),
        _ => panic!("m1 — доставка файла"),
    };
    let reward = w.missions.missions[0].reward;
    // игрок прислал содержимое файла группе
    push_sent(&mut w, &to, &format!("вот файл:\n{}\n", needle));
    missions::check(&mut w);
    assert_eq!(w.missions.missions[0].status, MissionStatus::Done, "m1 выполнено");
    assert_eq!(w.player.money, reward, "награда начислена");
    assert_eq!(w.missions.active, 1, "следующее задание открыто");
    assert_eq!(w.missions.missions[1].status, MissionStatus::Active);
    // ответ группы в почте
    assert!(w.machines[0].mail.iter().any(|m| m.from == "null@hw" && m.subject.contains("Принято")), "ответ группы пришёл");
}

#[test]
fn own_mission_completes() {
    let mut w = world();
    // найдём задание с целью Own
    let idx = w.missions.missions.iter().position(|m| matches!(m.goal, Goal::Own { .. })).expect("есть задание на доступ");
    let target = match &w.missions.missions[idx].goal {
        Goal::Own { target } => *target,
        _ => unreachable!(),
    };
    // сделаем его текущим
    w.missions.active = idx;
    w.missions.missions[idx].status = MissionStatus::Active;
    // ещё не выполнено
    missions::check(&mut w);
    assert_eq!(w.missions.missions[idx].status, MissionStatus::Active);
    // получили доступ к цели
    w.player.owned.push(target);
    missions::check(&mut w);
    assert_eq!(w.missions.missions[idx].status, MissionStatus::Done, "доступ — задание выполнено");
}

#[test]
fn plant_mission_completes() {
    let mut w = world();
    let idx = w.missions.missions.iter().position(|m| matches!(m.goal, Goal::Plant { .. })).expect("есть задание на метку");
    let (target, path, needle) = match &w.missions.missions[idx].goal {
        Goal::Plant { target, path, needle } => (*target, path.clone(), needle.clone()),
        _ => unreachable!(),
    };
    w.missions.active = idx;
    w.missions.missions[idx].status = MissionStatus::Active;
    // игрок создал файл с меткой
    let _ = w.machine_mut(target).fs.write_text(&path, &format!("{} — тень вернулась", needle), "root");
    missions::check(&mut w);
    assert_eq!(w.missions.missions[idx].status, MissionStatus::Done, "метка поставлена — задание выполнено");
}

#[test]
fn cover_tracks_completes_when_clean() {
    let mut w = world();
    let idx = w.missions.missions.iter().position(|m| matches!(m.goal, Goal::CoverTracks { .. })).expect("есть задание замести следы");
    w.missions.active = idx;
    w.missions.missions[idx].status = MissionStatus::Active;
    w.player.heat = 5.0; // чисто
    missions::check(&mut w);
    assert_eq!(w.missions.missions[idx].status, MissionStatus::Done, "чисто — задание выполнено");
}

#[test]
fn board_html_renders() {
    let w = world();
    let html = missions::board_html(&w);
    assert!(html.contains("Доска заданий"));
    assert!(html.contains("Стук в дверь"), "на доске видно первое задание");
}
