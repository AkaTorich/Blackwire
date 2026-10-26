//! Модули math/json/time/random, вызовы хоста, прерывание, синтаксические ошибки.

mod common;
use common::{err, lines, out, run, run_with, TestHost};

use blackwire_lang::{check, parse, Host, LangError, Value};

#[test]
fn math_module() {
    let o = out(r#"
print(math.sqrt(16), math.floor(2.7), math.ceil(2.1), math.floor(-2.5), round(math.sin(0), 3), math.cos(0))
print(round(math.pi, 5), round(math.e, 5), math.pow(2, 10), round(math.log(math.e), 5), round(math.log2(8)), round(math.log10(1000), 5))
print(round(math.atan2(1, 1), 4), math.tan(0), round(math.log(8, 2), 5))
"#);
    assert_eq!(
        o,
        lines(&["4.0 2 3 -3 0.0 1.0", "3.14159 2.71828 1024.0 1.0 3 3.0", "0.7854 0.0 3.0"])
    );
    assert_eq!(err("math.sqrt(-1)").message, "math.sqrt(): корень из отрицательного числа");
    assert_eq!(err("math.foo(1)").message, "модуль math не имеет атрибута 'foo'");
}

#[test]
fn json_module_keeps_order() {
    let o = out(r#"
d = json.loads('{"z": 1, "a": [1, 2.5, "x", null, true], "n": {"k": -3}}')
print(d, d["a"][1], type(d["n"]["k"]))
print(json.dumps(d))
print(json.dumps({"a": [1, {"b": 2}], "c": "стр\"ока"}, indent=2))
print(json.dumps([]), json.dumps({}), json.dumps("s"), json.dumps(1.5), json.dumps(null))
print(json.loads("[1e2, -0, \"\\u0410\\n\"]"))
"#);
    assert_eq!(
        o,
        lines(&[
            "{\"z\": 1, \"a\": [1, 2.5, \"x\", null, true], \"n\": {\"k\": -3}} 2.5 int",
            "{\"z\": 1, \"a\": [1, 2.5, \"x\", null, true], \"n\": {\"k\": -3}}",
            "{\n  \"a\": [\n    1,\n    {\n      \"b\": 2\n    }\n  ],\n  \"c\": \"стр\\\"ока\"\n}",
            "[] {} \"s\" 1.5 null",
            "[100.0, 0, \"А\\n\"]",
        ])
    );
    let e = err("json.loads('{bad}')");
    assert!(e.message.starts_with("json.loads(): "), "{}", e.message);
    assert_eq!(err("json.dumps(len)").message, "значение типа func нельзя записать в JSON");
}

#[test]
fn random_module_is_deterministic_with_seed() {
    let o = out(r#"
random.seed(42)
a = [random.randint(1, 6) for _ in range(20)]
random.seed(42)
b = [random.randint(1, 6) for _ in range(20)]
print(a == b, all(1 <= x <= 6 for x in a), len(a))
r = random.random()
print(0 <= r < 1, type(r))
xs = [1, 2, 3, 4, 5]
random.shuffle(xs)
print(sorted(xs), random.choice(["only"]), 1 <= random.uniform(1, 2) <= 2)
"#);
    assert_eq!(o, lines(&["true true 20", "true float", "[1, 2, 3, 4, 5] only true"]));
    assert_eq!(err("random.choice([])").message, "random.choice(): пустой список");
}

#[test]
fn time_module() {
    let o = out(r#"
print(time.format(0), time.format(1700000000, "%d.%m.%Y %H:%M"), time.format(86399, "%H:%M:%S %a %b %%"))
print(type(time.now()), type(time.ms()), time.ms() > 1600000000000, time.now() > 1600000000)
"#);
    assert_eq!(
        o,
        lines(&["1970-01-01 00:00:00 14.11.2023 22:13 23:59:59 Thu Jan %", "int int true true"])
    );
}

#[test]
fn host_calls_through_game_modules() {
    let (r, host) = run(r#"
print(sys.hostname(), sys.is_root())
fs.write("/tmp/a.txt", "данные")
print(fs.read("/tmp/a.txt"))
ports = net.scan("10.0.0.1")
print([p["port"] for p in ports], net.ping("10.0.0.1"), net.ping("10.0.0.9"))
res = shell.exec("ls")
print(res["out"], res["ok"])
f = fs.read
print(type(f), f("/tmp/a.txt"))
"#);
    r.unwrap();
    assert_eq!(
        host.out,
        lines(&["node-1 false", "данные", "[22, 80] 12 null", "done true", "func данные"])
    );
    let names: Vec<&str> = host.calls.iter().map(|(n, _)| n.as_str()).collect();
    assert_eq!(names, vec!["sys.hostname", "sys.is_root", "fs.write", "fs.read", "net.scan", "net.ping", "net.ping", "shell.exec", "fs.read"]);
    assert_eq!(host.calls[2].1, vec![Value::str("/tmp/a.txt"), Value::str("данные")]);
}

#[test]
fn host_errors_become_exceptions() {
    let o = out(r#"
try:
    fs.read("/нет")
except as e:
    print("ошибка:", e)
try:
    net.unknown_thing(1)
except as e:
    print(e)
"#);
    assert_eq!(o, lines(&["ошибка: нет такого файла: /нет", "неизвестная функция net.unknown_thing"]));
    let e = err("x = 1\nfs.read(\"/нет\")\n");
    assert_eq!(e.line, 2);
    assert_eq!(e.message, "нет такого файла: /нет");
}

#[test]
fn sys_args_and_example_script() {
    let src = r#"
targets = sys.args[1:]
if not targets:
    print("Использование: scan.adr ip [ip ...]")
    exit(1)

for ip in targets:
    ms = net.ping(ip)
    if ms == null:
        print(f"{ip:16} недоступен")
        continue
    ports = net.scan(ip)
    open_ssh = [p for p in ports if p["port"] == 22]
    mark = "ssh" if open_ssh else "---"
    print(f"{ip:16} {ms:4} мс  {mark}  {len(ports)} портов")
"#;
    let (r, host) = run_with(src, vec!["scan.adr".into(), "10.0.0.1".into(), "10.0.0.9".into()]);
    assert_eq!(r.unwrap(), Value::Null);
    assert_eq!(host.out, lines(&["10.0.0.1           12 мс  ssh  2 портов", "10.0.0.9         недоступен"]));
    let (r, host) = run_with(src, vec!["scan.adr".into()]);
    assert_eq!(r.unwrap(), Value::Int(1));
    assert_eq!(host.out, lines(&["Использование: scan.adr ip [ip ...]"]));
    let o = out("print(sys.args, len(sys.args), sys.args[0])");
    assert_eq!(o, lines(&["[\"test.adr\"] 1 test.adr"]));
}

#[test]
fn input_and_sleep_go_to_host() {
    let mut host = TestHost::default();
    host.inputs.push_back("Олег".to_string());
    let r = blackwire_lang::run("name = input(\"Имя: \")\nprint(\"привет,\", name)\nsleep(250)\n", &mut host, vec![]);
    r.unwrap();
    assert_eq!(host.out, lines(&["привет, Олег"]));
    assert_eq!(host.slept, vec![250]);
    let mut host = TestHost::default();
    let r = blackwire_lang::run("try:\n    input()\nexcept as e:\n    print(e)\n", &mut host, vec![]);
    r.unwrap();
    assert_eq!(host.out, lines(&["ввод закрыт"]));
}

#[test]
fn should_stop_interrupts_infinite_loop() {
    let mut host = TestHost { stop_after: Some(3), ..Default::default() };
    let r = blackwire_lang::run("i = 0\nwhile true:\n    i += 1\n", &mut host, vec![]);
    let e = r.unwrap_err();
    assert_eq!(e.message, "прервано");
    assert!(e.line >= 2 && e.line <= 3, "строка {}", e.line);
    // прерывание не ловится try/except
    let mut host = TestHost { stop_after: Some(1), ..Default::default() };
    let r = blackwire_lang::run("while true:\n    try:\n        x = 1\n    except:\n        pass\n", &mut host, vec![]);
    assert_eq!(r.unwrap_err().message, "прервано");
    // без запроса остановки длинный цикл дорабатывает
    let o = out("n = 0\nfor i in range(20000):\n    n += i\nprint(n)\n");
    assert_eq!(o, lines(&["199990000"]));
}

/// Хост, который считает обращения к should_stop.
struct CountingHost {
    checks: std::cell::Cell<usize>,
}

impl Host for CountingHost {
    fn print(&mut self, _: &str) {}
    fn input(&mut self, _: &str) -> Result<String, LangError> {
        Err(LangError::runtime(0, "нет ввода"))
    }
    fn call(&mut self, name: &str, _: Vec<Value>) -> Result<Value, LangError> {
        Err(LangError::runtime(0, format!("неизвестная функция {}", name)))
    }
    fn should_stop(&self) -> bool {
        self.checks.set(self.checks.get() + 1);
        false
    }
    fn sleep(&mut self, _: u64) {}
}

#[test]
fn should_stop_is_polled_every_500_steps() {
    let mut host = CountingHost { checks: std::cell::Cell::new(0) };
    blackwire_lang::run("i = 0\nwhile i < 5000:\n    i += 1\n", &mut host, vec![]).unwrap();
    let checks = host.checks.get();
    // ~3 шага на виток (проверка условия, тело, присваивание): около 15000 шагов → ~30 проверок
    assert!(checks >= 15 && checks <= 60, "проверок: {}", checks);
}

#[test]
fn syntax_errors_with_positions() {
    let e = err("x = (1 + \n");
    assert!(e.line >= 1 && e.col > 0, "{:?}", e);
    let e = err("if true:\nprint(1)\n");
    assert_eq!(e.message, "ожидался отступ");
    assert_eq!(e.line, 2);
    let e = err("x = 1\n    y = 2\n");
    assert_eq!(e.message, "неожиданный отступ");
    let e = err("if 1:\n    x = 1\n  y = 2\n");
    assert_eq!(e.message, "несогласованный отступ");
    let e = err("if 1:\n    x = 1\n\ty = 2\n");
    assert_eq!(e.message, "в файле смешаны отступы табами и пробелами");
    let e = err("x = \"незакрытая\n");
    assert_eq!(e.message, "незакрытая строка");
    let e = err("x = 1 +* 2");
    assert!(e.message.starts_with("неожиданный токен"), "{}", e.message);
    let e = err("def f(a=1, b): pass");
    assert_eq!(e.message, "параметр без значения по умолчанию после параметра со значением");
    let e = err("1 = x");
    assert_eq!(e.message, "недопустимая цель присваивания");
    let e = err("x = 1 $ 2");
    assert_eq!(e.message, "неизвестный символ '$'");
    let e = err("f(a=1, 2)");
    assert_eq!(e.message, "позиционный аргумент после именованного");
    assert_eq!(err("try:\n    pass\n").message, "после try ожидался except или finally");
    assert_eq!(err("else:\n    pass\n").message, "'else' без соответствующего начала блока");
    assert_eq!(err("x = f\"{}\"").message, "пустое выражение в f-строке");
    assert!(parse("x = 1\nif x:\n    print(x)\n").is_ok());
    assert!(check("x = ").len() == 1);
    assert!(check("x = 1").is_empty());
    assert_eq!(LangError::syntax(2, 5, "тест").to_string(), "Ошибка (строка 2, позиция 5): тест");
}

#[test]
fn tabs_as_indentation_work() {
    let o = out("if true:\n\tx = 1\n\tif x:\n\t\tprint(\"таб\")\n");
    assert_eq!(o, lines(&["таб"]));
}

#[test]
fn runtime_error_lines_inside_functions_and_loops() {
    let e = err("def f(xs):\n    total = 0\n    for x in xs:\n        total += 10 // x\n    return total\n\nprint(f([1, 2, 0]))\n");
    assert_eq!(e.line, 4);
    assert_eq!(e.message, "деление на ноль");
    let e = err("xs = [1,\n  2]\nprint(xs[5])\n");
    assert_eq!(e.line, 3);
    let e = err("x = {\n  'a': 1}\n\n\nprint(x['b'])");
    assert_eq!(e.line, 5);
}

#[test]
fn value_json_roundtrip_for_host() {
    let mut d = blackwire_lang::value::Dict::new();
    d.insert(blackwire_lang::value::Key::Str("a".into()), Value::list(vec![Value::Int(1), Value::Float(2.5), Value::Null]));
    d.insert(blackwire_lang::value::Key::Int(3), Value::Bool(true));
    let v = Value::dict(d);
    let j = v.to_json();
    assert_eq!(j, serde_json::json!({"a": [1, 2.5, null], "3": true}));
    let back = Value::from_json(&j);
    assert_eq!(back.repr(), "{\"3\": true, \"a\": [1, 2.5, null]}");
    assert_eq!(Value::from_json(&serde_json::json!("s")), Value::str("s"));
    assert_eq!(Value::Int(1), Value::Float(1.0));
}

#[test]
fn modules_can_be_shadowed_and_printed() {
    let o = out("print(math, type(sys), fs.read)\nmath = 5\nprint(math + 1)\n");
    assert_eq!(o, lines(&["<модуль math> module <функция fs.read>", "6"]));
    assert_eq!(err("sys()").message, "модуль sys нельзя вызвать");
    assert_eq!(err("fs.read(path=1)").message, "fs.read: именованные аргументы не поддерживаются");
}
