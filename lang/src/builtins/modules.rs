//! Модули без обращения к миру: `math`, `time`, `random`.
//! `time.now()` сначала спрашивает хост (`Host::call("time.now")` — игровое время),
//! а если хост такого не знает — берёт системные часы.

use super::{arg_float, arg_int, arg_list, lookup_table};
use crate::interp::{Interpreter, R};
use crate::value::{BuiltinFn, Call, Value};
use crate::Host;

// ---------- math ----------

const MATH: &[(&str, BuiltinFn)] = &[
    ("sqrt", math_sqrt),
    ("floor", math_floor),
    ("ceil", math_ceil),
    ("sin", math_sin),
    ("cos", math_cos),
    ("tan", math_tan),
    ("atan", math_atan),
    ("atan2", math_atan2),
    ("log", math_log),
    ("log2", math_log2),
    ("log10", math_log10),
    ("pow", math_pow),
    ("exp", math_exp),
    ("fabs", math_fabs),
    ("hypot", math_hypot),
];

pub fn math_attr(name: &str) -> Option<Value> {
    match name {
        "pi" => Some(Value::Float(std::f64::consts::PI)),
        "e" => Some(Value::Float(std::f64::consts::E)),
        "inf" => Some(Value::Float(f64::INFINITY)),
        "nan" => Some(Value::Float(f64::NAN)),
        _ => lookup_table(MATH, name),
    }
}

fn one(c: &Call, name: &str) -> R<f64> {
    c.check(name, 1, 1)?;
    arg_float(&c.args[0], c.line)
}

fn to_int_value(c: &Call, f: f64) -> R<Value> {
    if f.is_finite() && f.abs() < 9.2e18 {
        Ok(Value::Int(f as i64))
    } else {
        Err(c.err("число не помещается в int"))
    }
}

fn math_sqrt(_: &mut Interpreter, _: &mut dyn Host, c: Call) -> R<Value> {
    let x = one(&c, "math.sqrt")?;
    if x < 0.0 {
        return Err(c.err("math.sqrt(): корень из отрицательного числа"));
    }
    Ok(Value::Float(x.sqrt()))
}

fn math_floor(_: &mut Interpreter, _: &mut dyn Host, c: Call) -> R<Value> {
    let x = one(&c, "math.floor")?;
    to_int_value(&c, x.floor())
}

fn math_ceil(_: &mut Interpreter, _: &mut dyn Host, c: Call) -> R<Value> {
    let x = one(&c, "math.ceil")?;
    to_int_value(&c, x.ceil())
}

fn math_sin(_: &mut Interpreter, _: &mut dyn Host, c: Call) -> R<Value> {
    Ok(Value::Float(one(&c, "math.sin")?.sin()))
}

fn math_cos(_: &mut Interpreter, _: &mut dyn Host, c: Call) -> R<Value> {
    Ok(Value::Float(one(&c, "math.cos")?.cos()))
}

fn math_tan(_: &mut Interpreter, _: &mut dyn Host, c: Call) -> R<Value> {
    Ok(Value::Float(one(&c, "math.tan")?.tan()))
}

fn math_atan(_: &mut Interpreter, _: &mut dyn Host, c: Call) -> R<Value> {
    Ok(Value::Float(one(&c, "math.atan")?.atan()))
}

fn math_exp(_: &mut Interpreter, _: &mut dyn Host, c: Call) -> R<Value> {
    Ok(Value::Float(one(&c, "math.exp")?.exp()))
}

fn math_fabs(_: &mut Interpreter, _: &mut dyn Host, c: Call) -> R<Value> {
    Ok(Value::Float(one(&c, "math.fabs")?.abs()))
}

fn math_atan2(_: &mut Interpreter, _: &mut dyn Host, c: Call) -> R<Value> {
    c.check("math.atan2", 2, 2)?;
    let y = arg_float(&c.args[0], c.line)?;
    let x = arg_float(&c.args[1], c.line)?;
    Ok(Value::Float(y.atan2(x)))
}

fn math_hypot(_: &mut Interpreter, _: &mut dyn Host, c: Call) -> R<Value> {
    c.check("math.hypot", 2, 2)?;
    let x = arg_float(&c.args[0], c.line)?;
    let y = arg_float(&c.args[1], c.line)?;
    Ok(Value::Float(x.hypot(y)))
}

fn math_log(_: &mut Interpreter, _: &mut dyn Host, c: Call) -> R<Value> {
    c.check("math.log", 1, 2)?;
    let x = arg_float(&c.args[0], c.line)?;
    if x <= 0.0 {
        return Err(c.err("math.log(): аргумент должен быть положительным"));
    }
    match c.arg(1) {
        Some(b) => {
            let base = arg_float(b, c.line)?;
            if base <= 0.0 || base == 1.0 {
                return Err(c.err("math.log(): недопустимое основание"));
            }
            Ok(Value::Float(x.ln() / base.ln()))
        }
        None => Ok(Value::Float(x.ln())),
    }
}

fn math_log2(_: &mut Interpreter, _: &mut dyn Host, c: Call) -> R<Value> {
    let x = one(&c, "math.log2")?;
    if x <= 0.0 {
        return Err(c.err("math.log2(): аргумент должен быть положительным"));
    }
    Ok(Value::Float(x.log2()))
}

fn math_log10(_: &mut Interpreter, _: &mut dyn Host, c: Call) -> R<Value> {
    let x = one(&c, "math.log10")?;
    if x <= 0.0 {
        return Err(c.err("math.log10(): аргумент должен быть положительным"));
    }
    Ok(Value::Float(x.log10()))
}

fn math_pow(_: &mut Interpreter, _: &mut dyn Host, c: Call) -> R<Value> {
    c.check("math.pow", 2, 2)?;
    let x = arg_float(&c.args[0], c.line)?;
    let y = arg_float(&c.args[1], c.line)?;
    Ok(Value::Float(x.powf(y)))
}

// ---------- time ----------

const TIME: &[(&str, BuiltinFn)] = &[("now", time_now), ("format", time_format), ("ms", time_ms)];

pub fn time_attr(name: &str) -> Option<Value> {
    lookup_table(TIME, name)
}

fn system_seconds() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn time_now(_: &mut Interpreter, host: &mut dyn Host, c: Call) -> R<Value> {
    c.check("time.now", 0, 0)?;
    match host.call("time.now", Vec::new()) {
        Ok(v) if !matches!(v, Value::Null) => Ok(v),
        _ => Ok(Value::Int(system_seconds())),
    }
}

fn time_ms(_: &mut Interpreter, _: &mut dyn Host, c: Call) -> R<Value> {
    c.check("time.ms", 0, 0)?;
    let ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0);
    Ok(Value::Int(ms))
}

/// Дни от 1970-01-01 → (год, месяц, день); алгоритм Хиннанта.
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

const MONTHS: [&str; 12] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
const WEEKDAYS: [&str; 7] = ["Thu", "Fri", "Sat", "Sun", "Mon", "Tue", "Wed"];

/// Строка даты по шаблону `%Y-%m-%d %H:%M:%S` (UTC).
pub fn format_time(ts: i64, fmt: &str) -> String {
    let days = ts.div_euclid(86_400);
    let secs = ts.rem_euclid(86_400);
    let (y, m, d) = civil_from_days(days);
    let (hh, mm, ss) = (secs / 3600, (secs % 3600) / 60, secs % 60);
    let mut out = String::new();
    let mut chars = fmt.chars();
    while let Some(ch) = chars.next() {
        if ch != '%' {
            out.push(ch);
            continue;
        }
        match chars.next() {
            Some('Y') => out.push_str(&y.to_string()),
            Some('y') => out.push_str(&format!("{:02}", y.rem_euclid(100))),
            Some('m') => out.push_str(&format!("{:02}", m)),
            Some('d') => out.push_str(&format!("{:02}", d)),
            Some('H') => out.push_str(&format!("{:02}", hh)),
            Some('M') => out.push_str(&format!("{:02}", mm)),
            Some('S') => out.push_str(&format!("{:02}", ss)),
            Some('b') => out.push_str(MONTHS[(m - 1) as usize]),
            Some('a') => out.push_str(WEEKDAYS[days.rem_euclid(7) as usize]),
            Some('%') => out.push('%'),
            Some(other) => {
                out.push('%');
                out.push(other);
            }
            None => out.push('%'),
        }
    }
    out
}

fn time_format(_: &mut Interpreter, _: &mut dyn Host, c: Call) -> R<Value> {
    c.check("time.format", 1, 2)?;
    let ts = arg_float(&c.args[0], c.line)? as i64;
    let fmt = match c.arg(1) {
        Some(Value::Str(s)) => s.to_string(),
        Some(other) => return Err(c.err(format!("time.format(): шаблон должен быть строкой, а не {}", other.type_name()))),
        None => "%Y-%m-%d %H:%M:%S".to_string(),
    };
    Ok(Value::str(&format_time(ts, &fmt)))
}

// ---------- random ----------

const RANDOM: &[(&str, BuiltinFn)] = &[
    ("random", random_random),
    ("randint", random_randint),
    ("choice", random_choice),
    ("shuffle", random_shuffle),
    ("seed", random_seed),
    ("uniform", random_uniform),
];

pub fn random_attr(name: &str) -> Option<Value> {
    lookup_table(RANDOM, name)
}

/// xorshift64*: следующее число состояния.
fn next_u64(interp: &mut Interpreter) -> u64 {
    let mut x = interp.rng_state;
    if x == 0 {
        x = 0x9E37_79B9_7F4A_7C15;
    }
    x ^= x >> 12;
    x ^= x << 25;
    x ^= x >> 27;
    interp.rng_state = x;
    x.wrapping_mul(0x2545_F491_4F6C_DD1D)
}

fn next_f64(interp: &mut Interpreter) -> f64 {
    (next_u64(interp) >> 11) as f64 / (1u64 << 53) as f64
}

fn random_random(interp: &mut Interpreter, _: &mut dyn Host, c: Call) -> R<Value> {
    c.check("random.random", 0, 0)?;
    Ok(Value::Float(next_f64(interp)))
}

fn random_uniform(interp: &mut Interpreter, _: &mut dyn Host, c: Call) -> R<Value> {
    c.check("random.uniform", 2, 2)?;
    let a = arg_float(&c.args[0], c.line)?;
    let b = arg_float(&c.args[1], c.line)?;
    Ok(Value::Float(a + (b - a) * next_f64(interp)))
}

fn random_randint(interp: &mut Interpreter, _: &mut dyn Host, c: Call) -> R<Value> {
    c.check("random.randint", 2, 2)?;
    let a = arg_int(&c.args[0], c.line)?;
    let b = arg_int(&c.args[1], c.line)?;
    if b < a {
        return Err(c.err("random.randint(): нижняя граница больше верхней"));
    }
    let span = (b - a) as u64 + 1;
    Ok(Value::Int(a + (next_u64(interp) % span) as i64))
}

fn random_choice(interp: &mut Interpreter, _: &mut dyn Host, c: Call) -> R<Value> {
    c.check("random.choice", 1, 1)?;
    let items = crate::interp::ops::iter_to_vec(&c.args[0], c.line)?;
    if items.is_empty() {
        return Err(c.err("random.choice(): пустой список"));
    }
    let i = (next_u64(interp) % items.len() as u64) as usize;
    Ok(items[i].clone())
}

fn random_shuffle(interp: &mut Interpreter, _: &mut dyn Host, c: Call) -> R<Value> {
    c.check("random.shuffle", 1, 1)?;
    let l = arg_list(&c.args[0], c.line)?;
    let mut v = l.borrow_mut();
    let n = v.len();
    for i in (1..n).rev() {
        let j = (next_u64(interp) % (i as u64 + 1)) as usize;
        v.swap(i, j);
    }
    Ok(Value::Null)
}

fn random_seed(interp: &mut Interpreter, _: &mut dyn Host, c: Call) -> R<Value> {
    c.check("random.seed", 1, 1)?;
    let n = arg_int(&c.args[0], c.line)?;
    interp.rng_state = (n as u64) ^ 0x9E37_79B9_7F4A_7C15;
    if interp.rng_state == 0 {
        interp.rng_state = 1;
    }
    Ok(Value::Null)
}
