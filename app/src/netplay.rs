//! Сетевая игра: хост открывает порт, остальные подключаются по адресу —
//! локальный адрес даёт игру по локалке, публичный (с пробросом порта) — через
//! интернет. Это один и тот же TCP. Хост — ведущий (хаб): держит общую картину
//! (кто в игре, у кого какой прогресс, кто взял цель) и рассылает её всем.
//!
//! Два режима:
//! - «дуэль» (вдвоём, versus) — кто первым получит доступ к узлу-призу, тот и
//!   выиграл;
//! - «команда» (вместе, co-op) — общий список целей, берём их сообща, побеждает
//!   команда, когда взяты все.
//!
//! Партия идёт в обычном мире игры: при старте все строят мир из общего зерна
//! (сеть одинакова у всех), каждый — у своей машины. Часы мира (`ticker`) раз в
//! секунду считают прогресс игрока по целям и шлют его хосту.

use std::collections::BTreeSet;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream, UdpSocket};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use blackwire_core::world::World;
use blackwire_core::Game;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, State};

use crate::state::AppState;

/// Сколько целей в командном режиме.
const TEAM_TARGETS: usize = 3;

#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
pub enum Role {
    None,
    Host,
    Client,
}

/// Игрок в партии — для таблицы.
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct PeerView {
    pub name: String,
    /// 0 — ничего, 1 — виден, 2 — просканирован, 3 — доступ (взят).
    pub stage: u8,
    /// Сколько целей взял (командный режим).
    pub score: u32,
}

/// Общая картина партии (её держит хост, клиентам рассылает).
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Snapshot {
    pub mode: String,  // "duel" | "team"
    pub phase: String, // "lobby" | "playing" | "done"
    pub seed: u64,
    pub targets: Vec<String>,
    pub peers: Vec<PeerView>,
    pub captured: Vec<String>,
    pub winner: Option<String>,
}

/// Состояние сетевой игры (в оболочке, под мьютексом).
pub struct NetState {
    pub role: Role,
    pub my_name: String,
    pub snap: Snapshot,
    pub port: u16,
    pub error: Option<String>,
    // хост: потоки подключённых клиентов
    clients: Vec<Arc<Mutex<TcpStream>>>,
    stop: Arc<AtomicBool>,
    // клиент: поток к хосту
    host_stream: Option<Arc<Mutex<TcpStream>>>,
    // чтобы не слать один и тот же прогресс повторно
    sent_stage: u8,
    sent_caps: BTreeSet<String>,
    backed_up: bool,
}

impl Default for NetState {
    fn default() -> Self {
        NetState {
            role: Role::None,
            my_name: String::new(),
            snap: Snapshot::default(),
            port: 0,
            error: None,
            clients: Vec::new(),
            stop: Arc::new(AtomicBool::new(false)),
            host_stream: None,
            sent_stage: 0,
            sent_caps: BTreeSet::new(),
            backed_up: false,
        }
    }
}

impl NetState {
    fn reset(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        *self = NetState::default();
    }
}

// ---------------------------------------------------------------- вспомогательное

fn write_line(stream: &Arc<Mutex<TcpStream>>, line: &str) {
    if let Ok(mut s) = stream.lock() {
        let _ = s.write_all(line.as_bytes());
        let _ = s.write_all(b"\n");
        let _ = s.flush();
    }
}

/// Локальные адреса машины (для показа хосту).
fn local_ips() -> Vec<String> {
    let mut out = Vec::new();
    if let Ok(sock) = UdpSocket::bind("0.0.0.0:0") {
        // не отправляет пакетов — просто выбирает исходящий интерфейс
        if sock.connect("8.8.8.8:80").is_ok() {
            if let Ok(addr) = sock.local_addr() {
                out.push(addr.ip().to_string());
            }
        }
    }
    out.push("127.0.0.1".to_string());
    out.dedup();
    out
}

/// Текущее имя игрока из запущенной игры.
fn my_handle(st: &AppState) -> String {
    st.game().map(|g| g.world.player.handle.clone()).unwrap_or_else(|_| "player".to_string())
}

/// Стадия игрока по узлу-цели в его мире: 3 — доступ, 2 — скан, 1 — виден, 0 — нет.
fn stage_for(world: &World, host: &str) -> u8 {
    match world.find_by_hostname(host) {
        Some(id) => {
            if world.player.owned.contains(&id) {
                3
            } else if world.player.scanned.contains(&id) {
                2
            } else if world.is_known(id) {
                1
            } else {
                0
            }
        }
        None => 0,
    }
}

// ---------------------------------------------------------------- хост: ведение партии

/// Пересчитать победу и разослать картину всем.
fn host_recompute_and_broadcast(app: &AppHandle, net: &mut NetState) {
    if net.snap.phase == "playing" {
        if net.snap.mode == "duel" {
            if let Some(w) = net.snap.peers.iter().find(|p| p.stage >= 3) {
                net.snap.winner = Some(w.name.clone());
                net.snap.phase = "done".to_string();
            }
        } else if !net.snap.targets.is_empty() && net.snap.captured.len() >= net.snap.targets.len() {
            net.snap.winner = Some("команда".to_string());
            net.snap.phase = "done".to_string();
        }
    }
    let line = serde_json::to_string(&serde_json::json!({ "t": "state", "snap": net.snap })).unwrap_or_default();
    for c in &net.clients {
        write_line(c, &line);
    }
    let _ = app.emit("net_event", serde_json::json!({ "snap": net.snap }));
}

/// Поток-приёмник одного клиента: читает его прогресс, обновляет картину.
fn spawn_client_reader(app: AppHandle, stream: TcpStream, stop: Arc<AtomicBool>) {
    std::thread::spawn(move || {
        let reader = match stream.try_clone() {
            Ok(s) => BufReader::new(s),
            Err(_) => return,
        };
        let mut name = String::new();
        for line in reader.lines() {
            if stop.load(Ordering::Relaxed) {
                break;
            }
            let line = match line {
                Ok(l) => l,
                Err(_) => break,
            };
            let v: serde_json::Value = match serde_json::from_str(&line) {
                Ok(v) => v,
                Err(_) => continue,
            };
            let st = app.state::<AppState>();
            let Ok(mut net) = st.net.lock() else { break };
            match v.get("t").and_then(|t| t.as_str()) {
                Some("join") => {
                    name = v.get("name").and_then(|x| x.as_str()).unwrap_or("гость").to_string();
                    if !net.snap.peers.iter().any(|p| p.name == name) {
                        net.snap.peers.push(PeerView { name: name.clone(), stage: 0, score: 0 });
                    }
                    host_recompute_and_broadcast(&app, &mut net);
                }
                Some("prog") => {
                    let s = v.get("stage").and_then(|x| x.as_u64()).unwrap_or(0) as u8;
                    if let Some(p) = net.snap.peers.iter_mut().find(|p| p.name == name) {
                        if s > p.stage {
                            p.stage = s;
                        }
                    }
                    host_recompute_and_broadcast(&app, &mut net);
                }
                Some("cap") => {
                    let host = v.get("host").and_then(|x| x.as_str()).unwrap_or("").to_string();
                    if !host.is_empty() && !net.snap.captured.contains(&host) {
                        net.snap.captured.push(host);
                        if let Some(p) = net.snap.peers.iter_mut().find(|p| p.name == name) {
                            p.score += 1;
                            p.stage = 3;
                        }
                    }
                    host_recompute_and_broadcast(&app, &mut net);
                }
                _ => {}
            }
        }
        // клиент отключился — убрать из таблицы
        let st = app.state::<AppState>();
        let locked = st.net.lock();
        if let Ok(mut net) = locked {
            if !name.is_empty() {
                net.snap.peers.retain(|p| p.name != name);
                host_recompute_and_broadcast(&app, &mut net);
            }
        }
    });
}

// ---------------------------------------------------------------- клиент: приём картины

fn spawn_host_reader(app: AppHandle, stream: TcpStream, stop: Arc<AtomicBool>) {
    std::thread::spawn(move || {
        let reader = BufReader::new(stream);
        for line in reader.lines() {
            if stop.load(Ordering::Relaxed) {
                break;
            }
            let line = match line {
                Ok(l) => l,
                Err(_) => break,
            };
            let v: serde_json::Value = match serde_json::from_str(&line) {
                Ok(v) => v,
                Err(_) => continue,
            };
            if v.get("t").and_then(|t| t.as_str()) == Some("state") {
                if let Some(snap) = v.get("snap").and_then(|s| serde_json::from_value::<Snapshot>(s.clone()).ok()) {
                    let st = app.state::<AppState>();
                    let was_playing = {
                        let Ok(mut net) = st.net.lock() else { break };
                        let prev = net.snap.phase.clone();
                        net.snap = snap.clone();
                        prev != "playing" && snap.phase == "playing"
                    };
                    if was_playing {
                        start_match(&app, &st, snap.seed);
                    }
                    let _ = app.emit("net_event", serde_json::json!({ "snap": snap }));
                }
            }
        }
        let st = app.state::<AppState>();
        let locked = st.net.lock();
        if let Ok(mut net) = locked {
            if net.role == Role::Client {
                net.error = Some("связь с хостом потеряна".to_string());
                let _ = app.emit("net_event", serde_json::json!({ "snap": net.snap, "error": net.error }));
            }
        }
    });
}

// ---------------------------------------------------------------- старт партии в мире

/// Построить мир из общего зерна под своим именем и начать игру.
fn start_match(app: &AppHandle, st: &AppState, seed: u64) {
    // сохранить текущую игру в слот, чтобы вернуться после партии
    let name = my_handle(st);
    if let Ok(mut net) = st.net.lock() {
        if !net.backed_up {
            let _ = st.save_slot("до-сети");
            net.backed_up = true;
        }
        net.sent_stage = 0;
        net.sent_caps.clear();
    }
    let world = World::new(seed, &name, "");
    st.set_game(Game::new(world));
    let _ = st.save();
    let _ = app.emit("net_match_start", serde_json::json!({}));
}

// ---------------------------------------------------------------- команды

#[tauri::command]
pub fn net_host(app: AppHandle, st: State<'_, AppState>, mode: String, port: u16) -> Result<serde_json::Value, String> {
    do_host(&app, &st, &mode, port)
}

/// Открыть игру хостом. Отдельно от команды — чтобы звать из проверочного запуска.
pub fn do_host(app: &AppHandle, st: &AppState, mode: &str, port: u16) -> Result<serde_json::Value, String> {
    let port = if port == 0 { 7777 } else { port };
    let listener = TcpListener::bind(("0.0.0.0", port)).map_err(|e| format!("не удалось открыть порт {}: {}", port, e))?;
    let name = my_handle(st);
    let stop = Arc::new(AtomicBool::new(false));
    {
        let mut net = st.net.lock().map_err(|_| "сеть занята".to_string())?;
        net.reset();
        net.role = Role::Host;
        net.my_name = name.clone();
        net.port = port;
        net.stop = stop.clone();
        net.snap = Snapshot {
            mode: if mode == "team" { "team" } else { "duel" }.to_string(),
            phase: "lobby".to_string(),
            peers: vec![PeerView { name: name.clone(), stage: 0, score: 0 }],
            ..Snapshot::default()
        };
    }
    // поток приёма подключений
    let app2 = app.clone();
    let stop2 = stop.clone();
    std::thread::spawn(move || {
        for incoming in listener.incoming() {
            if stop2.load(Ordering::Relaxed) {
                break;
            }
            let stream = match incoming {
                Ok(s) => s,
                Err(_) => continue,
            };
            let _ = stream.set_nodelay(true);
            let shared = Arc::new(Mutex::new(stream.try_clone().expect("клон потока")));
            let st = app2.state::<AppState>();
            if let Ok(mut net) = st.net.lock() {
                net.clients.push(shared);
            }
            spawn_client_reader(app2.clone(), stream, stop2.clone());
        }
    });
    Ok(serde_json::json!({ "port": port, "addrs": local_ips() }))
}

#[tauri::command]
pub fn net_join(app: AppHandle, st: State<'_, AppState>, addr: String) -> Result<(), String> {
    do_join(&app, &st, &addr)
}

/// Подключиться к хосту. Отдельно от команды — чтобы звать из проверочного запуска.
pub fn do_join(app: &AppHandle, st: &AppState, addr: &str) -> Result<(), String> {
    let addr = addr.trim();
    let addr = if addr.contains(':') { addr.to_string() } else { format!("{}:7777", addr) };
    let stream = TcpStream::connect(&addr).map_err(|e| format!("не удалось подключиться к {}: {}", addr, e))?;
    let _ = stream.set_nodelay(true);
    let name = my_handle(st);
    let shared = Arc::new(Mutex::new(stream.try_clone().map_err(|e| e.to_string())?));
    let stop = Arc::new(AtomicBool::new(false));
    {
        let mut net = st.net.lock().map_err(|_| "сеть занята".to_string())?;
        net.reset();
        net.role = Role::Client;
        net.my_name = name.clone();
        net.host_stream = Some(shared.clone());
        net.stop = stop.clone();
    }
    spawn_host_reader(app.clone(), stream, stop);
    write_line(&shared, &serde_json::json!({ "t": "join", "name": name }).to_string());
    Ok(())
}

#[tauri::command]
pub fn net_start(app: AppHandle, st: State<'_, AppState>) -> Result<(), String> {
    let seed;
    {
        let mut net = st.net.lock().map_err(|_| "сеть занята".to_string())?;
        if net.role != Role::Host {
            return Err("начать партию может только хост".into());
        }
        if net.snap.peers.len() < 2 {
            return Err("нужен хотя бы один соперник".into());
        }
        // зерно — из времени, чтобы мир был новый
        seed = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(7) ^ 0x9e37_79b9;
        net.snap.seed = seed;
        net.snap.phase = "playing".to_string();
        net.snap.winner = None;
        net.snap.captured.clear();
        for p in &mut net.snap.peers {
            p.stage = 0;
            p.score = 0;
        }
    }
    // цели берём из мира хоста (у всех одинаковый seed → те же узлы)
    let targets = {
        let trophy_world = World::new(seed, "host", "");
        let all = blackwire_core::scenarios::targets(&trophy_world);
        let want = {
            let net = st.net.lock().map_err(|_| "сеть занята".to_string())?;
            if net.snap.mode == "team" { TEAM_TARGETS } else { 1 }
        };
        all.into_iter().take(want).collect::<Vec<_>>()
    };
    {
        let mut net = st.net.lock().map_err(|_| "сеть занята".to_string())?;
        net.snap.targets = targets;
        host_recompute_and_broadcast(&app, &mut net);
    }
    start_match(&app, &st, seed);
    Ok(())
}

#[tauri::command]
pub fn net_status(st: State<'_, AppState>) -> serde_json::Value {
    let net = match st.net.lock() {
        Ok(n) => n,
        Err(_) => return serde_json::json!({ "role": "None" }),
    };
    serde_json::json!({
        "role": net.role,
        "my_name": net.my_name,
        "port": net.port,
        "error": net.error,
        "addrs": if net.role == Role::Host { local_ips() } else { Vec::new() },
        "snap": net.snap,
    })
}

#[tauri::command]
pub fn net_leave(st: State<'_, AppState>) {
    if let Ok(mut net) = st.net.lock() {
        net.reset();
    }
}

// ---------------------------------------------------------------- ход времени

/// Раз в секунду: посчитать свой прогресс по целям и отправить хосту (или
/// применить у себя, если мы хост). Зовётся из `ticker`.
pub fn pump(app: &AppHandle, st: &AppState) {
    // снять нужное под коротким замком сети
    let (role, playing, mode, targets, my_name, host_stream) = {
        let Ok(net) = st.net.lock() else { return };
        if net.role == Role::None || net.snap.phase != "playing" {
            return;
        }
        (net.role, true, net.snap.mode.clone(), net.snap.targets.clone(), net.my_name.clone(), net.host_stream.clone())
    };
    if !playing || targets.is_empty() {
        return;
    }
    // прогресс по миру (отдельный короткий замок игры)
    let (stage, new_caps): (u8, Vec<String>) = {
        let Ok(g) = st.game() else { return };
        if mode == "duel" {
            (stage_for(&g.world, &targets[0]), Vec::new())
        } else {
            let caps: Vec<String> = targets.iter().filter(|t| stage_for(&g.world, t) >= 3).cloned().collect();
            (0, caps)
        }
    };

    let mut net = match st.net.lock() {
        Ok(n) => n,
        Err(_) => return,
    };
    if mode == "duel" {
        if stage > net.sent_stage {
            net.sent_stage = stage;
            match role {
                Role::Host => {
                    if let Some(p) = net.snap.peers.iter_mut().find(|p| p.name == my_name) {
                        p.stage = stage;
                    }
                    host_recompute_and_broadcast(app, &mut net);
                }
                Role::Client => {
                    if let Some(hs) = &host_stream {
                        write_line(hs, &serde_json::json!({ "t": "prog", "stage": stage }).to_string());
                    }
                }
                Role::None => {}
            }
        }
    } else {
        for host in new_caps {
            if net.sent_caps.contains(&host) {
                continue;
            }
            net.sent_caps.insert(host.clone());
            match role {
                Role::Host => {
                    if !net.snap.captured.contains(&host) {
                        net.snap.captured.push(host.clone());
                        if let Some(p) = net.snap.peers.iter_mut().find(|p| p.name == my_name) {
                            p.score += 1;
                            p.stage = 3;
                        }
                    }
                    host_recompute_and_broadcast(app, &mut net);
                }
                Role::Client => {
                    if let Some(hs) = &host_stream {
                        write_line(hs, &serde_json::json!({ "t": "cap", "host": host }).to_string());
                    }
                }
                Role::None => {}
            }
        }
    }
}
