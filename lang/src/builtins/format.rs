//! Форматирование по спецификации как в Python: `{x:16}` ширина, `{x:>8}` вправо,
//! `{x:^10}` по центру, `{x:.2f}` знаки после точки, `{x:04}` нули, `{x:,}` разряды,
//! `{x:x}`/`{x:b}`/`{x:o}` основания, `{x:+}` знак, `{x:.1%}` проценты, `{x:e}`.
//! Используется f-строками и `str.format(...)`.

use crate::error::{rt, LangError};
use crate::interp::{Interpreter, R};
use crate::value::{fmt_float, Call, Num, Value};
use crate::Host;

#[derive(Default)]
struct Spec {
    fill: Option<char>,
    align: Option<char>,
    sign: Option<char>,
    zero: bool,
    width: Option<usize>,
    grouping: Option<char>,
    precision: Option<usize>,
    ty: Option<char>,
}

fn parse_spec(spec: &str, line: u32) -> Result<Spec, LangError> {
    let chars: Vec<char> = spec.chars().collect();
    let mut s = Spec::default();
    let mut i = 0;
    let is_align = |c: char| matches!(c, '<' | '>' | '^' | '=');
    if chars.len() >= 2 && is_align(chars[1]) {
        s.fill = Some(chars[0]);
        s.align = Some(chars[1]);
        i = 2;
    } else if !chars.is_empty() && is_align(chars[0]) {
        s.align = Some(chars[0]);
        i = 1;
    }
    if i < chars.len() && matches!(chars[i], '+' | '-' | ' ') {
        s.sign = Some(chars[i]);
        i += 1;
    }
    if i < chars.len() && chars[i] == '0' {
        s.zero = true;
        i += 1;
    }
    let start = i;
    while i < chars.len() && chars[i].is_ascii_digit() {
        i += 1;
    }
    if i > start {
        s.width = chars[start..i].iter().collect::<String>().parse().ok();
    }
    if i < chars.len() && matches!(chars[i], ',' | '_') {
        s.grouping = Some(chars[i]);
        i += 1;
    }
    if i < chars.len() && chars[i] == '.' {
        i += 1;
        let start = i;
        while i < chars.len() && chars[i].is_ascii_digit() {
            i += 1;
        }
        if i == start {
            return Err(LangError::runtime(line, format!("неверный формат '{}': после '.' нужна точность", spec)));
        }
        s.precision = chars[start..i].iter().collect::<String>().parse().ok();
    }
    if i < chars.len() {
        s.ty = Some(chars[i]);
        i += 1;
    }
    if i < chars.len() {
        return Err(LangError::runtime(line, format!("неверный формат '{}'", spec)));
    }
    Ok(s)
}

/// Вставить разделители разрядов в целую часть числа (без знака).
fn group_digits(body: &str, sep: char) -> String {
    let (int_part, rest) = match body.find(|c: char| c == '.' || c == 'e' || c == 'E') {
        Some(i) => (&body[..i], &body[i..]),
        None => (body, ""),
    };
    let digits: Vec<char> = int_part.chars().collect();
    let mut out = String::new();
    for (i, ch) in digits.iter().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            out.push(sep);
        }
        out.push(*ch);
    }
    out.push_str(rest);
    out
}

/// Экспоненциальная запись в стиле Python: `1.500000e+02`.
fn fmt_exp(x: f64, prec: usize, upper: bool) -> String {
    let s = format!("{:.*e}", prec, x);
    let (mant, exp) = match s.split_once('e') {
        Some(p) => p,
        None => return s,
    };
    let exp_num: i32 = exp.parse().unwrap_or(0);
    let sign = if exp_num < 0 { '-' } else { '+' };
    let e = if upper { 'E' } else { 'e' };
    format!("{}{}{}{:02}", mant, e, sign, exp_num.abs())
}

/// Отформатировать значение по спецификации.
pub fn format_value(v: &Value, spec: &str, line: u32) -> Result<String, LangError> {
    if spec.is_empty() {
        return Ok(v.to_display_string());
    }
    let s = parse_spec(spec, line)?;
    let num = v.as_number();
    let is_num = num.is_some() && !matches!(v, Value::Bool(_)) || (matches!(v, Value::Bool(_)) && s.ty.is_some());
    let mut negative = false;
    let body = match (s.ty, num) {
        (Some(t @ ('f' | 'F' | 'e' | 'E' | '%' | 'g' | 'G')), Some(n)) => {
            let x = n.as_f64();
            negative = x.is_sign_negative() && x != 0.0;
            let x = x.abs();
            let prec = s.precision.unwrap_or(6);
            match t {
                'f' | 'F' => format!("{:.*}", prec, x),
                'e' | 'E' => fmt_exp(x, prec, t == 'E'),
                '%' => format!("{:.*}%", prec, x * 100.0),
                _ => {
                    let out = fmt_float(x);
                    out.strip_suffix(".0").map(String::from).unwrap_or(out)
                }
            }
        }
        (Some(t @ ('d' | 'x' | 'X' | 'b' | 'o' | 'c' | 'n')), Some(n)) => {
            let i = match n {
                Num::Int(i) => i,
                Num::Float(_) => {
                    return Err(LangError::runtime(line, format!("код формата '{}' не подходит для float", t)));
                }
            };
            negative = i < 0;
            let u = i.unsigned_abs();
            match t {
                'x' => format!("{:x}", u),
                'X' => format!("{:X}", u),
                'b' => format!("{:b}", u),
                'o' => format!("{:o}", u),
                'c' => char::from_u32(u as u32).map(String::from).unwrap_or_default(),
                _ => u.to_string(),
            }
        }
        (Some('s'), _) | (None, None) => {
            let mut text = v.to_display_string();
            if let Some(p) = s.precision {
                text = text.chars().take(p).collect();
            }
            text
        }
        (None, Some(n)) => match n {
            Num::Int(i) if !matches!(v, Value::Bool(_)) => {
                negative = i < 0;
                i.unsigned_abs().to_string()
            }
            Num::Int(_) => v.to_display_string(),
            Num::Float(x) => {
                negative = x.is_sign_negative() && x != 0.0;
                match s.precision {
                    Some(p) => format!("{:.*}", p, x.abs()),
                    None => fmt_float(x.abs()),
                }
            }
        },
        (Some(t), _) => {
            return Err(LangError::runtime(
                line,
                format!("неизвестный код формата '{}' для {}", t, v.type_name()),
            ));
        }
    };
    let body = match s.grouping {
        Some(sep) if is_num => group_digits(&body, sep),
        _ => body,
    };
    let sign = if negative {
        "-"
    } else {
        match s.sign {
            Some('+') if is_num => "+",
            Some(' ') if is_num => " ",
            _ => "",
        }
    };
    let width = s.width.unwrap_or(0);
    let len = sign.chars().count() + body.chars().count();
    if width <= len {
        return Ok(format!("{}{}", sign, body));
    }
    let padding = width - len;
    let (fill, align) = if s.zero && is_num && s.align.is_none() {
        ('0', '=')
    } else {
        (s.fill.unwrap_or(' '), s.align.unwrap_or(if is_num { '>' } else { '<' }))
    };
    let pad = |n: usize| std::iter::repeat(fill).take(n).collect::<String>();
    Ok(match align {
        '<' => format!("{}{}{}", sign, body, pad(padding)),
        '^' => {
            let left = padding / 2;
            format!("{}{}{}{}", pad(left), sign, body, pad(padding - left))
        }
        '=' => format!("{}{}{}", sign, pad(padding), body),
        _ => format!("{}{}{}", pad(padding), sign, body),
    })
}

/// `"{} и {name:>5}".format(a, name=b)`.
pub fn str_format(_: &mut Interpreter, _: &mut dyn Host, template: &str, c: &Call) -> R<Value> {
    let chars: Vec<char> = template.chars().collect();
    let mut out = String::new();
    let mut auto_index = 0usize;
    let mut i = 0;
    while i < chars.len() {
        let ch = chars[i];
        if ch == '{' {
            if chars.get(i + 1) == Some(&'{') {
                out.push('{');
                i += 2;
                continue;
            }
            let mut j = i + 1;
            let mut field = String::new();
            while j < chars.len() && chars[j] != '}' {
                field.push(chars[j]);
                j += 1;
            }
            if j >= chars.len() {
                return Err(rt(c.line, "незакрытая '{' в format()"));
            }
            i = j + 1;
            let (name, spec) = match field.find(':') {
                Some(p) => (field[..p].to_string(), field[p + 1..].to_string()),
                None => (field.clone(), String::new()),
            };
            let (name, conv) = match name.find('!') {
                Some(p) => (name[..p].to_string(), name[p + 1..].chars().next()),
                None => (name, None),
            };
            let value = if name.is_empty() {
                let v = c.arg(auto_index).ok_or_else(|| c.err("format(): не хватает позиционных аргументов"))?;
                auto_index += 1;
                v.clone()
            } else if let Ok(n) = name.parse::<usize>() {
                c.arg(n).ok_or_else(|| c.err(format!("format(): нет аргумента с номером {}", n)))?.clone()
            } else {
                c.kw(&name).ok_or_else(|| c.err(format!("format(): именованный аргумент '{}' не передан", name)))?.clone()
            };
            let value = match conv {
                Some('r') => Value::str(&value.repr()),
                Some('s') => Value::str(&value.to_display_string()),
                _ => value,
            };
            out.push_str(&format_value(&value, &spec, c.line)?);
        } else if ch == '}' {
            if chars.get(i + 1) == Some(&'}') {
                out.push('}');
                i += 2;
                continue;
            }
            return Err(rt(c.line, "одиночная '}' в format()"));
        } else {
            out.push(ch);
            i += 1;
        }
    }
    Ok(Value::str(&out))
}
