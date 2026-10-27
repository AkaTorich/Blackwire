# Adder — язык скриптов Blackwire

Файлы `*.adr`. Запуск из терминала: `adder имя.adr [аргументы]` или просто `./имя.adr`, если файл исполняемый.
Блоки задаются отступами (4 пробела или таб — главное, одинаково внутри файла). Комментарий — `#` до конца строки.
Строка заканчивается оператором; внутри незакрытых скобок `( [ {` переносы строк не считаются.

## Значения

| Тип     | Примеры                         | Примечание |
|---------|---------------------------------|------------|
| `null`  | `null`                          | пустота |
| `bool`  | `true`, `false`                 | |
| `int`   | `42`, `-7`, `0xFF`, `1_000_000` | 64 бита |
| `float` | `3.14`, `1e9`                   | |
| `str`   | `"текст"`, `'текст'`, `"""многострочный"""`, `f"x = {x}"` | экранирование `\n \t \\ \" \'`; f-строки с любыми выражениями в `{}` |
| `list`  | `[1, "a", [2]]`                 | изменяемый, нумерация с 0, отрицательные индексы с конца |
| `dict`  | `{"ip": "10.0.0.1", "port": 22}`| ключи — строки или числа, порядок вставки сохраняется |
| `func`  | `def`, `lambda`                 | функции — обычные значения |

Истинность: `null`, `false`, `0`, `0.0`, `""`, `[]`, `{}` — ложь, остальное — истина.

## Выражения

- Арифметика: `+ - * / // % **`; `/` всегда даёт float, `//` — целое деление, `%` — остаток, `**` — степень.
- Строки: `+` соединение, `*` повтор (`"-" * 40`), `in` вхождение, `s[i]`, срезы `s[1:3]`, `s[:-1]`, `s[::2]`.
- Списки: `+`, `in`, индексы и срезы как у строк.
- Сравнения: `== != < <= > >=` (цепочки `1 < x < 10` разрешены).
- Логика: `and`, `or`, `not` — ленивые, возвращают операнд как в Python.
- Условное выражение: `a if условие else b`.
- Приоритет как в Python.

## Операторы

```
x = 5                 # присваивание; a, b = b, a — обмен; a = b = 0
x += 1                # также -= *= /= //= %=
lst[0] = 9            # присваивание по индексу
d["k"] = v            # и по ключу

if a > b:
    ...
elif a == b:
    ...
else:
    ...

while условие:
    ...               # break, continue

for x in список:       # также по строке (по символам), по range, по dict (по ключам)
    ...
for i, x in enumerate(список):
    ...
for k, v in d.items():
    ...

def имя(a, b=10):      # параметры по умолчанию; return без значения даёт null
    return a + b

квадрат = lambda x: x * x

try:
    ...
except:                # любая ошибка
    ...
except as e:           # e — строка с текстом ошибки
    ...
finally:
    ...

raise "сообщение"      # ошибка с текстом
pass
exit()                 # завершить скрипт; exit(1) — с кодом
```

Функции видят глобальные переменные для чтения; чтобы менять глобальную внутри функции — `global имя`.
Функции можно вкладывать, замыкания работают.

## Встроенные функции

`print(*args, sep=" ", end="")` — печатает строку (каждый вызов — отдельная строка в терминале).
`input(prompt="")` — читает строку от игрока.
`len, str, int, float, bool, list, dict, range(stop | start, stop[, step]), type(x)` → `"int"`, `"str"`, …
`abs, min, max, sum, round(x[, n]), sorted(xs[, key][, reverse]), reversed, enumerate, zip, any, all`
`chr, ord, hex, bin, oct, repr`
`sleep(ms)` — пауза, `exit([code])`.
`isinstance(x, "str")` — проверка типа по имени.

Методы строк: `upper lower strip lstrip rstrip split(sep=None) join replace find count startswith endswith isdigit isalpha isspace format(…) center(n) ljust(n) rjust(n) zfill(n) encode_hex decode_hex`.
Методы списков: `append pop([i]) insert remove index count sort(key=None, reverse=False) reverse clear extend copy`.
Методы словарей: `keys values items get(k[, default]) pop(k) has(k) clear copy update(other)`.

Модули без обращения к миру (есть всегда): 
- `math`: `sqrt floor ceil sin cos tan atan2 log log2 log10 pow pi e`
- `json`: `loads(str)`, `dumps(value[, indent])`
- `time`: `now()` — игровое время в секундах, `format(ts[, fmt])` — строка даты, `ms()` — миллисекунды реального времени
- `random`: `random()`, `randint(a, b)`, `choice(list)`, `shuffle(list)`, `seed(n)`

## Игровые модули (через Host)

Все обращения к компьютеру и сети идут через четыре модуля; каждая функция — вызов `Host::call("модуль.имя", args)`.

**sys** — своя машина и сеанс
- `sys.args` — список аргументов (первый — имя скрипта)
- `sys.hostname()`, `sys.whoami()`, `sys.is_root()`, `sys.cwd()`, `sys.env(name)`, `sys.set_env(name, value)`
- `sys.users()` — список имён пользователей
- `sys.procs()` — список словарей `{pid, name, user, cpu, mem}`
- `sys.kill(pid)`, `sys.uptime()`, `sys.notify(title, text)` — всплывающее уведомление, `sys.pid()` — pid самого скрипта
- `sys.open(app, arg=null)` — открыть окно приложения (`editor`, `files`, `browser`, …)
- `sys.money()` — счёт игрока (₵); `sys.debt()` — остаток долга; `sys.missions()` → список `{code, title, status, objective, reward}`

**fs** — файлы текущей машины (той, куда мы сейчас подключены)
- `fs.read(path)` → str, `fs.write(path, text)`, `fs.append(path, text)`
- `fs.list(path=".")` → список словарей `{name, path, is_dir, size, owner, mode}`
- `fs.exists(path)`, `fs.is_dir(path)`, `fs.mkdir(path)`, `fs.delete(path)`, `fs.move(from, to)`, `fs.copy(from, to)`
- `fs.chmod(path, "rwxr-xr-x")`, `fs.chown(path, user)`

**net** — сеть
- `net.my_ip()`, `net.resolve(name)` → ip или null, `net.ping(ip)` → миллисекунды или null
- `net.scan(ip)` → список `{port, service, version, state}`
- `net.connect(ip, user, password)` → true/false (как `ssh`; дальше `fs.*`/`sys.*` работают на той машине)
- `net.disconnect()` — вернуться на шаг назад по цепочке
- `net.hops()` — список имён машин по цепочке
- `net.http(url)` → `{status, title, body}` — запрос к веб-службе машины

**net** — взлом (над машиной, адрес которой передан; «откуда» — текущий hop цепочки)
- `net.vulns(ip)` → список `{id, kind, version, service, port, name, access, local}`. Локальные уязвимости видны на текущей машине; их id используют research и privesc.
- `net.research(ip, id)` → `{id, name, service, port, version, local, kind, difficulty, fingerprint, layout, hints, fields, market, result, conditions, next}`. Выбирает конкретную уязвимость по id, имени или уникальной службе. Локальная исследуется после входа на машину.
- `net.probe(ip, id, payload)` → `{ok, applied:false, feedback, hint, id, kind}`. Проверяет payload, оставляет след и объясняет отказ; доступ не выдаёт.
- `net.exploit(ip, id, payload)` → `{ok, access, user, password, path, data, host, monitored}`; при отказе `{ok:false, feedback, hint, id, kind}`. Без payload автоматического взлома нет. fingerprint должен совпасть с исследованной целью. Shell добавляет hop, creds сохраняет логин.
- `net.crack(ip, user[, словарь])` → пароль (str) или null; словарь по умолчанию `~/wordlist.txt`
- `net.privesc(payload, id)` → `{ok, access, host, feedback}`; неверный payload даёт отказ с feedback. Требует локальную уязвимость текущей машины и уже полученную оболочку.
- `net.clear_logs([шаблон])` → число убранных записей (−1 — нет прав) — замести следы на текущей машине
- `net.heat()` → число 0..100 — подозрение к игроку; `net.owned()` → имена взломанных машин
- `net.watch()` → `{stage, progress, raids, locked, lead}` — слежка: стадия расследования, его ведущий орган, рейд/блокировка
- `net.proxy()` → `{chain, exit_ip, hidden, monitored}` — цепочка и выходной адрес

**shell** — выполнить команду терминала
- `shell.exec(line)` → `{out: str, ok: bool}` — та же оболочка, что у терминала, вывод одной строкой

Ошибки в этих функциях (нет файла, нет доступа, нет связи) поднимаются как исключения — ловятся `try/except`.

## Исполнение

Исполнитель считает шаги и не реже чем каждые 500 шагов спрашивает `Host::should_stop()`; так бесконечный цикл
останавливается командой `kill`, Ctrl+C или закрытием игры. Глубина рекурсии ограничена (1000) — глубже
ошибка «переполнение стека». Ошибка времени исполнения несёт номер строки: `Ошибка (строка 12): деление на ноль`.

## Пример

```
# scan.adr — найти машины с открытым ssh
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
    open_ssh = [p for p in ports if p["port"] == 22]   # генераторы списков поддерживаются
    mark = "ssh" if open_ssh else "---"
    print(f"{ip:16} {ms:4} мс  {mark}  {len(ports)} портов")
```

Поддерживаются генераторы списков `[выражение for x in xs if условие]` и словарей `{k: v for ...}`.
Форматирование в f-строках: `{x:16}` ширина, `{x:>8}` вправо, `{x:.2f}` знаки, `{x:04}` нули.

### Объекты взлома

`net.access(handle, method, ...args)` обращается к выданному эксплоитом объекту.
Список методов приходит в результате `net.exploit`. Объект действует в одном сеансе
300 игровых секунд. Частичный объект не выдаёт терминал. Полные параметры результатов,
условия и примеры описаны в SDK.md, раздел «Результаты взлома и условия».
