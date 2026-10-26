//! Методы строк: `upper lower strip split join replace find count startswith endswith
//! isdigit isalpha isspace format center ljust rjust zfill encode_hex decode_hex` и несколько
//! привычных из Python сверху (`title`, `capitalize`, `rfind`, `splitlines`, …).
//! Индексы — по символам, не по байтам.

use std::rc::Rc;

use super::{arg_int, arg_str, format};
use crate::error::rt;
use crate::interp::ops::iter_to_vec;
use crate::interp::{Interpreter, R};
use crate::value::{Call, Value};
use crate::Host;

pub const METHODS: &[&str] = &[
    "upper", "lower", "strip", "lstrip", "rstrip", "split", "join", "replace", "find", "rfind", "count",
    "startswith", "endswith", "isdigit", "isalpha", "isspace", "isalnum", "isupper", "islower", "format",
    "center", "ljust", "rjust", "zfill", "encode_hex", "decode_hex", "title", "capitalize", "swapcase",
    "splitlines", "index", "partition",
];

fn strs(items: Vec<String>) -> Value {
    Value::list(items.iter().map(|s| Value::str(s)).collect())
}

fn char_index(s: &str, byte_pos: usize) -> i64 {
    s[..byte_pos].chars().count() as i64
}

/// Проверить префикс/суффикс: строка или список строк.
fn affix(c: &Call, v: &Value, s: &str, start: bool) -> R<bool> {
    let candidates: Vec<Rc<str>> = match v {
        Value::Str(p) => vec![p.clone()],
        Value::List(l) => l.borrow().iter().map(|x| arg_str(x, c.line)).collect::<R<_>>()?,
        other => return Err(c.err(format!("ожидалась строка или список строк, получено {}", other.type_name()))),
    };
    Ok(candidates.iter().any(|p| if start { s.starts_with(&**p) } else { s.ends_with(&**p) }))
}

fn pad(s: &str, width: i64, fill: &str, align: char) -> String {
    let n = s.chars().count() as i64;
    if width <= n || fill.is_empty() {
        return s.to_string();
    }
    let total = (width - n) as usize;
    let (left, right) = match align {
        '<' => (0, total),
        '>' => (total, 0),
        _ => {
            let left = total / 2 + (total % 2) * (n as usize % 2);
            (left, total - left)
        }
    };
    format!("{}{}{}", fill.repeat(left), s, fill.repeat(right))
}

fn fill_arg(c: &Call, i: usize) -> R<String> {
    match c.arg(i) {
        None => Ok(" ".to_string()),
        Some(v) => {
            let f = arg_str(v, c.line)?;
            if f.chars().count() != 1 {
                return Err(c.err("символ заполнения должен быть одним символом"));
            }
            Ok(f.to_string())
        }
    }
}

fn split_whitespace_max(s: &str, maxsplit: i64) -> Vec<String> {
    if maxsplit < 0 {
        return s.split_whitespace().map(String::from).collect();
    }
    let mut out = Vec::new();
    let mut rest = s.trim_start();
    while !rest.is_empty() {
        if out.len() as i64 == maxsplit {
            out.push(rest.to_string());
            return out;
        }
        match rest.find(char::is_whitespace) {
            Some(i) => {
                out.push(rest[..i].to_string());
                rest = rest[i..].trim_start();
            }
            None => {
                out.push(rest.to_string());
                break;
            }
        }
    }
    out
}

pub fn call(interp: &mut Interpreter, host: &mut dyn Host, s: &Rc<str>, name: &str, c: Call) -> R<Value> {
    let s: &str = s;
    match name {
        "upper" => Ok(Value::str(&s.to_uppercase())),
        "lower" => Ok(Value::str(&s.to_lowercase())),
        "swapcase" => Ok(Value::str(
            &s.chars()
                .map(|ch| {
                    if ch.is_uppercase() {
                        ch.to_lowercase().collect::<String>()
                    } else {
                        ch.to_uppercase().collect::<String>()
                    }
                })
                .collect::<String>(),
        )),
        "capitalize" => {
            let mut chars = s.chars();
            let out = match chars.next() {
                Some(f) => f.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase(),
                None => String::new(),
            };
            Ok(Value::str(&out))
        }
        "title" => {
            let mut out = String::new();
            let mut start = true;
            for ch in s.chars() {
                if ch.is_alphanumeric() {
                    if start {
                        out.extend(ch.to_uppercase());
                    } else {
                        out.extend(ch.to_lowercase());
                    }
                    start = false;
                } else {
                    out.push(ch);
                    start = true;
                }
            }
            Ok(Value::str(&out))
        }
        "strip" | "lstrip" | "rstrip" => {
            c.check(name, 0, 1)?;
            let chars: Option<Vec<char>> = match c.arg(0) {
                None | Some(Value::Null) => None,
                Some(v) => Some(arg_str(v, c.line)?.chars().collect()),
            };
            let pred = |ch: char| match &chars {
                None => ch.is_whitespace(),
                Some(set) => set.contains(&ch),
            };
            let out = match name {
                "strip" => s.trim_matches(pred),
                "lstrip" => s.trim_start_matches(pred),
                _ => s.trim_end_matches(pred),
            };
            Ok(Value::str(out))
        }
        "split" => {
            c.check("split", 0, 2)?;
            c.allow_kwargs("split", &["sep", "maxsplit"])?;
            let maxsplit = match c.get(1, "maxsplit") {
                Some(v) => arg_int(v, c.line)?,
                None => -1,
            };
            match c.get(0, "sep") {
                None | Some(Value::Null) => Ok(strs(split_whitespace_max(s, maxsplit))),
                Some(v) => {
                    let sep = arg_str(v, c.line)?;
                    if sep.is_empty() {
                        return Err(c.err("разделитель не может быть пустым"));
                    }
                    let parts: Vec<String> = if maxsplit < 0 {
                        s.split(&*sep).map(String::from).collect()
                    } else {
                        s.splitn(maxsplit as usize + 1, &*sep).map(String::from).collect()
                    };
                    Ok(strs(parts))
                }
            }
        }
        "splitlines" => {
            c.check("splitlines", 0, 0)?;
            Ok(strs(s.lines().map(String::from).collect()))
        }
        "join" => {
            c.check("join", 1, 1)?;
            let items = iter_to_vec(&c.args[0], c.line)?;
            let mut parts = Vec::with_capacity(items.len());
            for it in &items {
                match it {
                    Value::Str(x) => parts.push(x.to_string()),
                    other => {
                        return Err(c.err(format!("join() ожидает список строк, встречен {}", other.type_name())));
                    }
                }
            }
            Ok(Value::str(&parts.join(s)))
        }
        "replace" => {
            c.check("replace", 2, 3)?;
            let old = arg_str(&c.args[0], c.line)?;
            let new = arg_str(&c.args[1], c.line)?;
            let count = match c.arg(2) {
                Some(v) => arg_int(v, c.line)?,
                None => -1,
            };
            if old.is_empty() {
                return Err(c.err("replace(): искомая подстрока не может быть пустой"));
            }
            let out = if count < 0 { s.replace(&*old, &new) } else { s.replacen(&*old, &new, count as usize) };
            Ok(Value::str(&out))
        }
        "find" | "index" => {
            c.check(name, 1, 2)?;
            let sub = arg_str(&c.args[0], c.line)?;
            let start = match c.arg(1) {
                Some(v) => arg_int(v, c.line)?.max(0) as usize,
                None => 0,
            };
            let byte_start = s.char_indices().nth(start).map(|(b, _)| b).unwrap_or(s.len());
            match s[byte_start..].find(&*sub) {
                Some(b) => Ok(Value::Int(char_index(s, byte_start + b))),
                None if name == "find" => Ok(Value::Int(-1)),
                None => Err(c.err("подстрока не найдена")),
            }
        }
        "rfind" => {
            c.check("rfind", 1, 1)?;
            let sub = arg_str(&c.args[0], c.line)?;
            Ok(Value::Int(s.rfind(&*sub).map(|b| char_index(s, b)).unwrap_or(-1)))
        }
        "count" => {
            c.check("count", 1, 1)?;
            let sub = arg_str(&c.args[0], c.line)?;
            if sub.is_empty() {
                return Ok(Value::Int(s.chars().count() as i64 + 1));
            }
            Ok(Value::Int(s.matches(&*sub).count() as i64))
        }
        "partition" => {
            c.check("partition", 1, 1)?;
            let sep = arg_str(&c.args[0], c.line)?;
            Ok(match s.find(&*sep) {
                Some(i) => strs(vec![s[..i].to_string(), sep.to_string(), s[i + sep.len()..].to_string()]),
                None => strs(vec![s.to_string(), String::new(), String::new()]),
            })
        }
        "startswith" => {
            c.check("startswith", 1, 1)?;
            Ok(Value::Bool(affix(&c, &c.args[0], s, true)?))
        }
        "endswith" => {
            c.check("endswith", 1, 1)?;
            Ok(Value::Bool(affix(&c, &c.args[0], s, false)?))
        }
        "isdigit" => Ok(Value::Bool(!s.is_empty() && s.chars().all(|ch| ch.is_ascii_digit()))),
        "isalpha" => Ok(Value::Bool(!s.is_empty() && s.chars().all(char::is_alphabetic))),
        "isalnum" => Ok(Value::Bool(!s.is_empty() && s.chars().all(char::is_alphanumeric))),
        "isspace" => Ok(Value::Bool(!s.is_empty() && s.chars().all(char::is_whitespace))),
        "isupper" => Ok(Value::Bool(s.chars().any(char::is_alphabetic) && !s.chars().any(char::is_lowercase))),
        "islower" => Ok(Value::Bool(s.chars().any(char::is_alphabetic) && !s.chars().any(char::is_uppercase))),
        "format" => format::str_format(interp, host, s, &c),
        "center" | "ljust" | "rjust" => {
            c.check(name, 1, 2)?;
            let width = arg_int(&c.args[0], c.line)?;
            let fill = fill_arg(&c, 1)?;
            let align = match name {
                "center" => '^',
                "ljust" => '<',
                _ => '>',
            };
            Ok(Value::str(&pad(s, width, &fill, align)))
        }
        "zfill" => {
            c.check("zfill", 1, 1)?;
            let width = arg_int(&c.args[0], c.line)?;
            let (sign, body) = match s.strip_prefix('-') {
                Some(rest) => ("-", rest),
                None => match s.strip_prefix('+') {
                    Some(rest) => ("+", rest),
                    None => ("", s),
                },
            };
            let padded = pad(body, width - sign.len() as i64, "0", '>');
            Ok(Value::str(&format!("{}{}", sign, padded)))
        }
        "encode_hex" => {
            c.check("encode_hex", 0, 0)?;
            Ok(Value::str(&s.bytes().map(|b| format!("{:02x}", b)).collect::<String>()))
        }
        "decode_hex" => {
            c.check("decode_hex", 0, 0)?;
            let clean: String = s.chars().filter(|ch| !ch.is_whitespace()).collect();
            if clean.len() % 2 != 0 || !clean.chars().all(|ch| ch.is_ascii_hexdigit()) {
                return Err(c.err("неверная hex-строка"));
            }
            let bytes: Vec<u8> =
                (0..clean.len()).step_by(2).map(|i| u8::from_str_radix(&clean[i..i + 2], 16).unwrap_or(0)).collect();
            Ok(Value::str(&String::from_utf8_lossy(&bytes)))
        }
        _ => Err(rt(c.line, format!("нет такого метода у строки: {}", name))),
    }
}
