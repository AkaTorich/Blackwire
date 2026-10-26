//! Монитор, карта сети, веб-страницы игровой сети, почта.

use blackwire_core::net::MailMessage;
use tauri::{AppHandle, Emitter, State};

use crate::state::AppState;

#[tauri::command]
pub fn sys_status(st: State<AppState>) -> Result<blackwire_core::sysinfo::Status, String> {
    Ok(st.game()?.status())
}

#[tauri::command]
pub fn sys_kill(st: State<AppState>, pid: u32) -> Result<(), String> {
    crate::scripts::request_stop(&st, pid);
    st.game()?.kill(pid)
}

#[tauri::command]
pub fn net_map(st: State<AppState>) -> Result<blackwire_core::game::NetMap, String> {
    Ok(st.game()?.net_map())
}

#[tauri::command]
pub fn web_get(st: State<AppState>, url: String) -> Result<blackwire_core::world::HttpPage, String> {
    Ok(st.game()?.http(&url))
}

#[tauri::command]
pub fn mail_list(st: State<AppState>, folder: Option<String>) -> Result<Vec<MailMessage>, String> {
    Ok(st.game()?.mail_list(folder.as_deref().unwrap_or("inbox")))
}

#[tauri::command]
pub fn mail_read(app: AppHandle, st: State<AppState>, id: u32) -> Result<MailMessage, String> {
    let message = st.game()?.mail_read(id)?;
    let _ = app.emit("mail_changed", serde_json::json!({}));
    Ok(message)
}

#[tauri::command]
pub fn mail_send(app: AppHandle, st: State<AppState>, to: String, subject: String, body: String) -> Result<(), String> {
    st.game()?.mail_send(&to, &subject, &body)?;
    let _ = app.emit("mail_changed", serde_json::json!({}));
    Ok(())
}

#[tauri::command]
pub fn mail_delete(app: AppHandle, st: State<AppState>, id: u32) -> Result<(), String> {
    st.game()?.mail_delete(id)?;
    let _ = app.emit("mail_changed", serde_json::json!({}));
    Ok(())
}

#[tauri::command]
pub fn mail_unread_count(st: State<AppState>) -> Result<u32, String> {
    Ok(st.game()?.mail_unread_count() as u32)
}

#[tauri::command]
pub fn ending_summary(st: State<AppState>) -> Result<blackwire_core::ending::EndingSummary, String> {
    Ok(st.game()?.ending_summary())
}
