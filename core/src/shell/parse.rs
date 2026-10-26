//! Разбор командной строки: кавычки `'` и `"`, экранирование `\`, комментарии `#`,
//! разделители `;` `&&` `||`, конвейер `|`, перенаправления `>` `>>` `2>` `<`.
//! Подстановка переменных `$X`/`${X}`, `~` и шаблонов `*`/`?` — отдельным шагом
//! (`expand_word`), когда известны окружение и файловая система.

use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Quote {
    None,
    Single,
    Double,
}

/// Кусок слова: текст и как он был записан (от этого зависит подстановка).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Seg {
    pub text: String,
    pub quote: Quote,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Word {
    pub segs: Vec<Seg>,
}

impl Word {
    /// Текст слова как написан, без подстановок.
    pub fn raw(&self) -> String {
        self.segs.iter().map(|s| s.text.as_str()).collect()
    }

    /// Слово целиком в кавычках (ничего подставлять не надо).
    pub fn fully_quoted(&self) -> bool {
        !self.segs.is_empty() && self.segs.iter().all(|s| s.quote != Quote::None)
    }
}

/// Связь с предыдущим конвейером.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sep {
    Seq,
    And,
    Or,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Redirs {
    /// Куда писать вывод и дописывать ли (`>>`).
    pub stdout: Option<(Word, bool)>,
    pub stderr: Option<Word>,
    pub stdin: Option<Word>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Simple {
    pub words: Vec<Word>,
    pub redirs: Redirs,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Pipeline {
    pub cmds: Vec<Simple>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Job {
    pub sep: Sep,
    pub pipeline: Pipeline,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Tok {
    Word(Word),
    Op(&'static str),
}

/// Разбить строку на слова и операторы.
fn lex(line: &str) -> Result<Vec<Tok>, String> {
    let chars: Vec<char> = line.chars().collect();
    let mut toks = Vec::new();
    let mut segs: Vec<Seg> = Vec::new();
    let mut buf = String::new();
    let mut quote = Quote::None;
    let mut i = 0;

    fn flush_seg(segs: &mut Vec<Seg>, buf: &mut String, quote: Quote) {
        if !buf.is_empty() || quote != Quote::None {
            segs.push(Seg { text: std::mem::take(buf), quote });
        }
    }
    fn flush_word(toks: &mut Vec<Tok>, segs: &mut Vec<Seg>, buf: &mut String) {
        flush_seg(segs, buf, Quote::None);
        if !segs.is_empty() {
            toks.push(Tok::Word(Word { segs: std::mem::take(segs) }));
        }
    }

    while i < chars.len() {
        let c = chars[i];
        match quote {
            Quote::Single => {
                if c == '\'' {
                    flush_seg(&mut segs, &mut buf, Quote::Single);
                    quote = Quote::None;
                } else {
                    buf.push(c);
                }
                i += 1;
            }
            Quote::Double => {
                if c == '"' {
                    flush_seg(&mut segs, &mut buf, Quote::Double);
                    quote = Quote::None;
                } else if c == '\\' && i + 1 < chars.len() && matches!(chars[i + 1], '"' | '\\' | '$' | '`') {
                    // экранированный символ в двойных кавычках — буквально, без подстановки
                    flush_seg(&mut segs, &mut buf, Quote::Double);
                    segs.push(Seg { text: chars[i + 1].to_string(), quote: Quote::Single });
                    i += 1;
                } else {
                    buf.push(c);
                }
                i += 1;
            }
            Quote::None => {
                match c {
                    ' ' | '\t' | '\n' | '\r' => {
                        flush_word(&mut toks, &mut segs, &mut buf);
                    }
                    '#' if segs.is_empty() && buf.is_empty() => break,
                    '\'' => {
                        flush_seg(&mut segs, &mut buf, Quote::None);
                        quote = Quote::Single;
                    }
                    '"' => {
                        flush_seg(&mut segs, &mut buf, Quote::None);
                        quote = Quote::Double;
                    }
                    '\\' => {
                        if i + 1 < chars.len() {
                            flush_seg(&mut segs, &mut buf, Quote::None);
                            segs.push(Seg { text: chars[i + 1].to_string(), quote: Quote::Single });
                            i += 1;
                        }
                    }
                    ';' => {
                        flush_word(&mut toks, &mut segs, &mut buf);
                        toks.push(Tok::Op(";"));
                    }
                    '&' => {
                        flush_word(&mut toks, &mut segs, &mut buf);
                        if i + 1 < chars.len() && chars[i + 1] == '&' {
                            toks.push(Tok::Op("&&"));
                            i += 1;
                        } else {
                            // фоновый запуск не поддерживается — считаем разделителем
                            toks.push(Tok::Op(";"));
                        }
                    }
                    '|' => {
                        flush_word(&mut toks, &mut segs, &mut buf);
                        if i + 1 < chars.len() && chars[i + 1] == '|' {
                            toks.push(Tok::Op("||"));
                            i += 1;
                        } else {
                            toks.push(Tok::Op("|"));
                        }
                    }
                    '<' => {
                        flush_word(&mut toks, &mut segs, &mut buf);
                        toks.push(Tok::Op("<"));
                    }
                    '>' => {
                        let is_stderr = segs.is_empty() && buf == "2";
                        if is_stderr {
                            buf.clear();
                        } else {
                            flush_word(&mut toks, &mut segs, &mut buf);
                        }
                        if i + 1 < chars.len() && chars[i + 1] == '>' {
                            toks.push(Tok::Op(if is_stderr { "2>>" } else { ">>" }));
                            i += 1;
                        } else {
                            toks.push(Tok::Op(if is_stderr { "2>" } else { ">" }));
                        }
                    }
                    _ => buf.push(c),
                }
                i += 1;
            }
        }
    }
    if quote != Quote::None {
        return Err("незакрытая кавычка".to_string());
    }
    flush_word(&mut toks, &mut segs, &mut buf);
    Ok(toks)
}

/// Разобрать строку в список конвейеров с разделителями.
pub fn parse(line: &str) -> Result<Vec<Job>, String> {
    let toks = lex(line)?;
    let mut jobs: Vec<Job> = Vec::new();
    let mut sep = Sep::Seq;
    let mut pipeline = Pipeline::default();
    let mut simple = Simple::default();
    let mut pending: Option<&'static str> = None;

    fn finish_simple(pipeline: &mut Pipeline, simple: &mut Simple) {
        if !simple.words.is_empty() || simple.redirs != Redirs::default() {
            pipeline.cmds.push(std::mem::take(simple));
        }
    }

    for tok in toks {
        match tok {
            Tok::Word(w) => {
                if let Some(op) = pending.take() {
                    match op {
                        ">" => simple.redirs.stdout = Some((w, false)),
                        ">>" => simple.redirs.stdout = Some((w, true)),
                        "2>" | "2>>" => simple.redirs.stderr = Some(w),
                        "<" => simple.redirs.stdin = Some(w),
                        _ => {}
                    }
                } else {
                    simple.words.push(w);
                }
            }
            Tok::Op(op) => {
                if pending.is_some() {
                    return Err(format!("после «{}» ожидалось имя файла", pending.unwrap()));
                }
                match op {
                    ">" | ">>" | "2>" | "2>>" | "<" => pending = Some(op),
                    "|" => {
                        if simple.words.is_empty() {
                            return Err("перед «|» ожидалась команда".to_string());
                        }
                        finish_simple(&mut pipeline, &mut simple);
                    }
                    ";" | "&&" | "||" => {
                        if simple.words.is_empty() && op != ";" {
                            return Err(format!("перед «{}» ожидалась команда", op));
                        }
                        finish_simple(&mut pipeline, &mut simple);
                        if !pipeline.cmds.is_empty() {
                            jobs.push(Job { sep, pipeline: std::mem::take(&mut pipeline) });
                        }
                        sep = match op {
                            "&&" => Sep::And,
                            "||" => Sep::Or,
                            _ => Sep::Seq,
                        };
                    }
                    _ => {}
                }
            }
        }
    }
    if let Some(op) = pending {
        return Err(format!("после «{}» ожидалось имя файла", op));
    }
    finish_simple(&mut pipeline, &mut simple);
    if !pipeline.cmds.is_empty() {
        jobs.push(Job { sep, pipeline });
    }
    Ok(jobs)
}

/// Всё, что нужно для подстановок в слове.
pub struct Expander<'a> {
    pub env: &'a BTreeMap<String, String>,
    pub home: &'a str,
    pub status: i32,
    /// Раскрыть шаблон с `*`/`?` в список имён (пустой — оставить как есть).
    pub glob: &'a dyn Fn(&str) -> Vec<String>,
}

/// Подставить `$X`, `${X}`, `$?` в тексте.
fn expand_vars(text: &str, ex: &Expander<'_>) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::new();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '$' && i + 1 < chars.len() {
            let next = chars[i + 1];
            if next == '?' {
                out.push_str(&ex.status.to_string());
                i += 2;
                continue;
            }
            if next == '{' {
                if let Some(end) = chars[i + 2..].iter().position(|c| *c == '}') {
                    let name: String = chars[i + 2..i + 2 + end].iter().collect();
                    out.push_str(ex.env.get(&name).map(|s| s.as_str()).unwrap_or(""));
                    i += end + 3;
                    continue;
                }
            }
            if next.is_alphabetic() || next == '_' {
                let mut j = i + 1;
                while j < chars.len() && (chars[j].is_alphanumeric() || chars[j] == '_') {
                    j += 1;
                }
                let name: String = chars[i + 1..j].iter().collect();
                out.push_str(ex.env.get(&name).map(|s| s.as_str()).unwrap_or(""));
                i = j;
                continue;
            }
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

/// Слово после подстановок без раскрытия шаблонов (для имён файлов перенаправления).
pub fn expand_literal(word: &Word, ex: &Expander<'_>) -> String {
    let mut out = String::new();
    for (i, seg) in word.segs.iter().enumerate() {
        match seg.quote {
            Quote::Single => out.push_str(&seg.text),
            Quote::Double => out.push_str(&expand_vars(&seg.text, ex)),
            Quote::None => {
                let mut t = expand_vars(&seg.text, ex);
                if i == 0 {
                    if t == "~" {
                        t = ex.home.to_string();
                    } else if let Some(rest) = t.strip_prefix("~/") {
                        t = format!("{}/{}", ex.home, rest);
                    }
                }
                out.push_str(&t);
            }
        }
    }
    out
}

/// Слово после всех подстановок; шаблон может дать несколько слов.
pub fn expand_word(word: &Word, ex: &Expander<'_>) -> Vec<String> {
    let text = expand_literal(word, ex);
    let has_pattern = word.segs.iter().any(|s| s.quote == Quote::None && (s.text.contains('*') || s.text.contains('?')));
    if has_pattern {
        let found = (ex.glob)(&text);
        if !found.is_empty() {
            return found;
        }
    }
    vec![text]
}
