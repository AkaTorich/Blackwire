//! Ответный удар и защита: при высокой угрозе враг подсаживает закладки и
//! штрафует; свой файрвол отражает; закладки вредят и чистятся.

use blackwire_core::hack::counter::{self, Implant, ImplantKind};
use blackwire_core::hack::watch::InvStage;
use blackwire_core::world::World;

fn world() -> World {
    World::new(7, "ghost", "secret")
}

/// Сымитировать далёкое расследование.
fn under_threat(w: &mut World) {
    w.player.traced = true;
    w.investigation.stage = InvStage::Tracing;
    w.investigation.progress = 80.0;
}

#[test]
fn calm_world_no_counter() {
    let mut w = world();
    for _ in 0..200 {
        w.clock.epoch_secs += 45;
        counter::tick(&mut w, 1);
    }
    assert!(w.defense.implants.is_empty(), "без угрозы врагу не за что бить");
    assert_eq!(w.defense.fines_total, 0);
}

#[test]
fn threat_triggers_counterattack() {
    let mut w = world();
    w.player.money = 5000;
    under_threat(&mut w);
    for _ in 0..300 {
        w.clock.epoch_secs += 45;
        counter::tick(&mut w, 1);
    }
    assert!(!w.defense.implants.is_empty() || w.defense.fines_total > 0, "под угрозой враг бьёт в ответ");
}

#[test]
fn shield_blocks_counter() {
    let mut w = world();
    w.player.money = 5000;
    under_threat(&mut w);
    counter::set_shield(&mut w, true);
    for _ in 0..300 {
        w.clock.epoch_secs += 45;
        counter::tick(&mut w, 1);
    }
    assert!(w.defense.implants.is_empty(), "файрвол отражает закладки");
    assert_eq!(w.defense.fines_total, 0, "файрвол отражает и штрафы");
}

#[test]
fn tracker_implant_raises_investigation() {
    let mut w = world();
    // спокойно, но стоит маяк
    w.investigation.progress = 10.0;
    w.defense.implants.push(Implant { id: 1, path: "/tmp/.sys1".into(), kind: ImplantKind::Tracker, planted: w.now() });
    let p0 = w.investigation.progress;
    for _ in 0..50 {
        counter::tick(&mut w, 1);
    }
    assert!(w.investigation.progress > p0, "маяк тянет на тебя следствие");
}

#[test]
fn clean_removes_implants() {
    let mut w = world();
    w.defense.implants.push(Implant { id: 1, path: "/tmp/.sys1".into(), kind: ImplantKind::Miner, planted: w.now() });
    w.defense.implants.push(Implant { id: 2, path: "/tmp/.sys2".into(), kind: ImplantKind::Backdoor, planted: w.now() });
    let n = counter::clean(&mut w, Some("all"));
    assert_eq!(n, 2);
    assert!(w.defense.implants.is_empty());
}

#[test]
fn shield_toggles() {
    let mut w = world();
    assert!(!w.defense.shield);
    counter::set_shield(&mut w, true);
    assert!(w.defense.shield);
    counter::set_shield(&mut w, false);
    assert!(!w.defense.shield);
}
