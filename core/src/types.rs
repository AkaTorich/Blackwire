//! Общие типы, которыми ядро отвечает интерфейсу. Все они сериализуются в JSON
//! один в один (snake_case) — интерфейс на JS читает их как есть.

use serde::{Deserialize, Serialize};

/// Вид строки вывода терминала — интерфейс красит по нему.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OutKind {
    Normal,
    Error,
    Dim,
    Accent,
    Success,
    Warn,
    Heading,
}

/// Одна строка вывода. `delay_ms` — пауза перед показом строки (сканирование,
/// пинг и прочие «долгие» дела печатаются построчно, как в кино).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct OutLine {
    pub text: String,
    pub kind: OutKind,
    pub delay_ms: u32,
}

impl OutLine {
    pub fn new(text: impl Into<String>, kind: OutKind) -> Self {
        OutLine { text: text.into(), kind, delay_ms: 0 }
    }
    pub fn normal(text: impl Into<String>) -> Self { Self::new(text, OutKind::Normal) }
    pub fn error(text: impl Into<String>) -> Self { Self::new(text, OutKind::Error) }
    pub fn dim(text: impl Into<String>) -> Self { Self::new(text, OutKind::Dim) }
    pub fn accent(text: impl Into<String>) -> Self { Self::new(text, OutKind::Accent) }
    pub fn success(text: impl Into<String>) -> Self { Self::new(text, OutKind::Success) }
    pub fn warn(text: impl Into<String>) -> Self { Self::new(text, OutKind::Warn) }
    pub fn heading(text: impl Into<String>) -> Self { Self::new(text, OutKind::Heading) }
    pub fn after(mut self, delay_ms: u32) -> Self { self.delay_ms = delay_ms; self }
}

/// Что интерфейс должен сделать после команды (кроме печати строк).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum UiAction {
    /// Очистить экран терминала.
    Clear,
    /// Открыть приложение: `editor`, `files`, `browser`, `monitor`, `map`, `settings`, `help`, `terminal`.
    /// `arg` — путь к файлу (editor/files), адрес (browser) и т. п.
    OpenApp { app: String, arg: Option<String> },
    /// Закрыть окно терминала (команда `exit` на своей машине).
    Exit,
    /// Сохранить мир и выйти из игры (shutdown на компьютере игрока).
    QuitGame,
    /// Всплывающее уведомление в углу экрана.
    Notify { title: String, text: String },
    /// Скрипт запущен как процесс: интерфейс дальше слушает события `script_out`/`script_done` по pid.
    ScriptStarted { pid: u32, name: String },
    /// `adder --check файл`: разобрать текст скрипта и показать ошибки (язык есть только у оболочки Tauri).
    CheckScript { path: String, src: String },
}

/// Строка подсказки терминала.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct Prompt {
    pub user: String,
    pub host: String,
    pub cwd: String,
    pub is_root: bool,
    /// Имена машин по цепочке подключений, начиная со своей (длина 1 — мы дома).
    pub hops: Vec<String>,
}

/// Ответ оболочки на одну строку команды.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct ShellResult {
    pub lines: Vec<OutLine>,
    pub actions: Vec<UiAction>,
    pub prompt: Prompt,
}

/// Запись каталога для файлового менеджера.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct FsEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
    /// Игровое время изменения, секунды от начала эпохи мира.
    pub modified: u64,
    pub owner: String,
    /// Права в виде `rwxr-xr-x`.
    pub mode: String,
}
