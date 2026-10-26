//! Слежка (второй этап): расследование центров безопасности и киберполиции
//! растёт от улик и подозрения, шлёт письма, на пике — рейд с блокировкой;
//! без активности — стихает. Всё над моделью игровой сети.

use blackwire_core::hack::watch::{self, InvEvent, InvStage};
use blackwire_core::hack::{self};
use blackwire_core::net::machine::LogKind;
use blackwire_core::world::World;
use blackwire_core::Game;

fn world() -> World {
    World::new(7, "ghost", "secret")
}

#[test]
fn lead_org_exists() {
    let mut w = world();
    watch::sweep(&mut w, 1);
    assert!(w.investigation.lead_org.is_some(), "в сети есть центр безопасности или киберполиция");
}

#[test]
fn investigation_rises_and_raids() {
    let mut w = world();
    // оставим след и зададим подозрение
    let my_ip = w.machines[0].ip.clone();
    let id = w.machines.iter().find(|m| m.id != 0 && m.up).map(|m| m.id).unwrap();
    let now = w.now();
    w.machine_mut(id).log(now, LogKind::Login, format!("вход с {}", my_ip));
    w.player.heat = 80.0;

    let mut raided = false;
    for _ in 0..3000 {
        watch::sweep(&mut w, 1);
        if w.investigation.raids > 0 {
            raided = true;
            break;
        }
    }
    assert!(raided, "при уликах и высоком подозрении дойдёт до рейда");
    assert!(watch::is_locked(&w), "после рейда канал заблокирован");
    assert!(watch::lock_remaining(&w) > 0);
}

#[test]
fn investigation_decays_when_quiet() {
    let mut w = world();
    w.investigation.progress = 50.0;
    w.player.heat = 0.0;
    // улик нет
    for _ in 0..200 {
        watch::sweep(&mut w, 1);
    }
    assert!(w.investigation.progress < 50.0, "без активности дело стихает");
}

#[test]
fn warning_mail_on_stage_rise() {
    let mut w = world();
    let before = w.machines[0].mail.len();
    w.player.heat = 60.0;
    // довести до стадии «замечена активность» и дальше
    for _ in 0..600 {
        watch::sweep(&mut w, 1);
        if matches!(w.investigation.stage, InvStage::Noticed | InvStage::Investigating | InvStage::Tracing) {
            break;
        }
    }
    assert!(w.investigation.stage != InvStage::Dormant, "стадия поднялась");
    let after = w.machines[0].mail.len();
    assert!(after > before, "следствие прислало предупреждение");
    assert!(w.machines[0].mail.iter().any(|m| m.from.contains("@")), "у письма есть отправитель-орган");
}

#[test]
fn raid_drops_hops_and_notifies() {
    let mut w = world();
    let target = w.machines.iter().find(|m| m.id != 0 && m.up).map(|m| m.id).unwrap();
    let mut g = {
        // соберём Game из мира
        Game::new(w.clone())
    };
    let sid = g.open_session();
    // войдём на машину (искусственно) — добавим hop
    let home = g.world.machine(target).users.home_of("root");
    g.sessions.get_mut(&sid).unwrap().push_hop(target, "root", &home);
    assert!(g.sessions.get(&sid).unwrap().hops.len() > 1);
    // подсунем событие рейда
    g.world.investigation.pending = Some(InvEvent {
        stage: InvStage::Raid,
        title: "Отдел К".to_string(),
        text: "рейд".to_string(),
        raided: true,
    });
    let ev = g.poll_surveillance().expect("событие слежки");
    assert!(ev.2, "это рейд");
    assert_eq!(g.sessions.get(&sid).unwrap().hops.len(), 1, "рейд оборвал цепочку — вернулись домой");
    let _ = &mut w;
}

#[test]
fn locked_blocks_attacks_via_host() {
    let mut w = world();
    let now = w.now();
    w.investigation.locked_until = now + 100;
    assert!(hack::is_locked(&w));
    assert!(hack::lock_remaining(&w) > 0);
}
