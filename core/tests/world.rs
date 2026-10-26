//! Мир: генерация из зерна, сеть, ssh и журнал, достижимость, http, почта,
//! сохранение и загрузка, скрипты, часы.

use blackwire_core::game::Game;
use blackwire_core::net::machine::{LogKind, MachineKind};
use blackwire_core::types::{OutKind, UiAction};
use blackwire_core::world::{GameClock, World, START_EPOCH};

fn game() -> (Game, u32) {
    let mut g = Game::new(World::new(7, "ghost", "pass"));
    let s = g.open_session();
    (g, s)
}

fn out(g: &mut Game, s: u32, line: &str) -> String {
    let r = g.exec(s, line);
    r.lines.iter().filter(|l| l.kind != OutKind::Error).map(|l| l.text.clone()).collect::<Vec<_>>().join("\n")
}

#[test]
fn world_generates_player_first() {
    let w = World::new(7, "ghost", "pass");
    assert!(!w.machines.is_empty());
    assert_eq!(w.machines[0].kind, MachineKind::Player);
    assert_eq!(w.machines[0].hostname, "ghost-pc");
    assert!(w.machines[0].users.check_password("ghost", "pass"));
}

#[test]
fn world_has_several_machines_and_orgs() {
    let w = World::new(7, "ghost", "pass");
    assert!(w.machines.len() >= 10, "машин должно быть немало, а их {}", w.machines.len());
    assert!(!w.orgs.is_empty());
    assert!(!w.player.known_machines.is_empty());
}

#[test]
fn deterministic_from_seed() {
    let a = World::new(42, "neo", "x").to_json();
    let b = World::new(42, "neo", "x").to_json();
    assert_eq!(a, b);
    let c = World::new(43, "neo", "x").to_json();
    assert_ne!(a, c);
}

#[test]
fn clock_formats_start() {
    assert_eq!(GameClock::format_time(START_EPOCH), "2031-03-14 03:00:00");
}

#[test]
fn clock_ticks_forward() {
    let mut w = World::new(7, "ghost", "pass");
    let t0 = w.now();
    w.tick(5000);
    assert_eq!(w.now(), t0 + 5);
}

#[test]
fn save_load_roundtrip_equal() {
    let w = World::new(7, "ghost", "pass");
    let json = w.to_json();
    let w2 = World::from_json(&json).unwrap();
    assert_eq!(json, w2.to_json());
}

#[test]
fn save_load_preserves_edits() {
    let (mut g, s) = game();
    out(&mut g, s, "echo данные > /home/ghost/note.txt");
    let json = g.save_json();
    let g2 = Game::from_json(&json).unwrap();
    assert!(g2.fs_exists(None, "/home/ghost/note.txt"));
}

#[test]
fn resolve_known_hostname() {
    let w = World::new(7, "ghost", "pass");
    // любой публичный узел кроме игрока
    let target = w.machines.iter().find(|m| m.id != 0 && m.is_public()).unwrap();
    let ip = w.resolve(&target.hostname);
    assert_eq!(ip.as_deref(), Some(target.ip.as_str()));
}

#[test]
fn ssh_right_password_adds_hop_and_logs() {
    let (mut g, s) = game();
    // найти публичную машину с ssh и известным пользователем
    let (ip, user, pass) = {
        let w = &g.world;
        let m = w
            .machines
            .iter()
            .find(|m| m.id != 0 && m.is_public() && m.up && m.service_named("ssh").is_some() && m.users.list().iter().any(|u| u.name != "root"))
            .expect("нужна машина с ssh");
        let u = m.users.list().iter().find(|u| u.name != "root").unwrap();
        (m.ip.clone(), u.name.clone(), u.password.clone())
    };
    let id = g.world.find_by_ip(&ip).unwrap();
    let before = g.world.machine(id).logs.len();
    let r = g.exec(s, &format!("ssh {}@{} {}", user, ip, pass));
    assert!(!r.lines.iter().any(|l| l.kind == OutKind::Error), "вход должен пройти");
    assert_eq!(out(&mut g, s, "whoami"), user);
    assert_eq!(out(&mut g, s, "hostname"), g.world.machine(id).hostname);
    // журнал машины пополнился записью входа
    let logs = &g.world.machine(id).logs;
    assert!(logs.len() > before);
    assert!(logs.iter().any(|e| e.kind == LogKind::Login));
    // exit снимает hop
    out(&mut g, s, "exit");
    assert_eq!(out(&mut g, s, "hostname"), "ghost-pc");
}

#[test]
fn ssh_wrong_password_logs_failure() {
    let (mut g, s) = game();
    let (ip, user) = {
        let w = &g.world;
        let m = w.machines.iter().find(|m| m.id != 0 && m.is_public() && m.up && m.service_named("ssh").is_some()).unwrap();
        (m.ip.clone(), m.users.list()[0].name.clone())
    };
    let id = g.world.find_by_ip(&ip).unwrap();
    let r = g.exec(s, &format!("ssh {}@{} совсем-не-тот-пароль", user, ip));
    assert!(r.lines.iter().any(|l| l.kind == OutKind::Error));
    assert_eq!(out(&mut g, s, "hostname"), "ghost-pc", "цепочка не должна удлиниться");
    assert!(g.world.machine(id).logs.iter().any(|e| e.kind == LogKind::LoginFailed));
}

#[test]
fn lan_address_unreachable_without_gateway() {
    let w = World::new(7, "ghost", "pass");
    // внутренняя машина организации: ip == lan_ip, не выставлена наружу
    let internal = w.machines.iter().find(|m| !m.is_public() && m.org.is_some());
    if let Some(m) = internal {
        // с машины игрока (0) внутренний адрес недоступен
        let r = w.lookup(0, &m.ip);
        assert!(r.is_err(), "внутренний адрес {} не должен быть доступен снаружи", m.ip);
    }
}

#[test]
fn scan_lists_services() {
    let (mut g, s) = game();
    let ip = {
        let w = &g.world;
        w.machines.iter().find(|m| m.id != 0 && m.is_public() && m.up && !m.services.is_empty()).unwrap().ip.clone()
    };
    let o = out(&mut g, s, &format!("scan {}", ip));
    assert!(o.contains("ПОРТ") || o.contains("tcp"));
    // машина помечена сканированной
    let id = g.world.find_by_ip(&ip).unwrap();
    assert!(g.world.player.scanned.contains(&id));
}

#[test]
fn ping_known_machine() {
    let (mut g, s) = game();
    let ip = {
        let w = &g.world;
        w.machines.iter().find(|m| m.id != 0 && m.is_public() && m.up).unwrap().ip.clone()
    };
    let o = out(&mut g, s, &format!("ping {}", ip));
    assert!(o.contains("PING"));
    assert!(o.contains("статистика"));
}

#[test]
fn http_home_page() {
    let mut g = Game::new(World::new(7, "ghost", "pass"));
    let page = g.http("home://");
    assert_eq!(page.status, 200);
    assert!(page.body.contains("ghost-pc"));
}

#[test]
fn http_unknown_host() {
    let mut g = Game::new(World::new(7, "ghost", "pass"));
    let page = g.http("http://nonexistent.invalid/");
    assert_eq!(page.status, 404);
}

#[test]
fn net_map_has_nodes_and_links() {
    let g = Game::new(World::new(7, "ghost", "pass"));
    let map = g.net_map();
    assert_eq!(map.nodes.len(), g.world.machines.len());
    assert!(!map.links.is_empty());
    assert!(map.nodes.iter().find(|n| n.id == 0).unwrap().known);
}

#[test]
fn status_is_live() {
    let g = Game::new(World::new(7, "ghost", "pass"));
    let st = g.status();
    assert_eq!(st.hostname, "ghost-pc");
    assert!(st.mem_total > 0);
    assert!(!st.procs.is_empty());
}

#[test]
fn script_start_registers_process_and_pending() {
    let (mut g, s) = game();
    let r = g.exec(s, "adder scripts/hello.adr");
    let pid = match r.actions.iter().find_map(|a| if let UiAction::ScriptStarted { pid, .. } = a { Some(*pid) } else { None }) {
        Some(p) => p,
        None => panic!("должно быть действие ScriptStarted"),
    };
    let pending = g.take_pending_scripts();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].pid, pid);
    assert!(pending[0].path.ends_with("hello.adr"));
    // процесс есть в таблице
    assert!(g.world.machines[0].procs.get(pid).is_some());
    // kill помечает, is_killed истинно
    g.kill(pid).unwrap();
    assert!(g.is_killed(pid));
    // завершение убирает процесс
    g.script_finished(pid);
    assert!(g.world.machines[0].procs.get(pid).is_none());
}

#[test]
fn adder_check_emits_action() {
    let (mut g, s) = game();
    let r = g.exec(s, "adder --check scripts/hello.adr");
    assert!(r.actions.iter().any(|a| matches!(a, UiAction::CheckScript { .. })));
}

#[test]
fn adder_non_adr_errors() {
    let (mut g, s) = game();
    let r = g.exec(s, "adder readme.txt");
    assert!(r.lines.iter().any(|l| l.kind == OutKind::Error));
}

#[test]
fn mail_send_to_self_and_list() {
    let mut g = Game::new(World::new(7, "ghost", "pass"));
    let addr = g.mail_address();
    assert_eq!(addr, "ghost@home.net");
    g.mail_send(&addr, "Проверка", "тело письма").unwrap();
    // копия в отправленных
    let sent = g.mail_list("sent");
    assert!(sent.iter().any(|m| m.subject == "Проверка"));
    // доставлено во входящие (игрок — домашний пользователь ghost)
    let inbox = g.mail_list("inbox");
    assert!(inbox.iter().any(|m| m.subject == "Проверка"));
}

#[test]
fn mail_unknown_address_errors() {
    let mut g = Game::new(World::new(7, "ghost", "pass"));
    let r = g.mail_send("ktoto@nesushestvuet.zzz", "x", "y");
    assert!(r.is_err());
    assert!(r.unwrap_err().contains("адрес не найден"));
}

#[test]
fn mail_read_marks_and_unread_count() {
    let mut g = Game::new(World::new(7, "ghost", "pass"));
    let addr = g.mail_address();
    g.mail_send(&addr, "Тема", "текст").unwrap();
    let before = g.mail_unread_count();
    let inbox = g.mail_list("inbox");
    let id = inbox[0].id;
    let msg = g.mail_read(id).unwrap();
    assert!(msg.read);
    assert_eq!(g.mail_unread_count(), before - 1);
    g.mail_delete(id).unwrap();
    assert!(g.mail_list("inbox").iter().all(|m| m.id != id));
}

#[test]
fn settings_defaults() {
    let g = Game::new(World::new(7, "ghost", "pass"));
    let s = g.settings();
    assert_eq!(s.theme, "swordfish");
    assert_eq!(s.font_size, 14);
    assert!((s.terminal_opacity - 0.85).abs() < 1e-6);
    assert!(s.sound);
    assert_eq!(s.default_monitor, 0);
    assert!(s.fx.wireframe && s.fx.hexrain && s.fx.scanlines && s.fx.glass);
}

#[test]
fn player_flags_present_and_empty() {
    let w = World::new(7, "ghost", "pass");
    assert!(w.player.flags.is_empty());
}

#[test]
fn logs_command_requires_admin_but_works_for_owner() {
    let (mut g, s) = game();
    // ghost — администратор на своей машине
    out(&mut g, s, "echo x > y.txt");
    let r = g.exec(s, "logs");
    // журнал читается (может быть пуст, но без ошибки доступа)
    assert!(!r.lines.iter().any(|l| l.kind == OutKind::Error && l.text.contains("администратор")));
}

#[test]
fn shell_doc_contains_markdown() {
    let g = Game::new(World::new(7, "ghost", "pass"));
    let doc = g.shell_doc();
    assert!(doc.contains("# Команды"));
    assert!(doc.contains("| `ls`"));
    assert!(doc.contains("ssh"));
}
