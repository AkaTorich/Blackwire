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

#[test]
fn active_raid_cannot_extend_its_own_lock() {
    let mut w = world();
    w.player.heat = 80.0;
    w.investigation.progress = 100.0;
    watch::sweep(&mut w, 1);
    let deadline = w.investigation.locked_until;
    for _ in 0..119 {
        w.clock.epoch_secs += 1;
        w.investigation.progress = 100.0;
        watch::sweep(&mut w, 1);
    }
    assert_eq!(w.investigation.raids, 1);
    assert_eq!(w.investigation.locked_until, deadline);
    w.clock.epoch_secs = deadline;
    assert!(!watch::is_locked(&w));
    for _ in 0..120 {
        w.investigation.progress = 100.0;
        watch::sweep(&mut w, 1);
        assert!(!watch::is_locked(&w), "must allow time to recover between raids");
        w.clock.epoch_secs += 1;
    }
}

#[test]
fn tracing_warning_does_not_repeat_after_progress_falls_or_reload() {
    let mut w = world();
    w.player.heat = 80.0;
    w.investigation.progress = 69.9;
    watch::sweep(&mut w, 1);
    let before = w.machines[0].mail.len();
    w.machines[0].mail.last_mut().unwrap().read = true;
    w = World::from_json(&w.to_json()).unwrap();
    for _ in 0..10 {
        w.investigation.progress = 55.0;
        watch::sweep(&mut w, 1);
        w.investigation.progress = 69.9;
        watch::sweep(&mut w, 1);
    }
    assert_eq!(w.machines[0].mail.len(), before);
    assert!(w.machines[0].mail.last().unwrap().read, "threshold oscillation must not mark a warning unread again");
}

#[test]
fn repeated_raids_update_one_notice_and_mark_it_unread() {
    let mut w = world();
    w.player.heat = 80.0;
    for _ in 0..5 {
        w.clock.epoch_secs += 241;
        w.investigation.progress = 100.0;
        watch::sweep(&mut w, 1);
        let notices: Vec<_> = w.machines[0].mail.iter_mut().filter(|m| m.subject == "Канал заблокирован").collect();
        assert_eq!(notices.len(), 1);
        for m in notices {
            assert!(!m.read);
            m.read = true;
        }
    }
    assert_eq!(w.investigation.raids, 5);
}

#[test]
fn old_saves_load_without_repeating_current_warning() {
    let mut w = world();
    w.investigation.stage = InvStage::Tracing;
    w.investigation.progress = 80.0;
    let mut json = serde_json::to_value(&w).unwrap();
    json["investigation"].as_object_mut().unwrap().remove("notified_stage");
    let mut loaded = World::from_json(&json.to_string()).unwrap();
    let before = loaded.machines[0].mail.len();
    watch::sweep(&mut loaded, 1);
    assert_eq!(loaded.machines[0].mail.len(), before);
}

#[test]
fn closed_case_can_notify_again_without_duplicating_mail() {
    let mut w = world();
    w.player.heat = 80.0;
    w.investigation.progress = 69.9;
    watch::sweep(&mut w, 1);
    w.machines[0].mail.last_mut().unwrap().read = true;
    let before = w.machines[0].mail.len();
    w.player.heat = 0.0;
    w.investigation.progress = 0.0;
    watch::sweep(&mut w, 1);
    w.clock.epoch_secs += 300;
    w.player.heat = 80.0;
    w.investigation.progress = 69.9;
    watch::sweep(&mut w, 1);
    assert_eq!(w.machines[0].mail.len(), before);
    assert!(!w.machines[0].mail.last().unwrap().read);
    assert!(watch::take_pending(&mut w).is_some());
}

#[test]
fn prolonged_investigation_keeps_mail_bounded() {
    use blackwire_core::hack::counter::{Implant, ImplantKind};
    let mut w = world();
    w.player.money = 0;
    w.player.heat = 80.0;
    let now = w.now();
    for id in 1..=244 {
        w.defense.implants.push(Implant {
            id,
            path: format!("/tmp/.test{}", id),
            kind: ImplantKind::Tracker,
            planted: now,
        });
    }
    let before = w.machines[0].mail.len();
    for _ in 0..3600 {
        w.tick(1000);
    }
    let police_mail = w.machines[0].mail[before..].iter().filter(|m| {
        m.from.starts_with("otdel-k@") || m.from.starts_with("soc@") || m.from.starts_with("court@")
    }).count();
    assert!(police_mail <= 4, "one game hour produced {} police messages", police_mail);
    assert!(w.investigation.raids <= 30, "raid cooldown was bypassed");
    assert!(w.investigation.raids > 0, "investigation must still work");
}
