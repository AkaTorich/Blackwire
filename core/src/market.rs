//! Хакерский маркет в скрытой сети hw: за ₵ покупают готовые скрипты-инструменты
//! на языке Adder. Купил — скрипт лёг в `~/scripts`, запускаешь командой `adder`.
//! Это поддерживает смысл игры: заработал — купил инструмент — взломал больше.

use serde::{Deserialize, Serialize};

use crate::world::World;

/// Товар маркета — готовый скрипт.
#[derive(Clone, Copy, Debug)]
pub struct MarketItem {
    pub id: &'static str,
    pub name: &'static str,
    pub price: i64,
    pub file: &'static str,
    pub desc: &'static str,
    pub code: &'static str,
}

/// Состояние маркета (что куплено) — в мире.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct MarketState {
    pub bought: std::collections::BTreeSet<String>,
}

pub const MARKET: &[MarketItem] = &[
    MarketItem {
        id: "netscan",
        name: "Сканер сети",
        price: 150,
        file: "netscan.adr",
        desc: "проверяет адреса на ssh и слабые места",
        code: NETSCAN,
    },
    MarketItem { id: "sweep", name: "Обходчик", price: 120, file: "sweep.adr", desc: "пингует все узлы из /etc/hosts", code: SWEEP },
    MarketItem { id: "autocrack", name: "Автоподбор", price: 350, file: "autocrack.adr", desc: "подбирает пароль и сразу входит", code: AUTOCRACK },
    MarketItem { id: "cleaner", name: "Чистильщик следов", price: 200, file: "cleaner.adr", desc: "убирает следы на текущей машине", code: CLEANER },
    MarketItem { id: "loot", name: "Сборщик добычи", price: 250, file: "loot.adr", desc: "показывает интересные файлы машины", code: LOOT },
    MarketItem { id: "botctl", name: "Вербовщик", price: 400, file: "botctl.adr", desc: "ставит кейлоггер и берёт машину в ботнет", code: BOTCTL },
];

// Скрипты хранятся «сырыми» строками, чтобы сохранялись отступы Adder.
const NETSCAN: &str = r##"# Сканер: проверяет адреса на открытый ssh и слабые места
targets = sys.args[1:]
if not targets:
    print("Использование: netscan.adr адрес [адрес ...]")
    exit(1)
for ip in targets:
    ms = net.ping(ip)
    if ms == null:
        print(f"{ip} недоступен")
        continue
    ports = net.scan(ip)
    ssh = [p for p in ports if p["service"] == "ssh"]
    vulns = net.vulns(ip)
    mark = "есть" if ssh else "нет"
    print(f"{ip}: портов {len(ports)}, ssh {mark}, слабых мест {len(vulns)}")
"##;

const SWEEP: &str = r##"# Обходчик: пинг всех известных узлов из /etc/hosts
hosts = fs.read("/etc/hosts")
for line in hosts.split("\n"):
    line = line.strip()
    if line == "" or line.startswith("#"):
        continue
    ip = line.split()[0]
    ms = net.ping(ip)
    mark = "жив" if ms != null else "нет"
    print(f"{ip:16} {mark}")
"##;

const AUTOCRACK: &str = r##"# Автоподбор: подбирает пароль по словарю и входит
if len(sys.args) < 3:
    print("Использование: autocrack.adr адрес пользователь")
    exit(1)
ip = sys.args[1]
user = sys.args[2]
pw = net.crack(ip, user)
if pw == null:
    print("Пароль не подобран")
    exit(1)
print(f"Пароль: {pw}")
if net.connect(ip, user, pw):
    print(f"Вход выполнен: {user}@{ip}")
else:
    print("Войти не удалось")
"##;

const CLEANER: &str = r##"# Чистильщик: убирает следы на текущей машине
n = net.clear_logs()
if n < 0:
    print("Нет прав — нужен root")
else:
    print(f"Убрано записей: {n}")
"##;

const LOOT: &str = r##"# Сборщик: ищет интересные файлы и показывает первую строку
roots = ["/root", "/home", "/var"]
found = 0
for root in roots:
    try:
        items = fs.list(root)
    except:
        continue
    for f in items:
        name = f["name"]
        if name.endswith(".txt") or name.endswith(".log") or name.endswith(".csv"):
            try:
                data = fs.read(f["path"])
                first = data.split("\n")[0]
                print(f"{f['path']}: {first}")
                found = found + 1
            except:
                pass
print(f"Найдено файлов: {found}")
"##;

const BOTCTL: &str = r##"# Вербовщик: ставит кейлоггер и добавляет машину в ботнет
ip = sys.args[1] if len(sys.args) > 1 else net.my_ip()
r1 = shell.exec("keylog plant " + ip)
print(r1["out"])
r2 = shell.exec("botnet add " + ip)
print(r2["out"])
"##;

pub fn item(id: &str) -> Option<&'static MarketItem> {
    MARKET.iter().find(|i| i.id == id)
}

/// Купить скрипт: списать ₵ и положить файл в `~/scripts`.
pub fn buy(world: &mut World, id: &str) -> Result<String, String> {
    let it = item(id).ok_or_else(|| format!("нет такого скрипта: {}", id))?;
    if world.market.bought.contains(id) {
        // перекупка не нужна, но переустановим файл бесплатно
        write_script(world, it);
        return Ok(format!("{} уже куплен — переустановлен в ~/scripts/{}", it.name, it.file));
    }
    if world.player.money < it.price {
        return Err(format!("не хватает: нужно {} ₵, есть {} ₵", it.price, world.player.money));
    }
    world.player.money -= it.price;
    world.market.bought.insert(id.to_string());
    write_script(world, it);
    Ok(format!("Куплено: {} (−{} ₵) → ~/scripts/{}. Запуск: adder scripts/{}", it.name, it.price, it.file, it.file))
}

fn write_script(world: &mut World, it: &MarketItem) {
    let handle = world.player.handle.clone();
    let home = world.machines[0].users.home_of(&handle);
    let dir = format!("{}/scripts", home);
    let _ = world.machines[0].fs.mkdir_all(&dir, &handle);
    let path = format!("{}/{}", dir, it.file);
    let _ = world.machines[0].fs.write_text(&path, it.code, &handle);
    let _ = world.machines[0].fs.set_mode(&path, 0o755);
}

/// Страница маркета (HTML) — узлы `gate.hw/market` и `market.hw`.
pub fn market_html(world: &World) -> String {
    let mut s = String::from("<header><h1>Маркет</h1></header><nav><a href=\"/board\">Доска</a> <a href=\"/shop\">Барахолка</a></nav><main>");
    s.push_str(&format!("<p>На счету: <b>{} ₵</b></p><p><small>Купить в терминале: <code>market buy id</code>. Скрипт ляжет в ~/scripts.</small></p><ul>", world.player.money));
    for it in MARKET {
        let owned = world.market.bought.contains(it.id);
        let mark = if owned { " — <i>куплено</i>" } else { "" };
        s.push_str(&format!("<li><b>{}</b> ({}) — {} ₵ · {}{}</li>", it.name, it.id, it.price, it.desc, mark));
    }
    s.push_str("</ul></main>");
    s
}
