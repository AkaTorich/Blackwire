//! Арифметика, строки, срезы, f-строки, сравнения, логика.

mod common;
use common::{err, lines, out};

#[test]
fn arithmetic_int_and_float() {
    let o = out(r#"
print(2 + 3 * 4, 7 / 2, 7 // 2, -7 // 2, 7 % 3, -7 % 3, 2 ** 10)
print(1.5 + 1, 3 / 1.5, 2 ** -1, 2 ** 0.5 > 1.41)
print(0xFF, 1_000_000, 1e3, 0b101, 0o17)
print(10 - 2 - 3, 2 ** 3 ** 2, -2 ** 2)
"#);
    assert_eq!(
        o,
        lines(&[
            "14 3.5 3 -4 1 2 1024",
            "2.5 2.0 0.5 true",
            "255 1000000 1000.0 5 15",
            "5 512 -4",
        ])
    );
}

#[test]
fn division_by_zero_has_line() {
    let e = err("x = 1\ny = 0\nz = x / y\n");
    assert_eq!(e.line, 3);
    assert_eq!(e.message, "деление на ноль");
    assert_eq!(e.to_string(), "Ошибка (строка 3): деление на ноль");
    assert_eq!(err("print(5 % 0)").message, "деление на ноль");
    assert_eq!(err("print(5 // 0)").message, "деление на ноль");
}

#[test]
fn type_errors_in_operations() {
    assert_eq!(err("print('a' + 1)").message, "неподдерживаемые операнды для +: str и int");
    assert_eq!(err("print([1] - [2])").message, "неподдерживаемые операнды для -: list и list");
    assert_eq!(err("print(1 < 'a')").message, "нельзя сравнить int и str");
}

#[test]
fn strings_concat_repeat_in_index() {
    let o = out(r#"
s = "привет"
print("-" * 5, "ab" + "cd", "ри" in s, "x" in s, s[0], s[-1], len(s))
print('одинарные', "кавычки \"внутри\"", 'и \'так\'', "таб\tконец")
t = """много
строк"""
print(t)
print(len("\n"), "\x41Ж")
"#);
    assert_eq!(
        o,
        lines(&[
            "----- abcd true false п т 6",
            "одинарные кавычки \"внутри\" и 'так' таб\tконец",
            "много\nстрок",
            "1 AЖ",
        ])
    );
}

#[test]
fn slices_with_step() {
    let o = out(r#"
s = "abcdefg"
xs = [0, 1, 2, 3, 4, 5, 6]
print(s[1:3], s[:-1], s[::2], s[::-1], s[5:], s[-3:], s[10:], s[2:1])
print(xs[1:3], xs[::-2], xs[:2], xs[-2:], xs[1:6:2])
print(s[null:2])
"#);
    assert_eq!(
        o,
        lines(&[
            "bc abcdef aceg gfedcba fg efg  ",
            "[1, 2] [6, 4, 2, 0] [0, 1] [5, 6] [1, 3, 5]",
            "ab",
        ])
    );
    assert_eq!(err("print([1,2][::0])").message, "шаг среза не может быть нулём");
}

#[test]
fn index_errors() {
    assert_eq!(err("xs = [1, 2]\nprint(xs[2])").message, "индекс списка вне диапазона");
    assert_eq!(err("print('ab'[5])").message, "индекс строки вне диапазона");
    assert_eq!(err("d = {'a': 1}\nprint(d['b'])").message, "ключ не найден: \"b\"");
    assert_eq!(err("print([1][1.5])").message, "индекс должен быть целым, а не float");
}

#[test]
fn fstrings_basic_and_nested() {
    let o = out(r#"
x = 7
name = "ip"
d = {"k": [1, 2]}
print(f"x = {x}, x*2 = {x * 2}, {name!r}, {d['k'][1]}, {'да' if x > 5 else 'нет'}")
print(f"скобки {{x}} и {len(name)}")
print(f"{x}{x}" + f"{x}")
print(f"{f'{x}!'}")
"#);
    assert_eq!(
        o,
        lines(&["x = 7, x*2 = 14, \"ip\", 2, да", "скобки {x} и 2", "777", "7!"])
    );
}

#[test]
fn fstring_format_specs() {
    let o = out(r#"
ip = "10.0.0.1"
ms = 42
pi = 3.14159
print(f"[{ip:16}][{ms:4}][{pi:.2f}][{ms:04}][{ip:>12}][{ms:<6}][{ip:^12}]")
print(f"[{1234567:,}][{255:x}][{255:X}][{5:b}][{8:o}][{ms:+}][{-ms:+}][{0.256:.1%}][{1234.5:e}]")
print(f"[{pi:8.3f}][{pi:08.3f}][{'abc':.2}][{ms:*>5}][{true}][{null}]")
print("{} + {} = {sum}".format(2, 3, sum=5), "{0}{0}{1}".format("a", "b"), "{:>4}|".format(7))
"#);
    assert_eq!(
        o,
        lines(&[
            "[10.0.0.1        ][  42][3.14][0042][    10.0.0.1][42    ][  10.0.0.1  ]",
            "[1,234,567][ff][FF][101][10][+42][-42][25.6%][1.234500e+03]",
            "[   3.142][0003.142][ab][***42][true][null]",
            "2 + 3 = 5 aab    7|",
        ])
    );
}

#[test]
fn comparisons_and_chains() {
    let o = out(r#"
x = 5
print(1 < x < 10, 1 < x > 10, 1 == 1.0, 1 != 2, "a" < "b", [1, 2] < [1, 3], 1 <= x <= 5 == 5)
print(true == 1, false == 0, null == null, null == 0, [1] == [1], {"a": 1} == {"a": 1})
print(2 in [1, 2], 3 not in [1, 2], "k" in {"k": 1}, "ab" in "cabd")
"#);
    assert_eq!(
        o,
        lines(&["true false true true true true true", "true true true false true true", "true true true true"])
    );
}

#[test]
fn logic_returns_operand_and_truthiness() {
    let o = out(r#"
print(0 or "дефолт", 5 and 7, null or [], "" or 0, not 0, not [1], 1 and 0 or 9)
print(bool(0), bool(0.0), bool(""), bool([]), bool({}), bool(null), bool("0"), bool([0]))
x = 3 if 1 > 2 else 4
print(x, "да" if [] else "нет")
"#);
    assert_eq!(o, lines(&["дефолт 7 [] 0 true false 9", "false false false false false false true true", "4 нет"]));
}

#[test]
fn print_sep_end_and_repr() {
    let o = out(r#"
print("a", "b", sep=", ")
print("x", end="!")
print()
print([1, "a", [2, 3.0]], {"k": "v", 1: null}, repr("s"), str(1.0), str(true))
print(1.0, 2.5, 1e20, 10 / 4, float("3"), int("42"), int(3.9), int("ff", 16), float(2))
"#);
    assert_eq!(
        o,
        lines(&[
            "a, b",
            "x!",
            "",
            "[1, \"a\", [2, 3.0]] {\"k\": \"v\", 1: null} \"s\" 1.0 true",
            "1.0 2.5 100000000000000000000.0 2.5 3.0 42 3 255 2.0",
        ])
    );
}

#[test]
fn conversions_and_type() {
    let o = out(r#"
print(type(1), type(1.0), type("s"), type([]), type({}), type(null), type(true), type(len), type(lambda: 0))
print(isinstance(1, "int"), isinstance("s", "int"), isinstance(2.0, ["int", "float"]))
print(list("abc"), list({"a": 1, "b": 2}), dict([["a", 1], ["b", 2]]), str(123) + "!", int("-7"), int(true))
print(abs(-3), abs(-2.5), round(2.5), round(3.5), round(2.675, 2), round(7, 1), min(3, 1, 2), max([4, 9, 2]))
print(chr(65), ord("A"), hex(255), bin(5), oct(8), hex(-1))
"#);
    assert_eq!(
        o,
        lines(&[
            "int float str list dict null bool func func",
            "true false true",
            "[\"a\", \"b\", \"c\"] [\"a\", \"b\"] {\"a\": 1, \"b\": 2} 123! -7 1",
            "3 2.5 2 4 2.68 7 1 9",
            "A 65 0xff 0b101 0o10 -0x1",
        ])
    );
    assert_eq!(err("int('abc')").message, "не удалось преобразовать \"abc\" в int");
}

#[test]
fn multiple_assignment_and_swap() {
    let o = out(r#"
a, b = 1, 2
a, b = b, a
c = d = 0
xs = [1, 2, 3]
xs[0], xs[2] = xs[2], xs[0]
[p, q] = "ab"
x = y = z = 5
x += 1
y -= 2
z *= 3
z //= 2
z %= 4
w = 2.0
w /= 4
print(a, b, c, d, xs, p, q, x, y, z, w)
"#);
    assert_eq!(o, lines(&["2 1 0 0 [3, 2, 1] a b 6 3 3 0.5"]));
    assert_eq!(err("a, b = [1, 2, 3]").message, "ожидалось 2 значений для распаковки, получено 3");
}

#[test]
fn comments_and_line_continuation_in_brackets() {
    let o = out(r#"
# комментарий
xs = [
    1,  # первый
    2,
    3,
]
d = {
    "a": 1,
    "b": (2 +
          3),
}
print(len(xs), d["b"])  # хвост
"#);
    assert_eq!(o, lines(&["3 5"]));
}
