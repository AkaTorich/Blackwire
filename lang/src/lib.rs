//! Язык Adder — скриптовый язык игры, по духу Python (блоки отступами).
//! Описание языка — `docs/ADDER.md`. Устройство:
//! - `lexer`    — текст → лексемы (с INDENT/DEDENT/NEWLINE).
//! - `ast`      — узлы дерева.
//! - `parser`   — лексемы → `ast::Program`.
//! - `value`    — значения времени исполнения.
//! - `interp`   — исполнитель (обход дерева), области видимости, вызовы.
//! - `builtins` — встроенные функции, методы строк/списков/словарей, модули `math`, `json`, `time`.
//! - `error`    — ошибки с номером строки.
//!
//! Скрипт общается с игрой только через `Host`: печать, ввод, вызовы игровых
//! модулей (`sys.*`, `fs.*`, `net.*`, `shell.*`) и проверку «пора остановиться».

pub mod lexer;
pub mod ast;
pub mod parser;
pub mod value;
pub mod interp;
pub mod builtins;
pub mod error;

pub use error::LangError;
pub use value::Value;

/// Диагностика для редактора кода.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct Diagnostic {
    pub line: u32,
    pub col: u32,
    pub message: String,
}

/// Мир, к которому обращается скрипт. Реализует `script_host::ScriptHost`.
pub trait Host {
    /// Напечатать одну строку (без завершающего `\n` — его добавит интерфейс).
    fn print(&mut self, text: &str);
    /// Прочитать строку от игрока. Может блокировать поток скрипта, пока игрок не ответит.
    fn input(&mut self, prompt: &str) -> Result<String, LangError>;
    /// Вызов игровой функции по полному имени: `"fs.read"`, `"net.scan"`, `"sys.hostname"`.
    /// Неизвестное имя — `Err(LangError::runtime(...))`.
    fn call(&mut self, name: &str, args: Vec<Value>) -> Result<Value, LangError>;
    /// Истина, когда скрипт надо прервать (`kill`, Ctrl+C, закрытие игры).
    /// Исполнитель проверяет это не реже чем раз в несколько сотен шагов.
    fn should_stop(&self) -> bool;
    /// Пауза скрипта на `ms` миллисекунд (встроенная `sleep`).
    fn sleep(&mut self, ms: u64);
}

/// Разобрать исходник в дерево.
pub fn parse(src: &str) -> Result<ast::Program, LangError> {
    let tokens = lexer::tokenize(src)?;
    parser::parse(tokens)
}

/// Проверить исходник без исполнения — для редактора (пустой список = ошибок нет).
pub fn check(src: &str) -> Vec<Diagnostic> {
    match parse(src) {
        Ok(_) => Vec::new(),
        Err(e) => vec![Diagnostic { line: e.line, col: e.col, message: e.message.clone() }],
    }
}

/// Выполнить исходник целиком. `args` попадает в `sys.args` (первый элемент — имя скрипта).
/// Возвращает значение `exit(code)` либо `Value::Null`.
pub fn run(src: &str, host: &mut dyn Host, args: Vec<String>) -> Result<Value, LangError> {
    let program = parse(src)?;
    let mut vm = interp::Interpreter::new(args);
    vm.run(&program, host)
}
