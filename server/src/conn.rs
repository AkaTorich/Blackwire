//! Одно подключение игры: вход (или регистрация), затем запросы `{id, cmd, args}`
//! и ответы `{id, ok}` / `{id, err}`. События сервер шлёт строками `{ev, data}`.

use std::net::TcpStream;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use blackwire_wire::{Reader, Writer};
use serde_json::{json, Value};

use crate::accounts::{self, Account};
use crate::hub::{Hub, Online};

/// Настройка TLS (без шифрования — `None`).
pub type ServerCfg = Option<Arc<blackwire_wire::TlsServerConfig>>;

static CONNS: AtomicUsize = AtomicUsize::new(0);
const MAX_CONNS: usize = 300;

fn reply(w: &Writer, id: &Value, r: Result<Value, String>) {
    let line = match r {
        Ok(v) => json!({ "id": id, "ok": v }),
        Err(e) => json!({ "id": id, "err": e }),
    };
    let _ = w.send(&line.to_string());
}

/// Вход: проверить или завести учётную запись. Возвращает имя игрока.
fn auth(hub: &Arc<Hub>, a: &Value) -> Result<String, String> {
    let proto = a.get("proto").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
    if proto != blackwire_wire::PROTOCOL {
        return Err(format!("версия игры не подходит к серверу (протокол {} ≠ {}) — обнови игру / update the game", proto, blackwire_wire::PROTOCOL));
    }
    let name = accounts::clean_name(a.get("user").and_then(|v| v.as_str()).unwrap_or(""))?;
    let pass = a.get("pass").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let register = a.get("register").and_then(|v| v.as_bool()).unwrap_or(false);
    let lang = if a.get("lang").and_then(|v| v.as_str()).unwrap_or("ru").starts_with("en") { "en" } else { "ru" };

    if register {
        accounts::check_password_rules(&pass)?;
        let account = Account::new(&pass);
        let mut st = hub.state.lock().map_err(|_| "мир недоступен".to_string())?;
        if st.accounts.contains_key(&name) {
            return Err("это имя уже занято / this name is taken".into());
        }
        if st.accounts.len() >= hub.cfg.max_accounts {
            return Err("на сервере нет мест / the server is full".into());
        }
        blackwire_core::i18n::set(lang);
        let pc_password = blackwire_core::net::names::strong_password(&mut st.game.world.rng);
        let mut ctx = st.game.world.mmo_add_player(&name, &pc_password, lang);
        blackwire_core::i18n::set(&hub.cfg.lang);
        ctx.next_session = hub.session_base_for_new(&st);
        st.players.insert(name.clone(), ctx);
        st.accounts.insert(name.clone(), account);
        drop(st);
        println!("новый игрок: {}", name);
        hub.save();
        return Ok(name);
    }

    let account = hub.state.lock().map_err(|_| "мир недоступен".to_string())?.accounts.get(&name).cloned();
    match account {
        Some(acc) if acc.check(&pass) => {
            if let Ok(mut st) = hub.state.lock() {
                if let Some(acc) = st.accounts.get_mut(&name) {
                    acc.last_seen = accounts::now();
                }
                if let Some(ctx) = st.players.get_mut(&name) {
                    ctx.lang = lang.to_string();
                }
            }
            Ok(name)
        }
        _ => {
            std::thread::sleep(std::time::Duration::from_millis(700));
            Err("неверное имя или пароль / wrong name or password".into())
        }
    }
}

/// Обслужить подключение до конца.
pub fn serve(hub: Arc<Hub>, sock: TcpStream, tls: ServerCfg) {
    if CONNS.fetch_add(1, Ordering::SeqCst) >= MAX_CONNS {
        CONNS.fetch_sub(1, Ordering::SeqCst);
        return;
    }
    let peer = sock.peer_addr().map(|a| a.to_string()).unwrap_or_default();
    match blackwire_wire::accept(sock, tls) {
        Ok((w, r)) => session(&hub, w, r, &peer),
        Err(e) => eprintln!("{}: рукопожатие не удалось: {}", peer, e),
    }
    CONNS.fetch_sub(1, Ordering::SeqCst);
}

fn session(hub: &Arc<Hub>, w: Writer, mut r: Reader, peer: &str) {
    blackwire_wire::set_idle_timeout(&r, 30);
    // первая строка — вход
    let first = match r.next_line() {
        Ok(Some(l)) => l,
        _ => return,
    };
    let v: Value = serde_json::from_str(&first).unwrap_or(Value::Null);
    let id = v.get("id").cloned().unwrap_or(Value::Null);
    if v.get("cmd").and_then(|c| c.as_str()) != Some("auth") {
        reply(&w, &id, Err("сначала вход / log in first".into()));
        return;
    }
    let args = v.get("args").cloned().unwrap_or(Value::Null);
    let me = match auth(hub, &args) {
        Ok(n) => n,
        Err(e) => {
            reply(&w, &id, Err(e));
            w.close();
            return;
        }
    };
    let conn = hub.new_conn_id();
    // тот же игрок с другого места — прежнее подключение закрываем
    let old = hub.online.lock().ok().and_then(|mut o| o.insert(me.clone(), Online { conn, writer: w.clone(), last_unread: None }));
    if let Some(old) = old {
        let _ = old.writer.send(&json!({ "ev": "kicked", "data": { "reason": "вход с другого места / logged in elsewhere" } }).to_string());
        old.writer.close();
    }
    println!("{} вошёл ({})", me, peer);
    let chat = hub.state.lock().map(|st| st.chat.iter().cloned().collect::<Vec<_>>()).unwrap_or_default();
    reply(&w, &id, Ok(json!({ "handle": me, "online": hub.online_names(), "chat": chat, "version": env!("CARGO_PKG_VERSION") })));
    hub.broadcast("presence", json!({ "online": hub.online_names(), "joined": me }));

    blackwire_wire::set_idle_timeout(&r, 120);
    loop {
        let line = match r.next_line() {
            Ok(Some(l)) => l,
            _ => break,
        };
        let v: Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(_) => continue,
        };
        let id = v.get("id").cloned().unwrap_or(Value::Null);
        let cmd = v.get("cmd").and_then(|c| c.as_str()).unwrap_or("").to_string();
        let args = v.get("args").cloned().unwrap_or_else(|| json!({}));
        let res = crate::api::call(hub, &me, &cmd, &args);
        reply(&w, &id, res);
    }

    // выход: убрать из сети, если это всё ещё наше подключение
    let still_ours = hub
        .online
        .lock()
        .map(|mut o| {
            if o.get(&me).map(|x| x.conn == conn).unwrap_or(false) {
                o.remove(&me);
                true
            } else {
                false
            }
        })
        .unwrap_or(false);
    if still_ours {
        hub.stop_scripts_of(&me);
        let _ = hub.with_player(&me, |g| {
            let ids: Vec<u32> = g.sessions.keys().copied().collect();
            for i in ids {
                g.close_session(i);
            }
        });
        if let Ok(mut st) = hub.state.lock() {
            if let Some(acc) = st.accounts.get_mut(&me) {
                acc.last_seen = accounts::now();
            }
        }
        println!("{} вышел", me);
        hub.broadcast("presence", json!({ "online": hub.online_names(), "left": me }));
    }
}
