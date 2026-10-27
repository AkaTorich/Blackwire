//! Общее состояние оболочки: игра под мьютексом, список окон, работающие скрипты,
//! путь сохранения. Доступ из команд — `tauri::State<AppState>`.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{mpsc, Arc, Mutex, MutexGuard};

use blackwire_core::world::World;
use blackwire_core::Game;
use tauri::Manager;

/// Окно приложения игры.
#[derive(Clone, Debug, serde::Serialize)]
pub struct WinInfo {
    pub label: String,
    pub app: String,
    pub arg: Option<String>,
}

/// Поток скрипта: флаг остановки и канал для ответов на `input()`.
pub struct ScriptHandle {
    pub stop: Arc<AtomicBool>,
    pub input_tx: mpsc::Sender<String>,
}

pub struct AppState {
    game: Mutex<Option<Game>>,
    pub windows: Mutex<Vec<WinInfo>>,
    pub scripts: Mutex<HashMap<u32, ScriptHandle>>,
    pub save_path: PathBuf,
    /// Папка именованных сохранений (снимков), которые делает сам игрок.
    pub slots_dir: PathBuf,
    /// Папка своих сценариев (из редактора сценариев).
    pub scenarios_dir: PathBuf,
    pub plugins_dir: PathBuf,
    pub next_win: AtomicU32,
    pub focus_revision: AtomicU32,
    pub focused_window: Mutex<Option<String>>,
    /// Монитор, на котором стоит подложка рабочего стола (по одному окну на монитор).
    pub desktops: Mutex<HashMap<u32, String>>,
    /// Проверочный режим `--size ШxВ`: все новые окна этого размера.
    pub forced_size: Mutex<Option<(f64, f64)>>,
    /// Сетевая игра: роль, партия, подключения.
    pub net: Mutex<crate::netplay::NetState>,
}

impl AppState {
    pub fn new(app: &tauri::AppHandle) -> Result<AppState, Box<dyn std::error::Error>> {
        let args:Vec<_>=std::env::args().collect();
        let dir = if let Some(i)=args.iter().position(|a|a=="--data-dir"){
            let value=args.get(i+1).ok_or("после --data-dir нужен путь")?;let path=PathBuf::from(value);if !path.is_absolute(){return Err("--data-dir должен быть абсолютным".into());}path
        }else{app.path().app_data_dir()?};
        std::fs::create_dir_all(&dir)?;
        let slots_dir = dir.join("slots");
        std::fs::create_dir_all(&slots_dir)?;
        let scenarios_dir = dir.join("scenarios");
        std::fs::create_dir_all(&scenarios_dir)?;
        Ok(AppState {
            game: Mutex::new(None),
            windows: Mutex::new(Vec::new()),
            scripts: Mutex::new(HashMap::new()),
            save_path: dir.join("save.json"),
            slots_dir,
            scenarios_dir,
            plugins_dir: dir.join("plugins"),
            next_win: AtomicU32::new(1),
            focus_revision: AtomicU32::new(0),
            focused_window: Mutex::new(None),
            desktops: Mutex::new(HashMap::new()),
            forced_size: Mutex::new(None),
            net: Mutex::new(crate::netplay::NetState::default()),
        })
    }

    /// Игра под замком. Пока игра не начата — ошибка по-русски.
    pub fn game(&self) -> Result<GameGuard<'_>, String> {
        let g = self.game.lock().map_err(|_| "состояние игры повреждено".to_string())?;
        if g.is_none() {
            return Err("игра ещё не начата".into());
        }
        Ok(GameGuard(g))
    }

    pub fn has_game(&self) -> bool {
        self.game.lock().map(|g| g.is_some()).unwrap_or(false)
    }

    pub fn set_game(&self, mut game: Game) {
        let _ = std::fs::create_dir_all(&self.plugins_dir);
        game.load_plugins(&self.plugins_dir);
        if let Ok(mut g) = self.game.lock() {
            *g = Some(game);
        }
    }

    pub fn has_save(&self) -> bool {
        self.save_path.is_file()
    }

    /// Прочитать сохранение с диска, не загружая его в игру (для экрана входа).
    pub fn peek_save(&self) -> Option<World> {
        let text = std::fs::read_to_string(&self.save_path).ok()?;
        World::from_json(&text).ok()
    }

    pub fn save(&self) -> Result<(), String> {
        let mut g = self.game()?;
        g.plugin_event("on_save",serde_json::json!({}));
        let text = g.world.to_json();
        std::fs::write(&self.save_path, text).map_err(|e| format!("не удалось сохранить игру: {e}"))
    }

    pub fn next_label(&self) -> String {
        format!("w{}", self.next_win.fetch_add(1, Ordering::Relaxed))
    }

    // ---------- именованные сохранения (слоты) ----------

    /// Имя слота → путь файла. Имя приводится к безопасному (без разделителей пути).
    fn slot_path(&self, name: &str) -> Result<PathBuf, String> {
        let clean = sanitize_slot(name);
        if clean.is_empty() {
            return Err("пустое имя сохранения".into());
        }
        Ok(self.slots_dir.join(format!("{clean}.json")))
    }

    /// Сохранить текущую игру в именованный слот.
    pub fn save_slot(&self, name: &str) -> Result<(), String> {
        let path = self.slot_path(name)?;
        let text = {
            let g = self.game()?;
            g.world.to_json()
        };
        std::fs::write(&path, text).map_err(|e| format!("не удалось сохранить: {e}"))
    }

    /// Загрузить мир из слота (в игру его ставит cmd_game).
    pub fn load_slot_world(&self, name: &str) -> Result<World, String> {
        let path = self.slot_path(name)?;
        let text = std::fs::read_to_string(&path).map_err(|_| "сохранение не найдено".to_string())?;
        World::from_json(&text)
    }

    /// Удалить слот.
    pub fn delete_slot(&self, name: &str) -> Result<(), String> {
        let path = self.slot_path(name)?;
        std::fs::remove_file(&path).map_err(|e| format!("не удалось удалить: {e}"))
    }

    /// Список слотов: имя, личность, игровое время и счёт для карточки.
    pub fn list_slots(&self) -> Vec<SlotInfo> {
        let mut out = Vec::new();
        let Ok(entries) = std::fs::read_dir(&self.slots_dir) else { return out };
        for e in entries.flatten() {
            let path = e.path();
            if path.extension().and_then(|x| x.to_str()) != Some("json") {
                continue;
            }
            let name = match path.file_stem().and_then(|x| x.to_str()) {
                Some(n) => n.to_string(),
                None => continue,
            };
            if let Some(w) = std::fs::read_to_string(&path).ok().and_then(|t| World::from_json(&t).ok()) {
                out.push(SlotInfo {
                    name,
                    handle: w.player.handle.clone(),
                    time: blackwire_core::world::GameClock::format_short(w.now()),
                    money: w.player.money,
                });
            }
        }
        out.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        out
    }
}

/// Карточка слота сохранения для экрана настроек.
#[derive(Clone, Debug, serde::Serialize)]
pub struct SlotInfo {
    pub name: String,
    pub handle: String,
    pub time: String,
    pub money: i64,
}

/// Карточка своего сценария для редактора.
#[derive(Clone, Debug, serde::Serialize)]
pub struct ScenarioInfo {
    pub name: String,
    pub brief: String,
    pub goal: String,
    pub reward: i64,
}

impl AppState {
    fn scenario_path(&self, name: &str) -> Result<PathBuf, String> {
        let clean = sanitize_slot(name);
        if clean.is_empty() {
            return Err("пустое имя сценария".into());
        }
        Ok(self.scenarios_dir.join(format!("{clean}.json")))
    }

    /// Сохранить сценарий файлом.
    pub fn save_scenario(&self, s: &blackwire_core::scenarios::Scenario) -> Result<(), String> {
        let path = self.scenario_path(&s.name)?;
        let text = serde_json::to_string(s).map_err(|e| format!("не удалось записать сценарий: {e}"))?;
        std::fs::write(&path, text).map_err(|e| format!("не удалось записать: {e}"))
    }

    pub fn load_scenario(&self, name: &str) -> Result<blackwire_core::scenarios::Scenario, String> {
        let path = self.scenario_path(name)?;
        let text = std::fs::read_to_string(&path).map_err(|_| "сценарий не найден".to_string())?;
        serde_json::from_str(&text).map_err(|e| format!("сценарий повреждён: {e}"))
    }

    pub fn delete_scenario(&self, name: &str) -> Result<(), String> {
        let path = self.scenario_path(name)?;
        std::fs::remove_file(&path).map_err(|e| format!("не удалось удалить: {e}"))
    }

    /// Список сценариев для карточек.
    pub fn list_scenarios(&self) -> Vec<ScenarioInfo> {
        let mut out = Vec::new();
        let Ok(entries) = std::fs::read_dir(&self.scenarios_dir) else { return out };
        for e in entries.flatten() {
            let path = e.path();
            if path.extension().and_then(|x| x.to_str()) != Some("json") {
                continue;
            }
            if let Some(s) = std::fs::read_to_string(&path)
                .ok()
                .and_then(|t| serde_json::from_str::<blackwire_core::scenarios::Scenario>(&t).ok())
            {
                out.push(ScenarioInfo {
                    goal: blackwire_core::scenarios::goal_text(&s.goal),
                    name: s.name,
                    brief: s.brief,
                    reward: s.reward,
                });
            }
        }
        out.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        out
    }
}

/// Безопасное имя файла слота: оставляем буквы, цифры, пробел, «_» и «-».
fn sanitize_slot(name: &str) -> String {
    let mut s: String = name
        .trim()
        .chars()
        .map(|c| if c.is_alphanumeric() || c == ' ' || c == '_' || c == '-' { c } else { '_' })
        .collect();
    s.truncate(40);
    s.trim().to_string()
}

/// Замок на игре с удобным разыменованием в `Game`.
pub struct GameGuard<'a>(MutexGuard<'a, Option<Game>>);

impl std::ops::Deref for GameGuard<'_> {
    type Target = Game;
    fn deref(&self) -> &Game {
        self.0.as_ref().expect("игра есть")
    }
}

impl std::ops::DerefMut for GameGuard<'_> {
    fn deref_mut(&mut self) -> &mut Game {
        self.0.as_mut().expect("игра есть")
    }
}
