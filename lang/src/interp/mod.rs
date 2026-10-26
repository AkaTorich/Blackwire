//! Исполнитель Adder: обход дерева. Области видимости — цепочка `Env`
//! (`Rc<RefCell<Env>>`), функции хранят своё замыкание. Здесь — операторы
//! (`if/while/for/try/def`, присваивания), счётчик шагов (каждые 500 шагов —
//! `Host::should_stop`), лимит рекурсии и защита стека. Выражения — в `expr.rs`,
//! операции и индексация — в `ops.rs`, вызовы функций и методов — в `call.rs`.

pub mod call;
pub mod expr;
pub mod ops;
pub mod stack;

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use crate::ast::*;
use crate::builtins;
use crate::error::{rt, Exc, LangError};
use crate::value::{Function, Value};
use crate::Host;

pub type R<T> = Result<T, Exc>;

/// Как раз в 500 шагов спрашиваем хост, не пора ли остановиться.
const STOP_CHECK_EVERY: u64 = 500;
/// Предел глубины вызовов пользовательских функций.
const MAX_DEPTH: u32 = 1000;
/// Сколько байт стека машины разрешаем занять исполнителю, когда границу стека потока
/// узнать нельзя (поток по умолчанию — 2 МБ).
const DEFAULT_STACK_BUDGET: usize = 1_400_000;

/// Область видимости.
pub struct Env {
    pub vars: HashMap<Rc<str>, Value>,
    pub parent: Option<Rc<RefCell<Env>>>,
    /// Имена, объявленные `global` в этой области.
    pub global_names: HashSet<Rc<str>>,
}

impl Env {
    pub fn new_global() -> Rc<RefCell<Env>> {
        Rc::new(RefCell::new(Env { vars: HashMap::new(), parent: None, global_names: HashSet::new() }))
    }

    pub fn child(parent: &Rc<RefCell<Env>>) -> Rc<RefCell<Env>> {
        Rc::new(RefCell::new(Env {
            vars: HashMap::new(),
            parent: Some(parent.clone()),
            global_names: HashSet::new(),
        }))
    }

    /// Поиск по цепочке областей.
    pub fn get(env: &Rc<RefCell<Env>>, name: &str) -> Option<Value> {
        let mut cur = env.clone();
        loop {
            let next = {
                let e = cur.borrow();
                if let Some(v) = e.vars.get(name) {
                    return Some(v.clone());
                }
                e.parent.clone()
            };
            match next {
                Some(p) => cur = p,
                None => return None,
            }
        }
    }
}

/// Результат выполнения оператора.
pub enum Flow {
    Normal,
    Break,
    Continue,
    Return(Value),
}

pub struct Interpreter {
    pub globals: Rc<RefCell<Env>>,
    pub args: Vec<String>,
    steps: u64,
    step_limit: u64,
    depth: u32,
    stack_base: usize,
    stack_bottom: Option<usize>,
    stack_budget: usize,
    /// Текст текущего исключения внутри `except` — для `raise` без аргумента.
    current_exc: Option<String>,
    /// Состояние генератора случайных чисел (`random`).
    pub rng_state: u64,
}

impl Interpreter {
    /// `args` — то, что увидит скрипт в `sys.args`.
    pub fn new(args: Vec<String>) -> Self {
        let globals = Env::new_global();
        builtins::install(&mut globals.borrow_mut());
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0x9E37_79B9_7F4A_7C15);
        Interpreter {
            globals,
            args,
            steps: 0,
            step_limit: u64::MAX,
            depth: 0,
            stack_base: 0,
            stack_bottom: None,
            stack_budget: DEFAULT_STACK_BUDGET,
            current_exc: None,
            rng_state: seed | 1,
        }
    }

    /// Сколько байт стека машины можно занять, когда границу стека потока узнать нельзя
    /// (не на Windows). В потоке с большим стеком поднимает достижимую глубину рекурсии.
    pub fn set_step_limit(&mut self, limit: u64) { self.steps = 0; self.step_limit = limit; }

    pub fn set_stack_budget(&mut self, bytes: usize) {
        self.stack_budget = bytes;
    }

    /// Выполнить программу. `exit(code)` → `Ok(Int(code))`, иначе `Ok(Null)`.
    pub fn run(&mut self, program: &Program, host: &mut dyn Host) -> Result<Value, LangError> {
        self.stack_base = stack::here();
        self.stack_bottom = stack::bottom(self.stack_base);
        let env = self.globals.clone();
        match self.exec_block(&program.body, &env, host) {
            Ok(_) => Ok(Value::Null),
            Err(Exc::Exit(code)) => Ok(Value::Int(code)),
            Err(Exc::Error(e)) => Err(e),
            Err(Exc::Interrupted(line)) => Err(LangError::runtime(line, "прервано")),
        }
    }

    /// Один шаг исполнения; раз в `STOP_CHECK_EVERY` шагов — вопрос хосту.
    pub fn tick(&mut self, line: u32, host: &mut dyn Host) -> R<()> {
        self.steps += 1;
        if self.steps > self.step_limit { return Err(Exc::Interrupted(line)); }
        if self.steps % STOP_CHECK_EVERY == 0 && host.should_stop() {
            return Err(Exc::Interrupted(line));
        }
        Ok(())
    }

    /// Проверка глубины рекурсии и остатка стека перед вызовом функции.
    pub(crate) fn enter_call(&mut self, line: u32) -> R<()> {
        if self.depth >= MAX_DEPTH {
            return Err(rt(line, "переполнение стека"));
        }
        let here = stack::here();
        let exhausted = match self.stack_bottom {
            Some(bottom) => here.saturating_sub(bottom) < stack::STACK_MARGIN,
            None => self.stack_base != 0 && self.stack_base.saturating_sub(here) > self.stack_budget,
        };
        if exhausted {
            return Err(rt(line, "переполнение стека"));
        }
        self.depth += 1;
        Ok(())
    }

    pub(crate) fn leave_call(&mut self) {
        self.depth = self.depth.saturating_sub(1);
    }

    pub fn lookup(&self, name: &str, env: &Rc<RefCell<Env>>) -> Option<Value> {
        if env.borrow().global_names.contains(name) {
            return self.globals.borrow().vars.get(name).cloned();
        }
        Env::get(env, name)
    }

    pub fn set_var(&self, env: &Rc<RefCell<Env>>, name: &Rc<str>, value: Value) {
        let is_global = env.borrow().global_names.contains(name);
        if is_global {
            self.globals.borrow_mut().vars.insert(name.clone(), value);
        } else {
            env.borrow_mut().vars.insert(name.clone(), value);
        }
    }

    pub fn exec_block(&mut self, stmts: &[Stmt], env: &Rc<RefCell<Env>>, host: &mut dyn Host) -> R<Flow> {
        for s in stmts {
            match self.exec_stmt(s, env, host)? {
                Flow::Normal => {}
                other => return Ok(other),
            }
        }
        Ok(Flow::Normal)
    }

    fn exec_stmt(&mut self, s: &Stmt, env: &Rc<RefCell<Env>>, host: &mut dyn Host) -> R<Flow> {
        self.tick(s.line, host)?;
        let line = s.line;
        match &s.kind {
            StmtKind::Expr(e) => {
                self.eval(e, env, host)?;
            }
            StmtKind::Assign { targets, value } => {
                let v = self.eval(value, env, host)?;
                for t in targets {
                    self.assign(t, v.clone(), env, host)?;
                }
            }
            StmtKind::AugAssign { target, op, value } => {
                if let ExprKind::Index(obj, idx) = &target.kind {
                    // объект и индекс вычисляем один раз: `xs[f()] += 1` зовёт f() однажды
                    let o = self.eval(obj, env, host)?;
                    let i = self.eval(idx, env, host)?;
                    let cur = ops::get_index(&o, &i, line)?;
                    let rhs = self.eval(value, env, host)?;
                    let res = ops::binary(*op, &cur, &rhs, line)?;
                    ops::set_index(&o, &i, res, line)?;
                } else {
                    let cur = self.eval(target, env, host)?;
                    let rhs = self.eval(value, env, host)?;
                    let res = ops::binary(*op, &cur, &rhs, line)?;
                    self.assign(target, res, env, host)?;
                }
            }
            StmtKind::If { branches, orelse } => {
                for (cond, body) in branches {
                    if self.eval(cond, env, host)?.truthy() {
                        return self.exec_block(body, env, host);
                    }
                }
                if let Some(body) = orelse {
                    return self.exec_block(body, env, host);
                }
            }
            StmtKind::While { cond, body } => return self.exec_while(cond, body, env, host, line),
            StmtKind::For { target, iter, body } => return self.exec_for(target, iter, body, env, host, line),
            StmtKind::Def(def) => {
                let f = self.make_function(def, env, host)?;
                self.set_var(env, &def.name, Value::Func(f));
            }
            StmtKind::Return(e) => {
                let v = match e {
                    Some(e) => self.eval(e, env, host)?,
                    None => Value::Null,
                };
                return Ok(Flow::Return(v));
            }
            StmtKind::Break => return Ok(Flow::Break),
            StmtKind::Continue => return Ok(Flow::Continue),
            StmtKind::Pass => {}
            StmtKind::Raise(e) => return self.exec_raise(e, env, host, line),
            StmtKind::Global(names) => {
                let mut e = env.borrow_mut();
                for n in names {
                    e.global_names.insert(n.clone());
                }
            }
            StmtKind::Try { body, handlers, finally } => return self.exec_try(body, handlers, finally, env, host),
        }
        Ok(Flow::Normal)
    }

    fn exec_while(
        &mut self,
        cond: &Expr,
        body: &[Stmt],
        env: &Rc<RefCell<Env>>,
        host: &mut dyn Host,
        line: u32,
    ) -> R<Flow> {
        while self.eval(cond, env, host)?.truthy() {
            self.tick(line, host)?;
            match self.exec_block(body, env, host)? {
                Flow::Break => break,
                Flow::Return(v) => return Ok(Flow::Return(v)),
                Flow::Normal | Flow::Continue => {}
            }
        }
        Ok(Flow::Normal)
    }

    fn exec_raise(&mut self, e: &Option<Expr>, env: &Rc<RefCell<Env>>, host: &mut dyn Host, line: u32) -> R<Flow> {
        let msg = match e {
            Some(e) => self.eval(e, env, host)?.to_display_string(),
            None => match &self.current_exc {
                Some(m) => m.clone(),
                None => return Err(rt(line, "raise без аргумента вне except")),
            },
        };
        Err(rt(line, msg))
    }

    /// Создать значение функции: значения по умолчанию вычисляются один раз, здесь.
    pub(crate) fn make_function(
        &mut self,
        def: &Rc<FuncDef>,
        env: &Rc<RefCell<Env>>,
        host: &mut dyn Host,
    ) -> R<Rc<Function>> {
        let mut defaults = Vec::with_capacity(def.params.len());
        for p in &def.params {
            defaults.push(match &p.default {
                Some(e) => Some(self.eval(e, env, host)?),
                None => None,
            });
        }
        Ok(Rc::new(Function { def: def.clone(), defaults, closure: env.clone() }))
    }

    fn exec_try(
        &mut self,
        body: &[Stmt],
        handlers: &[Handler],
        finally: &Option<Vec<Stmt>>,
        env: &Rc<RefCell<Env>>,
        host: &mut dyn Host,
    ) -> R<Flow> {
        let outcome = match self.exec_block(body, env, host) {
            Err(Exc::Error(e)) if !handlers.is_empty() => {
                let h = &handlers[0];
                let prev = self.current_exc.replace(e.message.clone());
                if let Some(name) = &h.name {
                    self.set_var(env, name, Value::str(&e.message));
                }
                let r = self.exec_block(&h.body, env, host);
                self.current_exc = prev;
                r
            }
            other => other,
        };
        if let Some(fin) = finally {
            match self.exec_block(fin, env, host)? {
                Flow::Normal => {}
                flow => return Ok(flow),
            }
        }
        outcome
    }

    fn exec_for(
        &mut self,
        target: &Expr,
        iter: &Expr,
        body: &[Stmt],
        env: &Rc<RefCell<Env>>,
        host: &mut dyn Host,
        line: u32,
    ) -> R<Flow> {
        // `for i in range(...)` без создания списка, если range не переопределён
        if let ExprKind::Call { func, args, kwargs } = &iter.kind {
            if let ExprKind::Name(n) = &func.kind {
                let is_range = kwargs.is_empty()
                    && &**n == "range"
                    && matches!(self.lookup(n, env), Some(Value::Builtin(b)) if b.name == "range");
                if is_range {
                    let mut vals = Vec::with_capacity(args.len());
                    for a in args {
                        vals.push(self.eval(a, env, host)?);
                    }
                    let (start, stop, step) = builtins::core::range_bounds(&vals, line)?;
                    let mut i = start;
                    while (step > 0 && i < stop) || (step < 0 && i > stop) {
                        self.tick(line, host)?;
                        self.assign(target, Value::Int(i), env, host)?;
                        match self.exec_block(body, env, host)? {
                            Flow::Break => break,
                            Flow::Return(v) => return Ok(Flow::Return(v)),
                            Flow::Normal | Flow::Continue => {}
                        }
                        i += step;
                    }
                    return Ok(Flow::Normal);
                }
            }
        }
        let it = self.eval(iter, env, host)?;
        if let Value::List(l) = &it {
            // живой список: смотрим длину на каждом шаге
            let mut i = 0;
            loop {
                let item = {
                    let b = l.borrow();
                    if i >= b.len() {
                        break;
                    }
                    b[i].clone()
                };
                i += 1;
                self.tick(line, host)?;
                self.assign(target, item, env, host)?;
                match self.exec_block(body, env, host)? {
                    Flow::Break => break,
                    Flow::Return(v) => return Ok(Flow::Return(v)),
                    Flow::Normal | Flow::Continue => {}
                }
            }
            return Ok(Flow::Normal);
        }
        let items = ops::iter_to_vec(&it, line)?;
        for item in items {
            self.tick(line, host)?;
            self.assign(target, item, env, host)?;
            match self.exec_block(body, env, host)? {
                Flow::Break => break,
                Flow::Return(v) => return Ok(Flow::Return(v)),
                Flow::Normal | Flow::Continue => {}
            }
        }
        Ok(Flow::Normal)
    }

    /// Присваивание цели: имя, индекс, распаковка списка.
    pub fn assign(&mut self, target: &Expr, value: Value, env: &Rc<RefCell<Env>>, host: &mut dyn Host) -> R<()> {
        match &target.kind {
            ExprKind::Name(n) => {
                self.set_var(env, n, value);
                Ok(())
            }
            ExprKind::Index(obj, idx) => {
                let o = self.eval(obj, env, host)?;
                let i = self.eval(idx, env, host)?;
                ops::set_index(&o, &i, value, target.line)
            }
            ExprKind::Tuple(items) | ExprKind::List(items) => {
                let vals = ops::iter_to_vec(&value, target.line)?;
                if vals.len() != items.len() {
                    return Err(rt(
                        target.line,
                        format!("ожидалось {} значений для распаковки, получено {}", items.len(), vals.len()),
                    ));
                }
                for (t, v) in items.iter().zip(vals) {
                    self.assign(t, v, env, host)?;
                }
                Ok(())
            }
            ExprKind::Attr(..) => Err(rt(target.line, "нельзя присваивать атрибуты")),
            _ => Err(rt(target.line, "недопустимая цель присваивания")),
        }
    }
}
