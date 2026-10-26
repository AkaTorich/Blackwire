//! Мелкие помощники команд: разбор флагов, таблицы с колонками, размеры.

use crate::types::{OutKind, OutLine};

/// Результат одной команды: строки вывода (ошибки — строки вида `Error`) и код.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CmdOutput {
    pub lines: Vec<OutLine>,
    pub status: i32,
}

impl CmdOutput {
    pub fn ok() -> CmdOutput {
        CmdOutput::default()
    }

    pub fn err(text: impl Into<String>) -> CmdOutput {
        CmdOutput { lines: vec![OutLine::error(text)], status: 1 }
    }

    pub fn text(text: impl Into<String>) -> CmdOutput {
        let mut o = CmdOutput::ok();
        o.push(text);
        o
    }

    /// Обычная строка; текст с переводами строк разбивается на несколько.
    pub fn push(&mut self, text: impl Into<String>) {
        let s: String = text.into();
        if s.is_empty() {
            self.lines.push(OutLine::normal(""));
            return;
        }
        for l in s.split('\n') {
            self.lines.push(OutLine::normal(l));
        }
    }

    pub fn push_kind(&mut self, text: impl Into<String>, kind: OutKind) {
        self.lines.push(OutLine::new(text, kind));
    }

    pub fn push_err(&mut self, text: impl Into<String>) {
        self.lines.push(OutLine::error(text));
        self.status = 1;
    }

    pub fn push_lines(&mut self, lines: impl IntoIterator<Item = String>) {
        for l in lines {
            self.lines.push(OutLine::normal(l));
        }
    }

    /// Текст стандартного вывода (без ошибок) одной строкой.
    pub fn stdout_text(&self) -> String {
        let v: Vec<&str> = self.lines.iter().filter(|l| l.kind != OutKind::Error).map(|l| l.text.as_str()).collect();
        v.join("\n")
    }
}

/// Разобранные аргументы: флаги (`-l`, `-la`, `--all`), флаги со значением, позиционные.
#[derive(Debug, Clone, Default)]
pub struct Args {
    pub flags: Vec<String>,
    pub values: Vec<(String, String)>,
    pub pos: Vec<String>,
}

impl Args {
    /// `with_value` — короткие флаги, за которыми идёт значение (`-n 5`, `-n5`).
    pub fn parse(args: &[String], with_value: &[&str]) -> Args {
        let mut a = Args::default();
        let mut i = 0;
        let mut only_pos = false;
        while i < args.len() {
            let s = &args[i];
            if only_pos || s == "-" || !s.starts_with('-') || s.len() == 1 {
                a.pos.push(s.clone());
            } else if s == "--" {
                only_pos = true;
            } else if let Some(long) = s.strip_prefix("--") {
                match long.split_once('=') {
                    Some((k, v)) => a.values.push((k.to_string(), v.to_string())),
                    None => a.flags.push(long.to_string()),
                }
            } else {
                let body: Vec<char> = s[1..].chars().collect();
                let mut j = 0;
                while j < body.len() {
                    let f = body[j].to_string();
                    if with_value.contains(&f.as_str()) {
                        let rest: String = body[j + 1..].iter().collect();
                        if !rest.is_empty() {
                            a.values.push((f, rest));
                        } else if i + 1 < args.len() {
                            i += 1;
                            a.values.push((f, args[i].clone()));
                        } else {
                            a.flags.push(f);
                        }
                        break;
                    }
                    // число сразу после минуса: -5 → значение для n (head/tail)
                    if body[j].is_ascii_digit() && j == 0 {
                        let num: String = body.iter().collect();
                        a.values.push(("n".to_string(), num));
                        break;
                    }
                    a.flags.push(f);
                    j += 1;
                }
            }
            i += 1;
        }
        a
    }

    pub fn has(&self, f: &str) -> bool {
        self.flags.iter().any(|x| x == f)
    }

    pub fn opt(&self, f: &str) -> Option<&str> {
        self.values.iter().rev().find(|(k, _)| k == f).map(|(_, v)| v.as_str())
    }

    pub fn opt_usize(&self, f: &str, default: usize) -> usize {
        self.opt(f).and_then(|v| v.parse().ok()).unwrap_or(default)
    }
}

/// Ширина строки в символах.
pub fn width(s: &str) -> usize {
    s.chars().count()
}

pub fn pad_right(s: &str, w: usize) -> String {
    let n = width(s);
    if n >= w {
        s.to_string()
    } else {
        format!("{}{}", s, " ".repeat(w - n))
    }
}

pub fn pad_left(s: &str, w: usize) -> String {
    let n = width(s);
    if n >= w {
        s.to_string()
    } else {
        format!("{}{}", " ".repeat(w - n), s)
    }
}

/// Выровнять строки таблицы по колонкам. `right[i]` — выравнивать колонку вправо.
pub fn table(rows: &[Vec<String>], right: &[bool]) -> Vec<String> {
    let cols = rows.iter().map(|r| r.len()).max().unwrap_or(0);
    let mut widths = vec![0usize; cols];
    for r in rows {
        for (i, c) in r.iter().enumerate() {
            widths[i] = widths[i].max(width(c));
        }
    }
    rows.iter()
        .map(|r| {
            let mut parts = Vec::new();
            for (i, c) in r.iter().enumerate() {
                let last = i + 1 == r.len();
                let is_right = right.get(i).copied().unwrap_or(false);
                let cell = if is_right {
                    pad_left(c, widths[i])
                } else if last {
                    c.clone()
                } else {
                    pad_right(c, widths[i])
                };
                parts.push(cell);
            }
            parts.join("  ").trim_end().to_string()
        })
        .collect()
}

/// Размер по-человечески: 1.2K, 3.4M, 512.
pub fn human_size(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["", "K", "M", "G", "T"];
    let mut v = bytes as f64;
    let mut u = 0;
    while v >= 1024.0 && u + 1 < UNITS.len() {
        v /= 1024.0;
        u += 1;
    }
    if u == 0 {
        format!("{}", bytes)
    } else if v >= 10.0 {
        format!("{:.0}{}", v, UNITS[u])
    } else {
        format!("{:.1}{}", v, UNITS[u])
    }
}

/// Разложить имена в несколько колонок шириной до `max_width` символов.
pub fn columns(names: &[String], max_width: usize) -> Vec<String> {
    if names.is_empty() {
        return Vec::new();
    }
    let w = names.iter().map(|n| width(n)).max().unwrap_or(1) + 2;
    let per_line = (max_width / w).max(1);
    names
        .chunks(per_line)
        .map(|chunk| {
            let mut line = String::new();
            for (i, n) in chunk.iter().enumerate() {
                if i + 1 == chunk.len() {
                    line.push_str(n);
                } else {
                    line.push_str(&pad_right(n, w));
                }
            }
            line
        })
        .collect()
}

/// Расстояние Левенштейна — для подсказки «возможно, вы имели в виду».
pub fn levenshtein(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    for i in 1..=a.len() {
        let mut cur = vec![i; b.len() + 1];
        for j in 1..=b.len() {
            let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };
            cur[j] = (prev[j] + 1).min(cur[j - 1] + 1).min(prev[j - 1] + cost);
        }
        prev = cur;
    }
    prev[b.len()]
}

/// Условный хеш строки в 32 шестнадцатеричных знака (похож на md5, но свой).
pub fn fake_md5(data: &[u8]) -> String {
    let mut h1: u64 = 0xcbf2_9ce4_8422_2325;
    let mut h2: u64 = 0x8422_2325_cbf2_9ce4;
    for (i, b) in data.iter().enumerate() {
        h1 ^= *b as u64;
        h1 = h1.wrapping_mul(0x0000_0100_0000_01b3);
        h2 ^= (*b as u64).rotate_left((i % 61) as u32);
        h2 = h2.wrapping_mul(0x9E37_79B9_7F4A_7C15).rotate_left(13);
    }
    format!("{:016x}{:016x}", h1, h2 ^ h1.rotate_left(29))
}

const B64: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

pub fn base64_encode(data: &[u8]) -> String {
    let mut out = String::new();
    for chunk in data.chunks(3) {
        let b = [chunk[0], *chunk.get(1).unwrap_or(&0), *chunk.get(2).unwrap_or(&0)];
        let n = ((b[0] as u32) << 16) | ((b[1] as u32) << 8) | b[2] as u32;
        out.push(B64[((n >> 18) & 63) as usize] as char);
        out.push(B64[((n >> 12) & 63) as usize] as char);
        out.push(if chunk.len() > 1 { B64[((n >> 6) & 63) as usize] as char } else { '=' });
        out.push(if chunk.len() > 2 { B64[(n & 63) as usize] as char } else { '=' });
    }
    out
}

pub fn base64_decode(text: &str) -> Option<Vec<u8>> {
    let mut out = Vec::new();
    let mut buf: u32 = 0;
    let mut bits = 0;
    for c in text.bytes() {
        if c == b'=' || c.is_ascii_whitespace() {
            continue;
        }
        let v = B64.iter().position(|x| *x == c)? as u32;
        buf = (buf << 6) | v;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push(((buf >> bits) & 0xff) as u8);
        }
    }
    Some(out)
}
