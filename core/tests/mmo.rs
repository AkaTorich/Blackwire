//! Общий мир онлайна: два игрока в одной сети, подстановка туда и обратно,
//! личный и общий ход времени.

use blackwire_core::game::Game;
use blackwire_core::mmo::PlayerCtx;
use blackwire_core::net::machine::MachineKind;
use blackwire_core::world::World;

fn two_players() -> (Game, PlayerCtx, PlayerCtx) {
    let mut world = World::mmo_new(42);
    let a = world.mmo_add_player("alice", "pa55-a", "ru");
    let b = world.mmo_add_player("bob", "pa55-b", "ru");
    (Game::new(world), a, b)
}

#[test]
fn players_get_own_machines() {
    let (g, a, b) = two_players();
    assert_ne!(a.home, b.home);
    assert_eq!(g.world.machines[0].ip, "0.0.0.0", "без игрока на месте 0 заглушка");
    assert!(!g.world.machines[0].up);
    let pcs: Vec<_> = g.world.mmo_player_machines().map(|m| m.hostname.clone()).collect();
    assert!(pcs.contains(&"alice-pc".to_string()) && pcs.contains(&"bob-pc".to_string()));
    // номер = место в списке
    for (i, m) in g.world.machines.iter().enumerate() {
        assert_eq!(m.id as usize, i);
    }
    assert!(g.world.player.handle.is_empty(), "личная часть мира пуста");
}

#[test]
fn swap_is_symmetric() {
    let (mut g, mut a, _b) = two_players();
    let before: Vec<String> = g.world.machines.iter().map(|m| m.hostname.clone()).collect();
    g.swap_player(&mut a);
    assert_eq!(g.world.machines[0].hostname, "alice-pc");
    assert_eq!(g.world.player.handle, "alice");
    assert!(g.mail_unread_count() > 0, "стартовые письма у игрока");
    assert!(g.world.missions.initialized && !g.world.missions.missions.is_empty());
    g.swap_player(&mut a);
    let after: Vec<String> = g.world.machines.iter().map(|m| m.hostname.clone()).collect();
    assert_eq!(before, after);
    assert!(g.world.player.handle.is_empty());
    for (i, m) in g.world.machines.iter().enumerate() {
        assert_eq!(m.id as usize, i);
    }
}

#[test]
fn shell_works_per_player_and_sees_the_other() {
    let (mut g, mut a, mut b) = two_players();
    let bob_ip = g.world.machines[b.home as usize].ip.clone();

    g.swap_player(&mut a);
    let s = g.open_session();
    let r = g.exec(s, "whoami");
    assert!(r.lines.iter().any(|l| l.text.contains("alice")), "{:?}", r.lines);
    let r = g.exec(s, &format!("ping {}", bob_ip));
    assert!(!r.lines.is_empty());
    let r = g.exec(s, "touch ~/from_alice.txt");
    let _ = r;
    g.swap_player(&mut a);

    g.swap_player(&mut b);
    let s2 = g.open_session();
    let r = g.exec(s2, "whoami");
    assert!(r.lines.iter().any(|l| l.text.contains("bob")), "{:?}", r.lines);
    let r = g.exec(s2, "ls ~");
    assert!(!r.lines.iter().any(|l| l.text.contains("from_alice")), "файлы Алисы не у Боба");
    g.swap_player(&mut b);

    // сеанс Алисы сохранился в её части
    g.swap_player(&mut a);
    let r = g.exec(s, "ls ~");
    assert!(r.lines.iter().any(|l| l.text.contains("from_alice")), "{:?}", r.lines);
    g.swap_player(&mut a);
}

#[test]
fn ticks_run_without_panic() {
    let (mut g, mut a, mut b) = two_players();
    for _ in 0..300 {
        let n = g.world.mmo_tick_shared(1000);
        for _ in 0..n {
            for c in [&mut a, &mut b] {
                g.swap_player(c);
                g.mmo_tick_player();
                let _ = g.poll_missions();
                let _ = g.poll_finance();
                g.swap_player(c);
            }
        }
    }
    assert!(g.world.player.handle.is_empty());
    // компьютеры игроков живой мир не выключает
    for m in g.world.machines.iter().filter(|m| m.kind == MachineKind::Player && m.id != 0) {
        assert!(m.up, "{} выключен", m.hostname);
    }
}

#[test]
fn save_round_trip() {
    let (g, a, _b) = two_players();
    let json = g.world.to_json();
    let w = World::from_json(&json).expect("мир читается");
    assert_eq!(w.machines.len(), g.world.machines.len());
    assert!(!w.invite.is_empty());
    let pj = serde_json::to_string(&a).unwrap();
    let back: PlayerCtx = serde_json::from_str(&pj).unwrap();
    assert_eq!(back.player.handle, "alice");
    assert_eq!(back.home, a.home);
}
