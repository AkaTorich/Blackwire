//! Дерево разбора Adder. Каждый оператор и выражение помнит строку исходника,
//! чтобы ошибки исполнения несли её номер.

use std::rc::Rc;

/// Разобранный скрипт целиком.
#[derive(Debug, Clone)]
pub struct Program {
    pub body: Vec<Stmt>,
}

/// Оператор.
#[derive(Debug, Clone)]
pub struct Stmt {
    pub line: u32,
    pub kind: StmtKind,
}

#[derive(Debug, Clone)]
pub enum StmtKind {
    /// Выражение ради побочного эффекта (вызов и т. п.).
    Expr(Expr),
    /// `a = b = value`, `a, b = value`, `xs[0] = value`.
    Assign { targets: Vec<Expr>, value: Expr },
    /// `x += 1` и прочие.
    AugAssign { target: Expr, op: BinOp, value: Expr },
    /// Ветви `if/elif` с условиями и необязательный `else`.
    If { branches: Vec<(Expr, Vec<Stmt>)>, orelse: Option<Vec<Stmt>> },
    While { cond: Expr, body: Vec<Stmt> },
    For { target: Expr, iter: Expr, body: Vec<Stmt> },
    Def(Rc<FuncDef>),
    Return(Option<Expr>),
    Break,
    Continue,
    Pass,
    /// `raise "текст"`; без аргумента — повторно поднять текущее исключение.
    Raise(Option<Expr>),
    Global(Vec<Rc<str>>),
    Try { body: Vec<Stmt>, handlers: Vec<Handler>, finally: Option<Vec<Stmt>> },
}

/// Ветка `except [as имя]:`.
#[derive(Debug, Clone)]
pub struct Handler {
    pub name: Option<Rc<str>>,
    pub body: Vec<Stmt>,
}

/// Определение функции (`def` или `lambda` — у лямбды тело из одного `return`).
#[derive(Debug, Clone)]
pub struct FuncDef {
    pub name: Rc<str>,
    pub params: Vec<Param>,
    pub body: Vec<Stmt>,
    pub line: u32,
}

#[derive(Debug, Clone)]
pub struct Param {
    pub name: Rc<str>,
    pub default: Option<Expr>,
}

/// Выражение.
#[derive(Debug, Clone)]
pub struct Expr {
    pub line: u32,
    pub col: u32,
    pub kind: ExprKind,
}

#[derive(Debug, Clone)]
pub enum ExprKind {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(Rc<str>),
    FString(Vec<FSeg>),
    Name(Rc<str>),
    List(Vec<Expr>),
    /// `a, b` без скобок или `(a, b)` — во время исполнения становится списком.
    Tuple(Vec<Expr>),
    Dict(Vec<(Expr, Expr)>),
    ListComp { elt: Box<Expr>, clauses: Vec<CompClause> },
    DictComp { key: Box<Expr>, value: Box<Expr>, clauses: Vec<CompClause> },
    Unary(UnOp, Box<Expr>),
    Binary(BinOp, Box<Expr>, Box<Expr>),
    /// `and` / `or` — ленивые, возвращают операнд.
    BoolOp(BoolOp, Box<Expr>, Box<Expr>),
    /// Цепочка сравнений `a < b <= c`.
    Compare { first: Box<Expr>, rest: Vec<(CmpOp, Expr)> },
    IfExpr { cond: Box<Expr>, then: Box<Expr>, orelse: Box<Expr> },
    Lambda(Rc<FuncDef>),
    Call { func: Box<Expr>, args: Vec<Expr>, kwargs: Vec<(Rc<str>, Expr)> },
    Attr(Box<Expr>, Rc<str>),
    Index(Box<Expr>, Box<Expr>),
    Slice { obj: Box<Expr>, lo: Option<Box<Expr>>, hi: Option<Box<Expr>>, step: Option<Box<Expr>> },
}

/// Кусок f-строки: текст или `{выражение[!r][:формат]}`.
#[derive(Debug, Clone)]
pub enum FSeg {
    Lit(Rc<str>),
    Expr { expr: Box<Expr>, conv: Option<char>, spec: Option<Rc<str>> },
}

/// `for цель in источник if условие ...` внутри генератора.
#[derive(Debug, Clone)]
pub struct CompClause {
    pub target: Expr,
    pub iter: Expr,
    pub conds: Vec<Expr>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnOp {
    Neg,
    Pos,
    Not,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    FloorDiv,
    Mod,
    Pow,
}

impl BinOp {
    pub fn symbol(self) -> &'static str {
        match self {
            BinOp::Add => "+",
            BinOp::Sub => "-",
            BinOp::Mul => "*",
            BinOp::Div => "/",
            BinOp::FloorDiv => "//",
            BinOp::Mod => "%",
            BinOp::Pow => "**",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoolOp {
    And,
    Or,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CmpOp {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    In,
    NotIn,
}
