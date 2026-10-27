//! Blackwire — оболочка Tauri. Каждое приложение игры — отдельное настоящее окно
//! Windows; состояние мира одно (`blackwire_core::Game` под мьютексом).
//!
//! Файлы:
//! - `state.rs`     — общее состояние: игра, окна, скрипты, пути сохранения.
//! - `windows.rs`   — создание окон приложений, мониторы, раскладка, подложка рабочего стола.
//! - `cmd_game.rs`  — запуск: boot / new_game / load_game / login / save / quit / настройки.
//! - `cmd_shell.rs` — сеансы терминала, команды оболочки, запуск и ввод скриптов.
//! - `cmd_fs.rs`    — файлы для окон, проверка и справка языка.
//! - `cmd_sys.rs`   — монитор, карта сети, веб-страницы, почта.
//! - `scripts.rs`   — потоки скриптов Adder и их связь с миром через `Host`.
//! - `ticker.rs`    — часы мира (раз в секунду) и автосохранение.
//! - `checks.rs`    — проверочный запуск из командной строки: `--new имя`, `--open приложение:аргумент`, `--size ШxВ`.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod cmd_plugins;
mod sdk_qa;
mod checks;
mod cmd_fs;
mod cmd_game;
mod cmd_shell;
mod cmd_sys;
mod netplay;
mod scripts;
mod state;
mod ticker;
mod windows;

use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let handle = app.handle().clone();
            let st = state::AppState::new(&handle)?;
            app.manage(st);
            if !checks::start_from_args(&handle)? {
                windows::open_login(&handle)?;
            }
            ticker::start(handle.clone());
            Ok(())
        })
        .on_window_event(|window, event| {
            match event {
                tauri::WindowEvent::Destroyed => windows::on_window_destroyed(window.app_handle(), window.label()),
                tauri::WindowEvent::Focused(_) => windows::on_focus_changed(window.app_handle()),
                _ => {}
            }
        })
        .invoke_handler(tauri::generate_handler![
            // запуск и настройки
            sdk_qa::sdk_ui_report,
            sdk_qa::sdk_ui_status,
            cmd_plugins::plugins_panel,
            cmd_plugins::plugins_ui,
            cmd_plugins::plugins_doc,
            cmd_plugins::game_doc,
            cmd_plugins::plugins_list,
            cmd_plugins::plugins_reload,
            cmd_plugins::plugins_enable,
            cmd_plugins::plugins_run,
            cmd_plugins::plugins_folder,
            cmd_plugins::plugins_example,
            cmd_game::boot,
            cmd_game::new_game,
            cmd_game::load_game,
            cmd_game::save_game,
            cmd_game::save_slots,
            cmd_game::save_to_slot,
            cmd_game::load_slot,
            cmd_game::delete_slot,
            cmd_game::scenario_targets,
            cmd_game::scenario_list,
            cmd_game::scenario_save,
            cmd_game::scenario_delete,
            cmd_game::scenario_start,
            cmd_game::scenario_stop,
            cmd_game::scenario_active,
            cmd_game::login,
            cmd_game::start_desktop,
            cmd_game::quit,
            cmd_game::settings_get,
            cmd_game::settings_set,
            // сетевая игра
            netplay::net_host,
            netplay::net_join,
            netplay::net_start,
            netplay::net_status,
            netplay::net_leave,
            // окна
            windows::open_app,
            windows::focus_window,
            windows::close_window,
            windows::list_windows,
            windows::monitors,
            windows::arrange_windows,
            windows::desktop_on,
            // терминал и скрипты
            cmd_shell::shell_open,
            cmd_shell::shell_close,
            cmd_shell::shell_exec,
            cmd_shell::shell_complete,
            cmd_shell::shell_prompt,
            cmd_shell::shell_history,
            cmd_shell::script_run,
            cmd_shell::script_input,
            cmd_shell::script_kill,
            // файлы и язык
            cmd_fs::fs_list,
            cmd_fs::fs_read,
            cmd_fs::fs_write,
            cmd_fs::fs_mkdir,
            cmd_fs::fs_delete,
            cmd_fs::fs_rename,
            cmd_fs::fs_copy,
            cmd_fs::lang_check,
            cmd_fs::lang_doc,
            cmd_fs::shell_doc,
            // система, сеть, почта
            cmd_sys::sys_status,
            cmd_sys::sys_kill,
            cmd_sys::net_map,
            cmd_sys::web_get,
            cmd_sys::mail_list,
            cmd_sys::mail_read,
            cmd_sys::mail_send,
            cmd_sys::mail_delete,
            cmd_sys::mail_unread_count,
            cmd_sys::ending_summary,
        ])
        .run(tauri::generate_context!())
        .expect("не удалось запустить Blackwire");
}
