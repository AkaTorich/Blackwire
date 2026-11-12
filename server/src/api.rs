//! Команды игры от лица игрока. Имена и параметры — те же, что у окон игры
//! (`app/ui/js/bridge.js`), поэтому окна в онлайне работают без переделки.

use std::sync::Arc;

use blackwire_core::{OutLine, UiAction};
use serde_json::{json, Value};

use crate::hub::Hub;
use crate::scripts::{self, take_pending};

fn s(a: &Value, k: &str) -> String {
    a.get(k).and_then(|v| v.as_str()).unwrap_or("").to_string()
}

fn u(a: &Value, k: &str) -> u32 {
    a.get(k).and_then(|v| v.as_u64()).unwrap_or(0) as u32
}

fn sid(a: &Value) -> Option<u32> {
    a.get("sessionId").and_then(|v| v.as_u64()).map(|v| v as u32)
}

fn to<T: serde::Serialize>(v: T) -> Result<Value, String> {
    serde_json::to_value(v).map_err(|e| e.to_string())
}

fn shell_quote(s: &str) -> String {
    if s.chars().all(|c| c.is_ascii_alphanumeric() || "/._-~:@".contains(c)) {
        s.to_string()
    } else {
        format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
    }
}

/// Строка терминала: ядро + разбор `adder --check` + запуск скриптов.
fn shell_exec(hub: &Arc<Hub>, me: &str, session: u32, line: &str) -> Result<Value, String> {
    let (mut result, pending) = hub.with_player(me, |g| {
        let r = g.exec(session, line);
        (r, take_pending(g))
    })?;
    let mut checks = Vec::new();
    result.actions.retain(|a| {
        if let UiAction::CheckScript { path, src } = a {
            checks.push((path.clone(), src.clone()));
            false
        } else {
            true
        }
    });
    for (path, src) in checks {
        let diags = blackwire_lang::check(&src);
        if diags.is_empty() {
            result.lines.push(OutLine::success(format!("{}: ok", path)));
        } else {
            for d in diags {
                result.lines.push(OutLine::error(format!("{}:{}:{}: {}", path, d.line, d.col, d.message)));
            }
        }
    }
    scripts::deliver(hub, me, pending);
    to(result)
}

pub fn call(hub: &Arc<Hub>, me: &str, cmd: &str, a: &Value) -> Result<Value, String> {
    match cmd {
        "ping" => Ok(json!("pong")),
        // ---- терминал и скрипты
        "shell_open" => to(hub.with_player(me, |g| g.open_session())?),
        "shell_close" => {
            let id = u(a, "sessionId");
            hub.with_player(me, |g| g.close_session(id))?;
            Ok(Value::Null)
        }
        "shell_exec" => shell_exec(hub, me, u(a, "sessionId"), &s(a, "line")),
        "shell_complete" => {
            let (id, line) = (u(a, "sessionId"), s(a, "line"));
            to(hub.with_player(me, |g| g.complete(id, &line))?)
        }
        "shell_prompt" => {
            let id = u(a, "sessionId");
            to(hub.with_player(me, |g| g.prompt(id))?)
        }
        "shell_history" => {
            let id = u(a, "sessionId");
            to(hub.with_player(me, |g| g.history(id))?)
        }
        "shell_doc" => to(hub.with_player(me, |g| g.shell_doc())?),
        "script_run" => {
            let mut line = format!("adder {}", shell_quote(&s(a, "path")));
            if let Some(args) = a.get("args").and_then(|v| v.as_array()) {
                for x in args {
                    line.push(' ');
                    line.push_str(&shell_quote(x.as_str().unwrap_or("")));
                }
            }
            shell_exec(hub, me, u(a, "sessionId"), &line)
        }
        "script_input" => {
            scripts::send_input(hub, me, u(a, "pid"), s(a, "text"))?;
            Ok(Value::Null)
        }
        "script_kill" | "sys_kill" => {
            let pid = u(a, "pid");
            scripts::request_stop(hub, me, pid);
            let r = hub.with_player(me, |g| g.kill(pid))?;
            if cmd == "sys_kill" {
                r?;
            }
            Ok(Value::Null)
        }
        // ---- файлы
        "fs_list" => {
            let (id, p) = (sid(a), s(a, "path"));
            to(hub.with_player(me, |g| g.fs_list(id, &p))??)
        }
        "fs_read" => {
            let (id, p) = (sid(a), s(a, "path"));
            to(hub.with_player(me, |g| g.fs_read(id, &p))??)
        }
        "fs_write" | "fs_mkdir" | "fs_delete" | "fs_rename" | "fs_copy" => {
            let id = sid(a);
            let (p, text, from, dest) = (s(a, "path"), s(a, "text"), s(a, "from"), s(a, "to"));
            hub.with_player(me, |g| match cmd {
                "fs_write" => g.fs_write(id, &p, &text),
                "fs_mkdir" => g.fs_mkdir(id, &p),
                "fs_delete" => g.fs_delete(id, &p),
                "fs_rename" => g.fs_rename(id, &from, &dest),
                _ => g.fs_copy(id, &from, &dest),
            })??;
            let changed = if p.is_empty() { dest } else { p };
            hub.send(me, "fs_changed", json!({ "path": changed }));
            Ok(Value::Null)
        }
        // ---- система, сеть, почта
        "sys_status" => to(hub.with_player(me, |g| g.status())?),
        "net_map" => to(hub.with_player(me, |g| g.net_map())?),
        "web_get" => {
            let url = s(a, "url");
            to(hub.with_player(me, |g| g.http(&url))?)
        }
        "mail_list" => {
            let folder = a.get("folder").and_then(|v| v.as_str()).unwrap_or("inbox").to_string();
            to(hub.with_player(me, |g| g.mail_list(&folder))?)
        }
        "mail_read" => {
            let id = u(a, "id");
            let m = hub.with_player(me, |g| g.mail_read(id))??;
            hub.send(me, "mail_changed", json!({}));
            to(m)
        }
        "mail_send" => {
            let (t, subj, body) = (s(a, "to"), s(a, "subject"), s(a, "body"));
            hub.with_player(me, |g| g.mail_send(&t, &subj, &body))??;
            hub.send(me, "mail_changed", json!({}));
            Ok(Value::Null)
        }
        "mail_delete" => {
            let id = u(a, "id");
            hub.with_player(me, |g| g.mail_delete(id))??;
            hub.send(me, "mail_changed", json!({}));
            Ok(Value::Null)
        }
        "mail_unread_count" => to(hub.with_player(me, |g| g.mail_unread_count() as u32)?),
        "ending_summary" => to(hub.with_player(me, |g| g.ending_summary())?),
        // ---- свой сценарий (файл сценария присылает игра)
        "scenario_targets" => to(hub.with_player(me, |g| blackwire_core::scenarios::targets(&g.world))?),
        "scenario_start" => {
            let sc: blackwire_core::scenarios::Scenario = serde_json::from_value(a.get("scenario").cloned().unwrap_or(Value::Null)).map_err(|e| e.to_string())?;
            hub.with_player(me, |g| blackwire_core::scenarios::start(&mut g.world, sc))?;
            Ok(Value::Null)
        }
        "scenario_stop" => {
            hub.with_player(me, |g| blackwire_core::scenarios::stop(&mut g.world))?;
            Ok(Value::Null)
        }
        "scenario_active" => to(hub.with_player(me, |g| g.world.scenarios.active.clone())?),
        // ---- плагины сервера (ставит владелец сервера)
        "plugins_list" => to(hub.with_player(me, |g| g.plugin_list())?),
        "plugins_run" => {
            let command = s(a, "command");
            let r = hub.with_player(me, |g| {
                let sid = g.open_session();
                let r = g.plugin_command(sid, &command);
                g.close_session(sid);
                r.map(|r| r.lines)
            })?;
            to(r.ok_or_else(|| "команда плагина не найдена / plugin command not found".to_string())?)
        }
        "plugins_panel" => {
            let id = s(a, "id");
            hub.with_player(me, |g| -> Result<Value, String> {
                let (owner, _) = id.split_once(':').ok_or("invalid window id")?;
                if g.world.plugins.disabled.contains(owner) || g.plugins.loaded.get(owner).map(|p| p.error.is_some()).unwrap_or(true) {
                    return Err("плагин не работает / plugin is not working".into());
                }
                g.world.plugins.windows.get(&id).cloned().ok_or_else(|| "окно не найдено / window not found".into())
            })?
        }
        "plugins_ui" => {
            let (id, action) = (s(a, "id"), s(a, "action"));
            let values = a.get("values").cloned().unwrap_or(Value::Null);
            to(hub.with_player(me, |g| g.plugin_ui(&id, &action, values))??)
        }
        // ---- онлайн
        "who" => Ok(json!(hub.online_names())),
        "chat_send" => {
            let text: String = s(a, "text").chars().filter(|c| !c.is_control()).take(400).collect();
            if text.trim().is_empty() {
                return Err("пустое сообщение / empty message".into());
            }
            hub.chat(me, text.trim());
            Ok(Value::Null)
        }
        "chat_history" => {
            let st = hub.state.lock().map_err(|_| "мир недоступен".to_string())?;
            to(st.chat.iter().cloned().collect::<Vec<_>>())
        }
        _ => Err(format!("команда недоступна в онлайне: {} / not available online", cmd)),
    }
}
