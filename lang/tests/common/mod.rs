//! Тестовый хост: печать складывается в вектор, игровые функции отвечают заготовками.
#![allow(dead_code)]

use std::cell::Cell;
use std::collections::{HashMap, VecDeque};

use blackwire_lang::value::{Dict, Key};
use blackwire_lang::{Host, LangError, Value};

#[derive(Default)]
pub struct TestHost {
    pub out: Vec<String>,
    pub calls: Vec<(String, Vec<Value>)>,
    pub inputs: VecDeque<String>,
    /// После какого по счёту вопроса `should_stop` отвечать «да».
    pub stop_after: Option<usize>,
    pub stop_checks: Cell<usize>,
    pub slept: Vec<u64>,
    pub files: HashMap<String, String>,
}

fn dict(pairs: &[(&str, Value)]) -> Value {
    let mut d = Dict::new();
    for (k, v) in pairs {
        d.insert(Key::Str((*k).into()), v.clone());
    }
    Value::dict(d)
}

impl Host for TestHost {
    fn print(&mut self, text: &str) {
        self.out.push(text.to_string());
    }

    fn input(&mut self, _prompt: &str) -> Result<String, LangError> {
        self.inputs.pop_front().ok_or_else(|| LangError::runtime(0, "ввод закрыт"))
    }

    fn call(&mut self, name: &str, args: Vec<Value>) -> Result<Value, LangError> {
        self.calls.push((name.to_string(), args.clone()));
        match name {
            "sys.hostname" => Ok(Value::str("node-1")),
            "sys.is_root" => Ok(Value::Bool(false)),
            "net.ping" => match &args[0] {
                Value::Str(ip) if &**ip == "10.0.0.9" => Ok(Value::Null),
                _ => Ok(Value::Int(12)),
            },
            "net.scan" => Ok(Value::list(vec![
                dict(&[("port", Value::Int(22)), ("service", Value::str("ssh"))]),
                dict(&[("port", Value::Int(80)), ("service", Value::str("http"))]),
            ])),
            "fs.read" => {
                let path = args[0].to_display_string();
                self.files
                    .get(&path)
                    .map(|s| Value::str(s))
                    .ok_or_else(|| LangError::runtime(0, format!("нет такого файла: {}", path)))
            }
            "fs.write" => {
                self.files.insert(args[0].to_display_string(), args[1].to_display_string());
                Ok(Value::Null)
            }
            "shell.exec" => Ok(dict(&[("out", Value::str("done")), ("ok", Value::Bool(true))])),
            _ => Err(LangError::runtime(0, format!("неизвестная функция {}", name))),
        }
    }

    fn should_stop(&self) -> bool {
        self.stop_checks.set(self.stop_checks.get() + 1);
        self.stop_after.map_or(false, |n| self.stop_checks.get() >= n)
    }

    fn sleep(&mut self, ms: u64) {
        self.slept.push(ms);
    }
}

pub fn run_with(src: &str, args: Vec<String>) -> (Result<Value, LangError>, TestHost) {
    let mut host = TestHost::default();
    let r = blackwire_lang::run(src, &mut host, args);
    (r, host)
}

pub fn run(src: &str) -> (Result<Value, LangError>, TestHost) {
    run_with(src, vec!["test.adr".to_string()])
}

/// Выполнить и вернуть напечатанные строки; ошибка — паника с текстом.
pub fn out(src: &str) -> Vec<String> {
    let (r, host) = run(src);
    if let Err(e) = r {
        panic!("скрипт упал: {}\nвывод: {:?}", e, host.out);
    }
    host.out
}

/// Выполнить и вернуть ошибку; успех — паника.
pub fn err(src: &str) -> LangError {
    let (r, host) = run(src);
    match r {
        Ok(v) => panic!("ожидалась ошибка, получено {:?}, вывод {:?}", v, host.out),
        Err(e) => e,
    }
}

pub fn lines(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| s.to_string()).collect()
}
