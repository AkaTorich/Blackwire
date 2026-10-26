//! Механики взлома (второй этап): уязвимости, эксплойты, подбор пароля,
//! повышение прав, следы и подозрение, прокси. Всё — над моделью игровой сети.

use blackwire_core::hack::{self, ExploitOutcome};
use blackwire_core::net::machine::{LogKind, MachineId};
use blackwire_core::session::Session;
use blackwire_core::world::World;

fn world() -> World {
    World::new(7, "ghost", "secret")
}

fn player_session(w: &World) -> Session {
    let home = w.machines[0].users.home_of("ghost");
    Session::new(1, w.machines[0].id, "ghost", &home)
}

/// Первая включённая машина (не игрок) с удалённой уязвимостью.
fn vulnerable_target(w: &World) -> Option<MachineId> {
    w.machines.iter().find(|m| m.id != 0 && m.up && !hack::remote_vulns(m).is_empty()).map(|m| m.id)
}

#[test]
fn vulns_exist_in_world() {
    let w = world();
    let total: usize = w.machines.iter().map(|m| hack::scan_vulns(m).len()).sum();
    assert!(total > 0, "в мире должны быть слабые места");
    assert!(vulnerable_target(&w).is_some(), "должна быть машина с удалённой уязвимостью");
}

#[test]
fn exploit_gives_access() {
    let mut w = world();
    let mut s = player_session(&w);
    let target = vulnerable_target(&w).expect("цель с уязвимостью");
    let report = hack::exploit(&mut w, &mut s, target, None).expect("эксплойт");
    assert!(!matches!(report.outcome, ExploitOutcome::Nothing), "эксплойт должен что-то дать");
    // журнал цели хранит попытку
    assert!(w.machine(target).logs.iter().any(|e| e.text.contains("эксплойт")), "попытка записана в журнал");
}

#[test]
fn exploit_shell_pushes_hop() {
    let mut w = world();
    // ищем машину, чья уязвимость даёт оболочку
    let target = w
        .machines
        .iter()
        .find(|m| m.id != 0 && m.up && hack::remote_vulns(m).iter().any(|v| matches!(v.access, hack::VulnAccess::Shell | hack::VulnAccess::Root)))
        .map(|m| m.id);
    if let Some(target) = target {
        let mut s = player_session(&w);
        let before = s.hops.len();
        let report = hack::exploit(&mut w, &mut s, target, None).expect("эксплойт");
        if matches!(report.outcome, ExploitOutcome::Shell { .. }) {
            assert_eq!(s.hops.len(), before + 1, "оболочка добавляет hop");
            assert_eq!(s.machine_id(), target, "текущий hop — цель");
            assert!(w.player.owned.contains(&target), "машина отмечена взломанной");
        }
    }
}

#[test]
fn crack_matches_known_password() {
    let mut w = world();
    let wl = w.machines[0].fs.read_text("/home/ghost/wordlist.txt").expect("словарь");
    let words: Vec<String> = wl.lines().map(|l| l.trim().to_string()).filter(|l| !l.is_empty()).collect();
    let set: std::collections::HashSet<&str> = words.iter().map(|s| s.as_str()).collect();
    let mut found_target = None;
    for m in w.machines.iter().filter(|m| m.id != 0 && m.up && m.service_named("ssh").is_some()) {
        if let Some(u) = m.users.list().iter().find(|u| set.contains(u.password.as_str())) {
            found_target = Some((m.id, u.name.clone(), u.password.clone()));
            break;
        }
    }
    let (id, user, pass) = found_target.expect("машина со слабым паролем");
    let s = player_session(&w);
    let report = hack::crack(&mut w, &s, id, &user, &words).expect("подбор");
    assert_eq!(report.found.as_deref(), Some(pass.as_str()), "пароль должен подобраться");
    assert!(w.player.found_creds.iter().any(|c| c.machine == id && c.user == user), "логин сохранён");
}

#[test]
fn privesc_grants_root() {
    let mut w = world();
    // машина с ssh, обычным пользователем и слабым паролем, чтобы войти честно
    let target = w.machines.iter().find(|m| m.id != 0 && m.up && m.service_named("ssh").is_some() && m.users.list().iter().any(|u| !u.is_admin)).map(|m| m.id);
    if let Some(id) = target {
        let (user, pass) = {
            let m = w.machine(id);
            let u = m.users.list().iter().find(|u| !u.is_admin).unwrap();
            (u.name.clone(), u.password.clone())
        };
        let mut s = player_session(&w);
        let home = w.machine(id).users.home_of(&user);
        // честный вход (проверим пароль)
        assert!(w.machine(id).users.check_password(&user, &pass));
        s.push_hop(id, &user, &home);
        let report = hack::privesc(&mut w, &mut s).expect("privesc");
        // заброшенная машина → повышение прав; аккуратная → нет (не ошибка)
        if report.ok {
            assert_eq!(s.user(), "root", "после privesc пользователь root");
            assert!(w.player.owned.contains(&id));
        }
    }
}

#[test]
fn heat_rises_on_monitored_and_decays() {
    let mut w = world();
    let monitored = w.machines.iter().find(|m| m.id != 0 && m.up && hack::is_monitored(&w, m.id) && !hack::remote_vulns(m).is_empty()).map(|m| m.id);
    if let Some(id) = monitored {
        let mut s = player_session(&w);
        assert_eq!(hack::heat(&w), 0.0);
        let _ = hack::exploit(&mut w, &mut s, id, None).expect("эксплойт");
        let after = hack::heat(&w);
        assert!(after > 0.0, "атака на наблюдаемую цель поднимает подозрение");
        // спад со временем
        for _ in 0..100 {
            hack::decay(&mut w, 1);
        }
        assert!(hack::heat(&w) < after, "подозрение спадает со временем");
    }
}

#[test]
fn clear_logs_removes_trace() {
    let mut w = world();
    let my_ip = w.machines[0].ip.clone();
    // возьмём любую включённую машину, оставим на ней след и войдём root'ом
    let id = w.machines.iter().find(|m| m.id != 0 && m.up).map(|m| m.id).unwrap();
    let now = w.now();
    w.machine_mut(id).log(now, LogKind::Login, format!("вход с {}", my_ip));
    let mut s = player_session(&w);
    s.push_hop(id, "root", "/root");
    assert!(w.machine(id).logs.iter().any(|e| e.text.contains(&my_ip)));
    let report = hack::clear_logs(&mut w, &s, None).expect("чистка");
    assert!(!report.denied, "root может чистить журналы");
    assert!(report.removed > 0, "запись со следом убрана");
    assert!(!w.machine(id).logs.iter().any(|e| e.text.contains(&my_ip)), "следа больше нет");
}

#[test]
fn proxy_hidden_through_chain() {
    let mut w = world();
    let mut s = player_session(&w);
    let info0 = hack::proxy_info(&w, &s);
    assert!(!info0.hidden, "со своей машины адрес виден");
    let id = w.machines.iter().find(|m| m.id != 0 && m.up).map(|m| m.id).unwrap();
    s.push_hop(id, "root", "/root");
    let info1 = hack::proxy_info(&w, &s);
    assert!(info1.hidden, "через цепочку адрес скрыт");
    assert_eq!(info1.exit_ip, w.machine(id).ip, "выходной адрес — последний hop");
    let _ = &mut w;
}
