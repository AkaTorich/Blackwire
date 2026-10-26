//! Ошибки языка Adder: синтаксические и времени исполнения, с номером строки.
//! `LangError` — то, что видит игрок и редактор. `Exc` — внутренний сигнал
//! исполнителя: обычная ошибка (её ловит `try/except`), `exit()` и прерывание хостом.

use std::fmt;

/// Ошибка с позицией в исходнике. `col == 0` — позиция неизвестна (ошибка исполнения).
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct LangError {
    pub line: u32,
    pub col: u32,
    pub message: String,
}

impl LangError {
    /// Ошибка времени исполнения: «Ошибка (строка 12): деление на ноль».
    pub fn runtime(line: u32, message: impl Into<String>) -> Self {
        LangError { line, col: 0, message: message.into() }
    }

    /// Синтаксическая ошибка с номером строки и позицией.
    pub fn syntax(line: u32, col: u32, message: impl Into<String>) -> Self {
        LangError { line, col, message: message.into() }
    }

    /// Подставить номер строки, если он ещё не известен (ошибки от хоста приходят без строки).
    pub fn with_line(mut self, line: u32) -> Self {
        if self.line == 0 {
            self.line = line;
        }
        self
    }
}

impl fmt::Display for LangError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.line == 0 {
            write!(f, "Ошибка: {}", self.message)
        } else if self.col == 0 {
            write!(f, "Ошибка (строка {}): {}", self.line, self.message)
        } else {
            write!(f, "Ошибка (строка {}, позиция {}): {}", self.line, self.col, self.message)
        }
    }
}

impl std::error::Error for LangError {}

/// Внутренний сигнал исполнителя.
/// - `Error` — ошибка, которую может поймать `try/except`;
/// - `Exit(code)` — вызван `exit()`, выполнение завершается, не ловится;
/// - `Interrupted(line)` — хост попросил остановиться (`should_stop`), не ловится.
#[derive(Debug, Clone)]
pub enum Exc {
    Error(LangError),
    Exit(i64),
    Interrupted(u32),
}

impl From<LangError> for Exc {
    fn from(e: LangError) -> Self {
        Exc::Error(e)
    }
}

/// Короткая запись ошибки исполнения в виде сигнала.
pub fn rt(line: u32, message: impl Into<String>) -> Exc {
    Exc::Error(LangError::runtime(line, message))
}
