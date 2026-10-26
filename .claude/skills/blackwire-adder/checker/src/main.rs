//! adder-check — проверка скрипта Adder (Blackwire) вне игры.
//!
//!   adder-check файл.adr            — только синтаксис
//!   adder-check файл.adr --run [арг…] — синтаксис + пробный запуск на заглушках
//!
//! При пробном запуске игровые функции отвечают пустыми значениями нужного вида,
//! а вызов несуществующей функции (опечатка) — ошибка с номером строки.

use std::time::{Duration, Instant};

use blackwire_lang::{Host, LangError, Value};
use serde_json::json;

/// Все игровые функции и вид пустого ответа.
fn stub(name: &str) -> Option<serde_json::Value> {
    Some(match name {
        "sys.hostname" | "sys.whoami" | "sys.cwd" | "sys.env" | "fs.read" | "net.my_ip" => json!(""),
        "sys.is_root" | "fs.exists" | "fs.is_dir" | "net.connect" => json!(false),
        "sys.users" | "sys.procs" | "sys.missions" | "fs.list" | "net.scan" | "net.hops" | "net.vulns"
        | "net.owned" => json!([]),
        "sys.uptime" | "sys.money" | "sys.debt" | "sys.pid" | "net.heat" | "net.clear_logs" | "time.now" => json!(0),
        "sys.set_env" | "sys.kill" | "sys.notify" | "sys.open" | "fs.write" | "fs.append" | "fs.mkdir"
        | "fs.delete" | "fs.move" | "fs.copy" | "fs.chmod" | "fs.chown" | "net.disconnect" | "net.resolve"
        | "net.ping" | "net.crack" => json!(null),
        "net.http" => json!({"status": 200, "title": "", "body": ""}),
        "net.exploit" | "net.privesc" => json!({"ok": false, "access": "", "user": "", "password": "", "path": "", "data": "", "host": "", "monitored": false}),
        "net.research" => json!({"id":"","name":"","service":"","port":0,"version":"","local":false,"kind":"","difficulty":1,"fingerprint":"","layout":{},"hints":[],"fields":[],"market":"","next":""}),
        "net.probe" => json!({"ok":false,"applied":false,"feedback":"проверка только на заглушке","hint":"","id":"","kind":""}),
        "net.watch" => json!({"stage": "", "progress": 0, "raids": 0, "locked": false, "lead": ""}),
        "net.proxy" => json!({"chain": [], "exit_ip": "", "hidden": false, "monitored": false}),
        "shell.exec" => json!({"out": "", "ok": true}),
        _ => return None,
    })
}

struct StubHost {
    deadline: Instant,
    calls: usize,
}

impl Host for StubHost {
    fn print(&mut self, text: &str) {
        println!("  | {}", text);
    }
    fn input(&mut self, _prompt: &str) -> Result<String, LangError> {
        Ok(String::new())
    }
    fn call(&mut self, name: &str, _args: Vec<Value>) -> Result<Value, LangError> {
        self.calls += 1;
        match stub(name) {
            Some(v) => Ok(Value::from_json(&v)),
            None => Err(LangError::runtime(0, format!("нет такой функции: {}", name))),
        }
    }
    fn should_stop(&self) -> bool {
        Instant::now() > self.deadline
    }
    fn sleep(&mut self, _ms: u64) {}
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(path) = args.first() else {
        eprintln!("использование: adder-check файл.adr [--run [аргументы…]]");
        std::process::exit(2);
    };
    let src = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("не прочитать {}: {}", path, e);
            std::process::exit(2);
        }
    };

    let diags = blackwire_lang::check(&src);
    if !diags.is_empty() {
        for d in &diags {
            println!("СИНТАКСИС строка {}, столбец {}: {}", d.line, d.col, d.message);
        }
        std::process::exit(1);
    }
    println!("синтаксис: OK");

    if args.get(1).map(|a| a == "--run").unwrap_or(false) {
        let mut script_args = vec![path.clone()];
        script_args.extend(args.iter().skip(2).cloned());
        let mut host = StubHost { deadline: Instant::now() + Duration::from_secs(3), calls: 0 };
        match blackwire_lang::run(&src, &mut host, script_args) {
            Ok(_) => println!("пробный запуск: OK (вызовов игры: {})", host.calls),
            Err(e) if host.should_stop() => println!("пробный запуск: остановлен по времени (бесконечный цикл?), строка {}", e.line),
            Err(e) => {
                println!("ИСПОЛНЕНИЕ строка {}: {}", e.line, e.message);
                std::process::exit(1);
            }
        }
    }
}
