# Взлом — механики второго этапа (договор)

Всё происходит только в выдуманной игровой сети. «Взлом» — это игровые действия над машинами `blackwire_core`:
найти слабое место по версии службы, получить доступ, подняться до root, пройти через чужие машины, замести следы.
Настоящих атак здесь нет — это числа и строки в модели мира.

Модуль `core/src/hack/` — вся логика; работает над `&mut World` и `&mut Session` (текущий hop сеанса — «откуда» идёт действие).
Команды оболочки (`shell/cmds_hack.rs`) и функции языка (`script_host.rs`) только зовут этот модуль и рисуют вывод.

## Новое состояние (добавляю я в общих файлах — помощники не трогают)

`world.rs`, `Player`:
```rust
pub owned: Vec<MachineId>,          // машины, где получен доступ (shell или root)
pub found_creds: Vec<FoundCred>,    // найденные логины
pub heat: f32,                      // подозрение к игроку, 0..100
pub heat_peak: f32,                 // пик (для статистики)
pub traced: bool,                   // игрока запеленговали (heat дошёл до 100)
```
`machine.rs` (новый тип):
```rust
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct FoundCred { pub machine: MachineId, pub user: String, pub password: String, pub note: String }
```
`services.rs`, `Vuln` расширяю и таблицу `VULNS` тоже (добавляю столбцы доступа и области):
```rust
pub struct Vuln {
    pub service: String, pub version_prefix: String, pub name: String, pub effect: String,
    pub access: String,   // "creds" | "shell" | "root" | "read:/путь" | "info"
    pub scope: String,    // "remote" | "local"
}
```

## API модуля `hack` (помощник A реализует ЭТИ сигнатуры точно, не меняя их)

`core/src/hack/mod.rs`: `pub mod vuln; pub mod exploit; pub mod crack; pub mod privesc; pub mod tracks; pub mod proxy;`
и `pub use` всех публичных типов/функций ниже. Все структуры — `#[derive(Serialize, Deserialize, Clone, Debug)]`.

### vuln.rs — классификация
```rust
pub enum VulnAccess { Creds, Shell, Root, Read(String), Info }  // из строки Vuln.access
pub struct VulnInfo { pub service: String, pub port: u16, pub version: String, pub name: String, pub access: VulnAccess, pub local: bool }
pub fn scan_vulns(m: &Machine) -> Vec<VulnInfo>;     // все уязвимости машины (по её службам и таблице VULNS)
pub fn remote_vulns(m: &Machine) -> Vec<VulnInfo>;   // scope=remote
pub fn local_vulns(m: &Machine) -> Vec<VulnInfo>;    // scope=local (для privesc)
pub fn access_from_str(s: &str) -> VulnAccess;
```

### exploit.rs — получить доступ к удалённой машине с текущего hop'а
```rust
pub enum ExploitOutcome {
    Creds { user: String, password: String },  // узнали логин (сохранить в found_creds)
    Shell { user: String },                     // прямой вход: push_hop как этот пользователь, машина → owned
    Read { path: String, data: String },        // утечка файла
    Nothing,                                     // подходящей уязвимости нет
}
pub struct ExploitReport {
    pub target: MachineId, pub target_host: String,
    pub vuln: Option<VulnInfo>, pub outcome: ExploitOutcome,
    pub source_ip: String,     // адрес, который увидела цель (текущий hop)
    pub monitored: bool,       // цель под наблюдением (SOC/киберполиция/высокая care/служба monitor)
    pub heat_added: f32,
}
// Атакует target службой service (None — выбрать первую удачную удалённую уязвимость).
// Внутри: находит remote-уязвимость; Creds → кладёт в found_creds; Shell → session.push_hop + owned;
// Read → читает файл; пишет в журнал цели попытку (LogKind::Warn, с source_ip); если monitored — поднимает heat.
pub fn exploit(world: &mut World, session: &mut Session, target: MachineId, service: Option<&str>) -> Result<ExploitReport, String>;
```

### crack.rs — подбор пароля по словарю (ssh)
```rust
pub struct CrackReport {
    pub target: MachineId, pub target_host: String, pub user: String,
    pub tried: usize,                 // сколько слов перебрано до результата
    pub found: Option<String>,        // найденный пароль или None
    pub attempts: Vec<String>,        // слова по порядку до результата включительно (для анимации прогресса)
    pub source_ip: String, pub monitored: bool, pub heat_added: f32,
}
// Перебирает слова по порядку; совпадение с паролем user'а на target → found. Каждая НЕУДАЧНАЯ попытка пишет
// LoginFailed в журнал цели (у monitored растёт heat). Успех кладёт логин в found_creds. ssh сам не делает.
// Требует на target запущенный ssh; иначе Err. wordlist передаётся готовым списком слов (команда читает файл).
pub fn crack(world: &mut World, session: &Session, target: MachineId, user: &str, wordlist: &[String]) -> Result<CrackReport, String>;
```

### privesc.rs — локальное повышение прав на текущей машине
```rust
pub struct PrivescReport { pub ok: bool, pub vuln: Option<VulnInfo>, pub host: String, pub was_user: String }
// На машине текущего hop'а: если есть local-уязвимость и пользователь не root — делает текущего пользователя root
// (session.current_mut().user = "root"), машина → owned. Нет уязвимости или уже root → ok=false (не ошибка).
pub fn privesc(world: &mut World, session: &mut Session) -> Result<PrivescReport, String>;
```

### tracks.rs — следы, чистка журналов, подозрение
```rust
pub struct ClearReport { pub machine: MachineId, pub host: String, pub removed: usize, pub denied: bool }
// На машине текущего hop'а (нужен root или владелец): убирает записи журнала и строки /var/log/*.log,
// содержащие pattern (None → след игрока: его прежние адреса-источники и строки входа/ошибок с них).
// denied=true без прав. Чистка снижает улики → уменьшает вклад этой машины в heat.
pub fn clear_logs(world: &mut World, session: &Session, pattern: Option<&str>) -> Result<ClearReport, String>;

pub fn source_ip(world: &World, session: &Session) -> String;  // адрес текущего hop'а (что видит цель)
pub fn is_monitored(world: &World, target: MachineId) -> bool;  // SOC/киберполиция/care=paranoid/служба monitor
pub fn add_heat(world: &mut World, amount: f32);               // с ограничением 0..100, обновляет heat_peak, ставит traced при 100
pub fn heat(world: &World) -> f32;
pub fn decay(world: &mut World, dt_secs: u64);                 // heat медленно падает со временем (зовётся из World::tick)
pub fn evidence(world: &World) -> Vec<(MachineId, usize)>;     // машины и число записей, где засветился реальный адрес игрока
```
Модель подозрения: действие из реального адреса игрока (цепочка длиной 1) против monitored-цели добавляет heat ощутимо;
через прокси-цепочку (hops>1) реальный адрес цель не видит — heat растёт только если сам прокси-hop уже «засвечен»
у наблюдателя. Чистка журналов на hop'е убирает его улики. heat=100 → traced=true (последствия — мягко: уведомление и
письмо-предупреждение, heat сбрасывается к 60; полные последствия — позже).

### proxy.rs — цепочка подключений
```rust
pub struct ProxyInfo {
    pub chain: Vec<(MachineId, String)>,  // (машина, пользователь) по hop'ам, [0] — своя
    pub exit_ip: String, pub exit_host: String,
    pub hidden: bool,                      // цепочка длиннее 1 — реальный адрес скрыт
    pub monitored_hops: Vec<MachineId>,    // hop'ы под наблюдением (риск)
}
pub fn proxy_info(world: &World, session: &Session) -> ProxyInfo;
```

## Команды оболочки (помощник B, новый файл `shell/cmds_hack.rs`, группа `Group::Hack`)

Функции `pub fn ИМЯ(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput` — имена ниже (их впишу в table.rs):
- `exploit` — `exploit [-s служба] адрес`: `hack::exploit`; удачу печатает строками с задержками 200–500 мс; при Shell
  показывает новый hop, при Creds — логин, при Read — файл; при monitored — предупреждение о следе и росте подозрения.
- `crack` — `crack [-w словарь] пользователь@адрес` (или `crack адрес пользователь`): читает словарь (по умолчанию
  `~/wordlist.txt`), зовёт `hack::crack`, печатает прогресс по `attempts` с задержкой ~90 мс на слово, итог — пароль/неудача.
- `probe` — `probe адрес`: тихая разведка: скан служб + `scan_vulns` (список слабых мест) + список пользователей.
- `privesc` — `privesc`: `hack::privesc`; успех — «вы root на <host>».
- `clearlogs` (алиас `wipe`) — `clearlogs [шаблон]`: `hack::clear_logs`; без прав — отказ.
- `proxy` — `proxy`: `hack::proxy_info`; цепочка, выходной адрес, скрыт ли реальный, какие hop'ы под наблюдением.
- `heat` — `heat`: уровень подозрения (0..100), traced, список улик (`hack::evidence`).

Вывод по-русски, аккуратными строками; опасные действия помечать видом строки `Warn`/`Error`. Подсказка Tab для адреса —
как у `ssh` (известные ip). Группа `Group::Hack` с заголовком «Взлом» — добавлю в `table.rs` я.

## Функции языка Adder (добавляю я в script_host.rs и ADDER.md)

- `net.vulns(addr)` → список `{service, port, name, access, local}`
- `net.exploit(addr[, service])` → `{ok, access, user, password, path, data, host, monitored}` (ok=false — Nothing)
- `net.crack(addr, user[, wordlist_path])` → пароль (str) или null; по умолчанию словарь `~/wordlist.txt`
- `net.privesc()` → true/false
- `net.clear_logs([pattern])` → число убранных записей (−1 — отказано)
- `net.heat()` → число; `net.owned()` → список имён машин; `net.proxy()` → `{chain, exit_ip, hidden, monitored}`

## Проверка

`cargo test -p blackwire-core` (новый `core/tests/hack.rs`). В игре — снимками: `exploit`, `crack`, `probe`, `privesc`,
`clearlogs`, `proxy`, `heat` в терминале; скрипт на Adder, который сканирует, подбирает пароль и входит.
