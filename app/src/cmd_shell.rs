//! Сеансы терминала и скрипты: команды оболочки идут в ядро, действия для окон
//! (`UiAction`) исполняются здесь, запуски скриптов уходят в потоки `scripts.rs`.

use blackwire_core::{Prompt, ShellResult, UiAction};
use tauri::{AppHandle, Emitter, State};

use crate::scripts;
use crate::state::AppState;
use crate::windows;

#[tauri::command]
pub fn shell_open(st: State<'_, AppState>) -> Result<u32, String> {
    Ok(st.game()?.open_session())
}

#[tauri::command]
pub fn shell_close(st: State<'_, AppState>, session_id: u32) -> Result<(), String> {
    st.game()?.close_session(session_id);
    Ok(())
}

/// Исполнить действия ядра, которые требуют оболочки Tauri (открыть окно, уведомить).
/// Остальные действия остаются в ответе — их разбирает терминал.
pub fn apply_actions(app: &AppHandle, st: &AppState, actions: &[UiAction]) {
    for a in actions {
        match a {
            UiAction::OpenApp { app: id, arg } => {
                let _ = windows::create_app_window(app, st, id, arg.as_deref(), None);
            }
            UiAction::Notify { title, text } => {
                let _ = app.emit("notify", serde_json::json!({ "title": title, "text": text }));
            }
            UiAction::QuitGame => {
                if let Err(error) = crate::cmd_game::save_and_quit(app, st) {
                    let _ = app.emit("notify", serde_json::json!({ "title": "Выход отменён", "text": error }));
                }
            }
            _ => {}
        }
    }
}

/// Запустить все скрипты, которые ядро поставило в очередь, и забрать отложенные действия.
pub fn flush_pending(app: &AppHandle, st: &AppState) {
    let (pending, ui) = match st.game() {
        Ok(mut g) => (g.take_pending_scripts(), g.take_pending_ui()),
        Err(_) => return,
    };
    for p in pending {
        scripts::spawn(app.clone(), p);
    }
    apply_actions(app, st, &ui);
}

#[tauri::command]
pub async fn shell_exec(app: AppHandle, st: State<'_, AppState>, session_id: u32, line: String) -> Result<ShellResult, String> {
    let mut result = { st.game()?.exec(session_id, &line) };
    // `adder --check` — разбор делает язык, которого у ядра нет
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
            result.lines.push(blackwire_core::OutLine::success(format!("{path}: ошибок нет")));
        } else {
            for d in diags {
                result.lines.push(blackwire_core::OutLine::error(format!("{path}:{}:{}: {}", d.line, d.col, d.message)));
            }
        }
    }
    apply_actions(&app, &st, &result.actions);
    // окна уже открыты здесь — терминалу их открывать не надо
    result.actions.retain(|a| !matches!(a, UiAction::OpenApp { .. } | UiAction::QuitGame));
    flush_pending(&app, &st);
    Ok(result)
}

#[tauri::command]
pub fn shell_complete(st: State<'_, AppState>, session_id: u32, line: String) -> Result<Vec<String>, String> {
    Ok(st.game()?.complete(session_id, &line))
}

#[tauri::command]
pub fn shell_prompt(st: State<'_, AppState>, session_id: u32) -> Result<Prompt, String> {
    Ok(st.game()?.prompt(session_id))
}

#[tauri::command]
pub fn shell_history(st: State<'_, AppState>, session_id: u32) -> Result<Vec<String>, String> {
    Ok(st.game()?.history(session_id))
}

/// Запуск скрипта из редактора — та же команда `adder`, что и в терминале.
#[tauri::command]
pub async fn script_run(app: AppHandle, st: State<'_, AppState>, session_id: u32, path: String, args: Option<Vec<String>>) -> Result<ShellResult, String> {
    let mut line = format!("adder {}", shell_quote(&path));
    for a in args.unwrap_or_default() {
        line.push(' ');
        line.push_str(&shell_quote(&a));
    }
    shell_exec(app, st, session_id, line).await
}

fn shell_quote(s: &str) -> String {
    if s.chars().all(|c| c.is_ascii_alphanumeric() || "/._-~:@".contains(c)) {
        s.to_string()
    } else {
        format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
    }
}

#[tauri::command]
pub fn script_input(st: State<'_, AppState>, pid: u32, text: String) -> Result<(), String> {
    let scripts = st.scripts.lock().map_err(|_| "скрипты недоступны".to_string())?;
    let h = scripts.get(&pid).ok_or_else(|| "скрипт уже завершён".to_string())?;
    h.input_tx.send(text).map_err(|_| "скрипт не ждёт ввода".to_string())
}

#[tauri::command]
pub fn script_kill(st: State<'_, AppState>, pid: u32) -> Result<(), String> {
    scripts::request_stop(&st, pid);
    if let Ok(mut g) = st.game() {
        let _ = g.kill(pid);
    }
    Ok(())
}
