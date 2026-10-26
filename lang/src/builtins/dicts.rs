//! Методы словарей: `keys values items get pop has clear copy update` (+ `setdefault`).

use crate::error::rt;
use crate::interp::ops::iter_to_vec;
use crate::interp::{Interpreter, R};
use crate::value::{Call, DictRef, Key, Value};
use crate::Host;

pub const METHODS: &[&str] = &["keys", "values", "items", "get", "pop", "has", "clear", "copy", "update", "setdefault"];

fn key_of(c: &Call, v: &Value) -> R<Key> {
    Key::from_value(v).map_err(|m| c.err(m))
}

pub fn call(_: &mut Interpreter, _: &mut dyn Host, d: &DictRef, name: &str, c: Call) -> R<Value> {
    match name {
        "keys" => {
            c.check("keys", 0, 0)?;
            Ok(Value::list(d.borrow().keys()))
        }
        "values" => {
            c.check("values", 0, 0)?;
            Ok(Value::list(d.borrow().values()))
        }
        "items" => {
            c.check("items", 0, 0)?;
            let items = d.borrow().iter().map(|(k, v)| Value::list(vec![k.to_value(), v.clone()])).collect();
            Ok(Value::list(items))
        }
        "get" => {
            c.check("get", 1, 2)?;
            let key = key_of(&c, &c.args[0])?;
            match d.borrow().get(&key) {
                Some(v) => Ok(v.clone()),
                None => Ok(c.arg(1).cloned().unwrap_or(Value::Null)),
            }
        }
        "pop" => {
            c.check("pop", 1, 2)?;
            let key = key_of(&c, &c.args[0])?;
            match d.borrow_mut().remove(&key) {
                Some(v) => Ok(v),
                None => match c.arg(1) {
                    Some(def) => Ok(def.clone()),
                    None => Err(c.err(format!("ключ не найден: {}", c.args[0].repr()))),
                },
            }
        }
        "has" => {
            c.check("has", 1, 1)?;
            match Key::from_value(&c.args[0]) {
                Ok(k) => Ok(Value::Bool(d.borrow().contains(&k))),
                Err(_) => Ok(Value::Bool(false)),
            }
        }
        "clear" => {
            c.check("clear", 0, 0)?;
            d.borrow_mut().clear();
            Ok(Value::Null)
        }
        "copy" => {
            c.check("copy", 0, 0)?;
            Ok(Value::dict(d.borrow().clone()))
        }
        "update" => {
            c.check("update", 0, 1)?;
            if let Some(src) = c.arg(0) {
                match src {
                    Value::Dict(other) => {
                        let pairs: Vec<(Key, Value)> = other.borrow().iter().cloned().collect();
                        let mut me = d.borrow_mut();
                        for (k, v) in pairs {
                            me.insert(k, v);
                        }
                    }
                    Value::List(_) => {
                        let pairs = iter_to_vec(src, c.line)?;
                        let mut me = d.borrow_mut();
                        for p in pairs {
                            let kv = iter_to_vec(&p, c.line)?;
                            if kv.len() != 2 {
                                return Err(c.err("update() ожидает словарь или список пар"));
                            }
                            me.insert(key_of(&c, &kv[0])?, kv[1].clone());
                        }
                    }
                    other => return Err(c.err(format!("update() не принимает {}", other.type_name()))),
                }
            }
            let mut me = d.borrow_mut();
            for (k, v) in &c.kwargs {
                me.insert(Key::Str(k.clone()), v.clone());
            }
            Ok(Value::Null)
        }
        "setdefault" => {
            c.check("setdefault", 1, 2)?;
            let key = key_of(&c, &c.args[0])?;
            let mut me = d.borrow_mut();
            if let Some(v) = me.get(&key) {
                return Ok(v.clone());
            }
            let v = c.arg(1).cloned().unwrap_or(Value::Null);
            me.insert(key, v.clone());
            Ok(v)
        }
        _ => Err(rt(c.line, format!("нет такого метода у словаря: {}", name))),
    }
}
