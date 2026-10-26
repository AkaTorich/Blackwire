//! Встроенные функции и модули. Здесь — установка глобальных имён, доступ к
//! атрибутам модулей (`math.sqrt`, `sys.args`, `fs.read` → `Native`) и общие
//! помощники для разбора аргументов. Сами функции — по файлам:
//! `core.rs` (print, len, range, sorted…), `strings.rs`, `lists.rs`, `dicts.rs`
//! (методы), `format.rs` (формат f-строк и `str.format`), `modules.rs`
//! (`math`, `time`, `random`), `json.rs` (`json.loads`/`json.dumps`).

pub mod core;
pub mod dicts;
pub mod format;
pub mod json;
pub mod lists;
pub mod modules;
pub mod strings;

use std::rc::Rc;

use crate::error::rt;
use crate::interp::{Env, Interpreter, R};
use crate::value::{Builtin, BuiltinFn, Call, Value};

/// Игровые модули: любой атрибут — функция хоста `модуль.имя`.
pub const GAME_MODULES: &[&str] = &["sys", "fs", "net", "shell", "plugin"];
/// Модули, реализованные внутри языка.
pub const LOCAL_MODULES: &[&str] = &["math", "json", "time", "random"];

pub fn builtin(name: &'static str, func: BuiltinFn) -> Value {
    Value::Builtin(Rc::new(Builtin { name, func }))
}

/// Положить встроенные функции и модули в глобальную область.
pub fn install(env: &mut Env) {
    for (name, f) in core::TABLE {
        env.vars.insert(Rc::from(*name), builtin(name, *f));
    }
    for m in GAME_MODULES.iter().chain(LOCAL_MODULES) {
        env.vars.insert(Rc::from(*m), Value::Module(m.to_string()));
    }
}

/// Атрибут модуля.
pub fn module_attr(interp: &Interpreter, module: &str, attr: &str, line: u32) -> R<Value> {
    if module == "sys" && attr == "args" {
        return Ok(Value::list(interp.args.iter().map(|a| Value::str(a)).collect()));
    }
    if GAME_MODULES.contains(&module) {
        return Ok(Value::Native(format!("{}.{}", module, attr)));
    }
    let found = match module {
        "math" => modules::math_attr(attr),
        "time" => modules::time_attr(attr),
        "random" => modules::random_attr(attr),
        "json" => json::attr(attr),
        _ => None,
    };
    found.ok_or_else(|| rt(line, format!("модуль {} не имеет атрибута '{}'", module, attr)))
}

/// Есть ли у строки/списка/словаря метод с таким именем.
pub fn has_method(obj: &Value, name: &str) -> bool {
    match obj {
        Value::Str(_) => strings::METHODS.contains(&name),
        Value::List(_) => lists::METHODS.contains(&name),
        Value::Dict(_) => dicts::METHODS.contains(&name),
        _ => false,
    }
}

/// «у списка», «у строки», «у словаря» — для текста ошибок.
pub fn type_genitive(obj: &Value) -> &'static str {
    match obj {
        Value::Str(_) => "строки",
        Value::List(_) => "списка",
        Value::Dict(_) => "словаря",
        _ => "значения",
    }
}

/// Найти функцию в таблице по имени.
pub fn lookup_table(table: &[(&'static str, BuiltinFn)], name: &str) -> Option<Value> {
    table.iter().find(|(n, _)| *n == name).map(|(n, f)| builtin(n, *f))
}

impl Call {
    /// Проверить число позиционных аргументов.
    pub fn check(&self, fname: &str, min: usize, max: usize) -> R<()> {
        let n = self.args.len();
        if n < min || n > max {
            let expected = if min == max {
                format!("{}", min)
            } else {
                format!("от {} до {}", min, max)
            };
            return Err(rt(
                self.line,
                format!("функция {}() принимает {} аргументов, передано {}", fname, expected, n),
            ));
        }
        Ok(())
    }

    /// Разрешённые именованные аргументы.
    pub fn allow_kwargs(&self, fname: &str, allowed: &[&str]) -> R<()> {
        for (k, _) in &self.kwargs {
            if !allowed.contains(&&**k) {
                return Err(rt(self.line, format!("функция {}() не имеет параметра '{}'", fname, k)));
            }
        }
        Ok(())
    }

    pub fn arg(&self, i: usize) -> Option<&Value> {
        self.args.get(i)
    }

    pub fn kw(&self, name: &str) -> Option<&Value> {
        self.kwargs.iter().find(|(k, _)| &**k == name).map(|(_, v)| v)
    }

    /// Аргумент по позиции или по имени.
    pub fn get(&self, i: usize, name: &str) -> Option<&Value> {
        self.args.get(i).or_else(|| self.kw(name))
    }

    pub fn err(&self, msg: impl Into<String>) -> crate::error::Exc {
        rt(self.line, msg)
    }
}

pub fn arg_int(v: &Value, line: u32) -> R<i64> {
    match v {
        Value::Int(n) => Ok(*n),
        Value::Bool(b) => Ok(*b as i64),
        Value::Float(f) if f.fract() == 0.0 => Ok(*f as i64),
        other => Err(rt(line, format!("ожидалось целое число, получено {}", other.type_name()))),
    }
}

pub fn arg_float(v: &Value, line: u32) -> R<f64> {
    match v.as_number() {
        Some(n) => Ok(n.as_f64()),
        None => Err(rt(line, format!("ожидалось число, получено {}", v.type_name()))),
    }
}

pub fn arg_str(v: &Value, line: u32) -> R<Rc<str>> {
    match v {
        Value::Str(s) => Ok(s.clone()),
        other => Err(rt(line, format!("ожидалась строка, получено {}", other.type_name()))),
    }
}

pub fn arg_list(v: &Value, line: u32) -> R<crate::value::ListRef> {
    match v {
        Value::List(l) => Ok(l.clone()),
        other => Err(rt(line, format!("ожидался список, получено {}", other.type_name()))),
    }
}

pub fn arg_dict(v: &Value, line: u32) -> R<crate::value::DictRef> {
    match v {
        Value::Dict(d) => Ok(d.clone()),
        other => Err(rt(line, format!("ожидался словарь, получено {}", other.type_name()))),
    }
}

/// Отсортировать с ключом и порядком; несравнимые элементы — ошибка.
pub fn sort_values(
    interp: &mut Interpreter,
    host: &mut dyn crate::Host,
    items: &mut Vec<Value>,
    key: Option<&Value>,
    reverse: bool,
    line: u32,
) -> R<()> {
    let keys: Vec<Value> = match key {
        Some(f) if !matches!(f, Value::Null) => {
            let mut ks = Vec::with_capacity(items.len());
            for it in items.iter() {
                ks.push(interp.call_simple(f, vec![it.clone()], line, host)?);
            }
            ks
        }
        _ => items.clone(),
    };
    let mut order: Vec<usize> = (0..items.len()).collect();
    let mut failed: Option<(String, String)> = None;
    order.sort_by(|&a, &b| match keys[a].py_cmp(&keys[b]) {
        Some(o) => o,
        None => {
            if failed.is_none() {
                failed = Some((keys[a].type_name().to_string(), keys[b].type_name().to_string()));
            }
            std::cmp::Ordering::Equal
        }
    });
    if let Some((a, b)) = failed {
        return Err(rt(line, format!("нельзя сравнить {} и {}", a, b)));
    }
    if reverse {
        order.reverse();
    }
    let sorted: Vec<Value> = order.into_iter().map(|i| items[i].clone()).collect();
    *items = sorted;
    Ok(())
}
