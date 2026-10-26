//! Вызовы: пользовательские функции (привязка позиционных и именованных аргументов,
//! значения по умолчанию, замыкание), встроенные функции, методы строк/списков/словарей,
//! функции игровых модулей через `Host::call`.

use std::cell::RefCell;
use std::rc::Rc;

use super::{Env, Flow, Interpreter, R};
use crate::ast::Expr;
use crate::builtins;
use crate::error::{rt, Exc};
use crate::value::{Call, Function, Value};
use crate::Host;

impl Interpreter {
    pub(crate) fn eval_call(
        &mut self,
        func: &Expr,
        args: &[Expr],
        kwargs: &[(Rc<str>, Expr)],
        env: &Rc<RefCell<Env>>,
        host: &mut dyn Host,
        line: u32,
    ) -> R<Value> {
        // `obj.method(...)` — без создания связанного метода
        if let crate::ast::ExprKind::Attr(obj_e, name) = &func.kind {
            let obj = self.eval(obj_e, env, host)?;
            let (a, k) = self.eval_args(args, kwargs, env, host)?;
            return self.call_method(obj, name, a, k, line, host);
        }
        let callee = self.eval(func, env, host)?;
        let (a, k) = self.eval_args(args, kwargs, env, host)?;
        self.call_value(&callee, a, k, line, host)
    }

    fn eval_args(
        &mut self,
        args: &[Expr],
        kwargs: &[(Rc<str>, Expr)],
        env: &Rc<RefCell<Env>>,
        host: &mut dyn Host,
    ) -> R<(Vec<Value>, Vec<(Rc<str>, Value)>)> {
        let mut a = Vec::with_capacity(args.len());
        for e in args {
            a.push(self.eval(e, env, host)?);
        }
        let mut k = Vec::with_capacity(kwargs.len());
        for (name, e) in kwargs {
            k.push((name.clone(), self.eval(e, env, host)?));
        }
        Ok((a, k))
    }

    /// Вызвать любое вызываемое значение.
    pub fn call_value(
        &mut self,
        callee: &Value,
        args: Vec<Value>,
        kwargs: Vec<(Rc<str>, Value)>,
        line: u32,
        host: &mut dyn Host,
    ) -> R<Value> {
        match callee {
            Value::Func(f) => self.call_function(f, args, kwargs, line, host),
            Value::Builtin(b) => (b.func)(self, host, Call { args, kwargs, line }),
            Value::Method(m) => self.call_method(m.recv.clone(), &m.name, args, kwargs, line, host),
            Value::Native(name) => self.call_native(name, args, kwargs, line, host),
            Value::Module(m) => Err(rt(line, format!("модуль {} нельзя вызвать", m))),
            other => Err(rt(line, format!("нельзя вызвать значение типа {}", other.type_name()))),
        }
    }

    /// Удобный вызов из встроенных функций (`sorted(key=...)`, `map`): без именованных аргументов.
    pub fn call_simple(&mut self, callee: &Value, args: Vec<Value>, line: u32, host: &mut dyn Host) -> R<Value> {
        self.call_value(callee, args, Vec::new(), line, host)
    }

    pub fn call_method(
        &mut self,
        recv: Value,
        name: &Rc<str>,
        args: Vec<Value>,
        kwargs: Vec<(Rc<str>, Value)>,
        line: u32,
        host: &mut dyn Host,
    ) -> R<Value> {
        let call = Call { args, kwargs, line };
        match &recv {
            Value::Module(m) => {
                let attr = builtins::module_attr(self, m, name, line)?;
                self.call_value(&attr, call.args, call.kwargs, line, host)
            }
            Value::Str(s) => builtins::strings::call(self, host, s, name, call),
            Value::List(l) => builtins::lists::call(self, host, l, name, call),
            Value::Dict(d) => builtins::dicts::call(self, host, d, name, call),
            other => Err(rt(line, format!("объект типа {} не имеет метода '{}'", other.type_name(), name))),
        }
    }

    fn call_native(
        &mut self,
        name: &str,
        args: Vec<Value>,
        kwargs: Vec<(Rc<str>, Value)>,
        line: u32,
        host: &mut dyn Host,
    ) -> R<Value> {
        if !kwargs.is_empty() {
            return Err(rt(line, format!("{}: именованные аргументы не поддерживаются", name)));
        }
        host.call(name, args).map_err(|e| Exc::Error(e.with_line(line)))
    }

    pub fn call_function(
        &mut self,
        f: &Rc<Function>,
        args: Vec<Value>,
        kwargs: Vec<(Rc<str>, Value)>,
        line: u32,
        host: &mut dyn Host,
    ) -> R<Value> {
        let def = &f.def;
        let fname = &def.name;
        let n = def.params.len();
        if args.len() > n {
            return Err(rt(
                line,
                format!("функция {}() принимает {} аргументов, передано {}", fname, n, args.len()),
            ));
        }
        let mut bound: Vec<Option<Value>> = args.into_iter().map(Some).collect();
        bound.resize(n, None);
        for (k, v) in kwargs {
            match def.params.iter().position(|p| p.name == k) {
                Some(i) => {
                    if bound[i].is_some() {
                        return Err(rt(line, format!("аргумент '{}' функции {}() передан дважды", k, fname)));
                    }
                    bound[i] = Some(v);
                }
                None => return Err(rt(line, format!("функция {}() не имеет параметра '{}'", fname, k))),
            }
        }
        let env = Env::child(&f.closure);
        {
            let mut e = env.borrow_mut();
            for (i, p) in def.params.iter().enumerate() {
                let v = match bound[i].take() {
                    Some(v) => v,
                    None => match &f.defaults[i] {
                        Some(d) => d.clone(),
                        None => {
                            return Err(rt(line, format!("функция {}() не получила аргумент '{}'", fname, p.name)));
                        }
                    },
                };
                e.vars.insert(p.name.clone(), v);
            }
        }
        self.enter_call(line)?;
        let r = self.exec_block(&def.body, &env, host);
        self.leave_call();
        match r? {
            Flow::Return(v) => Ok(v),
            _ => Ok(Value::Null),
        }
    }
}
