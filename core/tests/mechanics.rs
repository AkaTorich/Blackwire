//! Новые механики взлома: шифр, файрвол, ловушка, кейлоггер, ботнет.

use blackwire_core::hack::{self, cipher, defense, tools};
use blackwire_core::net::machine::MachineId;
use blackwire_core::session::Session;
use blackwire_core::world::World;

fn world() -> World {
    World::new(7, "ghost", "secret")
}

fn session(w: &World) -> Session {
    let home = w.machines[0].users.home_of("ghost");
    Session::new(1, w.machines[0].id, "ghost", &home)
}

#[test]
fn cipher_roundtrip() {
    let text = "Секрет: пароль 42, доступ к backup@srv-1.\nВторая строка.";
    let enc = cipher::encrypt(text, "ключ-слово");
    assert_ne!(enc, text);
    let dec = cipher::decrypt(&enc, "ключ-слово").expect("расшифровка верным ключом");
    assert_eq!(dec, text);
    // неверный ключ — не исходный текст (ошибка или мусор)
    match cipher::decrypt(&enc, "другой") {
        Ok(s) => assert_ne!(s, text),
        Err(_) => {}
    }
}

fn firewalled(w: &World) -> Option<MachineId> {
    w.machines.iter().map(|m| m.id).find(|id| *id != 0 && defense::has_firewall(w, *id))
}

#[test]
fn firewall_blocks_until_bypassed() {
    let mut w = world();
    let id = match firewalled(&w) {
        Some(id) => id,
        None => return, // в этом мире нет защищённых узлов — пропускаем
    };
    assert!(defense::firewall_up(&w, id));
    let mut s = session(&w);
    // эксплойт закрыт файрволом
    let r = hack::exploit(&mut w, &mut s, id, None);
    assert!(r.is_err(), "файрвол блокирует эксплойт");
    // обойдём принудительно
    w.fw_down.insert(id);
    assert!(!defense::firewall_up(&w, id), "после обхода узел открыт");
}

#[test]
fn honeypot_trap_spikes_heat() {
    let mut w = world();
    let trap = w.machines.iter().map(|m| m.id).find(|id| *id != 0 && defense::is_trap(&w, *id));
    if let Some(id) = trap {
        let h0 = hack::heat(&w);
        defense::spring_trap(&mut w, id);
        assert!(hack::heat(&w) > h0 + 20.0, "ловушка резко поднимает подозрение");
        assert_eq!(w.player.flags.get("trapped").map(|s| s.as_str()), Some("1"));
    }
}

#[test]
fn keylogger_captures_over_time() {
    let mut w = world();
    let id = w.machines.iter().map(|m| m.id).find(|id| *id != 0 && w.machine(*id).up && !w.machine(*id).users.list().is_empty()).unwrap();
    w.player.owned.push(id);
    tools::plant_keylog(&mut w, id).expect("установка");
    // промотаем время
    w.clock.epoch_secs += 100;
    tools::tick_keyloggers(&mut w, 1);
    assert!(w.keyloggers[0].captured.is_some(), "кейлоггер поймал логин");
    assert!(!w.player.found_creds.is_empty(), "логин попал в находки");
}

#[test]
fn botnet_earns_passively() {
    let mut w = world();
    let id = w.machines.iter().map(|m| m.id).find(|id| *id != 0 && w.machine(*id).up).unwrap();
    w.player.owned.push(id);
    tools::botnet_add(&mut w, id).expect("захват");
    let money0 = w.player.money;
    for _ in 0..130 {
        tools::tick_botnet(&mut w, 1);
    }
    assert!(w.player.money > money0, "ботнет приносит доход");
    assert!(tools::botnet_size(&w) >= 1);
}

#[test]
fn keylog_needs_access() {
    let mut w = world();
    let id = w.machines.iter().map(|m| m.id).find(|id| *id != 0).unwrap();
    assert!(tools::plant_keylog(&mut w, id).is_err(), "без доступа кейлоггер не поставить");
}
