//! Blackwire — онлайн-сервер общего мира. Один мир на всех: каждый игрок со
//! своим компьютером и своим сюжетом hello-world, компьютеры игроков — в общей
//! сети. Работает без окон (Linux-служба), мир хранит в папке данных.
//!
//! Запуск: `blackwire-server --port 7777 --data ./data --cert fullchain.pem --key privkey.pem`
//! Без `--cert/--key` — без шифрования (игра подключается адресом `tcp://хост:порт`).

mod accounts;
mod api;
mod conn;
mod hub;
mod scripts;
mod tick;

use std::net::TcpListener;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use hub::{Config, Hub};

const HELP: &str = "blackwire-server — онлайн-сервер Blackwire

  --port N          порт (по умолчанию 7777)
  --bind АДРЕС      на каком адресе слушать (по умолчанию 0.0.0.0)
  --data ПАПКА      где хранить мир и игроков (по умолчанию ./data)
  --cert ФАЙЛ       сертификат TLS (fullchain.pem)
  --key ФАЙЛ        ключ TLS (privkey.pem)
  --seed N          зерно нового мира (только при первом запуске)
  --lang ru|en      язык общего мира (по умолчанию ru)
  --plugins ПАПКА   плагины Blackwire для сервера
  --max-accounts N  сколько учётных записей принять (по умолчанию 1000)
  --max-scripts N   скриптов одновременно у игрока (по умолчанию 6)
  --version         версия
";

/// Сертификат перечитывается, когда файл обновился (Let's Encrypt продлевает его сам).
struct Tls {
    cert: String,
    key: String,
    stamp: Option<SystemTime>,
    cfg: Option<Arc<blackwire_wire::TlsServerConfig>>,
}

impl Tls {
    fn current(&mut self) -> Option<Arc<blackwire_wire::TlsServerConfig>> {
        let stamp = std::fs::metadata(&self.cert).and_then(|m| m.modified()).ok();
        if self.cfg.is_none() || stamp != self.stamp {
            match blackwire_wire::server_config(&self.cert, &self.key) {
                Ok(c) => {
                    self.cfg = Some(c);
                    self.stamp = stamp;
                    println!("сертификат TLS загружен: {}", self.cert);
                }
                Err(e) => eprintln!("сертификат TLS не прочитан: {}", e),
            }
        }
        self.cfg.clone()
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut port: u16 = blackwire_wire::DEFAULT_PORT;
    let mut bind = "0.0.0.0".to_string();
    let mut data = PathBuf::from("data");
    let mut cert: Option<String> = None;
    let mut key: Option<String> = None;
    let mut seed: Option<u64> = None;
    let mut lang = "ru".to_string();
    let mut plugins: Option<PathBuf> = None;
    let mut max_accounts = 1000usize;
    let mut max_scripts = 6usize;
    let mut i = 0;
    while i < args.len() {
        let next = args.get(i + 1).cloned().unwrap_or_default();
        match args[i].as_str() {
            "--port" => port = next.parse().unwrap_or(port),
            "--bind" => bind = next,
            "--data" => data = PathBuf::from(next),
            "--cert" => cert = Some(next),
            "--key" => key = Some(next),
            "--seed" => seed = next.parse().ok(),
            "--lang" => lang = next,
            "--plugins" => plugins = Some(PathBuf::from(next)),
            "--max-accounts" => max_accounts = next.parse().unwrap_or(max_accounts),
            "--max-scripts" => max_scripts = next.parse().unwrap_or(max_scripts),
            "--version" | "-V" => {
                println!("blackwire-server {} (протокол {})", env!("CARGO_PKG_VERSION"), blackwire_wire::PROTOCOL);
                return;
            }
            "--help" | "-h" => {
                print!("{}", HELP);
                return;
            }
            other => {
                eprintln!("неизвестный ключ: {}\n\n{}", other, HELP);
                std::process::exit(2);
            }
        }
        i += if args[i].starts_with("--") && !matches!(args[i].as_str(), "--version" | "--help") { 2 } else { 1 };
    }
    blackwire_core::i18n::set(&lang);
    let seed = seed.unwrap_or_else(accounts::random_u64);
    let cfg = Config { data_dir: data, lang: blackwire_core::i18n::code().to_string(), plugins, max_accounts, max_scripts };
    let hub = match Hub::open(cfg, seed) {
        Ok(h) => Arc::new(h),
        Err(e) => {
            eprintln!("мир не открыт: {}", e);
            std::process::exit(1);
        }
    };
    let tls = match (cert, key) {
        (Some(c), Some(k)) => {
            let mut t = Tls { cert: c, key: k, stamp: None, cfg: None };
            if t.current().is_none() {
                std::process::exit(1);
            }
            Some(Mutex::new(t))
        }
        _ => {
            println!("внимание: без --cert/--key связь идёт без шифрования");
            None
        }
    };
    let listener = match TcpListener::bind((bind.as_str(), port)) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("порт {} не открыт: {}", port, e);
            std::process::exit(1);
        }
    };
    tick::start(hub.clone());
    // сохранить мир при остановке службы (SIGTERM приходит как обычный выход нечасто,
    // поэтому мир и так пишется раз в минуту)
    println!("Blackwire server {} слушает {}:{}{}", env!("CARGO_PKG_VERSION"), bind, port, if tls.is_some() { " (TLS)" } else { "" });
    for stream in listener.incoming() {
        let Ok(sock) = stream else { continue };
        let cfg = tls.as_ref().and_then(|t| t.lock().ok().and_then(|mut t| t.current()));
        if tls.is_some() && cfg.is_none() {
            continue;
        }
        let hub = hub.clone();
        let _ = std::thread::Builder::new().name("conn".into()).spawn(move || conn::serve(hub, sock, cfg));
    }
}
