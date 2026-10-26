//! Файлы для окон (проводник, редактор) и справка по языку и командам.
//! `session_id` = null — своя машина; иначе машина, на которой сейчас сеанс терминала.

use blackwire_core::FsEntry;
use blackwire_lang::Diagnostic;
use tauri::{AppHandle, Emitter, State};

use crate::state::AppState;

const ADDER_DOC: &str = include_str!("../../docs/ADDER.md");

fn changed(app: &AppHandle, path: &str) {
    let _ = app.emit("fs_changed", serde_json::json!({ "path": path }));
}

#[tauri::command]
pub fn fs_list(st: State<AppState>, session_id: Option<u32>, path: String) -> Result<Vec<FsEntry>, String> {
    st.game()?.fs_list(session_id, &path)
}

#[tauri::command]
pub fn fs_read(st: State<AppState>, session_id: Option<u32>, path: String) -> Result<String, String> {
    st.game()?.fs_read(session_id, &path)
}

#[tauri::command]
pub fn fs_write(app: AppHandle, st: State<AppState>, session_id: Option<u32>, path: String, text: String) -> Result<(), String> {
    st.game()?.fs_write(session_id, &path, &text)?;
    changed(&app, &path);
    Ok(())
}

#[tauri::command]
pub fn fs_mkdir(app: AppHandle, st: State<AppState>, session_id: Option<u32>, path: String) -> Result<(), String> {
    st.game()?.fs_mkdir(session_id, &path)?;
    changed(&app, &path);
    Ok(())
}

#[tauri::command]
pub fn fs_delete(app: AppHandle, st: State<AppState>, session_id: Option<u32>, path: String) -> Result<(), String> {
    st.game()?.fs_delete(session_id, &path)?;
    changed(&app, &path);
    Ok(())
}

#[tauri::command]
pub fn fs_rename(app: AppHandle, st: State<AppState>, session_id: Option<u32>, from: String, to: String) -> Result<(), String> {
    st.game()?.fs_rename(session_id, &from, &to)?;
    changed(&app, &to);
    Ok(())
}

#[tauri::command]
pub fn fs_copy(app: AppHandle, st: State<AppState>, session_id: Option<u32>, from: String, to: String) -> Result<(), String> {
    st.game()?.fs_copy(session_id, &from, &to)?;
    changed(&app, &to);
    Ok(())
}

#[tauri::command]
pub fn lang_check(src: String) -> Vec<Diagnostic> {
    blackwire_lang::check(&src)
}

#[tauri::command]
pub fn lang_doc() -> &'static str {
    ADDER_DOC
}

#[tauri::command]
pub fn shell_doc(st: State<AppState>) -> Result<String, String> {
    Ok(st.game()?.shell_doc())
}
