//! Методы списков: `append pop insert remove index count sort reverse clear extend copy`.

use super::{arg_int, sort_values};
use crate::error::rt;
use crate::interp::ops::{iter_to_vec, norm_index};
use crate::interp::{Interpreter, R};
use crate::value::{Call, ListRef, Value};
use crate::Host;

pub const METHODS: &[&str] =
    &["append", "pop", "insert", "remove", "index", "count", "sort", "reverse", "clear", "extend", "copy"];

pub fn call(interp: &mut Interpreter, host: &mut dyn Host, l: &ListRef, name: &str, c: Call) -> R<Value> {
    match name {
        "append" => {
            c.check("append", 1, 1)?;
            l.borrow_mut().push(c.args[0].clone());
            Ok(Value::Null)
        }
        "pop" => {
            c.check("pop", 0, 1)?;
            let mut v = l.borrow_mut();
            if v.is_empty() {
                return Err(c.err("pop из пустого списка"));
            }
            let i = match c.arg(0) {
                Some(x) => arg_int(x, c.line)?,
                None => -1,
            };
            match norm_index(i, v.len()) {
                Some(j) => Ok(v.remove(j)),
                None => Err(c.err("индекс списка вне диапазона")),
            }
        }
        "insert" => {
            c.check("insert", 2, 2)?;
            let i = arg_int(&c.args[0], c.line)?;
            let mut v = l.borrow_mut();
            let n = v.len() as i64;
            let j = if i < 0 { (i + n).max(0) } else { i.min(n) } as usize;
            v.insert(j, c.args[1].clone());
            Ok(Value::Null)
        }
        "remove" => {
            c.check("remove", 1, 1)?;
            let mut v = l.borrow_mut();
            match v.iter().position(|x| x.py_eq(&c.args[0])) {
                Some(i) => {
                    v.remove(i);
                    Ok(Value::Null)
                }
                None => Err(c.err(format!("значение не найдено в списке: {}", c.args[0].repr()))),
            }
        }
        "index" => {
            c.check("index", 1, 1)?;
            match l.borrow().iter().position(|x| x.py_eq(&c.args[0])) {
                Some(i) => Ok(Value::Int(i as i64)),
                None => Err(c.err(format!("значение не найдено в списке: {}", c.args[0].repr()))),
            }
        }
        "count" => {
            c.check("count", 1, 1)?;
            Ok(Value::Int(l.borrow().iter().filter(|x| x.py_eq(&c.args[0])).count() as i64))
        }
        "sort" => {
            c.check("sort", 0, 0)?;
            c.allow_kwargs("sort", &["key", "reverse"])?;
            let reverse = c.kw("reverse").map_or(false, |v| v.truthy());
            // вынимаем содержимое, чтобы функция-ключ могла читать список без конфликта заимствований
            let mut items = std::mem::take(&mut *l.borrow_mut());
            let r = sort_values(interp, host, &mut items, c.kw("key"), reverse, c.line);
            *l.borrow_mut() = items;
            r?;
            Ok(Value::Null)
        }
        "reverse" => {
            c.check("reverse", 0, 0)?;
            l.borrow_mut().reverse();
            Ok(Value::Null)
        }
        "clear" => {
            c.check("clear", 0, 0)?;
            l.borrow_mut().clear();
            Ok(Value::Null)
        }
        "extend" => {
            c.check("extend", 1, 1)?;
            let items = iter_to_vec(&c.args[0], c.line)?;
            l.borrow_mut().extend(items);
            Ok(Value::Null)
        }
        "copy" => {
            c.check("copy", 0, 0)?;
            Ok(Value::list(l.borrow().clone()))
        }
        _ => Err(rt(c.line, format!("нет такого метода у списка: {}", name))),
    }
}
