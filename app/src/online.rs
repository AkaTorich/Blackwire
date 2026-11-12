//! Онлайн: игра подключается к серверу общего мира. Окна остаются своими, а
//! всё, что касается мира (терминал, файлы, почта, карта, браузер, скрипты),
//! уходит на сервер запросом `{id, cmd, args}`; сервер отвечает и сам шлёт
//! события (`world_tick`, `notify`, `script_out`, `chat` …) — их мы раздаём
//! окнам как обычные события Tauri.
//!
//! Своя одиночная игра при этом не трогается: в онлайне локальное сохранение
//! не пишется, а настройки вида живут в `online-settings.json`.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::time::Duration;

use blackwire_core::world::{Settings, World};
use blackwire_core::{Game, ShellResult, UiAction};
use blackwire_wire::{Reader, Writer};
use serde_json::{json, Value};
use tauri::{AppHandle, Emitter, Manager, State};

use crate::state::AppState;

/// Подключение к серверу.
pub struct OnlineConn {
    writer: Writer,
    pending: Mutex<HashMap<u64, mpsc::Sender<Result<Value, String>>>>,
    next: AtomicU64,
    alive: AtomicBool,
    pub handle: String,
    pub addr: String,
}

impl OnlineConn {
    /// Запрос и ожидание ответа.
    pub fn request(&self, cmd: &str, args: Value, timeout: Duration) -> Result<Value, String> {
        if !self.alive.load(Ordering::Relaxed) {
            return Err(tr!("нет связи с сервером", "no connection to the server").into());
        }
        let id = self.next.fetch_add(1, Ordering::Relaxed);
        let (tx, rx) = mpsc::channel();
        if let Ok(mut p) = self.pending.lock() {
            p.insert(id, tx);
        }
        let line = json!({ "id": id, "cmd": cmd, "args": args }).to_string();
        if let Err(e) = self.writer.send(&line) {
            if let Ok(mut p) = self.pending.lock() {
                p.remove(&id);
            }
            return Err(trf!("связь с сервером: {}", "server connection: {}", e));
        }
        match rx.recv_timeout(timeout) {
            Ok(r) => r,
            Err(_) => {
                if let Ok(mut p) = self.pending.lock() {
                    p.remove(&id);
                }
                Err(tr!("сервер не ответил", "the server did not respond").into())
            }
        }
    }

    fn fail_all(&self, why: &str) {
        self.alive.store(false, Ordering::Relaxed);
        if let Ok(mut p) = self.pending.lock() {
            for (_, tx) in p.drain() {
                let _ = tx.send(Err(why.to_string()));
            }
        }
    }
}

/// Текущее подключение (если игра в онлайне).
pub fn conn(st: &AppState) -> Option<Arc<OnlineConn>> {
    st.online.lock().ok().and_then(|o| o.clone())
}

pub fn is_online() -> bool {
    crate::state::ONLINE.load(Ordering::Relaxed)
}

/// Настройки вида в онлайне (своё сохранение не трогаем).
fn settings_path(st: &AppState) -> std::path::PathBuf {
    st.prefs_path.with_file_name("online-settings.json")
}

pub fn save_settings(st: &AppState) -> Result<(), String> {
    let s = st.game()?.world.settings.clone();
    std::fs::write(settings_path(st), serde_json::to_string(&s).unwrap_or_default()).map_err(|e| e.to_string())
}

// ---------------------------------------------------------------- события сервера

fn on_event(app: &AppHandle, ev: &str, data: Value) {
    let st = app.state::<AppState>();
    match ev {
        "open_app" => {
            let id = data.get("app").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let arg = data.get("arg").and_then(|v| v.as_str()).map(|s| s.to_string());
            if !id.is_empty() {
                let _ = crate::windows::create_app_window(app, &st, &id, arg.as_deref(), None);
            }
        }
        "mail_new" => {
            let _ = app.emit("mail_new", data);
            let _ = app.emit("notify", json!({ "title": tr!("Почта", "Mail"), "text": tr!("Новое письмо", "New message") }));
        }
        "kicked" => {
            let _ = app.emit("notify", json!({ "title": tr!("Онлайн", "Online"), "text": tr!("Вход под этим именем выполнен с другого места", "This name logged in from elsewhere") }));
        }
        _ => {
            let _ = app.emit(ev, data);
        }
    }
}

fn spawn_reader(app: AppHandle, conn: Arc<OnlineConn>, mut reader: Reader) {
    std::thread::Builder::new()
        .name("online-reader".into())
        .spawn(move || {
            loop {
                let line = match reader.next_line() {
                    Ok(Some(l)) => l,
                    _ => break,
                };
                let v: Value = match serde_json::from_str(&line) {
                    Ok(v) => v,
                    Err(_) => continue,
                };
                if let Some(id) = v.get("id").and_then(|x| x.as_u64()) {
                    let tx = conn.pending.lock().ok().and_then(|mut p| p.remove(&id));
                    if let Some(tx) = tx {
                        let r = match v.get("err") {
                            Some(e) if !e.is_null() => Err(e.as_str().unwrap_or("error").to_string()),
                            _ => Ok(v.get("ok").cloned().unwrap_or(Value::Null)),
                        };
                        let _ = tx.send(r);
                    }
                } else if let Some(ev) = v.get("ev").and_then(|x| x.as_str()) {
                    on_event(&app, ev, v.get("data").cloned().unwrap_or(Value::Null));
                }
            }
            let was_alive = conn.alive.load(Ordering::Relaxed);
            conn.fail_all(tr!("связь с сервером потеряна", "connection to the server lost"));
            if was_alive {
                let _ = app.emit("online_lost", json!({}));
                let _ = app.emit("notify", json!({ "title": tr!("Онлайн", "Online"), "text": tr!("Связь с сервером потеряна. Перезапусти игру, чтобы войти снова.", "Connection to the server lost. Restart the game to log in again.") }));
            }
        })
        .expect("поток онлайна");
}

fn spawn_pinger(conn: Arc<OnlineConn>) {
    let _ = std::thread::Builder::new().name("online-ping".into()).spawn(move || loop {
        std::thread::sleep(Duration::from_secs(30));
        if !conn.alive.load(Ordering::Relaxed) {
            break;
        }
        let _ = conn.request("ping", json!({}), Duration::from_secs(20));
    });
}

// ---------------------------------------------------------------- команды

/// Подключиться и войти (или зарегистрироваться). После этого окна игры
/// работают с миром сервера.
#[tauri::command]
pub async fn online_login(app: AppHandle, addr: String, user: String, pass: String, register: Option<bool>) -> Result<Value, String> {
    tauri::async_runtime::spawn_blocking(move || login_blocking(&app, &addr, &user, &pass, register.unwrap_or(false))).await.map_err(|e| e.to_string())?
}

/// Вход без окон (его зовёт и проверочный ключ `--online`).
pub fn login_blocking(app: &AppHandle, addr: &str, user: &str, pass: &str, register: bool) -> Result<Value, String> {
    let st = app.state::<AppState>();
    let lang = blackwire_core::i18n::code().to_string();
    let addr2 = addr.to_string();
    let addr = addr.to_string();
    let (writer, reader, info) = (move || -> Result<(Writer, Reader, Value), String> {
        let (w, mut r) = blackwire_wire::connect(&addr2).map_err(|e| trf!("не удалось подключиться к {}: {}", "failed to connect to {}: {}", addr2, e))?;
        let auth = json!({ "id": 0, "cmd": "auth", "args": {
            "user": user, "pass": pass, "register": register, "lang": lang, "proto": blackwire_wire::PROTOCOL } });
        w.send(&auth.to_string()).map_err(|e| e.to_string())?;
        loop {
            let line = r.next_line().map_err(|e| e.to_string())?.ok_or_else(|| tr!("сервер закрыл соединение", "the server closed the connection").to_string())?;
            let v: Value = serde_json::from_str(&line).unwrap_or(Value::Null);
            if v.get("id").and_then(|x| x.as_u64()) != Some(0) {
                continue;
            }
            if let Some(e) = v.get("err").and_then(|e| e.as_str()) {
                w.close();
                return Err(e.to_string());
            }
            return Ok((w, r, v.get("ok").cloned().unwrap_or(Value::Null)));
        }
    })()?;

    let handle = info.get("handle").and_then(|h| h.as_str()).unwrap_or("player").to_string();
    let conn = Arc::new(OnlineConn { writer, pending: Mutex::new(HashMap::new()), next: AtomicU64::new(1), alive: AtomicBool::new(true), handle: handle.clone(), addr: addr.clone() });
    // прежнее подключение — закрыть
    if let Some(old) = st.online.lock().ok().and_then(|mut o| o.replace(conn.clone())) {
        old.fail_all("closed");
        old.writer.close();
    }
    crate::state::ONLINE.store(true, Ordering::Relaxed);
    // местная «оболочка» для окон: настройки вида и учёт окон; мир — на сервере
    let mut world = World::new(1, &handle, "");
    if let Ok(text) = std::fs::read_to_string(settings_path(&st)) {
        if let Ok(s) = serde_json::from_str::<Settings>(&text) {
            world.settings = s;
        }
    }
    st.set_game(Game::new(world));
    spawn_reader(app.clone(), conn.clone(), reader);
    spawn_pinger(conn);
    Ok(info)
}

#[tauri::command]
pub fn online_status(st: State<'_, AppState>) -> Value {
    match conn(&st) {
        Some(c) => json!({ "online": c.alive.load(Ordering::Relaxed), "handle": c.handle, "addr": c.addr }),
        None => json!({ "online": false }),
    }
}

/// Запрос окна к миру сервера.
#[tauri::command]
pub async fn online_call(app: AppHandle, st: State<'_, AppState>, cmd: String, args: Value) -> Result<Value, String> {
    let c = conn(&st).ok_or_else(|| tr!("нет связи с сервером", "no connection to the server").to_string())?;
    let mut args = args;
    // свой сценарий лежит в файле у игрока — отправляем его целиком
    if cmd == "scenario_start" {
        let name = args.get("name").and_then(|n| n.as_str()).unwrap_or("").to_string();
        let sc = st.load_scenario(&name)?;
        args = json!({ "scenario": sc });
    }
    let cmd2 = cmd.clone();
    let c2 = c.clone();
    let result = tauri::async_runtime::spawn_blocking(move || c2.request(&cmd2, args, Duration::from_secs(60))).await.map_err(|e| e.to_string())??;
    if cmd == "shell_exec" || cmd == "script_run" {
        // окна открываются здесь, у игрока
        if let Ok(mut r) = serde_json::from_value::<ShellResult>(result.clone()) {
            crate::cmd_shell::apply_actions(&app, &st, &r.actions);
            r.actions.retain(|a| !matches!(a, UiAction::OpenApp { .. } | UiAction::QuitGame));
            return serde_json::to_value(r).map_err(|e| e.to_string());
        }
    }
    Ok(result)
}
