//! Модуль `json`: `loads(str)` и `dumps(value[, indent])`. Свой разбор и запись,
//! чтобы порядок ключей словаря сохранялся (serde_json без лишних крейтов его сортирует).

use std::rc::Rc;

use super::{arg_int, arg_str, lookup_table};
use crate::interp::{Interpreter, R};
use crate::value::{fmt_float, quote_str, BuiltinFn, Call, Dict, Key, Value};
use crate::Host;

const TABLE: &[(&str, BuiltinFn)] = &[("loads", loads), ("dumps", dumps)];

pub fn attr(name: &str) -> Option<Value> {
    lookup_table(TABLE, name)
}

fn loads(_: &mut Interpreter, _: &mut dyn Host, c: Call) -> R<Value> {
    c.check("json.loads", 1, 1)?;
    let s = arg_str(&c.args[0], c.line)?;
    parse(&s).map_err(|m| c.err(format!("json.loads(): {}", m)))
}

fn dumps(_: &mut Interpreter, _: &mut dyn Host, c: Call) -> R<Value> {
    c.check("json.dumps", 1, 2)?;
    c.allow_kwargs("json.dumps", &["indent"])?;
    let indent = match c.get(1, "indent") {
        None | Some(Value::Null) => None,
        Some(v) => Some(arg_int(v, c.line)?.max(0) as usize),
    };
    let mut out = String::new();
    write(&c.args[0], indent, 0, &mut out).map_err(|m| c.err(m))?;
    Ok(Value::str(&out))
}

/// Записать значение в JSON. `indent` — ширина отступа для многострочной записи.
pub fn write(v: &Value, indent: Option<usize>, level: usize, out: &mut String) -> Result<(), String> {
    match v {
        Value::Null => out.push_str("null"),
        Value::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        Value::Int(n) => out.push_str(&n.to_string()),
        Value::Float(f) => {
            if f.is_nan() {
                out.push_str("NaN");
            } else if f.is_infinite() {
                out.push_str(if *f > 0.0 { "Infinity" } else { "-Infinity" });
            } else {
                out.push_str(&fmt_float(*f));
            }
        }
        Value::Str(s) => out.push_str(&quote_str(s)),
        Value::List(l) => {
            let items = l.borrow();
            if items.is_empty() {
                out.push_str("[]");
                return Ok(());
            }
            out.push('[');
            for (i, it) in items.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                    if indent.is_none() {
                        out.push(' ');
                    }
                }
                newline(indent, level + 1, out);
                write(it, indent, level + 1, out)?;
            }
            newline(indent, level, out);
            out.push(']');
        }
        Value::Dict(d) => {
            let d = d.borrow();
            if d.is_empty() {
                out.push_str("{}");
                return Ok(());
            }
            out.push('{');
            for (i, (k, val)) in d.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                    if indent.is_none() {
                        out.push(' ');
                    }
                }
                newline(indent, level + 1, out);
                out.push_str(&quote_str(&k.to_string_key()));
                out.push_str(": ");
                write(val, indent, level + 1, out)?;
            }
            newline(indent, level, out);
            out.push('}');
        }
        other => return Err(format!("значение типа {} нельзя записать в JSON", other.type_name())),
    }
    Ok(())
}

fn newline(indent: Option<usize>, level: usize, out: &mut String) {
    if let Some(w) = indent {
        out.push('\n');
        out.push_str(&" ".repeat(w * level));
    }
}

/// Разобрать JSON-текст в значение.
pub fn parse(text: &str) -> Result<Value, String> {
    let chars: Vec<char> = text.chars().collect();
    let mut p = JsonParser { chars, pos: 0 };
    p.skip_ws();
    let v = p.value()?;
    p.skip_ws();
    if p.pos < p.chars.len() {
        return Err(format!("лишние данные на позиции {}", p.pos));
    }
    Ok(v)
}

struct JsonParser {
    chars: Vec<char>,
    pos: usize,
}

impl JsonParser {
    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn skip_ws(&mut self) {
        while matches!(self.peek(), Some(' ') | Some('\t') | Some('\n') | Some('\r')) {
            self.pos += 1;
        }
    }

    fn expect(&mut self, c: char) -> Result<(), String> {
        if self.peek() == Some(c) {
            self.pos += 1;
            Ok(())
        } else {
            Err(format!("ожидался '{}' на позиции {}", c, self.pos))
        }
    }

    fn literal(&mut self, word: &str, v: Value) -> Result<Value, String> {
        for ch in word.chars() {
            if self.peek() != Some(ch) {
                return Err(format!("неверное значение на позиции {}", self.pos));
            }
            self.pos += 1;
        }
        Ok(v)
    }

    fn value(&mut self) -> Result<Value, String> {
        match self.peek() {
            None => Err("неожиданный конец текста".to_string()),
            Some('{') => self.object(),
            Some('[') => self.array(),
            Some('"') => Ok(Value::str(&self.string()?)),
            Some('t') => self.literal("true", Value::Bool(true)),
            Some('f') => self.literal("false", Value::Bool(false)),
            Some('n') => self.literal("null", Value::Null),
            Some('N') => self.literal("NaN", Value::Float(f64::NAN)),
            Some('I') => self.literal("Infinity", Value::Float(f64::INFINITY)),
            Some(c) if c == '-' || c.is_ascii_digit() => self.number(),
            Some(c) => Err(format!("неожиданный символ '{}' на позиции {}", c, self.pos)),
        }
    }

    fn object(&mut self) -> Result<Value, String> {
        self.expect('{')?;
        let mut d = Dict::new();
        self.skip_ws();
        if self.peek() == Some('}') {
            self.pos += 1;
            return Ok(Value::dict(d));
        }
        loop {
            self.skip_ws();
            if self.peek() != Some('"') {
                return Err(format!("ожидался ключ-строка на позиции {}", self.pos));
            }
            let k = self.string()?;
            self.skip_ws();
            self.expect(':')?;
            self.skip_ws();
            let v = self.value()?;
            d.insert(Key::Str(Rc::from(k.as_str())), v);
            self.skip_ws();
            match self.peek() {
                Some(',') => self.pos += 1,
                Some('}') => {
                    self.pos += 1;
                    return Ok(Value::dict(d));
                }
                _ => return Err(format!("ожидалась ',' или '}}' на позиции {}", self.pos)),
            }
        }
    }

    fn array(&mut self) -> Result<Value, String> {
        self.expect('[')?;
        let mut items = Vec::new();
        self.skip_ws();
        if self.peek() == Some(']') {
            self.pos += 1;
            return Ok(Value::list(items));
        }
        loop {
            self.skip_ws();
            items.push(self.value()?);
            self.skip_ws();
            match self.peek() {
                Some(',') => self.pos += 1,
                Some(']') => {
                    self.pos += 1;
                    return Ok(Value::list(items));
                }
                _ => return Err(format!("ожидалась ',' или ']' на позиции {}", self.pos)),
            }
        }
    }

    fn string(&mut self) -> Result<String, String> {
        self.expect('"')?;
        let mut s = String::new();
        loop {
            let c = self.peek().ok_or_else(|| "незакрытая строка".to_string())?;
            self.pos += 1;
            match c {
                '"' => return Ok(s),
                '\\' => {
                    let e = self.peek().ok_or_else(|| "незакрытая строка".to_string())?;
                    self.pos += 1;
                    match e {
                        'n' => s.push('\n'),
                        't' => s.push('\t'),
                        'r' => s.push('\r'),
                        'b' => s.push('\u{8}'),
                        'f' => s.push('\u{c}'),
                        '/' => s.push('/'),
                        '\\' => s.push('\\'),
                        '"' => s.push('"'),
                        'u' => {
                            let code = self.hex4()?;
                            // суррогатная пара
                            if (0xD800..0xDC00).contains(&code) && self.peek() == Some('\\') {
                                self.pos += 1;
                                self.expect('u')?;
                                let low = self.hex4()?;
                                let full = 0x10000 + ((code - 0xD800) << 10) + (low.wrapping_sub(0xDC00) & 0x3FF);
                                s.push(char::from_u32(full).unwrap_or('\u{FFFD}'));
                            } else {
                                s.push(char::from_u32(code).unwrap_or('\u{FFFD}'));
                            }
                        }
                        other => return Err(format!("неверная экранированная последовательность '\\{}'", other)),
                    }
                }
                c => s.push(c),
            }
        }
    }

    fn hex4(&mut self) -> Result<u32, String> {
        let mut code = 0u32;
        for _ in 0..4 {
            let d = self.peek().and_then(|c| c.to_digit(16)).ok_or_else(|| "неверный \\u-код".to_string())?;
            self.pos += 1;
            code = code * 16 + d;
        }
        Ok(code)
    }

    fn number(&mut self) -> Result<Value, String> {
        let start = self.pos;
        if self.peek() == Some('-') {
            self.pos += 1;
            if self.peek() == Some('I') {
                return self.literal("Infinity", Value::Float(f64::NEG_INFINITY));
            }
        }
        let mut is_float = false;
        while let Some(c) = self.peek() {
            match c {
                '0'..='9' => {}
                '.' | 'e' | 'E' | '+' | '-' => is_float = true,
                _ => break,
            }
            self.pos += 1;
        }
        let text: String = self.chars[start..self.pos].iter().collect();
        if !is_float {
            if let Ok(n) = text.parse::<i64>() {
                return Ok(Value::Int(n));
            }
        }
        text.parse::<f64>().map(Value::Float).map_err(|_| format!("неверное число '{}'", text))
    }
}
