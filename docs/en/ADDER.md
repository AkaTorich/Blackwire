# Adder — the Blackwire scripting language

Files `*.adr`. Run from the terminal: `adder name.adr [arguments]` or simply `./name.adr` if the file is executable.
Blocks are defined by indentation (4 spaces or a tab — what matters is consistency within a file). A comment is `#` to the end of the line.
A line ends a statement; line breaks inside unclosed brackets `( [ {` are ignored.

## Values

| Type    | Examples                        | Note |
|---------|---------------------------------|------|
| `null`  | `null`                          | nothing |
| `bool`  | `true`, `false`                 | |
| `int`   | `42`, `-7`, `0xFF`, `1_000_000` | 64-bit |
| `float` | `3.14`, `1e9`                   | |
| `str`   | `"text"`, `'text'`, `"""multiline"""`, `f"x = {x}"` | escapes `\n \t \\ \" \'`; f-strings with any expression inside `{}` |
| `list`  | `[1, "a", [2]]`                 | mutable, zero-based, negative indices count from the end |
| `dict`  | `{"ip": "10.0.0.1", "port": 22}`| keys are strings or numbers, insertion order is preserved |
| `func`  | `def`, `lambda`                 | functions are ordinary values |

Truthiness: `null`, `false`, `0`, `0.0`, `""`, `[]`, `{}` are false, everything else is true.

## Expressions

- Arithmetic: `+ - * / // % **`; `/` always yields a float, `//` is integer division, `%` is the remainder, `**` is power.
- Strings: `+` concatenation, `*` repetition (`"-" * 40`), `in` containment, `s[i]`, slices `s[1:3]`, `s[:-1]`, `s[::2]`.
- Lists: `+`, `in`, indexing and slicing as with strings.
- Comparisons: `== != < <= > >=` (chains like `1 < x < 10` are allowed).
- Logic: `and`, `or`, `not` — short-circuit, return an operand as in Python.
- Conditional expression: `a if condition else b`.
- Precedence as in Python.

## Statements

```
x = 5                 # assignment; a, b = b, a — swap; a = b = 0
x += 1                # also -= *= /= //= %=
lst[0] = 9            # assignment by index
d["k"] = v            # and by key

if a > b:
    ...
elif a == b:
    ...
else:
    ...

while condition:
    ...               # break, continue

for x in items:        # also over a string (by character), a range, a dict (by key)
    ...
for i, x in enumerate(items):
    ...
for k, v in d.items():
    ...

def name(a, b=10):     # default parameters; return with no value yields null
    return a + b

square = lambda x: x * x

try:
    ...
except:                # any error
    ...
except as e:           # e is a string with the error text
    ...
finally:
    ...

raise "message"        # error with a text
pass
exit()                 # end the script; exit(1) — with a code
```

Functions can read global variables; to change a global inside a function, use `global name`.
Functions can be nested, closures work.

## Built-in functions

`print(*args, sep=" ", end="")` — prints a line (each call is a separate line in the terminal).
`input(prompt="")` — reads a line from the player.
`len, str, int, float, bool, list, dict, range(stop | start, stop[, step]), type(x)` → `"int"`, `"str"`, …
`abs, min, max, sum, round(x[, n]), sorted(xs[, key][, reverse]), reversed, enumerate, zip, any, all`
`chr, ord, hex, bin, oct, repr`
`sleep(ms)` — pause, `exit([code])`.
`isinstance(x, "str")` — type check by name.

String methods: `upper lower strip lstrip rstrip split(sep=None) join replace find count startswith endswith isdigit isalpha isspace format(…) center(n) ljust(n) rjust(n) zfill(n) encode_hex decode_hex`.
List methods: `append pop([i]) insert remove index count sort(key=None, reverse=False) reverse clear extend copy`.
Dict methods: `keys values items get(k[, default]) pop(k) has(k) clear copy update(other)`.

Modules that don't touch the world (always available):
- `math`: `sqrt floor ceil sin cos tan atan2 log log2 log10 pow pi e`
- `json`: `loads(str)`, `dumps(value[, indent])`
- `time`: `now()` — game time in seconds, `format(ts[, fmt])` — date string, `ms()` — real-time milliseconds
- `random`: `random()`, `randint(a, b)`, `choice(list)`, `shuffle(list)`, `seed(n)`

## Game modules (via Host)

All access to the computer and the network goes through four modules; each function is a `Host::call("module.name", args)` call.

**sys** — your own machine and session
- `sys.args` — list of arguments (the first is the script name)
- `sys.hostname()`, `sys.whoami()`, `sys.is_root()`, `sys.cwd()`, `sys.env(name)`, `sys.set_env(name, value)`
- `sys.users()` — list of user names
- `sys.procs()` — list of dicts `{pid, name, user, cpu, mem}`
- `sys.kill(pid)`, `sys.uptime()`, `sys.notify(title, text)` — pop-up notification, `sys.pid()` — the script's own pid
- `sys.open(app, arg=null)` — open an application window (`editor`, `files`, `browser`, …)
- `sys.lang()` — game language: `"ru"` or `"en"`
- `sys.money()` — the player's balance (₵); `sys.debt()` — remaining debt; `sys.missions()` → list of `{code, title, status, objective, reward}`

**fs** — files of the current machine (the one we are connected to right now)
- `fs.read(path)` → str, `fs.write(path, text)`, `fs.append(path, text)`
- `fs.list(path=".")` → list of dicts `{name, path, is_dir, size, owner, mode}`
- `fs.exists(path)`, `fs.is_dir(path)`, `fs.mkdir(path)`, `fs.delete(path)`, `fs.move(from, to)`, `fs.copy(from, to)`
- `fs.chmod(path, "rwxr-xr-x")`, `fs.chown(path, user)`

**net** — network
- `net.my_ip()`, `net.resolve(name)` → ip or null, `net.ping(ip)` → milliseconds or null
- `net.scan(ip)` → list of `{port, service, version, state}`
- `net.connect(ip, user, password)` → true/false (like `ssh`; after that `fs.*`/`sys.*` work on that machine)
- `net.disconnect()` — go one step back along the chain
- `net.hops()` — list of machine names along the chain
- `net.http(url)` → `{status, title, body}` — request to a machine's web service

**net** — hacking (against the machine whose address is passed; the "from" side is the current hop of the chain)
- `net.vulns(ip)` → list of `{id, kind, version, service, port, name, access, local}`. Local vulnerabilities are visible on the current machine; their ids are used by research and privesc.
- `net.research(ip, id)` → `{id, name, service, port, version, local, kind, difficulty, fingerprint, layout, hints, fields, market, result, conditions, next}`. Selects a specific vulnerability by id, name or a unique service. A local one is researched after logging in to the machine.
- `net.probe(ip, id, payload)` → `{ok, applied:false, feedback, hint, id, kind}`. Checks a payload, leaves a trace and explains a rejection; does not grant access.
- `net.exploit(ip, id, payload)` → `{ok, access, user, password, path, data, host, monitored}`; on rejection `{ok:false, feedback, hint, id, kind}`. There is no automatic hacking without a payload. The fingerprint must match the researched target. Shell adds a hop, creds saves the login.
- `net.crack(ip, user[, wordlist])` → password (str) or null; the default wordlist is `~/wordlist.txt`
- `net.privesc(payload, id)` → `{ok, access, host, feedback}`; a wrong payload gives a rejection with feedback. Requires a local vulnerability of the current machine and a shell already obtained.
- `net.clear_logs([pattern])` → number of removed entries (−1 — no permission) — cover your tracks on the current machine
- `net.heat()` → number 0..100 — heat on the player; `net.owned()` → names of compromised machines
- `net.watch()` → `{stage, progress, raids, locked, lead}` — surveillance: investigation stage, the agency leading it, raid/lockout
- `net.proxy()` → `{chain, exit_ip, hidden, monitored}` — the chain and the exit address

**shell** — run a terminal command
- `shell.exec(line)` → `{out: str, ok: bool}` — the same shell as the terminal, output as a single string

Errors in these functions (no file, no access, no connection) are raised as exceptions — caught with `try/except`.

## Execution

The executor counts steps and at least every 500 steps asks `Host::should_stop()`; this way an infinite loop
is stopped by the `kill` command, Ctrl+C or closing the game. Recursion depth is limited (1000) — beyond that
there is a "stack overflow" error. A runtime error carries the line number: `Error (line 12): division by zero`.

## Example

```
# scan.adr — find machines with ssh open
targets = sys.args[1:]
if not targets:
    print("Usage: scan.adr ip [ip ...]")
    exit(1)

for ip in targets:
    ms = net.ping(ip)
    if ms == null:
        print(f"{ip:16} unreachable")
        continue
    ports = net.scan(ip)
    open_ssh = [p for p in ports if p["port"] == 22]   # list comprehensions are supported
    mark = "ssh" if open_ssh else "---"
    print(f"{ip:16} {ms:4} ms  {mark}  {len(ports)} ports")
```

List comprehensions `[expression for x in xs if condition]` and dict comprehensions `{k: v for ...}` are supported.
Formatting in f-strings: `{x:16}` width, `{x:>8}` right-aligned, `{x:.2f}` decimals, `{x:04}` zero padding.

### Access objects

`net.access(handle, method, ...args)` works with an object granted by an exploit.
The list of methods comes in the `net.exploit` result. The object is valid in one session
for 300 game seconds. A partial object does not grant a terminal. Full result parameters,
conditions and examples are described in SDK.md, section "Exploit results and conditions".
