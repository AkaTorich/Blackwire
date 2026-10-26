//! Вычисление выражений: литералы, имена, f-строки, генераторы списков и словарей,
//! логика, цепочки сравнений, доступ к атрибутам (модули и методы).
//! Тяжёлые ветви вынесены в отдельные функции, чтобы кадр `eval` оставался маленьким —
//! от этого зависит достижимая глубина рекурсии скрипта.

use std::cell::RefCell;
use std::rc::Rc;

use super::{ops, Env, Interpreter, R};
use crate::ast::*;
use crate::builtins::{self, format};
use crate::error::rt;
use crate::value::{BoundMethod, Dict, Key, Value};
use crate::Host;

/// Куда складывает элементы генератор.
enum Acc {
    List(Vec<Value>),
    Dict(Dict),
}

/// Что вычислять на каждом витке генератора.
enum Elt<'a> {
    List(&'a Expr),
    Dict(&'a Expr, &'a Expr),
}

impl Interpreter {
    pub fn eval(&mut self, e: &Expr, env: &Rc<RefCell<Env>>, host: &mut dyn Host) -> R<Value> {
        match &e.kind {
            ExprKind::Null => Ok(Value::Null),
            ExprKind::Bool(b) => Ok(Value::Bool(*b)),
            ExprKind::Int(n) => Ok(Value::Int(*n)),
            ExprKind::Float(f) => Ok(Value::Float(*f)),
            ExprKind::Str(s) => Ok(Value::Str(s.clone())),
            ExprKind::Name(n) => match self.lookup(n, env) {
                Some(v) => Ok(v),
                None => Err(rt(e.line, format!("имя '{}' не определено", n))),
            },
            ExprKind::Call { func, args, kwargs } => self.eval_call(func, args, kwargs, env, host, e.line),
            ExprKind::Binary(op, a, b) => {
                let l = self.eval(a, env, host)?;
                let r = self.eval(b, env, host)?;
                ops::binary(*op, &l, &r, e.line)
            }
            ExprKind::Unary(op, inner) => {
                let v = self.eval(inner, env, host)?;
                ops::unary(*op, &v, e.line)
            }
            ExprKind::BoolOp(op, a, b) => self.eval_boolop(*op, a, b, env, host),
            ExprKind::IfExpr { cond, then, orelse } => {
                if self.eval(cond, env, host)?.truthy() {
                    self.eval(then, env, host)
                } else {
                    self.eval(orelse, env, host)
                }
            }
            ExprKind::Compare { first, rest } => self.eval_compare(first, rest, env, host, e.line),
            ExprKind::FString(segs) => self.eval_fstring(segs, env, host),
            ExprKind::List(items) | ExprKind::Tuple(items) => self.eval_list(items, env, host),
            ExprKind::Dict(pairs) => self.eval_dict(pairs, env, host),
            ExprKind::ListComp { elt, clauses } => self.eval_comp_expr(&Elt::List(elt), clauses, env, host),
            ExprKind::DictComp { key, value, clauses } => {
                self.eval_comp_expr(&Elt::Dict(key, value), clauses, env, host)
            }
            ExprKind::Lambda(def) => Ok(Value::Func(self.make_function(def, env, host)?)),
            ExprKind::Attr(obj, name) => {
                let o = self.eval(obj, env, host)?;
                self.get_attr(o, name, e.line)
            }
            ExprKind::Index(obj, idx) => {
                let o = self.eval(obj, env, host)?;
                let i = self.eval(idx, env, host)?;
                ops::get_index(&o, &i, e.line)
            }
            ExprKind::Slice { obj, lo, hi, step } => self.eval_slice(obj, lo, hi, step, env, host, e.line),
        }
    }

    fn eval_boolop(
        &mut self,
        op: BoolOp,
        a: &Expr,
        b: &Expr,
        env: &Rc<RefCell<Env>>,
        host: &mut dyn Host,
    ) -> R<Value> {
        let l = self.eval(a, env, host)?;
        let take_right = match op {
            BoolOp::And => l.truthy(),
            BoolOp::Or => !l.truthy(),
        };
        if take_right {
            self.eval(b, env, host)
        } else {
            Ok(l)
        }
    }

    fn eval_compare(
        &mut self,
        first: &Expr,
        rest: &[(CmpOp, Expr)],
        env: &Rc<RefCell<Env>>,
        host: &mut dyn Host,
        line: u32,
    ) -> R<Value> {
        let mut left = self.eval(first, env, host)?;
        for (op, re) in rest {
            let right = self.eval(re, env, host)?;
            if !ops::compare(*op, &left, &right, line)? {
                return Ok(Value::Bool(false));
            }
            left = right;
        }
        Ok(Value::Bool(true))
    }

    fn eval_fstring(&mut self, segs: &[FSeg], env: &Rc<RefCell<Env>>, host: &mut dyn Host) -> R<Value> {
        let mut out = String::new();
        for seg in segs {
            match seg {
                FSeg::Lit(s) => out.push_str(s),
                FSeg::Expr { expr, conv, spec } => {
                    let v = self.eval(expr, env, host)?;
                    let v = match conv {
                        Some('r') => Value::str(&v.repr()),
                        Some('s') => Value::str(&v.to_display_string()),
                        _ => v,
                    };
                    out.push_str(&format::format_value(&v, spec.as_deref().unwrap_or(""), expr.line)?);
                }
            }
        }
        Ok(Value::str(&out))
    }

    fn eval_list(&mut self, items: &[Expr], env: &Rc<RefCell<Env>>, host: &mut dyn Host) -> R<Value> {
        let mut out = Vec::with_capacity(items.len());
        for it in items {
            out.push(self.eval(it, env, host)?);
        }
        Ok(Value::list(out))
    }

    fn eval_dict(&mut self, pairs: &[(Expr, Expr)], env: &Rc<RefCell<Env>>, host: &mut dyn Host) -> R<Value> {
        let mut d = Dict::new();
        for (k, v) in pairs {
            let kv = self.eval(k, env, host)?;
            let key = Key::from_value(&kv).map_err(|m| rt(k.line, m))?;
            let vv = self.eval(v, env, host)?;
            d.insert(key, vv);
        }
        Ok(Value::dict(d))
    }

    fn eval_slice(
        &mut self,
        obj: &Expr,
        lo: &Option<Box<Expr>>,
        hi: &Option<Box<Expr>>,
        step: &Option<Box<Expr>>,
        env: &Rc<RefCell<Env>>,
        host: &mut dyn Host,
        line: u32,
    ) -> R<Value> {
        let o = self.eval(obj, env, host)?;
        let lo = self.eval_slice_bound(lo, env, host)?;
        let hi = self.eval_slice_bound(hi, env, host)?;
        let step = self.eval_slice_bound(step, env, host)?;
        ops::slice(&o, lo, hi, step, line)
    }

    fn eval_slice_bound(
        &mut self,
        e: &Option<Box<Expr>>,
        env: &Rc<RefCell<Env>>,
        host: &mut dyn Host,
    ) -> R<Option<i64>> {
        match e {
            None => Ok(None),
            Some(e) => match self.eval(e, env, host)? {
                Value::Null => Ok(None),
                Value::Int(n) => Ok(Some(n)),
                Value::Bool(b) => Ok(Some(b as i64)),
                other => Err(rt(e.line, format!("границы среза должны быть целыми, а не {}", other.type_name()))),
            },
        }
    }

    /// Генератор списка или словаря: своя область видимости, переменные цикла не утекают.
    fn eval_comp_expr(
        &mut self,
        elt: &Elt<'_>,
        clauses: &[CompClause],
        env: &Rc<RefCell<Env>>,
        host: &mut dyn Host,
    ) -> R<Value> {
        let scope = Env::child(env);
        let mut acc = match elt {
            Elt::List(_) => Acc::List(Vec::new()),
            Elt::Dict(..) => Acc::Dict(Dict::new()),
        };
        self.eval_comp(clauses, 0, elt, &scope, host, &mut acc)?;
        Ok(match acc {
            Acc::List(v) => Value::list(v),
            Acc::Dict(d) => Value::dict(d),
        })
    }

    /// Рекурсивный обход `for`-частей генератора; на последней — вычисляем элемент.
    fn eval_comp(
        &mut self,
        clauses: &[CompClause],
        idx: usize,
        elt: &Elt<'_>,
        env: &Rc<RefCell<Env>>,
        host: &mut dyn Host,
        acc: &mut Acc,
    ) -> R<()> {
        if idx == clauses.len() {
            match (elt, acc) {
                (Elt::List(e), Acc::List(out)) => {
                    let v = self.eval(e, env, host)?;
                    out.push(v);
                }
                (Elt::Dict(k, v), Acc::Dict(d)) => {
                    let kv = self.eval(k, env, host)?;
                    let key = Key::from_value(&kv).map_err(|m| rt(k.line, m))?;
                    let vv = self.eval(v, env, host)?;
                    d.insert(key, vv);
                }
                _ => {}
            }
            return Ok(());
        }
        let clause = &clauses[idx];
        let source = self.eval(&clause.iter, env, host)?;
        let items = ops::iter_to_vec(&source, clause.iter.line)?;
        'items: for item in items {
            self.tick(clause.iter.line, host)?;
            self.assign(&clause.target, item, env, host)?;
            for c in &clause.conds {
                if !self.eval(c, env, host)?.truthy() {
                    continue 'items;
                }
            }
            self.eval_comp(clauses, idx + 1, elt, env, host, acc)?;
        }
        Ok(())
    }

    /// Атрибут: у модуля — функция модуля (или `sys.args`), у строки/списка/словаря — метод.
    pub fn get_attr(&mut self, obj: Value, name: &Rc<str>, line: u32) -> R<Value> {
        match &obj {
            Value::Module(m) => builtins::module_attr(self, m, name, line),
            Value::Str(_) | Value::List(_) | Value::Dict(_) => {
                if !builtins::has_method(&obj, name) {
                    return Err(rt(line, format!("нет такого метода у {}: {}", builtins::type_genitive(&obj), name)));
                }
                Ok(Value::Method(Rc::new(BoundMethod { recv: obj.clone(), name: name.clone() })))
            }
            other => Err(rt(line, format!("объект типа {} не имеет атрибутов", other.type_name()))),
        }
    }
}
