//! Циклы, функции, замыкания, именованные аргументы, try/except, global, exit.

mod common;
use common::{err, lines, out, run};

use blackwire_lang::Value;

#[test]
fn if_elif_else_and_while() {
    let o = out(r#"
def sign(n):
    if n > 0:
        return "+"
    elif n < 0:
        return "-"
    else:
        return "0"
print(sign(5), sign(-1), sign(0))
i = 0
total = 0
while true:
    i += 1
    if i % 2 == 0:
        continue
    if i > 7:
        break
    total += i
print(i, total)
if 0: print("нет")
else: print("однострочный блок")
"#);
    assert_eq!(o, lines(&["+ - 0", "9 16", "однострочный блок"]));
}

#[test]
fn for_loops_over_everything() {
    let o = out(r#"
for i in range(3):
    print(i, end=";")
for i in range(1, 10, 4):
    print(i, end=";")
for ch in "ab":
    print(ch, end=";")
for k in {"x": 1, "y": 2}:
    print(k, end=";")
for a, b in [[1, 2], [3, 4]]:
    print(a + b, end=";")
xs = [1, 2]
for x in xs:
    if len(xs) < 4:
        xs.append(x * 10)
print(xs)
for i in range(10):
    if i == 2:
        break
else_marker = i
print(else_marker)
def first_even(xs):
    for x in xs:
        if x % 2 == 0:
            return x
    return null
print(first_even([1, 3, 4, 6]), first_even([1]))
"#);
    assert_eq!(o, lines(&["0;", "1;", "2;", "1;", "5;", "9;", "a;", "b;", "x;", "y;", "3;", "7;", "[1, 2, 10, 20]", "2", "4 null"]));
}

#[test]
fn functions_defaults_kwargs_and_errors() {
    let o = out(r#"
def greet(name, greeting="привет", punct="!"):
    return greeting + ", " + name + punct
print(greet("мир"), greet("мир", "здравствуй"), greet("мир", punct="?"), greet(punct=".", name="ты"))
def nothing():
    pass
def early():
    return
print(nothing(), early())
def fact(n):
    return 1 if n <= 1 else n * fact(n - 1)
print(fact(10), fact(20))
def pair():
    return 1, 2
a, b = pair()
print(a, b, pair())
"#);
    assert_eq!(
        o,
        lines(&["привет, мир! здравствуй, мир! привет, мир? привет, ты.", "null null", "3628800 2432902008176640000", "1 2 [1, 2]"])
    );
    assert_eq!(err("def f(a): pass\nf(1, 2)").message, "функция f() принимает 1 аргументов, передано 2");
    assert_eq!(err("def f(a, b): pass\nf(1)").message, "функция f() не получила аргумент 'b'");
    assert_eq!(err("def f(a): pass\nf(1, a=2)").message, "аргумент 'a' функции f() передан дважды");
    assert_eq!(err("def f(a): pass\nf(1, z=2)").message, "функция f() не имеет параметра 'z'");
    assert_eq!(err("x = 5\nx()").message, "нельзя вызвать значение типа int");
    assert_eq!(err("print(y)").message, "имя 'y' не определено");
    assert_eq!(err("sorted([1], foo=1)").message, "функция sorted() не имеет параметра 'foo'");
}

#[test]
fn default_values_evaluated_once() {
    let o = out(r#"
n = 1
def f(x=n):
    return x
n = 2
print(f(), f(5))
"#);
    assert_eq!(o, lines(&["1 5"]));
}

#[test]
fn closures_lambdas_and_nested_functions() {
    let o = out(r#"
def make_adder(n):
    def add(x):
        return x + n
    return add
add5 = make_adder(5)
add10 = make_adder(10)
print(add5(1), add10(1))
def make_counter():
    count = [0]
    def inc():
        count[0] += 1
        return count[0]
    return inc
c = make_counter()
c()
c()
print(c())
square = lambda x: x * x
compose = lambda f, g: lambda x: f(g(x))
print(square(4), compose(square, add5)(1), (lambda: 42)(), (lambda a, b=2: a * b)(3))
funcs = [lambda x, i=i: x + i for i in range(3)]
print([f(10) for f in funcs])
"#);
    assert_eq!(o, lines(&["6 11", "3", "16 36 42 6", "[10, 11, 12]"]));
}

#[test]
fn named_args_for_builtins() {
    let o = out(r#"
xs = [{"n": "b", "v": 2}, {"n": "a", "v": 3}]
print(sorted(xs, key=lambda p: p["n"]), sorted([3, 1, 2], reverse=true))
print([p["n"] for p in sorted(xs, key=lambda p: p["v"], reverse=true)])
print(1, 2, 3, sep="-", end="|")
print(list(enumerate("ab", start=5)))
"#);
    assert_eq!(
        o,
        lines(&[
            "[{\"n\": \"a\", \"v\": 3}, {\"n\": \"b\", \"v\": 2}] [3, 2, 1]",
            "[\"a\", \"b\"]",
            "1-2-3|",
            "[[5, \"a\"], [6, \"b\"]]",
        ])
    );
}

#[test]
fn global_statement() {
    let o = out(r#"
counter = 0
def bump():
    global counter
    counter += 1
def shadow():
    counter = 100
    return counter
bump()
bump()
print(counter, shadow(), counter)
def reads():
    return counter * 2
print(reads())
def creates():
    global fresh
    fresh = "новая"
creates()
print(fresh)
"#);
    assert_eq!(o, lines(&["2 100 2", "4", "новая"]));
}

#[test]
fn try_except_finally_raise() {
    let o = out(r#"
try:
    x = 1 / 0
except:
    print("поймали")
try:
    raise "своя ошибка"
except as e:
    print("e =", e)
finally:
    print("finally 1")
try:
    print("без ошибки")
finally:
    print("finally 2")
def risky(n):
    try:
        if n:
            raise "n=" + str(n)
        return "ок"
    except as e:
        return "ошибка: " + e
    finally:
        print("убираем", n)
print(risky(0), risky(3))
try:
    try:
        [][0]
    except as e:
        raise
except as outer:
    print("повторно:", outer)
try:
    undefined_name
except as e:
    print(e)
try:
    try:
        raise "внутри"
    finally:
        print("finally при ошибке")
except as e:
    print("снаружи:", e)
"#);
    assert_eq!(
        o,
        lines(&[
            "поймали",
            "e = своя ошибка",
            "finally 1",
            "без ошибки",
            "finally 2",
            "убираем 0",
            "убираем 3",
            "ок ошибка: n=3",
            "повторно: индекс списка вне диапазона",
            "имя 'undefined_name' не определено",
            "finally при ошибке",
            "снаружи: внутри",
        ])
    );
    let e = err("def f():\n    raise \"стоп\"\n\nf()\n");
    assert_eq!(e.line, 2);
    assert_eq!(e.message, "стоп");
}

#[test]
fn finally_return_overrides_and_exit_passes_through() {
    let o = out(r#"
def f():
    try:
        return 1
    finally:
        return 2
print(f())
try:
    exit(3)
except:
    print("exit нельзя поймать")
finally:
    print("но finally выполняется")
print("сюда не дойдём")
"#);
    assert_eq!(o, lines(&["2", "но finally выполняется"]));
}

#[test]
fn exit_codes() {
    let (r, host) = run("print('a')\nexit(7)\nprint('b')");
    assert_eq!(r.unwrap(), Value::Int(7));
    assert_eq!(host.out, lines(&["a"]));
    let (r, _) = run("exit()");
    assert_eq!(r.unwrap(), Value::Int(0));
    let (r, _) = run("x = 1");
    assert_eq!(r.unwrap(), Value::Null);
    let (r, _) = run("def f():\n    exit(2)\nf()\n");
    assert_eq!(r.unwrap(), Value::Int(2));
}

#[test]
fn recursion_limit_is_stack_overflow() {
    let e = err("def f(n):\n    return f(n + 1)\nf(0)\n");
    assert_eq!(e.message, "переполнение стека");
    assert_eq!(e.line, 2);
    // умеренная рекурсия работает
    let o = out("def s(n):\n    return 0 if n == 0 else n + s(n - 1)\nprint(s(60))\n");
    assert_eq!(o, lines(&["1830"]));
}

#[test]
fn stack_overflow_is_catchable_and_depth_is_reasonable() {
    let o = out("depth = 0\ndef f():\n    global depth\n    depth += 1\n    f()\ntry:\n    f()\nexcept as e:\n    print(depth >= 100, e)\n");
    assert_eq!(o, lines(&["true переполнение стека"]));
}

#[test]
fn pass_and_empty_blocks() {
    let o = out(r#"
def todo():
    pass
if true:
    pass
else:
    print("нет")
while false:
    pass
for _ in []:
    pass
print(todo())
"#);
    assert_eq!(o, lines(&["null"]));
}
