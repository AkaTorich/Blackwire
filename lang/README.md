# blackwire-lang — язык Adder

Реализация скриптового языка игры Blackwire по `docs/ADDER.md`. Зависимости — только `serde`/`serde_json`.
Проверка: `cargo test -p blackwire-lang --jobs 2` (49 интеграционных тестов в `tests/`).

## Что где лежит

| Файл | Что внутри |
|------|-----------|
| `src/lib.rs` | Публичный API: `parse`, `check` (диагностика для редактора), `run`, трейт `Host`. |
| `src/error.rs` | `LangError { line, col, message }` с `runtime()`, `syntax()`, `Display` («Ошибка (строка 12): …»); внутренний сигнал `Exc` (ошибка / `Exit` / `Interrupted`). |
| `src/lexer.rs` | `tokenize`: отступы → INDENT/DEDENT/NEWLINE (`handle_indent`, смешение табов и пробелов — ошибка), переносы внутри скобок игнорируются, числа `0xFF` `1_000` `1e9` (`number`), строки и тройные кавычки с экранированием (`string_body`, `escape`), f-строки (`fstring_body`, `fexpr` — выражение, `!r`/`!s`, формат после `:`), комментарии `#`, `\` + перенос. `tokenize_expr` — для вложенных выражений f-строк. |
| `src/ast.rs` | Узлы дерева: `Stmt`/`StmtKind`, `Expr`/`ExprKind`, `FuncDef`, `CompClause`, `FSeg`. |
| `src/parser.rs` | Рекурсивный спуск с приоритетами Python: присваивания (цепочки, распаковка, `+=` и др. — `parse_expr_stmt`), `if/elif/else`, `while`, `for`, `def` с параметрами по умолчанию (`parse_params`), `try/except [as e]/finally` (`parse_try`), `raise`, `global`, `lambda`, условное выражение, цепочки сравнений (`parse_comparison`), `not in`, вызовы с именованными аргументами (`parse_call_args`), срезы (`parse_subscript`), генераторы списков/словарей (`parse_comp_clauses`), f-строки (`fstring` — рекурсивный разбор выражений). Однострочные блоки `if x: return 1` разрешены. |
| `src/value.rs` | `Value` (Null, Bool, Int, Float, Str, List, Dict, Func, Builtin, Method, Native, Module), `Dict` с порядком вставки (вектор пар + индекс), `Key` (строка/число; `true` и `2.0` сводятся к `1`/`2`), истинность, `py_eq`/`py_cmp` как в Python, `repr`/`to_display_string`, `to_json`/`from_json`, `fmt_float`. |
| `src/interp/mod.rs` | `Interpreter`, `Env` (цепочка областей, `global_names`), `run`, операторы (`exec_stmt`, `exec_for` с ленивым `range`, `exec_while`, `exec_try`, `exec_raise`), присваивание целей (`assign`), счётчик шагов (`tick` — каждые 500 шагов `Host::should_stop` → «прервано»), лимит рекурсии 1000 и защита стека (`enter_call`). |
| `src/interp/stack.rs` | Запас стека потока: на Windows — точная нижняя граница через `VirtualQuery`, иначе бюджет 1,4 МБ (`Interpreter::set_stack_budget`). Глубже — ошибка «переполнение стека» вместо аварии. |
| `src/interp/expr.rs` | Вычисление выражений: `eval`, f-строки (`eval_fstring`), генераторы в своей области (`eval_comp`), ленивые `and`/`or`, цепочки сравнений, срезы, атрибуты (`get_attr`: модуль → функция, строка/список/словарь → связанный метод). |
| `src/interp/ops.rs` | Арифметика (`/` → float, `//` и `%` округляют вниз, `**`, проверка переполнения, «деление на ноль»), сравнения и `in`, индексация с отрицательными индексами, срезы с шагом (`slice_indices`), перебор (`iter_to_vec`). |
| `src/interp/call.rs` | Вызовы: `call_function` (позиционные, именованные, значения по умолчанию, замыкание), `call_value`, `call_method`, `call_native` (`Native("fs.read")` → `host.call("fs.read", args)`, ошибка хоста становится исключением). |
| `src/builtins/mod.rs` | Установка глобальных имён (`install`), атрибуты модулей (`module_attr`: `sys.args` из аргументов запуска, `sys/fs/net/shell` → `Native`, `math/json/time/random` → встроенные), помощники разбора аргументов, `sort_values` (ключ, `reverse`). |
| `src/builtins/core.rs` | `print(sep, end)`, `input`, `len`, `str int float bool list dict range type`, `abs min max sum round sorted reversed enumerate zip any all map filter`, `chr ord hex bin oct repr`, `sleep`, `exit`, `isinstance`. |
| `src/builtins/strings.rs` | Методы строк из спецификации + `title capitalize swapcase rfind index partition splitlines isalnum isupper islower`. |
| `src/builtins/lists.rs` | `append pop insert remove index count sort(key, reverse) reverse clear extend copy`. |
| `src/builtins/dicts.rs` | `keys values items get pop has clear copy update setdefault`. |
| `src/builtins/format.rs` | Спецификация формата (`format_value`): ширина, выравнивание `< > ^ =`, заполнитель, знак, нули, `,`, точность, типы `f e % d x X b o c s`; `str.format` с `{}` `{0}` `{name}` (`str_format`). |
| `src/builtins/modules.rs` | `math` (`sqrt floor ceil sin cos tan atan atan2 log log2 log10 pow exp hypot pi e`), `time` (`now` — сначала `Host::call("time.now")`, иначе системные часы; `format(ts, fmt)` с `%Y %m %d %H %M %S %y %b %a`; `ms`), `random` (xorshift64*: `random randint choice shuffle seed uniform`). |
| `src/builtins/json.rs` | `json.loads`/`json.dumps(indent)` — свой разбор и запись, порядок ключей сохраняется. |

## Тесты (`tests/`)

`common/mod.rs` — тестовый `Host` (печать в `Vec<String>`, заготовки `sys.*`/`fs.*`/`net.*`/`shell.*`, счётчик `should_stop`).
`basics.rs` — арифметика, строки, срезы, f-строки и форматы, сравнения, логика, преобразования, присваивания.
`collections.rs` — списки, словари, их методы, генераторы, методы строк.
`control.rs` — ветвления, циклы, функции, замыкания, именованные аргументы, `global`, `try/except/finally`, `exit`, рекурсия.
`modules.rs` — `math/json/time/random`, вызовы хоста и его ошибки, `sys.args`, пример из спецификации, прерывание по `should_stop`, синтаксические ошибки и номера строк, JSON-обмен с хостом.

## Отличия и ограничения

- Присваивание внутри функции создаёт локальную переменную (как в Python); менять глобальную — `global`; `nonlocal` нет.
- Функции игровых модулей (`Native`) принимают только позиционные аргументы — `Host::call` получает `Vec<Value>`.
- Достижимая глубина рекурсии зависит от стека потока: лимит 1000, но при нехватке стека ошибка «переполнение стека» приходит раньше (в отладочной сборке на потоке 2 МБ — около 150 уровней). Для глубокой рекурсии запускать исполнитель в потоке с большим стеком.
- `time.now()` без ответа хоста — секунды системных часов (UTC).
