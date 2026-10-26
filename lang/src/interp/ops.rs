//! Операции над значениями: арифметика (int/float как в Python: `/` даёт float,
//! `//` и `%` округляют к минус бесконечности), сравнения, `in`, индексация,
//! срезы с шагом, перебор.

use std::cmp::Ordering;

use super::R;
use crate::ast::{BinOp, CmpOp, UnOp};
use crate::error::rt;
use crate::value::{Key, Num, Value};

pub fn unary(op: UnOp, v: &Value, line: u32) -> R<Value> {
    match op {
        UnOp::Not => Ok(Value::Bool(!v.truthy())),
        UnOp::Neg => match v.as_number() {
            Some(Num::Int(n)) => n
                .checked_neg()
                .map(Value::Int)
                .ok_or_else(|| rt(line, "переполнение целого")),
            Some(Num::Float(f)) => Ok(Value::Float(-f)),
            None => Err(rt(line, format!("неподдерживаемый операнд для -: {}", v.type_name()))),
        },
        UnOp::Pos => match v.as_number() {
            Some(Num::Int(n)) => Ok(Value::Int(n)),
            Some(Num::Float(f)) => Ok(Value::Float(f)),
            None => Err(rt(line, format!("неподдерживаемый операнд для +: {}", v.type_name()))),
        },
    }
}

fn type_err(op: BinOp, a: &Value, b: &Value, line: u32) -> crate::error::Exc {
    rt(
        line,
        format!("неподдерживаемые операнды для {}: {} и {}", op.symbol(), a.type_name(), b.type_name()),
    )
}

fn repeat_str(s: &str, n: i64) -> Value {
    if n <= 0 {
        Value::str("")
    } else {
        Value::str(&s.repeat(n as usize))
    }
}

fn repeat_list(items: &[Value], n: i64) -> Value {
    let mut out = Vec::new();
    for _ in 0..n.max(0) {
        out.extend(items.iter().cloned());
    }
    Value::list(out)
}

/// Деление с округлением вниз, как в Python.
fn floor_div_int(a: i64, b: i64) -> Option<i64> {
    let q = a.checked_div(b)?;
    let r = a % b;
    if r != 0 && ((r < 0) != (b < 0)) {
        Some(q - 1)
    } else {
        Some(q)
    }
}

/// Остаток со знаком делителя, как в Python.
fn mod_int(a: i64, b: i64) -> i64 {
    let r = a.wrapping_rem(b);
    if r != 0 && ((r < 0) != (b < 0)) {
        r + b
    } else {
        r
    }
}

fn mod_float(a: f64, b: f64) -> f64 {
    let r = a % b;
    if r != 0.0 && ((r < 0.0) != (b < 0.0)) {
        r + b
    } else {
        r
    }
}

pub fn binary(op: BinOp, a: &Value, b: &Value, line: u32) -> R<Value> {
    // строки и списки
    match (op, a, b) {
        (BinOp::Add, Value::Str(x), Value::Str(y)) => {
            let mut s = String::with_capacity(x.len() + y.len());
            s.push_str(x);
            s.push_str(y);
            return Ok(Value::str(&s));
        }
        (BinOp::Add, Value::List(x), Value::List(y)) => {
            let mut v = x.borrow().clone();
            v.extend(y.borrow().iter().cloned());
            return Ok(Value::list(v));
        }
        (BinOp::Mul, Value::Str(s), Value::Int(n)) | (BinOp::Mul, Value::Int(n), Value::Str(s)) => {
            return Ok(repeat_str(s, *n));
        }
        (BinOp::Mul, Value::List(l), Value::Int(n)) | (BinOp::Mul, Value::Int(n), Value::List(l)) => {
            return Ok(repeat_list(&l.borrow(), *n));
        }
        _ => {}
    }
    let (x, y) = match (a.as_number(), b.as_number()) {
        (Some(x), Some(y)) => (x, y),
        _ => return Err(type_err(op, a, b, line)),
    };
    let overflow = || rt(line, "переполнение целого");
    let zero = || rt(line, "деление на ноль");
    if let (Num::Int(p), Num::Int(q)) = (x, y) {
        return match op {
            BinOp::Add => p.checked_add(q).map(Value::Int).ok_or_else(overflow),
            BinOp::Sub => p.checked_sub(q).map(Value::Int).ok_or_else(overflow),
            BinOp::Mul => p.checked_mul(q).map(Value::Int).ok_or_else(overflow),
            BinOp::Div => {
                if q == 0 {
                    Err(zero())
                } else {
                    Ok(Value::Float(p as f64 / q as f64))
                }
            }
            BinOp::FloorDiv => {
                if q == 0 {
                    Err(zero())
                } else {
                    floor_div_int(p, q).map(Value::Int).ok_or_else(overflow)
                }
            }
            BinOp::Mod => {
                if q == 0 {
                    Err(zero())
                } else {
                    Ok(Value::Int(mod_int(p, q)))
                }
            }
            BinOp::Pow => {
                if q >= 0 {
                    match u32::try_from(q).ok().and_then(|e| p.checked_pow(e)) {
                        Some(v) => Ok(Value::Int(v)),
                        None => Ok(Value::Float((p as f64).powf(q as f64))),
                    }
                } else {
                    if p == 0 {
                        return Err(zero());
                    }
                    Ok(Value::Float((p as f64).powf(q as f64)))
                }
            }
        };
    }
    let (p, q) = (x.as_f64(), y.as_f64());
    let v = match op {
        BinOp::Add => p + q,
        BinOp::Sub => p - q,
        BinOp::Mul => p * q,
        BinOp::Div => {
            if q == 0.0 {
                return Err(zero());
            }
            p / q
        }
        BinOp::FloorDiv => {
            if q == 0.0 {
                return Err(zero());
            }
            (p / q).floor()
        }
        BinOp::Mod => {
            if q == 0.0 {
                return Err(zero());
            }
            mod_float(p, q)
        }
        BinOp::Pow => {
            if p == 0.0 && q < 0.0 {
                return Err(zero());
            }
            p.powf(q)
        }
    };
    Ok(Value::Float(v))
}

pub fn compare(op: CmpOp, a: &Value, b: &Value, line: u32) -> R<bool> {
    match op {
        CmpOp::Eq => Ok(a.py_eq(b)),
        CmpOp::Ne => Ok(!a.py_eq(b)),
        CmpOp::In => contains(b, a, line),
        CmpOp::NotIn => contains(b, a, line).map(|r| !r),
        _ => {
            let ord = a
                .py_cmp(b)
                .ok_or_else(|| rt(line, format!("нельзя сравнить {} и {}", a.type_name(), b.type_name())))?;
            Ok(match op {
                CmpOp::Lt => ord == Ordering::Less,
                CmpOp::Le => ord != Ordering::Greater,
                CmpOp::Gt => ord == Ordering::Greater,
                CmpOp::Ge => ord != Ordering::Less,
                _ => false,
            })
        }
    }
}

/// `item in container`.
pub fn contains(container: &Value, item: &Value, line: u32) -> R<bool> {
    match container {
        Value::Str(s) => match item {
            Value::Str(sub) => Ok(s.contains(&**sub)),
            other => Err(rt(line, format!("операнд 'in' для строки должен быть строкой, а не {}", other.type_name()))),
        },
        Value::List(l) => Ok(l.borrow().iter().any(|v| v.py_eq(item))),
        Value::Dict(d) => match Key::from_value(item) {
            Ok(k) => Ok(d.borrow().contains(&k)),
            Err(_) => Ok(false),
        },
        other => Err(rt(line, format!("объект типа {} не поддерживает 'in'", other.type_name()))),
    }
}

/// Индекс с учётом отрицательных значений; `None` — вне диапазона.
pub fn norm_index(i: i64, len: usize) -> Option<usize> {
    let n = len as i64;
    let j = if i < 0 { i + n } else { i };
    if j < 0 || j >= n {
        None
    } else {
        Some(j as usize)
    }
}

fn index_value(idx: &Value, line: u32) -> R<i64> {
    match idx {
        Value::Int(n) => Ok(*n),
        Value::Bool(b) => Ok(*b as i64),
        other => Err(rt(line, format!("индекс должен быть целым, а не {}", other.type_name()))),
    }
}

pub fn get_index(obj: &Value, idx: &Value, line: u32) -> R<Value> {
    match obj {
        Value::List(l) => {
            let i = index_value(idx, line)?;
            let l = l.borrow();
            match norm_index(i, l.len()) {
                Some(j) => Ok(l[j].clone()),
                None => Err(rt(line, "индекс списка вне диапазона")),
            }
        }
        Value::Str(s) => {
            let i = index_value(idx, line)?;
            let n = s.chars().count();
            match norm_index(i, n) {
                Some(j) => Ok(Value::str(&s.chars().nth(j).map(String::from).unwrap_or_default())),
                None => Err(rt(line, "индекс строки вне диапазона")),
            }
        }
        Value::Dict(d) => {
            let key = Key::from_value(idx).map_err(|m| rt(line, m))?;
            match d.borrow().get(&key) {
                Some(v) => Ok(v.clone()),
                None => Err(rt(line, format!("ключ не найден: {}", idx.repr()))),
            }
        }
        other => Err(rt(line, format!("объект типа {} не поддерживает индексацию", other.type_name()))),
    }
}

pub fn set_index(obj: &Value, idx: &Value, value: Value, line: u32) -> R<()> {
    match obj {
        Value::List(l) => {
            let i = index_value(idx, line)?;
            let mut l = l.borrow_mut();
            let len = l.len();
            match norm_index(i, len) {
                Some(j) => {
                    l[j] = value;
                    Ok(())
                }
                None => Err(rt(line, "индекс списка вне диапазона")),
            }
        }
        Value::Dict(d) => {
            let key = Key::from_value(idx).map_err(|m| rt(line, m))?;
            d.borrow_mut().insert(key, value);
            Ok(())
        }
        Value::Str(_) => Err(rt(line, "строки неизменяемы")),
        other => Err(rt(line, format!("объект типа {} не поддерживает присваивание по индексу", other.type_name()))),
    }
}

/// Индексы среза по правилам Python (`lo:hi:step`, шаг может быть отрицательным).
pub fn slice_indices(len: usize, lo: Option<i64>, hi: Option<i64>, step: Option<i64>, line: u32) -> R<Vec<usize>> {
    let step = step.unwrap_or(1);
    if step == 0 {
        return Err(rt(line, "шаг среза не может быть нулём"));
    }
    let n = len as i64;
    let clamp = |i: i64| -> i64 {
        let i = if i < 0 { i + n } else { i };
        if step > 0 {
            i.clamp(0, n)
        } else {
            i.clamp(-1, n - 1)
        }
    };
    let start = match lo {
        Some(i) => clamp(i),
        None => if step > 0 { 0 } else { n - 1 },
    };
    let stop = match hi {
        Some(i) => clamp(i),
        None => if step > 0 { n } else { -1 },
    };
    let mut out = Vec::new();
    let mut i = start;
    while (step > 0 && i < stop) || (step < 0 && i > stop) {
        out.push(i as usize);
        i += step;
    }
    Ok(out)
}

pub fn slice(obj: &Value, lo: Option<i64>, hi: Option<i64>, step: Option<i64>, line: u32) -> R<Value> {
    match obj {
        Value::List(l) => {
            let l = l.borrow();
            let idx = slice_indices(l.len(), lo, hi, step, line)?;
            Ok(Value::list(idx.into_iter().map(|i| l[i].clone()).collect()))
        }
        Value::Str(s) => {
            let chars: Vec<char> = s.chars().collect();
            let idx = slice_indices(chars.len(), lo, hi, step, line)?;
            Ok(Value::str(&idx.into_iter().map(|i| chars[i]).collect::<String>()))
        }
        other => Err(rt(line, format!("объект типа {} не поддерживает срезы", other.type_name()))),
    }
}

/// Элементы для перебора: список (копия), символы строки, ключи словаря.
pub fn iter_to_vec(v: &Value, line: u32) -> R<Vec<Value>> {
    match v {
        Value::List(l) => Ok(l.borrow().clone()),
        Value::Str(s) => Ok(s.chars().map(|c| Value::str(&c.to_string())).collect()),
        Value::Dict(d) => Ok(d.borrow().keys()),
        other => Err(rt(line, format!("объект типа {} не является итерируемым", other.type_name()))),
    }
}
