//! Списки, словари, их методы, генераторы, методы строк.

mod common;
use common::{err, lines, out};

#[test]
fn list_methods() {
    let o = out(r#"
xs = [3, 1, 2]
xs.append(5)
xs.insert(0, 9)
xs.insert(-1, 7)
print(xs, xs.pop(), xs.pop(0), xs)
xs.extend([8, 1])
print(xs.index(1), xs.count(1), xs)
xs.remove(1)
xs.sort()
print(xs)
xs.sort(reverse=true)
print(xs)
ys = xs.copy()
ys.reverse()
ys.clear()
print(xs, ys, len(ys))
words = ["bb", "a", "ccc"]
words.sort(key=len, reverse=true)
print(words, sorted(words), sorted(words, key=lambda w: w[-1]))
"#);
    assert_eq!(
        o,
        lines(&[
            "[3, 1, 2, 7] 5 9 [3, 1, 2, 7]",
            "1 2 [3, 1, 2, 7, 8, 1]",
            "[1, 2, 3, 7, 8]",
            "[8, 7, 3, 2, 1]",
            "[8, 7, 3, 2, 1] [] 0",
            "[\"ccc\", \"bb\", \"a\"] [\"a\", \"bb\", \"ccc\"] [\"a\", \"bb\", \"ccc\"]",
        ])
    );
    assert_eq!(err("[].pop()").message, "pop из пустого списка");
    assert_eq!(err("[1].remove(2)").message, "значение не найдено в списке: 2");
    assert_eq!(err("[1].foo()").message, "нет такого метода у списка: foo");
    assert!(err("[1, 'a'].sort()").message.starts_with("нельзя сравнить"));
}

#[test]
fn list_aliasing_and_equality() {
    let o = out(r#"
a = [1, 2]
b = a
b.append(3)
c = a + [4]
print(a, b, c, a == b, a == [1, 2, 3], "ok")
"#);
    assert_eq!(o, lines(&["[1, 2, 3] [1, 2, 3] [1, 2, 3, 4] true true ok"]));
}

#[test]
fn dict_methods_and_order() {
    let o = out(r#"
d = {"b": 1, "a": 2}
d["c"] = 3
d["b"] = 10
print(d, d.keys(), d.values(), d.items(), len(d))
print(d.get("a"), d.get("zzz"), d.get("zzz", -1), d.has("a"), d.has("zzz"), "c" in d)
print(d.pop("a"), d)
d.update({"x": 0, "b": 11})
e = d.copy()
e.clear()
print(d, e, d.setdefault("y", 5), d.setdefault("y", 6))
n = {1: "один", 2.0: "два", true: "правда"}
print(n, n[1], n[2])
for k, v in {"k1": 1, "k2": 2}.items():
    print(k, v)
for k in {"q": 1, "w": 2}:
    print(k)
"#);
    assert_eq!(
        o,
        lines(&[
            "{\"b\": 10, \"a\": 2, \"c\": 3} [\"b\", \"a\", \"c\"] [10, 2, 3] [[\"b\", 10], [\"a\", 2], [\"c\", 3]] 3",
            "2 null -1 true false true",
            "2 {\"b\": 10, \"c\": 3}",
            "{\"b\": 11, \"c\": 3, \"x\": 0, \"y\": 5} {} 5 5",
            "{1: \"правда\", 2: \"два\"} правда два",
            "k1 1",
            "k2 2",
            "q",
            "w",
        ])
    );
    assert_eq!(err("{}.pop('k')").message, "ключ не найден: \"k\"");
    assert_eq!(err("{}.foo()").message, "нет такого метода у словаря: foo");
    assert_eq!(err("d = {[1]: 2}").message, "ключ словаря должен быть строкой или числом, а не list");
}

#[test]
fn comprehensions() {
    let o = out(r#"
xs = [1, 2, 3, 4, 5]
print([x * x for x in xs if x % 2], [c.upper() for c in "abc"], [[i, j] for i in range(2) for j in range(2) if i != j])
print({k: len(k) for k in ["a", "bb"]}, {i: i * i for i in range(4) if i > 1})
ports = [{"port": 22}, {"port": 80}]
print([p for p in ports if p["port"] == 22], sum(x for x in xs), list(x + 1 for x in xs))
x = 100
print([x for x in range(3)], x)
"#);
    assert_eq!(
        o,
        lines(&[
            "[1, 9, 25] [\"A\", \"B\", \"C\"] [[0, 1], [1, 0]]",
            "{\"a\": 1, \"bb\": 2} {2: 4, 3: 9}",
            "[{\"port\": 22}] 15 [2, 3, 4, 5, 6]",
            "[0, 1, 2] 100",
        ])
    );
}

#[test]
fn string_methods() {
    let o = out(r#"
s = "  Hello, World  "
print("[" + s.strip() + "]", "[" + s.lstrip() + "]", "[" + s.rstrip() + "]", "xxhixx".strip("x"))
print("a,b,,c".split(","), "a b  c".split(), "a,b,c".split(",", 1), ", ".join(["x", "y"]), "-".join("abc"))
print("hello".upper(), "HeLLo".lower(), "hello world".title(), "hello".capitalize(), "aBc".swapcase())
print("banana".replace("a", "o"), "banana".replace("a", "o", 1), "banana".find("na"), "banana".find("x"), "banana".rfind("na"), "banana".count("a"))
print("file.adr".startswith("file"), "file.adr".endswith([".txt", ".adr"]), "123".isdigit(), "12a".isdigit(), "abc".isalpha(), "  ".isspace(), "".isdigit())
print("[" + "hi".center(6) + "]", "[" + "hi".ljust(5, ".") + "]", "[" + "hi".rjust(5) + "]", "42".zfill(5), "-42".zfill(5))
print("hi".encode_hex(), "6869".decode_hex(), "привет".encode_hex().decode_hex(), "x = {}".format(5))
print("a\nb\nc".splitlines(), "ab".partition("x"), "key=val".partition("="), "ab".index("b"))
f = "abc".upper
print(f(), type(f))
"#);
    assert_eq!(
        o,
        lines(&[
            "[Hello, World] [Hello, World  ] [  Hello, World] hi",
            "[\"a\", \"b\", \"\", \"c\"] [\"a\", \"b\", \"c\"] [\"a\", \"b,c\"] x, y a-b-c",
            "HELLO hello Hello World Hello AbC",
            "bonono bonana 2 -1 4 3",
            "true true true false true true false",
            "[  hi  ] [hi...] [   hi] 00042 -0042",
            "6869 hi привет x = 5",
            "[\"a\", \"b\", \"c\"] [\"ab\", \"\", \"\"] [\"key\", \"=\", \"val\"] 1",
            "ABC func",
        ])
    );
    assert_eq!(err("'a'.foo()").message, "нет такого метода у строки: foo");
    assert_eq!(err("'a'.join([1])").message, "join() ожидает список строк, встречен int");
    assert_eq!(err("print(1.foo)").message, "объект типа int не имеет атрибутов");
}

#[test]
fn builtin_iteration_helpers() {
    let o = out(r#"
xs = ["a", "b", "c"]
for i, x in enumerate(xs):
    print(i, x)
print(list(enumerate(xs, 1)), list(zip([1, 2, 3], "ab")), list(reversed(xs)))
print(any([0, "", 3]), all([1, "x"]), all([]), any([]), sum([1, 2, 3.5]), sum([], 10))
print(list(map(lambda x: x * 2, [1, 2])), list(filter(lambda x: x > 1, [1, 2, 3])), list(range(5, 0, -2)), list(range(3)))
print(min([3, 1, 2]), max("b", "a"), max([], default=0), min(["aaa", "b"], key=len), sorted([3, 1, 2], reverse=true))
"#);
    assert_eq!(
        o,
        lines(&[
            "0 a",
            "1 b",
            "2 c",
            "[[1, \"a\"], [2, \"b\"], [3, \"c\"]] [[1, \"a\"], [2, \"b\"]] [\"c\", \"b\", \"a\"]",
            "true true true false 6.5 10",
            "[2, 4] [2, 3] [5, 3, 1] [0, 1, 2]",
            "1 b 0 b [3, 2, 1]",
        ])
    );
    assert_eq!(err("len(5)").message, "объект типа int не имеет длины");
    assert_eq!(err("for x in 5:\n    pass").message, "объект типа int не является итерируемым");
}

#[test]
fn nested_structures_and_index_assignment() {
    let o = out(r#"
grid = [[0] * 3 for _ in range(2)]
grid[0][1] = 5
d = {"list": [1, 2], "d": {"x": 1}}
d["list"].append(3)
d["d"]["y"] = 2
d["d"]["x"] += 10
counts = {}
for ch in "abca":
    counts[ch] = counts.get(ch, 0) + 1
print(grid, d, counts)
"#);
    assert_eq!(
        o,
        lines(&["[[0, 5, 0], [0, 0, 0]] {\"list\": [1, 2, 3], \"d\": {\"x\": 11, \"y\": 2}} {\"a\": 2, \"b\": 1, \"c\": 1}"])
    );
    assert_eq!(err("s = 'ab'\ns[0] = 'x'").message, "строки неизменяемы");
}
