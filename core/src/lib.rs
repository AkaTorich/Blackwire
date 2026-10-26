//! Blackwire — ядро игры. Здесь нет ни окон, ни Tauri: только модель
//! виртуального компьютера и сети, оболочка (shell) и язык Adder.
//! Оболочка Tauri (`app`) держит `Game` под мьютексом и дёргает отсюда функции.
//!
//! Устройство:
//! - `types`   — общие типы ответа оболочки (строки вывода, действия для интерфейса, подсказка).
//! - `rng`     — свой детерминированный генератор случайных чисел (мир строится из зерна).
//! - `vfs`     — виртуальная файловая система одной машины.
//! - `users`   — учётные записи машины.
//! - `procs`   — таблица процессов машины (демоны, скрипты).
//! - `net`     — машины, адреса, службы на портах, журналы, генератор сети.
//! - `shell`   — разбор командной строки и все команды.
//! - язык Adder — отдельный крейт `blackwire-lang` (папка `lang`); связка со скриптами — `script_host` (добавляется после).
//! - `script_host` — связь скрипта Adder с миром (модули sys/fs/net/shell).
//! - `session` — сеанс терминала: цепочка подключений (ssh), текущая папка, переменные.
//! - `sysinfo` — условные показатели машины (процессор, память, сеть) для монитора.
//! - `world`   — весь мир целиком: машины, часы, игрок, настройки; сохранение и загрузка.
//! - `game`    — мир + сеансы терминалов; точка входа для оболочки Tauri.

pub mod plugins;
pub mod types;
pub mod rng;
pub mod vfs;
pub mod users;
pub mod procs;
pub mod net;
pub mod hack;
pub mod missions;
pub mod money;
pub mod market;
pub mod ending;
pub mod life;
pub mod gigs;
pub mod base;
pub mod achievements;
pub mod scenarios;
pub mod shell;
pub mod session;
pub mod sysinfo;
pub mod world;
pub mod game;
pub mod script_host;

pub use world::World;

pub use game::Game;
pub use types::*;
