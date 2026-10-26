//! Лексер Adder: текст → лексемы.
//! Отступы превращаются в INDENT/DEDENT, конец логической строки — в NEWLINE.
//! Внутри незакрытых скобок переносы строк не считаются. Разбирает числа
//! (`0xFF`, `1_000`, `1e9`), строки с экранированием, тройные кавычки и f-строки
//! (выражение внутри `{}` отдаётся парсеру как текст, он разбирает его рекурсивно).

use crate::error::LangError;

#[derive(Debug, Clone, PartialEq)]
pub enum Tok {
    Int(i64),
    Float(f64),
    Str(String),
    FStr(Vec<FPart>),
    Name(String),
    Kw(&'static str),
    Op(&'static str),
    Newline,
    Indent,
    Dedent,
    Eof,
}

/// Часть f-строки: текст или выражение с необязательными преобразованием (`!r`) и форматом.
#[derive(Debug, Clone, PartialEq)]
pub enum FPart {
    Lit(String),
    Expr { src: String, conv: Option<char>, spec: Option<String>, line: u32, col: u32 },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub tok: Tok,
    pub line: u32,
    pub col: u32,
}

const KEYWORDS: &[&str] = &[
    "and", "or", "not", "if", "elif", "else", "while", "for", "in", "def", "return", "lambda", "try",
    "except", "finally", "raise", "pass", "break", "continue", "global", "true", "false", "null", "as",
];

/// Операторы — от длинных к коротким, чтобы жадно брать самый длинный.
const OPS: &[&str] = &[
    "**=", "//=", "**", "//", "==", "!=", "<=", ">=", "+=", "-=", "*=", "/=", "%=", "+", "-", "*", "/",
    "%", "<", ">", "=", "(", ")", "[", "]", "{", "}", ",", ":", ".",
];

/// Разобрать исходник целиком.
pub fn tokenize(src: &str) -> Result<Vec<Token>, LangError> {
    let mut lx = Lexer::new(src);
    lx.run()?;
    Ok(lx.tokens)
}

/// Лексемы одного выражения (вложенное выражение f-строки). Номера строк и позиций
/// сдвигаются так, чтобы указывать на место в исходном файле.
pub fn tokenize_expr(src: &str, line: u32, col: u32) -> Result<Vec<Token>, LangError> {
    let shift = |mut e: LangError| {
        if e.line == 1 {
            e.col += col.saturating_sub(1);
        }
        e.line += line.saturating_sub(1);
        e
    };
    let mut toks = tokenize(src.trim()).map_err(shift)?;
    for t in &mut toks {
        if t.line == 1 {
            t.col += col.saturating_sub(1);
        }
        t.line += line.saturating_sub(1);
    }
    Ok(toks)
}

struct Lexer {
    chars: Vec<char>,
    pos: usize,
    line: u32,
    col: u32,
    tokens: Vec<Token>,
    indents: Vec<usize>,
    depth: usize,
    indent_char: Option<char>,
    at_line_start: bool,
}

impl Lexer {
    fn new(src: &str) -> Self {
        Lexer {
            chars: src.chars().collect(),
            pos: 0,
            line: 1,
            col: 1,
            tokens: Vec::new(),
            indents: vec![0],
            depth: 0,
            indent_char: None,
            at_line_start: true,
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn peek_at(&self, n: usize) -> Option<char> {
        self.chars.get(self.pos + n).copied()
    }

    fn bump(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.pos += 1;
        if c == '\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
        Some(c)
    }

    fn push(&mut self, tok: Tok, line: u32, col: u32) {
        self.tokens.push(Token { tok, line, col });
    }

    fn err(&self, msg: impl Into<String>) -> LangError {
        LangError::syntax(self.line, self.col, msg)
    }

    fn run(&mut self) -> Result<(), LangError> {
        loop {
            if self.at_line_start && self.depth == 0 {
                self.handle_indent()?;
            }
            let c = match self.peek() {
                Some(c) => c,
                None => break,
            };
            let (line, col) = (self.line, self.col);
            match c {
                '\n' => {
                    self.bump();
                    if self.depth == 0 {
                        self.newline(line, col);
                        self.at_line_start = true;
                    }
                }
                ' ' | '\t' | '\r' => {
                    self.bump();
                }
                '#' => self.skip_comment(),
                '\\' if self.peek_at(1) == Some('\n')
                    || (self.peek_at(1) == Some('\r') && self.peek_at(2) == Some('\n')) =>
                {
                    self.bump();
                    if self.peek() == Some('\r') {
                        self.bump();
                    }
                    self.bump();
                }
                '0'..='9' => self.number(line, col)?,
                '"' | '\'' => self.string(false, line, col)?,
                'f' | 'F' if matches!(self.peek_at(1), Some('"') | Some('\'')) => {
                    self.bump();
                    self.string(true, line, col)?;
                }
                c if is_ident_start(c) => self.name(line, col),
                _ => self.operator(line, col)?,
            }
        }
        if !matches!(self.tokens.last().map(|t| &t.tok), None | Some(Tok::Newline)) {
            let (line, col) = (self.line, self.col);
            self.push(Tok::Newline, line, col);
        }
        while self.indents.len() > 1 {
            self.indents.pop();
            let (line, col) = (self.line, self.col);
            self.push(Tok::Dedent, line, col);
        }
        let (line, col) = (self.line, self.col);
        self.push(Tok::Eof, line, col);
        Ok(())
    }

    fn skip_comment(&mut self) {
        while let Some(c) = self.peek() {
            if c == '\n' {
                break;
            }
            self.bump();
        }
    }

    fn newline(&mut self, line: u32, col: u32) {
        if matches!(self.tokens.last().map(|t| &t.tok), Some(Tok::Newline) | Some(Tok::Indent) | Some(Tok::Dedent) | None) {
            return;
        }
        self.push(Tok::Newline, line, col);
    }

    /// Начало логической строки: измерить отступ, пропустить пустые строки и комментарии,
    /// выдать INDENT/DEDENT.
    fn handle_indent(&mut self) -> Result<(), LangError> {
        loop {
            let mut width = 0usize;
            let mut has_tab = false;
            let mut has_space = false;
            while let Some(c) = self.peek() {
                match c {
                    ' ' => has_space = true,
                    '\t' => has_tab = true,
                    _ => break,
                }
                width += 1;
                self.bump();
            }
            match self.peek() {
                None => return Ok(()),
                Some('\n') | Some('\r') => {
                    self.bump();
                    continue;
                }
                Some('#') => {
                    self.skip_comment();
                    continue;
                }
                _ => {}
            }
            if has_tab && has_space {
                return Err(self.err("смешение табов и пробелов в отступе"));
            }
            if width > 0 {
                let ch = if has_tab { '\t' } else { ' ' };
                match self.indent_char {
                    None => self.indent_char = Some(ch),
                    Some(prev) if prev != ch => {
                        return Err(self.err("в файле смешаны отступы табами и пробелами"));
                    }
                    _ => {}
                }
            }
            self.at_line_start = false;
            let top = *self.indents.last().unwrap_or(&0);
            let (line, col) = (self.line, self.col);
            if width > top {
                if self.tokens.is_empty() {
                    return Err(self.err("неожиданный отступ"));
                }
                self.indents.push(width);
                self.push(Tok::Indent, line, col);
            } else if width < top {
                while self.indents.len() > 1 && *self.indents.last().unwrap() > width {
                    self.indents.pop();
                    self.push(Tok::Dedent, line, col);
                }
                if *self.indents.last().unwrap() != width {
                    return Err(self.err("несогласованный отступ"));
                }
            }
            return Ok(());
        }
    }

    fn number(&mut self, line: u32, col: u32) -> Result<(), LangError> {
        let mut text = String::new();
        if self.peek() == Some('0') {
            let radix = match self.peek_at(1) {
                Some('x') | Some('X') => Some(16),
                Some('b') | Some('B') => Some(2),
                Some('o') | Some('O') => Some(8),
                _ => None,
            };
            if let Some(radix) = radix {
                self.bump();
                self.bump();
                while let Some(c) = self.peek() {
                    if c.is_ascii_alphanumeric() || c == '_' {
                        if c != '_' {
                            text.push(c);
                        }
                        self.bump();
                    } else {
                        break;
                    }
                }
                let value = i64::from_str_radix(&text, radix)
                    .map_err(|_| LangError::syntax(line, col, "неверная запись числа"))?;
                self.push(Tok::Int(value), line, col);
                return Ok(());
            }
        }
        let mut is_float = false;
        self.digits(&mut text);
        if self.peek() == Some('.') && self.peek_at(1).map_or(false, |c| c.is_ascii_digit()) {
            is_float = true;
            text.push('.');
            self.bump();
            self.digits(&mut text);
        }
        if matches!(self.peek(), Some('e') | Some('E')) {
            let next = self.peek_at(1);
            let exp_ok = next.map_or(false, |c| c.is_ascii_digit())
                || (matches!(next, Some('+') | Some('-')) && self.peek_at(2).map_or(false, |c| c.is_ascii_digit()));
            if exp_ok {
                is_float = true;
                text.push('e');
                self.bump();
                if let Some(sign @ ('+' | '-')) = self.peek() {
                    text.push(sign);
                    self.bump();
                }
                self.digits(&mut text);
            }
        }
        if is_float {
            let v: f64 = text.parse().map_err(|_| LangError::syntax(line, col, "неверная запись числа"))?;
            self.push(Tok::Float(v), line, col);
        } else {
            match text.parse::<i64>() {
                Ok(v) => self.push(Tok::Int(v), line, col),
                Err(_) => return Err(LangError::syntax(line, col, "слишком большое целое число")),
            }
        }
        Ok(())
    }

    fn digits(&mut self, out: &mut String) {
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                out.push(c);
                self.bump();
            } else if c == '_' && self.peek_at(1).map_or(false, |d| d.is_ascii_digit()) {
                self.bump();
            } else {
                break;
            }
        }
    }

    fn name(&mut self, line: u32, col: u32) {
        let mut s = String::new();
        while let Some(c) = self.peek() {
            if is_ident_char(c) {
                s.push(c);
                self.bump();
            } else {
                break;
            }
        }
        if let Some(kw) = KEYWORDS.iter().find(|k| **k == s) {
            self.push(Tok::Kw(kw), line, col);
        } else {
            self.push(Tok::Name(s), line, col);
        }
    }

    fn operator(&mut self, line: u32, col: u32) -> Result<(), LangError> {
        for op in OPS {
            let matches = op.chars().enumerate().all(|(i, c)| self.peek_at(i) == Some(c));
            if matches {
                for _ in 0..op.chars().count() {
                    self.bump();
                }
                match *op {
                    "(" | "[" | "{" => self.depth += 1,
                    ")" | "]" | "}" => self.depth = self.depth.saturating_sub(1),
                    _ => {}
                }
                self.push(Tok::Op(op), line, col);
                return Ok(());
            }
        }
        let c = self.peek().unwrap_or(' ');
        Err(self.err(format!("неизвестный символ '{}'", c)))
    }

    /// Строка или f-строка; открывающая кавычка ещё не прочитана.
    fn string(&mut self, is_f: bool, line: u32, col: u32) -> Result<(), LangError> {
        let quote = self.bump().unwrap_or('"');
        let triple = self.peek() == Some(quote) && self.peek_at(1) == Some(quote);
        if triple {
            self.bump();
            self.bump();
        }
        if is_f {
            let parts = self.fstring_body(quote, triple)?;
            self.push(Tok::FStr(parts), line, col);
        } else {
            let s = self.string_body(quote, triple)?;
            self.push(Tok::Str(s), line, col);
        }
        Ok(())
    }

    /// Закрывающая кавычка на текущей позиции? Если да — съесть её.
    fn try_close(&mut self, quote: char, triple: bool) -> bool {
        if self.peek() != Some(quote) {
            return false;
        }
        if triple {
            if self.peek_at(1) == Some(quote) && self.peek_at(2) == Some(quote) {
                self.bump();
                self.bump();
                self.bump();
                return true;
            }
            return false;
        }
        self.bump();
        true
    }

    fn string_body(&mut self, quote: char, triple: bool) -> Result<String, LangError> {
        let mut s = String::new();
        loop {
            if self.try_close(quote, triple) {
                return Ok(s);
            }
            let c = match self.peek() {
                Some(c) => c,
                None => return Err(self.err("незакрытая строка")),
            };
            if c == '\n' && !triple {
                return Err(self.err("незакрытая строка"));
            }
            if c == '\\' {
                self.bump();
                self.escape(&mut s)?;
                continue;
            }
            s.push(c);
            self.bump();
        }
    }

    /// Экранированная последовательность после `\`.
    fn escape(&mut self, out: &mut String) -> Result<(), LangError> {
        let c = match self.bump() {
            Some(c) => c,
            None => return Err(self.err("незакрытая строка")),
        };
        match c {
            'n' => out.push('\n'),
            't' => out.push('\t'),
            'r' => out.push('\r'),
            '0' => out.push('\0'),
            '\\' => out.push('\\'),
            '"' => out.push('"'),
            '\'' => out.push('\''),
            '\n' => {}
            'x' | 'u' => {
                let n = if c == 'x' { 2 } else { 4 };
                let mut code = 0u32;
                for _ in 0..n {
                    let d = self.bump().and_then(|d| d.to_digit(16));
                    match d {
                        Some(d) => code = code * 16 + d,
                        None => return Err(self.err("неверная экранированная последовательность")),
                    }
                }
                match char::from_u32(code) {
                    Some(ch) => out.push(ch),
                    None => return Err(self.err("неверный код символа")),
                }
            }
            other => {
                out.push('\\');
                out.push(other);
            }
        }
        Ok(())
    }

    fn fstring_body(&mut self, quote: char, triple: bool) -> Result<Vec<FPart>, LangError> {
        let mut parts = Vec::new();
        let mut lit = String::new();
        loop {
            if self.try_close(quote, triple) {
                if !lit.is_empty() {
                    parts.push(FPart::Lit(lit));
                }
                return Ok(parts);
            }
            let c = match self.peek() {
                Some(c) => c,
                None => return Err(self.err("незакрытая f-строка")),
            };
            match c {
                '\n' if !triple => return Err(self.err("незакрытая f-строка")),
                '\\' => {
                    self.bump();
                    self.escape(&mut lit)?;
                }
                '{' if self.peek_at(1) == Some('{') => {
                    self.bump();
                    self.bump();
                    lit.push('{');
                }
                '}' if self.peek_at(1) == Some('}') => {
                    self.bump();
                    self.bump();
                    lit.push('}');
                }
                '}' => return Err(self.err("одиночная '}' в f-строке")),
                '{' => {
                    if !lit.is_empty() {
                        parts.push(FPart::Lit(std::mem::take(&mut lit)));
                    }
                    self.bump();
                    let (line, col) = (self.line, self.col);
                    let (src, conv, spec) = self.fexpr()?;
                    if src.trim().is_empty() {
                        return Err(LangError::syntax(line, col, "пустое выражение в f-строке"));
                    }
                    parts.push(FPart::Expr { src, conv, spec, line, col });
                }
                _ => {
                    lit.push(c);
                    self.bump();
                }
            }
        }
    }

    /// Выражение внутри `{}` f-строки: читаем до `}` на нулевой глубине скобок;
    /// `:` на нулевой глубине начинает формат, `!r`/`!s` — преобразование.
    fn fexpr(&mut self) -> Result<(String, Option<char>, Option<String>), LangError> {
        let mut src = String::new();
        let mut depth = 0i32;
        let mut conv = None;
        loop {
            let c = match self.peek() {
                Some(c) => c,
                None => return Err(self.err("незакрытая '{' в f-строке")),
            };
            match c {
                '(' | '[' | '{' => {
                    depth += 1;
                    src.push(c);
                    self.bump();
                }
                ')' | ']' => {
                    depth -= 1;
                    src.push(c);
                    self.bump();
                }
                '}' if depth > 0 => {
                    depth -= 1;
                    src.push(c);
                    self.bump();
                }
                '}' => {
                    self.bump();
                    return Ok((src, conv, None));
                }
                '"' | '\'' => {
                    src.push(c);
                    self.bump();
                    loop {
                        let d = match self.bump() {
                            Some(d) => d,
                            None => return Err(self.err("незакрытая строка в f-строке")),
                        };
                        src.push(d);
                        if d == '\\' {
                            if let Some(e) = self.bump() {
                                src.push(e);
                            }
                        } else if d == c {
                            break;
                        }
                    }
                }
                '!' if depth == 0
                    && matches!(self.peek_at(1), Some('r') | Some('s'))
                    && matches!(self.peek_at(2), Some('}') | Some(':')) =>
                {
                    self.bump();
                    conv = self.bump();
                }
                ':' if depth == 0 => {
                    self.bump();
                    let mut spec = String::new();
                    loop {
                        match self.bump() {
                            Some('}') => break,
                            Some('\n') | None => return Err(self.err("незакрытая '{' в f-строке")),
                            Some(d) => spec.push(d),
                        }
                    }
                    return Ok((src, conv, Some(spec)));
                }
                '\n' => return Err(self.err("перенос строки внутри выражения f-строки")),
                _ => {
                    src.push(c);
                    self.bump();
                }
            }
        }
    }
}

fn is_ident_start(c: char) -> bool {
    c == '_' || c.is_alphabetic()
}

fn is_ident_char(c: char) -> bool {
    c == '_' || c.is_alphanumeric()
}
