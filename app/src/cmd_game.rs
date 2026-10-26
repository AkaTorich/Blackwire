//! Запуск игры: экран входа, новая личность, загрузка, сохранение, настройки, выход.

use blackwire_core::world::{Settings, World};
use blackwire_core::Game;
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};

use crate::state::AppState;
use crate::windows;

pub const OS_NAME: &str = "Blackwire OS";
pub const OS_VERSION: &str = "0.1";

#[derive(Serialize)]
pub struct ThemeInfo {
    id: &'static str,
    name: &'static str,
    accent: &'static str,
}

const THEMES: &[ThemeInfo] = &[
    ThemeInfo { id: "swordfish", name: "Рыба-меч", accent: "#35d6ff" },
    ThemeInfo { id: "ice", name: "Лёд", accent: "#8fd3ff" },
    ThemeInfo { id: "amber", name: "Янтарь", accent: "#ffb347" },
    ThemeInfo { id: "phosphor", name: "Фосфор", accent: "#4cff7a" },
    ThemeInfo { id: "violet", name: "Фиолет", accent: "#b48cff" },
    ThemeInfo { id: "crimson", name: "Кармин", accent: "#ff4d6d" },
    ThemeInfo { id: "paper", name: "Бумага", accent: "#1b5fb3" },
    ThemeInfo { id: "midnight", name: "Полночь", accent: "#5b8def" },
    ThemeInfo { id: "steel", name: "Сталь", accent: "#a9b7c6" },
    ThemeInfo { id: "gold", name: "Золото", accent: "#e6c66e" },
];

#[derive(Serialize)]
pub struct BootInfo {
    has_save: bool,
    handle: Option<String>,
    os_name: &'static str,
    version: &'static str,
    settings: Settings,
    theme_list: &'static [ThemeInfo],
}

fn boot_info(st: &AppState) -> BootInfo {
    let (handle, settings) = if let Ok(g) = st.game() {
        (Some(g.world.player.handle.clone()), g.world.settings.clone())
    } else if let Some(w) = st.peek_save() {
        (Some(w.player.handle.clone()), w.settings.clone())
    } else {
        (None, Settings::default())
    };
    BootInfo { has_save: st.has_save(), handle, os_name: OS_NAME, version: OS_VERSION, settings, theme_list: THEMES }
}

#[tauri::command]
pub fn boot(st: State<'_, AppState>) -> BootInfo {
    boot_info(&st)
}

fn seed_from_handle(handle: &str) -> u64 {
    // зерно из имени и времени запуска — каждый новый мир другой
    let mut h: u64 = 0xcbf29ce484222325;
    for b in handle.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    let t = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(1);
    h ^ t.rotate_left(17)
}

#[tauri::command]
pub fn new_game(st: State<'_, AppState>, handle: String, password: Option<String>, seed: Option<u64>) -> Result<BootInfo, String> {
    let handle = handle.trim().to_string();
    if handle.len() < 3 || handle.len() > 16 || !handle.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') {
        return Err("имя — латиница, цифры, «_» и «-», от 3 до 16 знаков".into());
    }
    let seed = seed.unwrap_or_else(|| seed_from_handle(&handle));
    let world = World::new(seed, &handle, password.as_deref().unwrap_or(""));
    st.set_game(Game::new(world));
    st.save()?;
    Ok(boot_info(&st))
}

#[tauri::command]
pub fn load_game(st: State<'_, AppState>) -> Result<BootInfo, String> {
    let text = std::fs::read_to_string(&st.save_path).map_err(|_| "сохранение не найдено".to_string())?;
    let world = World::from_json(&text)?;
    st.set_game(Game::new(world));
    Ok(boot_info(&st))
}

#[tauri::command]
pub fn save_game(st: State<'_, AppState>) -> Result<(), String> {
    st.save()
}

// ---------- именованные сохранения (слоты) ----------

#[tauri::command]
pub fn save_slots(st: State<'_, AppState>) -> Vec<crate::state::SlotInfo> {
    st.list_slots()
}

#[tauri::command]
pub fn save_to_slot(st: State<'_, AppState>, name: String) -> Result<Vec<crate::state::SlotInfo>, String> {
    st.save_slot(&name)?;
    Ok(st.list_slots())
}

#[tauri::command]
pub fn load_slot(st: State<'_, AppState>, name: String) -> Result<BootInfo, String> {
    let world = st.load_slot_world(&name)?;
    st.set_game(Game::new(world));
    // продолжаем и дальше с этого места: переносим снимок в основное сохранение
    st.save()?;
    Ok(boot_info(&st))
}

#[tauri::command]
pub fn delete_slot(st: State<'_, AppState>, name: String) -> Result<Vec<crate::state::SlotInfo>, String> {
    st.delete_slot(&name)?;
    Ok(st.list_slots())
}

// ---------- свои сценарии (редактор сценариев) ----------

#[tauri::command]
pub fn scenario_targets(st: State<'_, AppState>) -> Vec<String> {
    st.game().map(|g| blackwire_core::scenarios::targets(&g.world)).unwrap_or_default()
}

#[tauri::command]
pub fn scenario_list(st: State<'_, AppState>) -> Vec<crate::state::ScenarioInfo> {
    st.list_scenarios()
}

#[tauri::command]
pub fn scenario_save(st: State<'_, AppState>, scenario: blackwire_core::scenarios::Scenario) -> Result<Vec<crate::state::ScenarioInfo>, String> {
    if scenario.name.trim().is_empty() {
        return Err("у сценария должно быть имя".into());
    }
    st.save_scenario(&scenario)?;
    Ok(st.list_scenarios())
}

#[tauri::command]
pub fn scenario_delete(st: State<'_, AppState>, name: String) -> Result<Vec<crate::state::ScenarioInfo>, String> {
    st.delete_scenario(&name)?;
    Ok(st.list_scenarios())
}

#[tauri::command]
pub fn scenario_start(st: State<'_, AppState>, name: String) -> Result<(), String> {
    let scenario = st.load_scenario(&name)?;
    {
        let mut g = st.game()?;
        blackwire_core::scenarios::start(&mut g.world, scenario);
    }
    let _ = st.save();
    Ok(())
}

#[tauri::command]
pub fn scenario_stop(st: State<'_, AppState>) -> Result<(), String> {
    {
        let mut g = st.game()?;
        blackwire_core::scenarios::stop(&mut g.world);
    }
    let _ = st.save();
    Ok(())
}

#[tauri::command]
pub fn scenario_active(st: State<'_, AppState>) -> Option<blackwire_core::scenarios::Scenario> {
    st.game().ok().and_then(|g| g.world.scenarios.active.clone())
}

/// Вход по паролю своей учётной записи (пустой пароль допустим, если так создана личность).
#[tauri::command]
pub fn login(st: State<'_, AppState>, password: Option<String>) -> Result<bool, String> {
    if !st.has_game() {
        load_game(st.clone())?;
    }
    let g = st.game()?;
    let handle = g.world.player.handle.clone();
    Ok(g.world.machines[0].users.check_password(&handle, password.as_deref().unwrap_or("")))
}

/// После входа: открыть подложку рабочего стола и первый терминал, закрыть окно входа.
#[tauri::command]
pub async fn start_desktop(app: AppHandle, st: State<'_, AppState>) -> Result<(), String> {
    let monitor = st.game()?.world.settings.default_monitor as u32;
    windows::open_desktop(&app, &st, monitor)?;
    windows::create_app_window(&app, &st, "terminal", None, Some(monitor))?;
    if let Some(w) = app.get_webview_window("login") {
        let _ = w.close();
    }
    Ok(())
}

#[tauri::command]
pub async fn quit(app: AppHandle, st: State<'_, AppState>) -> Result<(), String> {
    let _ = st.save();
    app.exit(0);
    Ok(())
}

#[tauri::command]
pub fn settings_get(st: State<'_, AppState>) -> Settings {
    st.game().map(|g| g.world.settings.clone()).unwrap_or_else(|_| st.peek_save().map(|w| w.settings).unwrap_or_default())
}

#[tauri::command]
pub fn settings_set(app: AppHandle, st: State<'_, AppState>, settings: Settings) -> Result<(), String> {
    {
        let mut g = st.game()?;
        g.world.settings = settings.clone();
    }
    let _ = st.save();
    let _ = app.emit("theme_changed", serde_json::json!({ "settings": settings }));
    Ok(())
}
