//! Парсер Adder: лексемы → дерево (`ast::Program`). Рекурсивный спуск,
//! приоритеты как в Python. Выражения внутри f-строк разбираются рекурсивно
//! (лексер отдаёт их текстом, здесь они снова проходят лексер и парсер).

use std::rc::Rc;

use crate::ast::*;
use crate::error::LangError;
use crate::lexer::{self, FPart, Tok, Token};

type PResult<T> = Result<T, LangError>;

/// Разобрать программу из лексем.
pub fn parse(tokens: Vec<Token>) -> PResult<Program> {
    let mut p = Parser { toks: tokens, pos: 0 };
    let mut body = Vec::new();
    while !matches!(p.peek(), Tok::Eof) {
        body.push(p.parse_stmt()?);
    }
    Ok(Program { body })
}

/// Разобрать одно выражение (для вложенных выражений f-строк).
pub fn parse_expression(tokens: Vec<Token>) -> PResult<Expr> {
    let mut p = Parser { toks: tokens, pos: 0 };
    let e = p.parse_expr_list()?;
    if matches!(p.peek(), Tok::Newline) {
        p.advance();
    }
    if !matches!(p.peek(), Tok::Eof) {
        return Err(p.err(format!("лишний текст в выражении: {}", describe(p.peek()))));
    }
    Ok(e)
}

struct Parser {
    toks: Vec<Token>,
    pos: usize,
}

fn describe(t: &Tok) -> String {
    match t {
        Tok::Int(n) => format!("'{}'", n),
        Tok::Float(f) => format!("'{}'", f),
        Tok::Str(_) | Tok::FStr(_) => "строка".to_string(),
        Tok::Name(n) => format!("'{}'", n),
        Tok::Kw(k) => format!("'{}'", k),
        Tok::Op(o) => format!("'{}'", o),
        Tok::Newline => "конец строки".to_string(),
        Tok::Indent => "отступ".to_string(),
        Tok::Dedent => "конец блока".to_string(),
        Tok::Eof => "конец файла".to_string(),
    }
}

fn aug_op(t: &Tok) -> Option<BinOp> {
    match t {
        Tok::Op("+=") => Some(BinOp::Add),
        Tok::Op("-=") => Some(BinOp::Sub),
        Tok::Op("*=") => Some(BinOp::Mul),
        Tok::Op("/=") => Some(BinOp::Div),
        Tok::Op("//=") => Some(BinOp::FloorDiv),
        Tok::Op("%=") => Some(BinOp::Mod),
        Tok::Op("**=") => Some(BinOp::Pow),
        _ => None,
    }
}

impl Parser {
    fn tok(&self) -> &Token {
        &self.toks[self.pos.min(self.toks.len() - 1)]
    }

    fn peek(&self) -> &Tok {
        &self.tok().tok
    }

    fn peek_at(&self, n: usize) -> &Tok {
        &self.toks[(self.pos + n).min(self.toks.len() - 1)].tok
    }

    fn line(&self) -> u32 {
        self.tok().line
    }

    fn col(&self) -> u32 {
        self.tok().col
    }

    fn advance(&mut self) -> Token {
        let t = self.tok().clone();
        if self.pos < self.toks.len() - 1 {
            self.pos += 1;
        }
        t
    }

    fn err(&self, msg: impl Into<String>) -> LangError {
        LangError::syntax(self.line(), self.col(), msg)
    }

    fn is_op(&self, s: &str) -> bool {
        matches!(self.peek(), Tok::Op(o) if *o == s)
    }

    fn is_kw(&self, s: &str) -> bool {
        matches!(self.peek(), Tok::Kw(k) if *k == s)
    }

    fn eat_op(&mut self, s: &str) -> bool {
        if self.is_op(s) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn eat_kw(&mut self, s: &str) -> bool {
        if self.is_kw(s) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn expect_op(&mut self, s: &str) -> PResult<()> {
        if self.eat_op(s) {
            Ok(())
        } else {
            Err(self.err(format!("ожидалось '{}', найдено {}", s, describe(self.peek()))))
        }
    }

    fn expect_kw(&mut self, s: &str) -> PResult<()> {
        if self.eat_kw(s) {
            Ok(())
        } else {
            Err(self.err(format!("ожидалось '{}', найдено {}", s, describe(self.peek()))))
        }
    }

    fn expect_name(&mut self) -> PResult<Rc<str>> {
        match self.peek() {
            Tok::Name(n) => {
                let n: Rc<str> = Rc::from(n.as_str());
                self.advance();
                Ok(n)
            }
            other => Err(self.err(format!("ожидалось имя, найдено {}", describe(other)))),
        }
    }

    fn expect_newline(&mut self) -> PResult<()> {
        match self.peek() {
            Tok::Newline => {
                self.advance();
                Ok(())
            }
            Tok::Eof => Ok(()),
            other => Err(self.err(format!("ожидался конец строки, найдено {}", describe(other)))),
        }
    }

    fn mk(&self, line: u32, col: u32, kind: ExprKind) -> Expr {
        Expr { line, col, kind }
    }

    // ---------- операторы ----------

    /// Блок после `:` — либо с новой строки с отступом, либо один простой оператор в той же строке.
    fn parse_block(&mut self) -> PResult<Vec<Stmt>> {
        self.expect_op(":")?;
        if !matches!(self.peek(), Tok::Newline) {
            return Ok(vec![self.parse_simple_stmt()?]);
        }
        self.advance();
        if !matches!(self.peek(), Tok::Indent) {
            return Err(self.err("ожидался отступ"));
        }
        self.advance();
        let mut body = Vec::new();
        while !matches!(self.peek(), Tok::Dedent | Tok::Eof) {
            body.push(self.parse_stmt()?);
        }
        if matches!(self.peek(), Tok::Dedent) {
            self.advance();
        }
        Ok(body)
    }

    fn parse_stmt(&mut self) -> PResult<Stmt> {
        match self.peek() {
            Tok::Indent => Err(self.err("неожиданный отступ")),
            Tok::Kw("if") => self.parse_if(),
            Tok::Kw("while") => self.parse_while(),
            Tok::Kw("for") => self.parse_for(),
            Tok::Kw("def") => self.parse_def(),
            Tok::Kw("try") => self.parse_try(),
            _ => self.parse_simple_stmt(),
        }
    }

    fn parse_simple_stmt(&mut self) -> PResult<Stmt> {
        let line = self.line();
        let kind = match self.peek() {
            Tok::Kw("return") => {
                self.advance();
                if matches!(self.peek(), Tok::Newline | Tok::Eof) {
                    StmtKind::Return(None)
                } else {
                    StmtKind::Return(Some(self.parse_expr_list()?))
                }
            }
            Tok::Kw("raise") => {
                self.advance();
                if matches!(self.peek(), Tok::Newline | Tok::Eof) {
                    StmtKind::Raise(None)
                } else {
                    StmtKind::Raise(Some(self.parse_expr()?))
                }
            }
            Tok::Kw("global") => {
                self.advance();
                let mut names = vec![self.expect_name()?];
                while self.eat_op(",") {
                    names.push(self.expect_name()?);
                }
                StmtKind::Global(names)
            }
            Tok::Kw("pass") => {
                self.advance();
                StmtKind::Pass
            }
            Tok::Kw("break") => {
                self.advance();
                StmtKind::Break
            }
            Tok::Kw("continue") => {
                self.advance();
                StmtKind::Continue
            }
            Tok::Kw(k @ ("elif" | "else" | "except" | "finally")) => {
                return Err(self.err(format!("'{}' без соответствующего начала блока", k)));
            }
            _ => self.parse_expr_stmt()?,
        };
        self.expect_newline()?;
        Ok(Stmt { line, kind })
    }

    /// Выражение, присваивание или составное присваивание.
    fn parse_expr_stmt(&mut self) -> PResult<StmtKind> {
        let first = self.parse_expr_list()?;
        if self.is_op("=") {
            let mut targets = vec![first];
            let value;
            loop {
                self.advance();
                let e = self.parse_expr_list()?;
                if self.is_op("=") {
                    targets.push(e);
                } else {
                    value = e;
                    break;
                }
            }
            for t in &targets {
                self.check_target(t)?;
            }
            return Ok(StmtKind::Assign { targets, value });
        }
        if let Some(op) = aug_op(self.peek()) {
            self.advance();
            match first.kind {
                ExprKind::Name(_) | ExprKind::Index(..) => {}
                _ => return Err(LangError::syntax(first.line, first.col, "недопустимая цель присваивания")),
            }
            let value = self.parse_expr_list()?;
            return Ok(StmtKind::AugAssign { target: first, op, value });
        }
        Ok(StmtKind::Expr(first))
    }

    fn check_target(&self, e: &Expr) -> PResult<()> {
        match &e.kind {
            ExprKind::Name(_) | ExprKind::Index(..) | ExprKind::Attr(..) => Ok(()),
            ExprKind::Tuple(items) | ExprKind::List(items) => {
                for it in items {
                    self.check_target(it)?;
                }
                Ok(())
            }
            _ => Err(LangError::syntax(e.line, e.col, "недопустимая цель присваивания")),
        }
    }

    fn parse_if(&mut self) -> PResult<Stmt> {
        let line = self.line();
        self.advance();
        let cond = self.parse_expr()?;
        let body = self.parse_block()?;
        let mut branches = vec![(cond, body)];
        let mut orelse = None;
        loop {
            if self.is_kw("elif") {
                self.advance();
                let cond = self.parse_expr()?;
                let body = self.parse_block()?;
                branches.push((cond, body));
            } else if self.is_kw("else") {
                self.advance();
                orelse = Some(self.parse_block()?);
                break;
            } else {
                break;
            }
        }
        Ok(Stmt { line, kind: StmtKind::If { branches, orelse } })
    }

    fn parse_while(&mut self) -> PResult<Stmt> {
        let line = self.line();
        self.advance();
        let cond = self.parse_expr()?;
        let body = self.parse_block()?;
        Ok(Stmt { line, kind: StmtKind::While { cond, body } })
    }

    fn parse_for(&mut self) -> PResult<Stmt> {
        let line = self.line();
        self.advance();
        let target = self.parse_target_list()?;
        self.expect_kw("in")?;
        let iter = self.parse_expr_list()?;
        let body = self.parse_block()?;
        Ok(Stmt { line, kind: StmtKind::For { target, iter, body } })
    }

    fn parse_def(&mut self) -> PResult<Stmt> {
        let line = self.line();
        self.advance();
        let name = self.expect_name()?;
        self.expect_op("(")?;
        let params = self.parse_params(")")?;
        let body = self.parse_block()?;
        let def = FuncDef { name, params, body, line };
        Ok(Stmt { line, kind: StmtKind::Def(Rc::new(def)) })
    }

    /// Параметры до закрывающей лексемы `end` (`)` у def, `:` у lambda).
    fn parse_params(&mut self, end: &str) -> PResult<Vec<Param>> {
        let mut params: Vec<Param> = Vec::new();
        loop {
            if self.is_op(end) {
                break;
            }
            let name = self.expect_name()?;
            let default = if self.eat_op("=") { Some(self.parse_expr()?) } else { None };
            if default.is_none() && params.iter().any(|p| p.default.is_some()) {
                return Err(self.err("параметр без значения по умолчанию после параметра со значением"));
            }
            if params.iter().any(|p| p.name == name) {
                return Err(self.err(format!("повторяющийся параметр '{}'", name)));
            }
            params.push(Param { name, default });
            if !self.eat_op(",") {
                break;
            }
        }
        self.expect_op(end)?;
        Ok(params)
    }

    fn parse_try(&mut self) -> PResult<Stmt> {
        let line = self.line();
        self.advance();
        let body = self.parse_block()?;
        let mut handlers = Vec::new();
        let mut finally = None;
        while self.is_kw("except") {
            self.advance();
            // допускаем `except Имя as e` в духе Python — имя типа пропускаем
            if matches!(self.peek(), Tok::Name(_)) {
                self.advance();
            }
            let name = if self.eat_kw("as") { Some(self.expect_name()?) } else { None };
            let hbody = self.parse_block()?;
            handlers.push(Handler { name, body: hbody });
        }
        if self.is_kw("finally") {
            self.advance();
            finally = Some(self.parse_block()?);
        }
        if handlers.is_empty() && finally.is_none() {
            return Err(self.err("после try ожидался except или finally"));
        }
        Ok(Stmt { line, kind: StmtKind::Try { body, handlers, finally } })
    }

    // ---------- выражения ----------

    /// Список выражений через запятую: один элемент — само выражение, несколько — Tuple.
    fn parse_expr_list(&mut self) -> PResult<Expr> {
        let (line, col) = (self.line(), self.col());
        let first = self.parse_expr()?;
        if !self.is_op(",") {
            return Ok(first);
        }
        let mut items = vec![first];
        while self.eat_op(",") {
            if !self.starts_expr() {
                break;
            }
            items.push(self.parse_expr()?);
        }
        Ok(self.mk(line, col, ExprKind::Tuple(items)))
    }

    /// Может ли текущая лексема начинать выражение.
    fn starts_expr(&self) -> bool {
        match self.peek() {
            Tok::Int(_) | Tok::Float(_) | Tok::Str(_) | Tok::FStr(_) | Tok::Name(_) => true,
            Tok::Kw(k) => matches!(*k, "not" | "lambda" | "true" | "false" | "null"),
            Tok::Op(o) => matches!(*o, "(" | "[" | "{" | "-" | "+"),
            _ => false,
        }
    }

    /// Цель `for`/генератора: без сравнений, чтобы `in` не был съеден.
    fn parse_target_list(&mut self) -> PResult<Expr> {
        let (line, col) = (self.line(), self.col());
        let first = self.parse_primary()?;
        self.check_target(&first)?;
        if !self.is_op(",") {
            return Ok(first);
        }
        let mut items = vec![first];
        while self.eat_op(",") {
            if self.is_kw("in") {
                break;
            }
            let t = self.parse_primary()?;
            self.check_target(&t)?;
            items.push(t);
        }
        Ok(self.mk(line, col, ExprKind::Tuple(items)))
    }

    /// Условное выражение или lambda.
    fn parse_expr(&mut self) -> PResult<Expr> {
        if self.is_kw("lambda") {
            return self.parse_lambda();
        }
        let (line, col) = (self.line(), self.col());
        let e = self.parse_or()?;
        if self.is_kw("if") {
            self.advance();
            let cond = self.parse_or()?;
            self.expect_kw("else")?;
            let orelse = self.parse_expr()?;
            return Ok(self.mk(
                line,
                col,
                ExprKind::IfExpr { cond: Box::new(cond), then: Box::new(e), orelse: Box::new(orelse) },
            ));
        }
        Ok(e)
    }

    fn parse_lambda(&mut self) -> PResult<Expr> {
        let (line, col) = (self.line(), self.col());
        self.advance();
        let params = self.parse_params(":")?;
        let body_expr = self.parse_expr()?;
        let body = vec![Stmt { line, kind: StmtKind::Return(Some(body_expr)) }];
        let def = FuncDef { name: Rc::from("lambda"), params, body, line };
        Ok(self.mk(line, col, ExprKind::Lambda(Rc::new(def))))
    }

    fn parse_or(&mut self) -> PResult<Expr> {
        let (line, col) = (self.line(), self.col());
        let mut left = self.parse_and()?;
        while self.eat_kw("or") {
            let right = self.parse_and()?;
            left = self.mk(line, col, ExprKind::BoolOp(BoolOp::Or, Box::new(left), Box::new(right)));
        }
        Ok(left)
    }

    fn parse_and(&mut self) -> PResult<Expr> {
        let (line, col) = (self.line(), self.col());
        let mut left = self.parse_not()?;
        while self.eat_kw("and") {
            let right = self.parse_not()?;
            left = self.mk(line, col, ExprKind::BoolOp(BoolOp::And, Box::new(left), Box::new(right)));
        }
        Ok(left)
    }

    fn parse_not(&mut self) -> PResult<Expr> {
        let (line, col) = (self.line(), self.col());
        if self.eat_kw("not") {
            let e = self.parse_not()?;
            return Ok(self.mk(line, col, ExprKind::Unary(UnOp::Not, Box::new(e))));
        }
        self.parse_comparison()
    }

    fn parse_comparison(&mut self) -> PResult<Expr> {
        let (line, col) = (self.line(), self.col());
        let first = self.parse_arith()?;
        let mut rest = Vec::new();
        loop {
            let op = match self.peek() {
                Tok::Op("==") => CmpOp::Eq,
                Tok::Op("!=") => CmpOp::Ne,
                Tok::Op("<") => CmpOp::Lt,
                Tok::Op("<=") => CmpOp::Le,
                Tok::Op(">") => CmpOp::Gt,
                Tok::Op(">=") => CmpOp::Ge,
                Tok::Kw("in") => CmpOp::In,
                Tok::Kw("not") if matches!(self.peek_at(1), Tok::Kw("in")) => {
                    self.advance();
                    CmpOp::NotIn
                }
                _ => break,
            };
            self.advance();
            let right = self.parse_arith()?;
            rest.push((op, right));
        }
        if rest.is_empty() {
            Ok(first)
        } else {
            Ok(self.mk(line, col, ExprKind::Compare { first: Box::new(first), rest }))
        }
    }

    fn parse_arith(&mut self) -> PResult<Expr> {
        let (line, col) = (self.line(), self.col());
        let mut left = self.parse_term()?;
        loop {
            let op = match self.peek() {
                Tok::Op("+") => BinOp::Add,
                Tok::Op("-") => BinOp::Sub,
                _ => break,
            };
            self.advance();
            let right = self.parse_term()?;
            left = self.mk(line, col, ExprKind::Binary(op, Box::new(left), Box::new(right)));
        }
        Ok(left)
    }

    fn parse_term(&mut self) -> PResult<Expr> {
        let (line, col) = (self.line(), self.col());
        let mut left = self.parse_factor()?;
        loop {
            let op = match self.peek() {
                Tok::Op("*") => BinOp::Mul,
                Tok::Op("/") => BinOp::Div,
                Tok::Op("//") => BinOp::FloorDiv,
                Tok::Op("%") => BinOp::Mod,
                _ => break,
            };
            self.advance();
            let right = self.parse_factor()?;
            left = self.mk(line, col, ExprKind::Binary(op, Box::new(left), Box::new(right)));
        }
        Ok(left)
    }

    fn parse_factor(&mut self) -> PResult<Expr> {
        let (line, col) = (self.line(), self.col());
        if self.eat_op("-") {
            let e = self.parse_factor()?;
            // `-5` — сразу число, чтобы литералы были простыми
            if let ExprKind::Int(n) = e.kind {
                return Ok(self.mk(line, col, ExprKind::Int(n.wrapping_neg())));
            }
            if let ExprKind::Float(f) = e.kind {
                return Ok(self.mk(line, col, ExprKind::Float(-f)));
            }
            return Ok(self.mk(line, col, ExprKind::Unary(UnOp::Neg, Box::new(e))));
        }
        if self.eat_op("+") {
            let e = self.parse_factor()?;
            return Ok(self.mk(line, col, ExprKind::Unary(UnOp::Pos, Box::new(e))));
        }
        self.parse_power()
    }

    fn parse_power(&mut self) -> PResult<Expr> {
        let (line, col) = (self.line(), self.col());
        let base = self.parse_primary()?;
        if self.eat_op("**") {
            let exp = self.parse_factor()?;
            return Ok(self.mk(line, col, ExprKind::Binary(BinOp::Pow, Box::new(base), Box::new(exp))));
        }
        Ok(base)
    }

    /// Атом с хвостами: вызовы, индексы/срезы, атрибуты.
    fn parse_primary(&mut self) -> PResult<Expr> {
        let mut e = self.parse_atom()?;
        loop {
            let (line, col) = (self.line(), self.col());
            if self.eat_op("(") {
                let (args, kwargs) = self.parse_call_args()?;
                e = self.mk(line, col, ExprKind::Call { func: Box::new(e), args, kwargs });
            } else if self.eat_op("[") {
                e = self.parse_subscript(e, line, col)?;
            } else if self.eat_op(".") {
                let name = self.expect_name()?;
                e = self.mk(line, col, ExprKind::Attr(Box::new(e), name));
            } else {
                break;
            }
        }
        Ok(e)
    }

    fn parse_call_args(&mut self) -> PResult<(Vec<Expr>, Vec<(Rc<str>, Expr)>)> {
        let mut args = Vec::new();
        let mut kwargs: Vec<(Rc<str>, Expr)> = Vec::new();
        loop {
            if self.eat_op(")") {
                break;
            }
            if let (Tok::Name(n), Tok::Op("=")) = (self.peek(), self.peek_at(1)) {
                let name: Rc<str> = Rc::from(n.as_str());
                self.advance();
                self.advance();
                if kwargs.iter().any(|(k, _)| *k == name) {
                    return Err(self.err(format!("именованный аргумент '{}' передан дважды", name)));
                }
                kwargs.push((name, self.parse_expr()?));
            } else {
                if !kwargs.is_empty() {
                    return Err(self.err("позиционный аргумент после именованного"));
                }
                let (line, col) = (self.line(), self.col());
                let mut e = self.parse_expr()?;
                if self.is_kw("for") {
                    let clauses = self.parse_comp_clauses()?;
                    e = self.mk(line, col, ExprKind::ListComp { elt: Box::new(e), clauses });
                }
                args.push(e);
            }
            if !self.eat_op(",") {
                self.expect_op(")")?;
                break;
            }
        }
        Ok((args, kwargs))
    }

    /// После `[`: индекс или срез `lo:hi:step`.
    fn parse_subscript(&mut self, obj: Expr, line: u32, col: u32) -> PResult<Expr> {
        let mut lo = None;
        if !self.is_op(":") {
            let idx = self.parse_expr()?;
            if self.eat_op("]") {
                return Ok(self.mk(line, col, ExprKind::Index(Box::new(obj), Box::new(idx))));
            }
            lo = Some(Box::new(idx));
        }
        self.expect_op(":")?;
        let mut hi = None;
        let mut step = None;
        if !self.is_op(":") && !self.is_op("]") {
            hi = Some(Box::new(self.parse_expr()?));
        }
        if self.eat_op(":") && !self.is_op("]") {
            step = Some(Box::new(self.parse_expr()?));
        }
        self.expect_op("]")?;
        Ok(self.mk(line, col, ExprKind::Slice { obj: Box::new(obj), lo, hi, step }))
    }

    fn parse_comp_clauses(&mut self) -> PResult<Vec<CompClause>> {
        let mut clauses = Vec::new();
        while self.is_kw("for") {
            self.advance();
            let target = self.parse_target_list()?;
            self.expect_kw("in")?;
            let iter = self.parse_or()?;
            let mut conds = Vec::new();
            while self.is_kw("if") {
                self.advance();
                conds.push(self.parse_or()?);
            }
            clauses.push(CompClause { target, iter, conds });
        }
        Ok(clauses)
    }

    fn parse_atom(&mut self) -> PResult<Expr> {
        let (line, col) = (self.line(), self.col());
        let tok = self.advance();
        let kind = match tok.tok {
            Tok::Int(n) => ExprKind::Int(n),
            Tok::Float(f) => ExprKind::Float(f),
            Tok::Str(s) => ExprKind::Str(Rc::from(s.as_str())),
            Tok::FStr(parts) => self.fstring(parts)?,
            Tok::Name(n) => ExprKind::Name(Rc::from(n.as_str())),
            Tok::Kw("true") => ExprKind::Bool(true),
            Tok::Kw("false") => ExprKind::Bool(false),
            Tok::Kw("null") => ExprKind::Null,
            Tok::Kw("lambda") => {
                self.pos -= 1;
                return self.parse_lambda();
            }
            Tok::Op("(") => return self.parse_paren(line, col),
            Tok::Op("[") => return self.parse_list(line, col),
            Tok::Op("{") => return self.parse_dict(line, col),
            other => {
                return Err(LangError::syntax(line, col, format!("неожиданный токен: {}", describe(&other))));
            }
        };
        Ok(self.mk(line, col, kind))
    }

    fn fstring(&mut self, parts: Vec<FPart>) -> PResult<ExprKind> {
        let mut segs = Vec::new();
        for p in parts {
            match p {
                FPart::Lit(s) => segs.push(FSeg::Lit(Rc::from(s.as_str()))),
                FPart::Expr { src, conv, spec, line, col } => {
                    let toks = lexer::tokenize_expr(&src, line, col)?;
                    let expr = parse_expression(toks)?;
                    segs.push(FSeg::Expr { expr: Box::new(expr), conv, spec: spec.map(|s| Rc::from(s.as_str())) });
                }
            }
        }
        Ok(ExprKind::FString(segs))
    }

    /// После `(`: группировка, кортеж или генератор.
    fn parse_paren(&mut self, line: u32, col: u32) -> PResult<Expr> {
        if self.eat_op(")") {
            return Ok(self.mk(line, col, ExprKind::Tuple(Vec::new())));
        }
        let first = self.parse_expr()?;
        if self.is_kw("for") {
            let clauses = self.parse_comp_clauses()?;
            self.expect_op(")")?;
            return Ok(self.mk(line, col, ExprKind::ListComp { elt: Box::new(first), clauses }));
        }
        if self.eat_op(")") {
            return Ok(first);
        }
        let mut items = vec![first];
        while self.eat_op(",") {
            if self.is_op(")") {
                break;
            }
            items.push(self.parse_expr()?);
        }
        self.expect_op(")")?;
        Ok(self.mk(line, col, ExprKind::Tuple(items)))
    }

    fn parse_list(&mut self, line: u32, col: u32) -> PResult<Expr> {
        if self.eat_op("]") {
            return Ok(self.mk(line, col, ExprKind::List(Vec::new())));
        }
        let first = self.parse_expr()?;
        if self.is_kw("for") {
            let clauses = self.parse_comp_clauses()?;
            self.expect_op("]")?;
            return Ok(self.mk(line, col, ExprKind::ListComp { elt: Box::new(first), clauses }));
        }
        let mut items = vec![first];
        while self.eat_op(",") {
            if self.is_op("]") {
                break;
            }
            items.push(self.parse_expr()?);
        }
        self.expect_op("]")?;
        Ok(self.mk(line, col, ExprKind::List(items)))
    }

    fn parse_dict(&mut self, line: u32, col: u32) -> PResult<Expr> {
        if self.eat_op("}") {
            return Ok(self.mk(line, col, ExprKind::Dict(Vec::new())));
        }
        let key = self.parse_expr()?;
        if !self.eat_op(":") {
            return Err(self.err("ожидалось ':' между ключом и значением словаря"));
        }
        let value = self.parse_expr()?;
        if self.is_kw("for") {
            let clauses = self.parse_comp_clauses()?;
            self.expect_op("}")?;
            return Ok(self.mk(
                line,
                col,
                ExprKind::DictComp { key: Box::new(key), value: Box::new(value), clauses },
            ));
        }
        let mut pairs = vec![(key, value)];
        while self.eat_op(",") {
            if self.is_op("}") {
                break;
            }
            let k = self.parse_expr()?;
            self.expect_op(":")?;
            let v = self.parse_expr()?;
            pairs.push((k, v));
        }
        self.expect_op("}")?;
        Ok(self.mk(line, col, ExprKind::Dict(pairs)))
    }
}
