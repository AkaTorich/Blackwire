//! Потоки скриптов Adder. Каждый запуск — свой поток; скрипт говорит с миром
//! через `Host`: печать и ввод идут событиями в окна, игровые функции — через
//! `blackwire_core::script_host::call` под замком игры.

use std::cell::Cell;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};
use std::time::Duration;

use blackwire_core::session::PendingScript;
use blackwire_lang::{Host, LangError, Value};
use tauri::{AppHandle, Emitter, Manager};

use crate::state::{AppState, ScriptHandle};

struct TauriHost {
    app: AppHandle,
    pid: u32,
    session: u32,
    stop: Arc<AtomicBool>,
    input_rx: mpsc::Receiver<String>,
    steps_since_check: Cell<u32>,
}

impl TauriHost {
    fn killed_in_game(&self) -> bool {
        let st = self.app.state::<AppState>();
        st.game().map(|g| g.is_killed(self.pid)).unwrap_or(true)
    }
}

impl Host for TauriHost {
    fn print(&mut self, text: &str) {
        let _ = self.app.emit("script_out", serde_json::json!({ "pid": self.pid, "text": text }));
    }

    fn input(&mut self, prompt: &str) -> Result<String, LangError> {
        let _ = self.app.emit("script_input_wanted", serde_json::json!({ "pid": self.pid, "prompt": prompt }));
        loop {
            match self.input_rx.recv_timeout(Duration::from_millis(100)) {
                Ok(s) => return Ok(s),
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    if self.should_stop() {
                        return Err(LangError::runtime(0, "прервано"));
                    }
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => return Err(LangError::runtime(0, "прервано")),
            }
        }
    }

    fn call(&mut self, name: &str, args: Vec<Value>) -> Result<Value, LangError> {
        let json_args: Vec<serde_json::Value> = args.iter().map(|v| v.to_json()).collect();
        let st = self.app.state::<AppState>();
        let result = {
            let mut g = st.game().map_err(|e| LangError::runtime(0, &e))?;
            blackwire_core::script_host::call(&mut g, self.session, self.pid, name, json_args)
        };
        // sys.open / sys.notify оставляют действия для оболочки
        crate::cmd_shell::flush_pending(&self.app, &st);
        match result {
            Ok(v) => Ok(Value::from_json(&v)),
            Err(e) => Err(LangError::runtime(0, &e)),
        }
    }

    fn should_stop(&self) -> bool {
        if self.stop.load(Ordering::Relaxed) {
            return true;
        }
        // заглядывать в игру на каждом вызове дорого — раз в несколько проверок
        let n = self.steps_since_check.get() + 1;
        self.steps_since_check.set(n);
        if n >= 10 {
            self.steps_since_check.set(0);
            if self.killed_in_game() {
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

/// Запустить скрипт из очереди ядра в своём потоке.
pub fn spawn(app: AppHandle, p: PendingScript) {
    let stop = Arc::new(AtomicBool::new(false));
    let (tx, rx) = mpsc::channel::<String>();
    {
        let st = app.state::<AppState>();
        let guard = st.scripts.lock();
        if let Ok(mut s) = guard {
            s.insert(p.pid, ScriptHandle { stop: stop.clone(), input_tx: tx });
        }
    }
    let name = format!("adder-{}", p.pid);
    // стек побольше: глубокая рекурсия в скрипте не должна ронять игру
    let _ = std::thread::Builder::new().name(name).stack_size(64 * 1024 * 1024).spawn(move || {
        let mut host = TauriHost { app: app.clone(), pid: p.pid, session: p.session, stop, input_rx: rx, steps_since_check: Cell::new(0) };
        let result = blackwire_lang::run(&p.src, &mut host, p.args.clone());
        let (ok, error, line) = match &result {
            Ok(_) => (true, None, 0),
            Err(e) => (false, Some(e.to_string()), e.line),
        };
        let _ = app.emit("script_done", serde_json::json!({ "pid": p.pid, "ok": ok, "error": error, "line": line }));
        let st = app.state::<AppState>();
        let guard = st.scripts.lock();
        if let Ok(mut s) = guard {
            s.remove(&p.pid);
        }
        let game = st.game();
        if let Ok(mut g) = game {
            g.script_finished(p.pid);
        }
    });
}

pub fn request_stop(st: &AppState, pid: u32) {
    if let Ok(s) = st.scripts.lock() {
        if let Some(h) = s.get(&pid) {
            h.stop.store(true, Ordering::Relaxed);
        }
    }
}
