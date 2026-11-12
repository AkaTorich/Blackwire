//! Сердце сервера: общий мир, части игроков, учётные записи, кто сейчас в сети.
//!
//! Замки: `state` (мир) и `online` (подключения) никогда не берутся в обратном
//! порядке; отправка в сеть идёт уже без замков.

use std::collections::{BTreeMap, HashMap, VecDeque};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{mpsc, Arc, Mutex};

use blackwire_core::mmo::PlayerCtx;
use blackwire_core::world::World;
use blackwire_core::Game;
use blackwire_wire::Writer;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::accounts::Account;

#[derive(Serialize, Deserialize, Clone)]
pub struct ChatLine {
    pub from: String,
    pub text: String,
    pub time: u64,
}

pub struct State {
    pub game: Game,
    pub players: BTreeMap<String, PlayerCtx>,
    pub accounts: BTreeMap<String, Account>,
    pub chat: VecDeque<ChatLine>,
}

/// Подключённый игрок.
pub struct Online {
    pub conn: u64,
    pub writer: Writer,
    pub last_unread: Option<usize>,
}

pub struct ScriptHandle {
    pub stop: Arc<AtomicBool>,
    pub input_tx: mpsc::Sender<String>,
}

pub struct Config {
    pub data_dir: PathBuf,
    pub lang: String,
    pub plugins: Option<PathBuf>,
    pub max_accounts: usize,
    pub max_scripts: usize,
}

pub struct Hub {
    pub state: Mutex<State>,
    pub online: Mutex<HashMap<String, Online>>,
    pub scripts: Mutex<HashMap<(String, u32), ScriptHandle>>,
    pub cfg: Config,
    next_conn: AtomicU64,
}

/// Файл с игроками и учётными записями (мир лежит отдельно — world.json).
#[derive(Serialize, Deserialize, Default)]
struct PlayersFile {
    players: BTreeMap<String, PlayerCtx>,
    accounts: BTreeMap<String, Account>,
    #[serde(default)]
    chat: VecDeque<ChatLine>,
}

fn session_base(i: usize) -> u32 {
    1 + (i as u32 % 40_000) * 100_000
}

impl Hub {
    /// Открыть сохранённый мир или создать новый.
    pub fn open(cfg: Config, seed: u64) -> Result<Hub, String> {
        std::fs::create_dir_all(&cfg.data_dir).map_err(|e| format!("{}: {}", cfg.data_dir.display(), e))?;
        let world_path = cfg.data_dir.join("world.json");
        let players_path = cfg.data_dir.join("players.json");
        let (world, mut pf) = if world_path.is_file() {
            let text = std::fs::read_to_string(&world_path).map_err(|e| e.to_string())?;
            let world = World::from_json(&text)?;
            let pf: PlayersFile = match std::fs::read_to_string(&players_path) {
                Ok(t) => serde_json::from_str(&t).map_err(|e| format!("players.json: {}", e))?,
                Err(_) => PlayersFile::default(),
            };
            println!("мир загружен: {} машин, {} игроков", world.machines.len(), pf.players.len());
            (world, pf)
        } else {
            let world = World::mmo_new(seed);
            println!("создан новый мир (зерно {}): {} машин", seed, world.machines.len());
            (world, PlayersFile::default())
        };
        // номера сеансов у каждого свои — не пересекаются в плагинах
        for (i, ctx) in pf.players.values_mut().enumerate() {
            ctx.next_session = session_base(i);
        }
        let mut game = Game::new(world);
        if let Some(dir) = &cfg.plugins {
            game.load_plugins(dir);
            println!("плагины: {}", game.plugin_list());
        }
        let hub = Hub {
            state: Mutex::new(State { game, players: pf.players, accounts: pf.accounts, chat: pf.chat }),
            online: Mutex::new(HashMap::new()),
            scripts: Mutex::new(HashMap::new()),
            cfg,
            next_conn: AtomicU64::new(1),
        };
        hub.save();
        Ok(hub)
    }

    pub fn new_conn_id(&self) -> u64 {
        self.next_conn.fetch_add(1, Ordering::Relaxed)
    }

    /// Записать мир и игроков на диск (через временный файл).
    pub fn save(&self) {
        let (world_json, players_json) = {
            let Ok(st) = self.state.lock() else { return };
            let pf = PlayersFileRef { players: &st.players, accounts: &st.accounts, chat: &st.chat };
            (st.game.world.to_json(), serde_json::to_string(&pf).unwrap_or_default())
        };
        if players_json.is_empty() || world_json.starts_with("{\"error\"") {
            eprintln!("сохранение пропущено: ошибка сериализации");
            return;
        }
        for (name, text) in [("world.json", world_json), ("players.json", players_json)] {
            let path = self.cfg.data_dir.join(name);
            let tmp = self.cfg.data_dir.join(format!("{}.tmp", name));
            if std::fs::write(&tmp, text).and_then(|_| std::fs::rename(&tmp, &path)).is_err() {
                eprintln!("не удалось записать {}", path.display());
            }
        }
    }

    /// Выполнить действие от лица игрока: его часть подставляется в мир, после
    /// действия убирается. Паника внутри не ломает мир.
    pub fn with_player<R>(&self, handle: &str, f: impl FnOnce(&mut Game) -> R) -> Result<R, String> {
        let mut st = self.state.lock().map_err(|_| "мир недоступен".to_string())?;
        let st = &mut *st;
        let mut ctx = st.players.remove(handle).ok_or_else(|| "игрок не найден".to_string())?;
        blackwire_core::i18n::set(if ctx.lang.is_empty() { &self.cfg.lang } else { &ctx.lang });
        st.game.swap_player(&mut ctx);
        let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| f(&mut st.game)));
        st.game.swap_player(&mut ctx);
        st.players.insert(handle.to_string(), ctx);
        blackwire_core::i18n::set(&self.cfg.lang);
        r.map_err(|_| "внутренняя ошибка сервера / internal server error".to_string())
    }

    /// Событие одному игроку (если он в сети).
    pub fn send(&self, handle: &str, ev: &str, data: Value) {
        let w = self.online.lock().ok().and_then(|o| o.get(handle).map(|x| x.writer.clone()));
        if let Some(w) = w {
            let _ = w.send(&json!({ "ev": ev, "data": data }).to_string());
        }
    }

    /// Событие всем в сети.
    pub fn broadcast(&self, ev: &str, data: Value) {
        let ws: Vec<Writer> = self.online.lock().map(|o| o.values().map(|x| x.writer.clone()).collect()).unwrap_or_default();
        let line = json!({ "ev": ev, "data": data }).to_string();
        for w in ws {
            let _ = w.send(&line);
        }
    }

    pub fn online_names(&self) -> Vec<String> {
        let mut v: Vec<String> = self.online.lock().map(|o| o.keys().cloned().collect()).unwrap_or_default();
        v.sort();
        v
    }

    /// Новая строка чата: в историю и всем.
    pub fn chat(&self, from: &str, text: &str) {
        let line = ChatLine { from: from.to_string(), text: text.to_string(), time: crate::accounts::now() };
        if let Ok(mut st) = self.state.lock() {
            st.chat.push_back(line.clone());
            while st.chat.len() > 100 {
                st.chat.pop_front();
            }
        }
        self.broadcast("chat", serde_json::to_value(&line).unwrap_or(Value::Null));
    }

    /// Остановить все скрипты игрока (выход из сети).
    pub fn stop_scripts_of(&self, handle: &str) {
        if let Ok(s) = self.scripts.lock() {
            for ((h, _), sh) in s.iter() {
                if h == handle {
                    sh.stop.store(true, Ordering::Relaxed);
                }
            }
        }
    }

    pub fn scripts_of(&self, handle: &str) -> usize {
        self.scripts.lock().map(|s| s.keys().filter(|(h, _)| h == handle).count()).unwrap_or(0)
    }

    /// Номер первого сеанса для нового игрока.
    pub fn session_base_for_new(&self, st: &State) -> u32 {
        session_base(st.players.len())
    }
}

#[derive(Serialize)]
struct PlayersFileRef<'a> {
    players: &'a BTreeMap<String, PlayerCtx>,
    accounts: &'a BTreeMap<String, Account>,
    chat: &'a VecDeque<ChatLine>,
}
