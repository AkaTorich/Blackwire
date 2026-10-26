# Ядро (crate `blackwire-core`) — договорённости

Ядро не знает про окна. Оболочка Tauri (`app`) держит `Game` под `Mutex` и вызывает его методы;
скрипты Adder работают в отдельных потоках и обращаются к `Game` через `script_host`.
Всё состояние мира сериализуется `serde` в JSON (сохранение игры).

## Модель

```rust
// world.rs
pub struct World {
    pub seed: u64,
    pub clock: GameClock,          // игровое время: epoch_secs (старт 2031-03-14 03:00), tick(dt_ms)
    pub machines: Vec<Machine>,    // machines[0] — компьютер игрока
    pub player: Player,            // handle, money, known_machines: Vec<MachineId>, notes
    pub settings: Settings,        // theme: String, fx: Fx {wireframe, hexrain, scanlines, glass}, font_size, terminal_opacity, sound
}
impl World {
    pub fn new(seed: u64, handle: &str) -> World;        // строит сеть через net::gen
    pub fn to_json(&self) -> String;
    pub fn from_json(s: &str) -> Result<World, String>;
    pub fn tick(&mut self, dt_ms: u64);                   // часы, фон процессов, сетевой шум
    pub fn machine(&self, id: MachineId) -> &Machine;  pub fn machine_mut(...)
    pub fn find_by_ip(&self, ip: &str) -> Option<MachineId>;
    pub fn resolve(&self, name: &str) -> Option<String>;  // dns: hostname / домен → ip
}

// net/machine.rs
pub type MachineId = u32;
pub struct Machine {
    pub id: MachineId, pub hostname: String, pub ip: String, pub domain: Option<String>,
    pub os: String, pub cpu: String, pub ram_mb: u32, pub disk_mb: u32,
    pub fs: vfs::Vfs, pub users: users::UserDb, pub procs: procs::ProcTable,
    pub services: Vec<net::services::Service>,   // {port, name, version, running, banner}
    pub logs: Vec<net::machine::LogEntry>,       // {time, kind, text}: вход, выход, ошибка пароля, запуск…
    pub kind: MachineKind,                       // Player, Router, Server, Workstation, …
    pub router: Option<MachineId>,               // за каким маршрутизатором (локальная сеть)
    pub lan_ip: Option<String>,
    pub up: bool,
}

// vfs.rs — файловая система одной машины
pub struct Vfs { ... }   // дерево узлов: Dir { children }, File { data: FileData::Text(String) | Binary(Vec<u8>) }
pub struct Meta { owner: String, mode: u16 /* 0o755 */, modified: u64, size }
impl Vfs {
    pub fn new_unix(owner_user: &str) -> Vfs;             // /bin /etc /home/<user> /var/log /tmp /root … с типичными файлами
    pub fn read(&self, path) -> Result<&str, FsError>;   pub fn read_bytes
    pub fn write(&mut self, path, text, who: &User) -> Result<(), FsError>;   // проверка прав
    pub fn list(&self, path) -> Result<Vec<FsEntry>, FsError>;
    pub fn mkdir / remove / rename / copy / exists / is_dir / chmod / chown / stat
    pub fn resolve(cwd: &str, path: &str) -> String;     // "..", ".", "~" уже раскрыт сеансом
    pub fn find(&self, root, pattern) -> Vec<String>;
}
pub enum FsError { NotFound(String), NotDir(String), IsDir(String), Denied(String), Exists(String), ... }  // Display по-русски

// users.rs
pub struct User { name, password: String /* открытым текстом — игра */, uid, home, shell, is_admin }
pub struct UserDb { users: Vec<User> } // add, remove, get, check_password, set_password, is_root(name)

// procs.rs
pub struct Proc { pid: u32, name: String, user: String, cpu: f32, mem_mb: u32, started: u64, kind: ProcKind /* Daemon, Script{owner_session}, System */ }
pub struct ProcTable { next_pid, procs: Vec<Proc> }  // spawn, kill, list, get, tick (колебания cpu)

// session.rs — один терминал
pub struct Hop { pub machine: MachineId, pub user: String }
pub struct Session {
    pub id: u32, pub hops: Vec<Hop>,        // hops[0] — своя машина; ssh добавляет, exit снимает
    pub cwd: String, pub env: BTreeMap<String, String>, pub history: Vec<String>,
    pub last_status: i32,
}
impl Session { pub fn current(&self) -> &Hop; pub fn home(&self, world) -> String; }

// game.rs — точка входа для Tauri
pub struct Game { pub world: World, pub sessions: HashMap<u32, Session>, next_session: u32 }
impl Game {
    pub fn new(world: World) -> Game;
    pub fn open_session(&mut self) -> u32;   pub fn close_session(&mut self, id);
    pub fn exec(&mut self, session: u32, line: &str) -> ShellResult;      // shell::exec
    pub fn complete(&self, session: u32, line: &str) -> Vec<String>;
    pub fn prompt(&self, session: u32) -> Prompt;
    // для окон (работают с машиной, на которой сейчас сеанс; session=None → своя машина)
    pub fn fs_list(&self, session: Option<u32>, path) -> Result<Vec<FsEntry>, String>;
    pub fn fs_read / fs_write / fs_mkdir / fs_delete / fs_rename / fs_copy
    pub fn status(&self) -> sysinfo::Status;   // cpu, mem_used, mem_total, net_in_kbps, net_out_kbps, uptime, procs: Vec<Proc>, hostname, ip
    pub fn net_map(&self) -> NetMap;           // nodes: {id, hostname, ip, kind, known: bool, x, y}, links: {a, b}
    pub fn http(&mut self, url: &str) -> HttpPage;  // {status, title, body(html)} — страницы веб-служб машин
    pub fn kill(&mut self, pid) -> Result<(), String>;
}
```

## Оболочка (shell)

`shell::exec(world, session, line) -> ShellResult`. Разбор строки: кавычки `' "`, экранирование `\`, переменные `$X` и `${X}`,
`~`, подстановка `*` и `?` в именах файлов, `;`, `&&`, `||`, конвейер `|`, перенаправления `>`, `>>`, `2>`, `<`.
Команда возвращает `CmdOutput { lines, status }`; в конвейере следующая получает stdin текстом.

Команды (все с `--help` / `help имя`; сообщения по-русски):

- Файлы: `ls [-l -a -h] [путь]`, `cd`, `pwd`, `cat`, `head [-n]`, `tail [-n]`, `less` (= cat), `touch`, `mkdir [-p]`, `rm [-r -f]`,
  `mv`, `cp [-r]`, `echo [-n]`, `grep [-i -n -r] шаблон [файлы]`, `wc [-l -w -c]`, `find [путь] [-name шаблон]`, `du`, `df`,
  `chmod режим файл`, `chown пользователь файл`, `file`, `tree [путь]`, `sort [-r -n]`, `uniq`, `cut -d x -f n`, `tr a b`, `diff`,
  `tar`/`zip` — архив `.tar` как папка-в-файле (пакует/распаковывает), `md5` (условный хеш строки)
- Система: `whoami`, `hostname`, `uname [-a]`, `date`, `uptime`, `ps [-a]`, `top` (один снимок), `kill pid`, `free`, `env`, `export X=1`,
  `unset`, `alias`, `history`, `clear`, `help`, `man команда`, `passwd [пользователь]`, `useradd имя [пароль]`, `userdel`, `users`,
  `su [пользователь]` (просит пароль: команда возвращает `UiAction::AskSecret`? — нет: пароль передаётся третьим словом `su root пароль`,
  а без него — подсказка; то же `ssh user@ip пароль`), `sudo команда` (если пользователь админ), `shutdown`/`reboot` (своя машина — перезапуск
  экрана; чужая — машина уходит в `up=false` на время), `logs [-n]` (журнал машины `/var/log/auth.log` и `system.log`), `exit`/`logout`
- Сеть: `ifconfig`/`ip`, `ping ip|имя` (4 строки с задержками 300–600 мс), `nslookup имя`, `scan ip` (= `nmap`: порты, службы, версии,
  строки с задержкой), `ssh user@ip пароль`, `scp [user@ip:]src [user@ip:]dst пароль`, `netstat`, `whois ip`, `traceroute ip`, `wget url [-O файл]`
  (через `Game::http`), `curl url`, `arp`, `route`
- Adder: `adder файл.adr [args]` → запускает скрипт процессом, `UiAction::ScriptStarted`; `./файл.adr` то же;
  `adder -c "print(1)"`; `adder --check файл`
- Окна: `edit файл` → `OpenApp{editor}`, `open приложение [аргумент]`, `files [путь]`, `browser [url]`, `map`, `monitor`, `settings`, `help`
- Прочее для вида: `banner текст` (крупные буквы из `#`), `matrix` (несколько строк кода-дождя с задержкой), `fortune`, `cal`, `bc выражение` (калькулятор),
  `hexdump файл`, `base64 [-d]`, `rev`, `yes` (ограничено 20 строками), `sleep сек` (строки с delay_ms), `neofetch` (карточка машины, ASCII-логотип)

Подсказка (`complete`): команды, имена файлов по текущей папке, имена пользователей после `su`, известные ip после `ssh`/`ping`/`scan`.

Все действия пишутся в журналы машин (вход по ssh, неверный пароль, запуск скрипта, удаление файлов) — основа для второго этапа (следы и слежка).

## Скрипты (`script_host.rs`)

```rust
pub struct ScriptHost { game: Arc<Mutex<Game>>, session: u32, pid: u32, stop: Arc<AtomicBool>,
                        out: Box<dyn FnMut(&str) + Send>, input: Box<dyn FnMut(&str) -> Option<String> + Send> }
impl lang::Host for ScriptHost { ... }   // реализует sys.* fs.* net.* shell.* из docs/ADDER.md
pub fn run_script(host: ScriptHost, src: String, args: Vec<String>) -> Result<Value, LangError>;  // блокирующий, зовётся из потока
```
`shell.exec` внутри скрипта исполняет команду в том же сеансе (вывод — строкой). `sys.open` и `sys.notify` возвращаются оболочке Tauri
через очередь `Game::pending_ui: Vec<UiAction>`, которую Tauri забирает после каждого вызова.

## Генератор сети (`net/gen.rs`) — первый этап

Из `seed` строится небольшая, но живая сеть: компьютер игрока (`hostname` = handle + "-pc", ip вида `87.x.x.x`), домашний маршрутизатор,
провайдер (`dns`, `gateway`), 12–20 машин разных видов с осмысленными именами и доменами (почтовый сервер, веб-сервер новостей,
университетский шлюз, маленький хостинг, чей-то домашний ПК, камера, банк-заглушка), у каждой службы с портами и версиями,
пользователи с паролями (часть слабых), файлы по дому (`notes.txt`, `.bash_history`, `passwords.txt` у неосторожных), журналы.
Две-три машины уже известны игроку (`player.known_machines`) — чтобы было куда пинговать и сканировать. Содержание — второй этап, но
генератор обязан быть расширяемым: виды машин и шаблоны файлов — таблицами, не хардкодом по месту.

На компьютере игрока: `/home/<handle>/` с `readme.txt` (приветствие, с чего начать), `scripts/` с 3–4 готовыми примерами `.adr`
(`hello.adr`, `scan.adr`, `bruteforce_demo.adr` — перебор паролей по словарю `wordlist.txt` через `net.connect`, `backup.adr`), `wordlist.txt`
(200 частых паролей), `/etc/motd`, `/etc/hosts`, `/var/log/*.log`.
