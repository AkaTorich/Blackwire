//! Скрипты Adder игроков на сервере: каждый — свой поток. Игровые вызовы идут
//! в общий мир от лица владельца, печать и ввод — событиями ему в игру.

use std::cell::Cell;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};
use std::time::Duration;

use blackwire_core::session::PendingScript;
use blackwire_core::{Game, UiAction};
use blackwire_lang::{Host, LangError, Value};
use serde_json::json;

use crate::hub::{Hub, ScriptHandle};

struct ServerHost {
    hub: Arc<Hub>,
    handle: String,
    pid: u32,
    session: u32,
    stop: Arc<AtomicBool>,
    input_rx: mpsc::Receiver<String>,
    steps: Cell<u32>,
}

impl Host for ServerHost {
    fn print(&mut self, text: &str) {
        self.hub.send(&self.handle, "script_out", json!({ "pid": self.pid, "text": text }));
    }

    fn input(&mut self, prompt: &str) -> Result<String, LangError> {
        self.hub.send(&self.handle, "script_input_wanted", json!({ "pid": self.pid, "prompt": prompt }));
        loop {
            match self.input_rx.recv_timeout(Duration::from_millis(100)) {
                Ok(s) => return Ok(s),
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    if self.should_stop() {
                        return Err(LangError::runtime(0, "interrupted"));
                    }
                }
                Err(_) => return Err(LangError::runtime(0, "interrupted")),
            }
        }
    }

    fn call(&mut self, name: &str, args: Vec<Value>) -> Result<Value, LangError> {
        let json_args: Vec<serde_json::Value> = args.iter().map(|v| v.to_json()).collect();
        let (session, pid) = (self.session, self.pid);
        let out = self
            .hub
            .with_player(&self.handle, |g| {
                let r = blackwire_core::script_host::call(g, session, pid, name, json_args);
                (r, take_pending(g))
            })
            .map_err(|e| LangError::runtime(0, &e))?;
        let (result, pending) = out;
        deliver(&self.hub, &self.handle, pending);
        match result {
            Ok(v) => Ok(Value::from_json(&v)),
            Err(e) => Err(LangError::runtime(0, &e)),
        }
    }

    fn should_stop(&self) -> bool {
        if self.stop.load(Ordering::Relaxed) {
            return true;
        }
        let n = self.steps.get() + 1;
        self.steps.set(n);
        if n >= 10 {
            self.steps.set(0);
            let pid = self.pid;
            let killed = self.hub.with_player(&self.handle, |g| g.is_killed(pid)).unwrap_or(true);
            if killed {
                self.stop.store(true, Ordering::Relaxed);
                return true;
            }
        }
        false
    }

    fn sleep(&mut self, ms: u64) {
        let mut left = ms;
        while left > 0 {
            let step = left.min(50);
            std::thread::sleep(Duration::from_millis(step));
            left -= step;
            if self.should_stop() {
                return;
            }
        }
    }
}

/// Отложенное ядром: скрипты к запуску и действия для окон игрока.
pub struct Pending {
    pub scripts: Vec<PendingScript>,
    pub ui: Vec<UiAction>,
}

pub fn take_pending(g: &mut Game) -> Pending {
    Pending { scripts: g.take_pending_scripts(), ui: g.take_pending_ui() }
}

/// Запустить скрипты и разослать действия окон игроку.
pub fn deliver(hub: &Arc<Hub>, handle: &str, p: Pending) {
    for s in p.scripts {
        spawn(hub.clone(), handle.to_string(), s);
    }
    for a in p.ui {
        match a {
            UiAction::OpenApp { app, arg } => hub.send(handle, "open_app", json!({ "app": app, "arg": arg })),
            UiAction::Notify { title, text } => hub.send(handle, "notify", json!({ "title": title, "text": text })),
            _ => {}
        }
    }
}

/// Запустить скрипт в своём потоке.
pub fn spawn(hub: Arc<Hub>, handle: String, p: PendingScript) {
    if hub.scripts_of(&handle) >= hub.cfg.max_scripts {
        let pid = p.pid;
        let _ = hub.with_player(&handle, |g| {
            let _ = g.kill(pid);
            g.script_finished(pid);
        });
        hub.send(&handle, "script_done", json!({ "pid": pid, "ok": false, "line": 0,
            "error": format!("слишком много скриптов сразу (не больше {}) / too many scripts at once", hub.cfg.max_scripts) }));
        return;
    }
    let stop = Arc::new(AtomicBool::new(false));
    let (tx, rx) = mpsc::channel::<String>();
    if let Ok(mut s) = hub.scripts.lock() {
        s.insert((handle.clone(), p.pid), ScriptHandle { stop: stop.clone(), input_tx: tx });
    }
    let name = format!("adder-{}-{}", handle, p.pid);
    let _ = std::thread::Builder::new().name(name).stack_size(64 * 1024 * 1024).spawn(move || {
        let mut host = ServerHost { hub: hub.clone(), handle: handle.clone(), pid: p.pid, session: p.session, stop, input_rx: rx, steps: Cell::new(0) };
        let result = blackwire_lang::run(&p.src, &mut host, p.args.clone());
        let (ok, error, line) = match &result {
            Ok(_) => (true, None, 0),
            Err(e) => (false, Some(e.to_string()), e.line),
        };
        hub.send(&handle, "script_done", json!({ "pid": p.pid, "ok": ok, "error": error, "line": line }));
        if let Ok(mut s) = hub.scripts.lock() {
            s.remove(&(handle.clone(), p.pid));
        }
        let pid = p.pid;
        let _ = hub.with_player(&handle, |g| g.script_finished(pid));
    });
}

pub fn request_stop(hub: &Hub, handle: &str, pid: u32) {
    if let Ok(s) = hub.scripts.lock() {
        if let Some(h) = s.get(&(handle.to_string(), pid)) {
            h.stop.store(true, Ordering::Relaxed);
        }
    }
}

pub fn send_input(hub: &Hub, handle: &str, pid: u32, text: String) -> Result<(), String> {
    let s = hub.scripts.lock().map_err(|_| "scripts unavailable".to_string())?;
    let h = s.get(&(handle.to_string(), pid)).ok_or_else(|| "скрипт уже завершён / script already finished".to_string())?;
    h.input_tx.send(text).map_err(|_| "скрипт не ждёт ввода / script is not waiting for input".to_string())
}
