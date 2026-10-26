//! Автоматическая проверка UI плагинов из тестового запуска.
//!
//! Разрешена только при явно отдельном профиле, чтобы сохранение игрока не
//! затрагивалось: нужны одновременно флаги `--sdk-ui-test` и `--data-dir`.
//! Результат самопроверки окно отправляет через `sdk_ui_report`, а скрипт
//! проверки — `tools/sdk-ui-test.js`, подкладывается в окно при его запуске.

use tauri::{AppHandle, Manager, State};

use crate::state::AppState;

/// Истина, если игра запущена с проверочными флагами.
fn allowed() -> Result<(), String> {
    let args: Vec<String> = std::env::args().collect();
    let has_test = args.iter().any(|a| a == "--sdk-ui-test");
    let has_dir = args.iter().any(|a| a == "--data-dir");
    if has_test && has_dir {
        Ok(())
    } else {
        Err("тестовый режим не включён".into())
    }
}

const MAX_REPORT: usize = 65_536;
const STAGES: &[&str] = &["manager", "panel"];

/// Записать отчёт самопроверки окна.
#[tauri::command]
pub fn sdk_ui_report(
    st: State<'_, AppState>,
    stage: String,
    result: serde_json::Value,
) -> Result<(), String> {
    allowed()?;
    if !STAGES.contains(&stage.as_str()) || result.to_string().len() > MAX_REPORT {
        return Err("неверный отчёт".into());
    }
    let path = st.save_path.with_file_name("sdk-ui-test-report.json");
    let mut report: serde_json::Value = std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_else(|| serde_json::json!({}));
    report[stage] = result;
    let text = serde_json::to_string_pretty(&report).unwrap();
    std::fs::write(&path, text).map_err(|e| e.to_string())?;
    st.save()
}

/// Прочитать текущий отчёт (для сторонних проверок).
#[tauri::command]
pub fn sdk_ui_status(st: State<'_, AppState>) -> Result<serde_json::Value, String> {
    allowed()?;
    let path = st.save_path.with_file_name("sdk-ui-test-report.json");
    let value = std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_else(|| serde_json::json!({}));
    Ok(value)
}

/// Подсунуть окну проверочный скрипт — он сам запустит шаги и пришлёт отчёт.
pub fn inject(app: &AppHandle, label: String) {
    let app = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(1500));
        if let Some(win) = app.get_webview_window(&label) {
            let _ = win.eval(include_str!("../../tools/sdk-ui-test.js"));
        }
    });
}
