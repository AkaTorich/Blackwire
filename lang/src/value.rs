//! Значения времени исполнения Adder: типы, истинность, равенство и порядок
//! (как в Python: int и float сравниваются по числу), текстовое представление
//! (`repr` — с кавычками у строк, `to_display_string` — без), обмен с хостом через JSON.
//! Словарь `Dict` хранит порядок вставки сам (вектор пар + индекс по ключу).

use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::rc::Rc;

use crate::ast::FuncDef;
use crate::error::Exc;
use crate::interp::{Env, Interpreter};
use crate::Host;

pub type ListRef = Rc<RefCell<Vec<Value>>>;
pub type DictRef = Rc<RefCell<Dict>>;

#[derive(Clone)]
pub enum Value {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(Rc<str>),
    List(ListRef),
    Dict(DictRef),
    /// Пользовательская функция с замыканием.
    Func(Rc<Function>),
    /// Встроенная функция.
    Builtin(Rc<Builtin>),
    /// Метод строки/списка/словаря, привязанный к объекту (`f = s.upper`).
    Method(Rc<BoundMethod>),
    /// Функция игрового модуля: вызов `Native("fs.read")` → `host.call("fs.read", args)`.
    Native(String),
    /// Пространство имён: `sys`, `fs`, `net`, `shell`, `math`, `json`, `time`, `random`.
    Module(String),
}

/// Пользовательская функция: определение, вычисленные значения по умолчанию и замыкание.
pub struct Function {
    pub def: Rc<FuncDef>,
    pub defaults: Vec<Option<Value>>,
    pub closure: Rc<RefCell<Env>>,
}

/// Аргументы вызова встроенной функции.
pub struct Call {
    pub args: Vec<Value>,
    pub kwargs: Vec<(Rc<str>, Value)>,
    pub line: u32,
}

pub type BuiltinFn = fn(&mut Interpreter, &mut dyn Host, Call) -> Result<Value, Exc>;

pub struct Builtin {
    pub name: &'static str,
    pub func: BuiltinFn,
}

pub struct BoundMethod {
    pub recv: Value,
    pub name: Rc<str>,
}

/// Ключ словаря: строка или число. `true`/`false` и целые float сводятся к Int.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Key {
    Str(Rc<str>),
    Int(i64),
    Float(u64),
}

impl Key {
    pub fn from_value(v: &Value) -> Result<Key, String> {
        match v {
            Value::Str(s) => Ok(Key::Str(s.clone())),
            Value::Int(n) => Ok(Key::Int(*n)),
            Value::Bool(b) => Ok(Key::Int(*b as i64)),
            Value::Float(f) => {
                if f.fract() == 0.0 && f.abs() < 9.0e18 {
                    Ok(Key::Int(*f as i64))
                } else {
                    Ok(Key::Float(f.to_bits()))
                }
            }
            other => Err(format!("ключ словаря должен быть строкой или числом, а не {}", other.type_name())),
        }
    }

    pub fn to_value(&self) -> Value {
        match self {
            Key::Str(s) => Value::Str(s.clone()),
            Key::Int(n) => Value::Int(*n),
            Key::Float(b) => Value::Float(f64::from_bits(*b)),
        }
    }

    /// Ключ в виде строки (для JSON).
    pub fn to_string_key(&self) -> String {
        match self {
            Key::Str(s) => s.to_string(),
            other => other.to_value().to_display_string(),
        }
    }
}

/// Словарь с сохранением порядка вставки.
#[derive(Clone, Default)]
pub struct Dict {
    entries: Vec<(Key, Value)>,
    index: HashMap<Key, usize>,
}

impl Dict {
    pub fn new() -> Self {
        Dict::default()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn get(&self, key: &Key) -> Option<&Value> {
        self.index.get(key).map(|&i| &self.entries[i].1)
    }

    pub fn contains(&self, key: &Key) -> bool {
        self.index.contains_key(key)
    }

    /// Вставка: существующий ключ остаётся на своём месте.
    pub fn insert(&mut self, key: Key, value: Value) {
        if let Some(&i) = self.index.get(&key) {
            self.entries[i].1 = value;
        } else {
            self.index.insert(key.clone(), self.entries.len());
            self.entries.push((key, value));
        }
    }

    pub fn remove(&mut self, key: &Key) -> Option<Value> {
        let i = self.index.remove(key)?;
        let (_, v) = self.entries.remove(i);
        for (k, _) in &self.entries[i..] {
            if let Some(pos) = self.index.get_mut(k) {
                *pos -= 1;
            }
        }
        Some(v)
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.index.clear();
    }

    pub fn iter(&self) -> impl Iterator<Item = &(Key, Value)> {
        self.entries.iter()
    }

    pub fn keys(&self) -> Vec<Value> {
        self.entries.iter().map(|(k, _)| k.to_value()).collect()
    }

    pub fn values(&self) -> Vec<Value> {
        self.entries.iter().map(|(_, v)| v.clone()).collect()
    }
}

impl Value {
    pub fn str(s: &str) -> Value {
        Value::Str(Rc::from(s))
    }

    pub fn list(items: Vec<Value>) -> Value {
        Value::List(Rc::new(RefCell::new(items)))
    }

    pub fn dict(d: Dict) -> Value {
        Value::Dict(Rc::new(RefCell::new(d)))
    }

    /// Имя типа — то, что возвращает `type(x)`.
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Null => "null",
            Value::Bool(_) => "bool",
            Value::Int(_) => "int",
            Value::Float(_) => "float",
            Value::Str(_) => "str",
            Value::List(_) => "list",
            Value::Dict(_) => "dict",
            Value::Func(_) | Value::Builtin(_) | Value::Method(_) | Value::Native(_) => "func",
            Value::Module(_) => "module",
        }
    }

    pub fn truthy(&self) -> bool {
        match self {
            Value::Null => false,
            Value::Bool(b) => *b,
            Value::Int(n) => *n != 0,
            Value::Float(f) => *f != 0.0,
            Value::Str(s) => !s.is_empty(),
            Value::List(l) => !l.borrow().is_empty(),
            Value::Dict(d) => !d.borrow().is_empty(),
            _ => true,
        }
    }

    /// Число для арифметики: int/bool → i64, float → f64.
    pub fn as_number(&self) -> Option<Num> {
        match self {
            Value::Int(n) => Some(Num::Int(*n)),
            Value::Bool(b) => Some(Num::Int(*b as i64)),
            Value::Float(f) => Some(Num::Float(*f)),
            _ => None,
        }
    }

    /// Равенство как в Python.
    pub fn py_eq(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::Null, Value::Null) => true,
            (Value::Str(a), Value::Str(b)) => a == b,
            (Value::List(a), Value::List(b)) => {
                if Rc::ptr_eq(a, b) {
                    return true;
                }
                let (a, b) = (a.borrow(), b.borrow());
                a.len() == b.len() && a.iter().zip(b.iter()).all(|(x, y)| x.py_eq(y))
            }
            (Value::Dict(a), Value::Dict(b)) => {
                if Rc::ptr_eq(a, b) {
                    return true;
                }
                let (a, b) = (a.borrow(), b.borrow());
                a.len() == b.len() && a.iter().all(|(k, v)| b.get(k).map_or(false, |w| v.py_eq(w)))
            }
            (Value::Func(a), Value::Func(b)) => Rc::ptr_eq(a, b),
            (Value::Builtin(a), Value::Builtin(b)) => a.name == b.name,
            (Value::Native(a), Value::Native(b)) => a == b,
            (Value::Module(a), Value::Module(b)) => a == b,
            (Value::Method(a), Value::Method(b)) => a.name == b.name && a.recv.py_eq(&b.recv),
            _ => match (self.as_number(), other.as_number()) {
                (Some(a), Some(b)) => a.cmp_num(&b) == Some(Ordering::Equal),
                _ => false,
            },
        }
    }

    /// Порядок как в Python; `None` — значения несравнимы.
    pub fn py_cmp(&self, other: &Value) -> Option<Ordering> {
        match (self, other) {
            (Value::Str(a), Value::Str(b)) => Some(a.cmp(b)),
            (Value::List(a), Value::List(b)) => {
                let (a, b) = (a.borrow(), b.borrow());
                for (x, y) in a.iter().zip(b.iter()) {
                    match x.py_cmp(y)? {
                        Ordering::Equal => continue,
                        ord => return Some(ord),
                    }
                }
                Some(a.len().cmp(&b.len()))
            }
            _ => match (self.as_number(), other.as_number()) {
                (Some(a), Some(b)) => a.cmp_num(&b),
                _ => None,
            },
        }
    }

    /// Представление «как в коде»: строки в кавычках.
    pub fn repr(&self) -> String {
        match self {
            Value::Str(s) => quote_str(s),
            Value::List(l) => {
                let items: Vec<String> = l.borrow().iter().map(|v| v.repr()).collect();
                format!("[{}]", items.join(", "))
            }
            Value::Dict(d) => {
                let items: Vec<String> =
                    d.borrow().iter().map(|(k, v)| format!("{}: {}", k.to_value().repr(), v.repr())).collect();
                format!("{{{}}}", items.join(", "))
            }
            other => other.to_display_string(),
        }
    }

    /// Представление для `print`/`str`: строки без кавычек (внутри списков — с кавычками).
    pub fn to_display_string(&self) -> String {
        match self {
            Value::Null => "null".to_string(),
            Value::Bool(b) => if *b { "true".to_string() } else { "false".to_string() },
            Value::Int(n) => n.to_string(),
            Value::Float(f) => fmt_float(*f),
            Value::Str(s) => s.to_string(),
            Value::List(_) | Value::Dict(_) => self.repr(),
            Value::Func(f) => format!("<функция {}>", f.def.name),
            Value::Builtin(b) => format!("<встроенная функция {}>", b.name),
            Value::Method(m) => format!("<метод {}>", m.name),
            Value::Native(n) => format!("<функция {}>", n),
            Value::Module(m) => format!("<модуль {}>", m),
        }
    }

    /// В JSON для обмена с хостом. Функции и модули становятся строками.
    pub fn to_json(&self) -> serde_json::Value {
        use serde_json::Value as J;
        match self {
            Value::Null => J::Null,
            Value::Bool(b) => J::Bool(*b),
            Value::Int(n) => J::from(*n),
            Value::Float(f) => serde_json::Number::from_f64(*f).map(J::Number).unwrap_or(J::Null),
            Value::Str(s) => J::String(s.to_string()),
            Value::List(l) => J::Array(l.borrow().iter().map(|v| v.to_json()).collect()),
            Value::Dict(d) => {
                let mut map = serde_json::Map::new();
                for (k, v) in d.borrow().iter() {
                    map.insert(k.to_string_key(), v.to_json());
                }
                J::Object(map)
            }
            other => J::String(other.to_display_string()),
        }
    }

    /// Из JSON (ответы хоста).
    pub fn from_json(j: &serde_json::Value) -> Value {
        use serde_json::Value as J;
        match j {
            J::Null => Value::Null,
            J::Bool(b) => Value::Bool(*b),
            J::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Value::Int(i)
                } else {
                    Value::Float(n.as_f64().unwrap_or(0.0))
                }
            }
            J::String(s) => Value::str(s),
            J::Array(items) => Value::list(items.iter().map(Value::from_json).collect()),
            J::Object(map) => {
                let mut d = Dict::new();
                for (k, v) in map {
                    d.insert(Key::Str(Rc::from(k.as_str())), Value::from_json(v));
                }
                Value::dict(d)
            }
        }
    }
}

impl std::fmt::Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.repr())
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Value) -> bool {
        self.py_eq(other)
    }
}

/// Число для арифметики и сравнений.
#[derive(Clone, Copy, Debug)]
pub enum Num {
    Int(i64),
    Float(f64),
}

impl Num {
    pub fn as_f64(self) -> f64 {
        match self {
            Num::Int(n) => n as f64,
            Num::Float(f) => f,
        }
    }

    pub fn cmp_num(&self, other: &Num) -> Option<Ordering> {
        match (self, other) {
            (Num::Int(a), Num::Int(b)) => Some(a.cmp(b)),
            _ => self.as_f64().partial_cmp(&other.as_f64()),
        }
    }
}

/// Запись float так, чтобы он отличался от целого: `1.0`, `2.5`, `inf`, `nan`.
pub fn fmt_float(f: f64) -> String {
    if f.is_nan() {
        return "nan".to_string();
    }
    if f.is_infinite() {
        return if f > 0.0 { "inf".to_string() } else { "-inf".to_string() };
    }
    let s = format!("{}", f);
    if s.contains('.') || s.contains('e') {
        s
    } else {
        s + ".0"
    }
}

/// Строка в двойных кавычках с экранированием.
pub fn quote_str(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}
