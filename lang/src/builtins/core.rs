//! Глобальные встроенные функции: `print`, `input`, `len`, преобразования типов,
//! `range`, `sorted`, `enumerate`, `zip`, `chr/ord/hex/bin/oct`, `sleep`, `exit`…

use std::rc::Rc;

use super::{arg_float, arg_int, arg_str, sort_values};
use crate::error::{rt, Exc};
use crate::interp::ops::iter_to_vec;
use crate::interp::{Interpreter, R};
use crate::value::{BuiltinFn, Call, Dict, Key, Value};
use crate::Host;

pub const TABLE: &[(&str, BuiltinFn)] = &[
    ("print", print),
    ("input", input),
    ("len", len),
    ("str", to_str),
    ("int", to_int),
    ("float", to_float),
    ("bool", to_bool),
    ("list", to_list),
    ("dict", to_dict),
    ("range", range),
    ("type", type_of),
    ("abs", abs),
    ("min", min),
    ("max", max),
    ("sum", sum),
    ("round", round),
    ("sorted", sorted),
    ("reversed", reversed),
    ("enumerate", enumerate),
    ("zip", zip),
    ("any", any),
    ("all", all),
    ("map", map),
    ("filter", filter),
    ("chr", chr),
    ("ord", ord),
    ("hex", hex),
    ("bin", bin),
    ("oct", oct),
    ("repr", repr),
    ("sleep", sleep),
    ("exit", exit),
    ("isinstance", isinstance),
];

fn print(_: &mut Interpreter, host: &mut dyn Host, c: Call) -> R<Value> {
    c.allow_kwargs("print", &["sep", "end"])?;
    let sep = match c.kw("sep") {
        Some(v) => v.to_display_string(),
        None => " ".to_string(),
    };
    let end = match c.kw("end") {
        Some(v) => v.to_display_string(),
        None => String::new(),
    };
    let parts: Vec<String> = c.args.iter().map(|v| v.to_display_string()).collect();
    let mut line = parts.join(&sep);
    line.push_str(&end);
    host.print(&line);
    Ok(Value::Null)
}

fn input(_: &mut Interpreter, host: &mut dyn Host, c: Call) -> R<Value> {
    c.check("input", 0, 1)?;
    let prompt = c.arg(0).map(|v| v.to_display_string()).unwrap_or_default();
    let s = host.input(&prompt).map_err(|e| Exc::Error(e.with_line(c.line)))?;
    Ok(Value::str(&s))
}

fn len(_: &mut Interpreter, _: &mut dyn Host, c: Call) -> R<Value> {
    c.check("len", 1, 1)?;
    let n = match &c.args[0] {
        Value::Str(s) => s.chars().count(),
        Value::List(l) => l.borrow().len(),
        Value::Dict(d) => d.borrow().len(),
        other => return Err(c.err(format!("объект типа {} не имеет длины", other.type_name()))),
    };
    Ok(Value::Int(n as i64))
}

fn to_str(_: &mut Interpreter, _: &mut dyn Host, c: Call) -> R<Value> {
    c.check("str", 0, 1)?;
    Ok(Value::str(&c.arg(0).map(|v| v.to_display_string()).unwrap_or_default()))
}

fn to_int(_: &mut Interpreter, _: &mut dyn Host, c: Call) -> R<Value> {
    c.check("int", 0, 2)?;
    let v = match c.arg(0) {
        None => return Ok(Value::Int(0)),
        Some(v) => v,
    };
    match v {
        Value::Int(n) => Ok(Value::Int(*n)),
        Value::Bool(b) => Ok(Value::Int(*b as i64)),
        Value::Float(f) => {
            if f.is_finite() && f.abs() < 9.2e18 {
                Ok(Value::Int(f.trunc() as i64))
            } else {
                Err(c.err("число не помещается в int"))
            }
        }
        Value::Str(s) => {
            let base = match c.arg(1) {
                Some(b) => arg_int(b, c.line)? as u32,
                None => 10,
            };
            if !(2..=36).contains(&base) {
                return Err(c.err("основание int() должно быть от 2 до 36"));
            }
            let t = s.trim().replace('_', "");
            let t = t.as_str();
            let (neg, digits) = match t.strip_prefix('-') {
                Some(rest) => (true, rest),
                None => (false, t.strip_prefix('+').unwrap_or(t)),
            };
            let digits = match base {
                16 => digits.strip_prefix("0x").or_else(|| digits.strip_prefix("0X")).unwrap_or(digits),
                2 => digits.strip_prefix("0b").or_else(|| digits.strip_prefix("0B")).unwrap_or(digits),
                8 => digits.strip_prefix("0o").or_else(|| digits.strip_prefix("0O")).unwrap_or(digits),
                _ => digits,
            };
            match i64::from_str_radix(digits, base) {
                Ok(n) => Ok(Value::Int(if neg { -n } else { n })),
                Err(_) => Err(c.err(format!("не удалось преобразовать {} в int", v.repr()))),
            }
        }
        other => Err(c.err(format!("int() не принимает {}", other.type_name()))),
    }
}

fn to_float(_: &mut Interpreter, _: &mut dyn Host, c: Call) -> R<Value> {
    c.check("float", 0, 1)?;
    let v = match c.arg(0) {
        None => return Ok(Value::Float(0.0)),
        Some(v) => v,
    };
    match v {
        Value::Str(s) => match s.trim().replace('_', "").parse::<f64>() {
            Ok(f) => Ok(Value::Float(f)),
            Err(_) => Err(c.err(format!("не удалось преобразовать {} в float", v.repr()))),
        },
        other => match other.as_number() {
            Some(n) => Ok(Value::Float(n.as_f64())),
            None => Err(c.err(format!("float() не принимает {}", other.type_name()))),
        },
    }
}

fn to_bool(_: &mut Interpreter, _: &mut dyn Host, c: Call) -> R<Value> {
    c.check("bool", 0, 1)?;
    Ok(Value::Bool(c.arg(0).map_or(false, |v| v.truthy())))
}

fn to_list(_: &mut Interpreter, _: &mut dyn Host, c: Call) -> R<Value> {
    c.check("list", 0, 1)?;
    match c.arg(0) {
        None => Ok(Value::list(Vec::new())),
        Some(v) => Ok(Value::list(iter_to_vec(v, c.line)?)),
    }
}

fn to_dict(_: &mut Interpreter, _: &mut dyn Host, c: Call) -> R<Value> {
    c.check("dict", 0, 1)?;
    let mut d = Dict::new();
    match c.arg(0) {
        None => {}
        Some(Value::Dict(src)) => d = src.borrow().clone(),
        Some(Value::List(items)) => {
            for pair in items.borrow().iter() {
                let kv = iter_to_vec(pair, c.line)?;
                if kv.len() != 2 {
                    return Err(c.err("dict() ожидает список пар [ключ, значение]"));
                }
                let key = Key::from_value(&kv[0]).map_err(|m| c.err(m))?;
                d.insert(key, kv[1].clone());
            }
        }
        Some(other) => return Err(c.err(format!("dict() не принимает {}", other.type_name()))),
    }
    for (k, v) in &c.kwargs {
        d.insert(Key::Str(k.clone()), v.clone());
    }
    Ok(Value::dict(d))
}

/// Границы `range(stop)`, `range(start, stop[, step])`.
pub fn range_bounds(args: &[Value], line: u32) -> R<(i64, i64, i64)> {
    let ints: Vec<i64> = args.iter().map(|a| arg_int(a, line)).collect::<R<_>>()?;
    let (start, stop, step) = match ints.len() {
        1 => (0, ints[0], 1),
        2 => (ints[0], ints[1], 1),
        3 => (ints[0], ints[1], ints[2]),
        n => return Err(rt(line, format!("функция range() принимает от 1 до 3 аргументов, передано {}", n))),
    };
    if step == 0 {
        return Err(rt(line, "шаг range() не может быть нулём"));
    }
    Ok((start, stop, step))
}

fn range(_: &mut Interpreter, _: &mut dyn Host, c: Call) -> R<Value> {
    let (start, stop, step) = range_bounds(&c.args, c.line)?;
    let mut out = Vec::new();
    let mut i = start;
    while (step > 0 && i < stop) || (step < 0 && i > stop) {
        out.push(Value::Int(i));
        i += step;
    }
    Ok(Value::list(out))
}

fn type_of(_: &mut Interpreter, _: &mut dyn Host, c: Call) -> R<Value> {
    c.check("type", 1, 1)?;
    Ok(Value::str(c.args[0].type_name()))
}

fn abs(_: &mut Interpreter, _: &mut dyn Host, c: Call) -> R<Value> {
    c.check("abs", 1, 1)?;
    match &c.args[0] {
        Value::Int(n) => n.checked_abs().map(Value::Int).ok_or_else(|| c.err("переполнение целого")),
        Value::Bool(b) => Ok(Value::Int(*b as i64)),
        Value::Float(f) => Ok(Value::Float(f.abs())),
        other => Err(c.err(format!("abs() не принимает {}", other.type_name()))),
    }
}

/// Общая часть min/max: один список или несколько аргументов, необязательный `key`.
fn extreme(
    interp: &mut Interpreter,
    host: &mut dyn Host,
    c: &Call,
    fname: &str,
    want_greater: bool,
) -> R<Value> {
    c.allow_kwargs(fname, &["key", "default"])?;
    let items = if c.args.len() == 1 {
        iter_to_vec(&c.args[0], c.line)?
    } else {
        c.args.clone()
    };
    if items.is_empty() {
        return match c.kw("default") {
            Some(d) => Ok(d.clone()),
            None => Err(c.err(format!("{}() от пустого списка", fname))),
        };
    }
    let key = c.kw("key").filter(|k| !matches!(k, Value::Null)).cloned();
    let mut best = items[0].clone();
    let mut best_key = match &key {
        Some(f) => interp.call_simple(f, vec![best.clone()], c.line, host)?,
        None => best.clone(),
    };
    for it in items.into_iter().skip(1) {
        let k = match &key {
            Some(f) => interp.call_simple(f, vec![it.clone()], c.line, host)?,
            None => it.clone(),
        };
        let ord = k
            .py_cmp(&best_key)
            .ok_or_else(|| c.err(format!("нельзя сравнить {} и {}", k.type_name(), best_key.type_name())))?;
        let better = if want_greater { ord == std::cmp::Ordering::Greater } else { ord == std::cmp::Ordering::Less };
        if better {
            best = it;
            best_key = k;
        }
    }
    Ok(best)
}

fn min(interp: &mut Interpreter, host: &mut dyn Host, c: Call) -> R<Value> {
    c.check("min", 1, usize::MAX)?;
    extreme(interp, host, &c, "min", false)
}

fn max(interp: &mut Interpreter, host: &mut dyn Host, c: Call) -> R<Value> {
    c.check("max", 1, usize::MAX)?;
    extreme(interp, host, &c, "max", true)
}

fn sum(_: &mut Interpreter, _: &mut dyn Host, c: Call) -> R<Value> {
    c.check("sum", 1, 2)?;
    let items = iter_to_vec(&c.args[0], c.line)?;
    let mut acc = c.arg(1).cloned().unwrap_or(Value::Int(0));
    for it in items {
        acc = crate::interp::ops::binary(crate::ast::BinOp::Add, &acc, &it, c.line)?;
    }
    Ok(acc)
}

/// Округление к ближайшему, при .5 — к чётному (как в Python).
fn round_half_even(x: f64) -> f64 {
    let r = x.round();
    if (x - x.trunc()).abs() == 0.5 && r % 2.0 != 0.0 {
        r - x.signum()
    } else {
        r
    }
}

fn round(_: &mut Interpreter, _: &mut dyn Host, c: Call) -> R<Value> {
    c.check("round", 1, 2)?;
    let digits = match c.arg(1) {
        Some(Value::Null) | None => None,
        Some(v) => Some(arg_int(v, c.line)?),
    };
    match (&c.args[0], digits) {
        (Value::Int(n), None) => Ok(Value::Int(*n)),
        (Value::Int(n), Some(d)) if d >= 0 => Ok(Value::Int(*n)),
        (v, None) => {
            let f = arg_float(v, c.line)?;
            let r = round_half_even(f);
            if r.is_finite() && r.abs() < 9.2e18 {
                Ok(Value::Int(r as i64))
            } else {
                Err(c.err("число не помещается в int"))
            }
        }
        (v, Some(d)) => {
            let f = arg_float(v, c.line)?;
            let m = 10f64.powi(d as i32);
            let r = round_half_even(f * m) / m;
            if let Value::Int(_) = v {
                return Ok(Value::Int(r as i64));
            }
            Ok(Value::Float(r))
        }
    }
}

fn sorted(interp: &mut Interpreter, host: &mut dyn Host, c: Call) -> R<Value> {
    c.check("sorted", 1, 1)?;
    c.allow_kwargs("sorted", &["key", "reverse"])?;
    let mut items = iter_to_vec(&c.args[0], c.line)?;
    let reverse = c.kw("reverse").map_or(false, |v| v.truthy());
    sort_values(interp, host, &mut items, c.kw("key"), reverse, c.line)?;
    Ok(Value::list(items))
}

fn reversed(_: &mut Interpreter, _: &mut dyn Host, c: Call) -> R<Value> {
    c.check("reversed", 1, 1)?;
    let mut items = iter_to_vec(&c.args[0], c.line)?;
    items.reverse();
    Ok(Value::list(items))
}

fn enumerate(_: &mut Interpreter, _: &mut dyn Host, c: Call) -> R<Value> {
    c.check("enumerate", 1, 2)?;
    c.allow_kwargs("enumerate", &["start"])?;
    let start = match c.get(1, "start") {
        Some(v) => arg_int(v, c.line)?,
        None => 0,
    };
    let items = iter_to_vec(&c.args[0], c.line)?;
    Ok(Value::list(
        items
            .into_iter()
            .enumerate()
            .map(|(i, v)| Value::list(vec![Value::Int(start + i as i64), v]))
            .collect(),
    ))
}

fn zip(_: &mut Interpreter, _: &mut dyn Host, c: Call) -> R<Value> {
    let lists: Vec<Vec<Value>> = c.args.iter().map(|a| iter_to_vec(a, c.line)).collect::<R<_>>()?;
    let n = lists.iter().map(|l| l.len()).min().unwrap_or(0);
    let out = (0..n).map(|i| Value::list(lists.iter().map(|l| l[i].clone()).collect())).collect();
    Ok(Value::list(out))
}

fn any(_: &mut Interpreter, _: &mut dyn Host, c: Call) -> R<Value> {
    c.check("any", 1, 1)?;
    Ok(Value::Bool(iter_to_vec(&c.args[0], c.line)?.iter().any(|v| v.truthy())))
}

fn all(_: &mut Interpreter, _: &mut dyn Host, c: Call) -> R<Value> {
    c.check("all", 1, 1)?;
    Ok(Value::Bool(iter_to_vec(&c.args[0], c.line)?.iter().all(|v| v.truthy())))
}

fn map(interp: &mut Interpreter, host: &mut dyn Host, c: Call) -> R<Value> {
    c.check("map", 2, 2)?;
    let items = iter_to_vec(&c.args[1], c.line)?;
    let mut out = Vec::with_capacity(items.len());
    for it in items {
        out.push(interp.call_simple(&c.args[0], vec![it], c.line, host)?);
    }
    Ok(Value::list(out))
}

fn filter(interp: &mut Interpreter, host: &mut dyn Host, c: Call) -> R<Value> {
    c.check("filter", 2, 2)?;
    let items = iter_to_vec(&c.args[1], c.line)?;
    let mut out = Vec::new();
    for it in items {
        let keep = match &c.args[0] {
            Value::Null => it.truthy(),
            f => interp.call_simple(f, vec![it.clone()], c.line, host)?.truthy(),
        };
        if keep {
            out.push(it);
        }
    }
    Ok(Value::list(out))
}

fn chr(_: &mut Interpreter, _: &mut dyn Host, c: Call) -> R<Value> {
    c.check("chr", 1, 1)?;
    let n = arg_int(&c.args[0], c.line)?;
    let ch = u32::try_from(n).ok().and_then(char::from_u32).ok_or_else(|| c.err("chr(): код вне диапазона"))?;
    Ok(Value::str(&ch.to_string()))
}

fn ord(_: &mut Interpreter, _: &mut dyn Host, c: Call) -> R<Value> {
    c.check("ord", 1, 1)?;
    let s = arg_str(&c.args[0], c.line)?;
    let mut chars = s.chars();
    match (chars.next(), chars.next()) {
        (Some(ch), None) => Ok(Value::Int(ch as i64)),
        _ => Err(c.err("ord() ожидает строку из одного символа")),
    }
}

fn radix(c: &Call, fname: &str, prefix: &str, base: u32) -> R<Value> {
    c.check(fname, 1, 1)?;
    let n = arg_int(&c.args[0], c.line)?;
    let digits = match base {
        16 => format!("{:x}", n.unsigned_abs()),
        2 => format!("{:b}", n.unsigned_abs()),
        _ => format!("{:o}", n.unsigned_abs()),
    };
    let sign = if n < 0 { "-" } else { "" };
    Ok(Value::str(&format!("{}{}{}", sign, prefix, digits)))
}

fn hex(_: &mut Interpreter, _: &mut dyn Host, c: Call) -> R<Value> {
    radix(&c, "hex", "0x", 16)
}

fn bin(_: &mut Interpreter, _: &mut dyn Host, c: Call) -> R<Value> {
    radix(&c, "bin", "0b", 2)
}

fn oct(_: &mut Interpreter, _: &mut dyn Host, c: Call) -> R<Value> {
    radix(&c, "oct", "0o", 8)
}

fn repr(_: &mut Interpreter, _: &mut dyn Host, c: Call) -> R<Value> {
    c.check("repr", 1, 1)?;
    Ok(Value::str(&c.args[0].repr()))
}

fn sleep(_: &mut Interpreter, host: &mut dyn Host, c: Call) -> R<Value> {
    c.check("sleep", 1, 1)?;
    let ms = arg_float(&c.args[0], c.line)?;
    if ms < 0.0 {
        return Err(c.err("sleep(): время не может быть отрицательным"));
    }
    host.sleep(ms as u64);
    Ok(Value::Null)
}

fn exit(_: &mut Interpreter, _: &mut dyn Host, c: Call) -> R<Value> {
    c.check("exit", 0, 1)?;
    let code = match c.arg(0) {
        None | Some(Value::Null) => 0,
        Some(v) => arg_int(v, c.line)?,
    };
    Err(Exc::Exit(code))
}

fn isinstance(_: &mut Interpreter, _: &mut dyn Host, c: Call) -> R<Value> {
    c.check("isinstance", 2, 2)?;
    let tn = c.args[0].type_name();
    let names: Vec<Rc<str>> = match &c.args[1] {
        Value::Str(s) => vec![s.clone()],
        Value::List(l) => l.borrow().iter().map(|v| arg_str(v, c.line)).collect::<R<_>>()?,
        other => return Err(c.err(format!("isinstance() ожидает имя типа, получено {}", other.type_name()))),
    };
    Ok(Value::Bool(names.iter().any(|n| &**n == tn || (&**n == "number" && (tn == "int" || tn == "float")))))
}
