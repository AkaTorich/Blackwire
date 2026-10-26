# blackwire-core — ядро игры Blackwire

Виртуальный компьютер и сеть, оболочка (shell) и всё, что нужно оболочке Tauri.
Окон и Tauri здесь нет: оболочка держит `Game` под мьютексом и дёргает его методы.
Всё состояние мира сериализуется `serde` в JSON (сохранение игры).

Язык скриптов Adder — отдельный крейт `blackwire-lang`; связка со скриптами
(`script_host.rs`) добавляется поверх. Сеть (`src/net`) пишет другой помощник;
ядро работает против его сигнатур.

## Подсистемы

### Случайные числа — `src/rng.rs`
Детерминированный xorshift64*: один и тот же `seed` даёт один и тот же мир.
- `Rng::new`, `next_u64`, `next_f64`, `range`, `range_f`, `chance`, `pick`, `shuffle`, `jitter`.

### Файловая система — `src/vfs/`
Дерево каталогов и файлов одной машины с владельцами, правами rwx и временем.
- `vfs/mod.rs` — `Vfs`, `Node`, `Meta`, `FsError` (Display по-русски).
  - без проверки прав (установка): `new_unix`, `add_user_home`, `write_text`, `write_binary`,
    `mkdir_all`, `set_mode`, `set_owner`, `read_text`, `append_text`.
  - с проверкой пользователя: `read`, `read_bytes`, `write`, `write_bytes`, `append`, `touch`,
    `mkdir`, `remove`, `rename`, `copy`, `list`, `chmod`, `chown`.
  - запросы: `exists`, `is_dir`, `is_file`, `stat`, `find`, `walk`, `deep_size`, `total_size`.
- `vfs/path.rs` — `normalize`, `resolve`, `split_parent`, `basename`, `dirname`, `join`,
  `mode_string`, `parse_mode` (`755`, `u+x`, `rwxr-xr-x`), `glob_match`, `has_glob`.
- `vfs/template.rs` — типичное дерево Unix (`/bin /etc /home /root /var/log …`).

### Пользователи — `src/users.rs`
`User`, `UserDb`: `new_root`, `add`, `remove`, `get`, `check_password`, `set_password`,
`is_root`, `is_admin`, `names`, `home_of`, `passwd_text`.

### Процессы — `src/procs.rs`
`Proc`, `ProcKind` (System/Daemon/Script), `ProcTable`: `new`, `spawn_daemon`, `spawn_script`,
`kill`, `kill_as`, `mark_killed`, `is_killed`, `remove`, `tick` (колебание CPU), `total_cpu/mem`.

### Сеанс терминала — `src/session.rs`
`Session`, `Hop`, `PendingScript`: цепочка подключений (`hops`), текущая папка, окружение,
история, псевдонимы. `current`, `at_home`, `push_hop`, `pop_hop`, `home`, `set_cwd`.

### Показатели машины — `src/sysinfo.rs`
`Status` (hostname, ip, uptime, cpu, память, сеть, процессы) — живые значения из таблицы
процессов. `status(world, id)`.

### Мир — `src/world.rs`
`World`: `new` (через `net::gen::generate`), `to_json`/`from_json`, `tick`, часы `GameClock`
(старт 2031-03-14 03:00, `format_time`, `format_duration`, календарь), `Settings`, `Player`
(в т. ч. `flags` для сюжета). Сеть: `resolve`, `find_by_ip`, `lookup`, `reachable`,
`address_for` (локальные адреса видны только изнутри организации / со шлюза), `http`
(`http://host[:port]/path`, `home://`), `log`, `shutdown_machine`, `reboot_player`.

### Оболочка — `src/shell/`
- `mod.rs` — `exec` (конвейеры `|`, перенаправления `> >> 2> <`, `; && ||`, подстановки),
  `complete` (Tab), `prompt`, `shell_doc`; `Ctx` — окружение команды.
- `parse.rs` — разбор строки: кавычки `' "`, экранирование, `$X`/`${X}`, `~`, `*`/`?`.
- `table.rs` — список всех команд (имя, группа, использование, описание, функция).
- `help.rs` — тексты `--help`/`man`, таблица `help`, Markdown `shell_doc()`.
- `util.rs` — `CmdOutput`, разбор флагов `Args`, таблицы-колонки, размеры, base64, хеш.
- `font.rs` — крупный шрифт 5×5 для `banner`.
- `cmds_fs.rs` — ls, cd, pwd, cat, touch, mkdir, rm, rmdir, mv, cp, echo, find, du, df,
  chmod, chown, file, stat, tree, ln.
- `cmds_text.rs` — head, tail, grep, wc, sort, uniq, cut, tr, rev, tee, diff, md5, base64,
  hexdump, tar/zip/unzip (архив — папка в одном файле).
- `cmds_sys.rs` — whoami, id, hostname, uname, date, uptime, ps, top, kill, free, env,
  export, unset, alias, history, clear, help, man, which, passwd, useradd, userdel, users,
  su, sudo, shutdown, reboot, logs, exit, true, false.
- `cmds_net.rs` — ifconfig, ip, ping, nslookup, scan/nmap, ssh, scp, netstat, whois,
  traceroute, wget, curl, arp, route.
- `cmds_misc.rs` — adder (запуск скрипта процессом, `-c`, `--check`), edit/open/files/browser/
  map/monitor/settings/mail (→ `UiAction::OpenApp`), banner, matrix, fortune, cal, bc, yes,
  sleep, seq, neofetch.

### Точка входа — `src/game.rs`
`Game` = мир + сеансы. Для моста Tauri: `open_session`/`close_session`, `exec`, `complete`,
`prompt`, `history`, `shell_doc`; файлы для окон `fs_list/read/write/mkdir/delete/rename/copy/exists`;
`status`, `net_map`, `http`, `settings`/`set_settings`; скрипты `take_pending_scripts`,
`script_finished`, `kill`, `is_killed`, `pending_ui`/`take_pending_ui`/`notify`; почта
`mail_address`, `mail_list`, `mail_read`, `mail_send`, `mail_delete`, `mail_unread_count`.

## Сеть и мир — `src/net`

Машины, службы на портах, организации со своими подсетями, почта, DNS и генератор
мира из зерна. Данные отделены от кода генерации: таблицы в отдельных файлах.

- `net/mod.rs` — подмодули и повторный экспорт `machine`, `services`, `orgs`.
- `net/machine.rs` — `Machine` (адреса, железо, fs, пользователи, процессы, службы,
  журнал `LogEntry`, почта `MailMessage`), `MachineKind`, `MailFolder`; игровое время
  (`WORLD_EPOCH`, `civil`, `fmt_time`, `fmt_syslog`, `fmt_date_ru`).
- `net/services.rs` — `Service`, `WebPage`, выбор версии по «возрасту» (`Age`), набор служб
  по виду машины, уязвимости (`Vuln`, `known_vulns`, `find_vulns`).
- `net/services_data.rs` — таблицы служб (порты, версии, баннеры, демоны), наборы по видам
  машин, таблица уязвимых версий.
- `net/orgs.rs` — `Org`, `OrgKind`, аккуратность `Care`, шаблоны (`OrgTemplate`, `Role`,
  `NameVariant`).
- `net/orgs_data.rs` — сами шаблоны организаций: имена, домены, описания, состав машин,
  порядок размещения в мире.
- `net/names.rs` — люди (логин, имя, фамилия), 200+ частых паролей (тот же список, что
  в словаре игрока), генераторы слабого/сильного пароля, код приглашения.
- `net/files_data.rs` — тексты файлов: заметки, дела, история с утечками, забытые пароли,
  договоры, база пациентов, журнал транзакций, график смен, настройки камеры,
  `sshd_config`, строки `auth.log`, `motd`.
- `net/web.rs` — веб-страницы: сайт организации, статус провайдера, сайты хостинга, страницы
  скрытой сети `gate.hw`. HTML простой, на `/` — `h1` с именем организации.
- `net/mail.rs` — начальный ящик игрока (банк, бывший работодатель, провайдер, спам,
  письмо hello-world со ссылкой-приглашением) и рабочая почта сотрудников с зацепками.
- `net/gen.rs` — генератор: организации по подсетям, публичные/локальные адреса, карта,
  компьютер игрока и его почта, известные узлы. `generate(rng, handle, password) -> Generated`.
- `net/gen_machine.rs` — сборка одной машины (железо, учётки, службы, процессы, файлы, журнал, почта).
- `net/player.rs` + `net/player_data.rs` — компьютер игрока `<handle>-pc` (адрес `87.x.x.x`,
  `readme.txt`, `scripts/*.adr`, `wordlist.txt`, `/etc/hosts`, `/etc/motd`) и домашний маршрутизатор.
- `net/dns.rs` — `resolve` (hostname, домен, `www.`/`mail.`, особое `home.net`) и `reverse`.

Скрытая сеть hello-world: `OrgKind::Underground`, домен `hw`, шлюз `gate.hw`
(`/`, `/invite/<код>`, `/manifesto`, `/board`) и узлы `board.hw`, `drop.hw`, `relay.hw`.
Код приглашения детерминирован от зерна и совпадает со ссылкой в письме игрока.
Детерминизм: один `seed` — один мир; при зерне 7 — 109 машин и 21 организация.
Тесты генератора — `tests/net_gen.rs` (15).

## Проверка

Из корня репозитория:

```
cargo test -p blackwire-core --jobs 2
```

Тесты ядра — в `core/tests/`: `vfs.rs` (16), `parse.rs` (11), `shell.rs` (30), `world.rs` (27), `net_gen.rs` (15),
`scripts_parse.rs` (1 — готовые скрипты игрока разбираются языком Adder), `hack.rs` (8 — уязвимости, эксплойты, подбор,
повышение прав, следы, прокси), `watch.rs` (6 — слежка), `missions.rs` (7 — цепочка заданий: построение, проверка,
награда, ответ), `money.rs` (10 — долг, выплаты, магазин, эффекты, сбыт), `ending.rs` (5 — условие победы, финал, игра продолжается,
разовый доход с логина), `life.rs` (3 — живой мир), `gigs.rs` (4 — подработки), `mechanics.rs` (6 — ловушки, файрволы, шифр,
кейлоггер, ботнет), `counter.rs` (6 — ответный удар и защита), `market.rs` (4 — маркет скриптов), `base.rs` (5 — аренда релеев),
`achievements.rs` (6 — открытие по состоянию мира, без повтора, переживают сохранение),
`scenarios.rs` (5 — свои сценарии: цели, награда, сохранение); всего 175, без `#[ignore]`.

## Концовка (`src/ending.rs`)

Победа — пройдена вся цепочка заданий и закрыт кредит (`check` из `World::tick`): флаг `won`, последнее письмо группы,
событие `poll_ending` (оболочка авто-открывает окно финала). Игра продолжается и после. `summary` — сводка пути для окна
и команды `ending`. Экономика ограничена: `World::add_found_cred` даёт доход с каждого логина один раз (`Player.harvested`).

## Деньги и долг (`src/money/`, содержание)

`Finance` в `World` (кредит, ставка, кредитор — банк из сюжета, выплаты, `cleared`); `ensure_init` в `World::new` заводит
долг, `tick` набегают проценты. `pay` гасит долг (квитанция/освобождение письмом), `buy`/`sell` — магазин (`shop.rs`:
каталог `ITEMS`, `apply` для мгновенных эффектов; vpn/scrubber/rootkit — постоянные, читаются в `hack`). Команды —
`shell/cmds_money.rs` (группа «Деньги»), страница `gate.hw/shop` (`shop_html`), язык `sys.money`/`sys.debt`.

## Задания hello-world (`src/missions/`, содержание)

Цепочка из 7 миссий. `MissionState` в `World`; `missions::check` из `World::tick` строит цепочку (находит цели по ролям
через `resolve_target`), проверяет выполнение текущего задания (`goal_met`), платит награду (`Player.money`) и шлёт ответ
группы со следующим брифингом. Доска — `board_html` (узел `gate.hw/board` в `World::http`). Команды — `shell/cmds_jobs.rs`
(группа «Задания»), функции языка — `sys.money`/`sys.missions`. Содержание и голос группы — `missions/data.rs` (`CHAIN`).

## Взлом (`src/hack/`, второй этап)

Механики взлома над моделью игровой сети (договор — `docs/HACK.md`): `vuln` (классификация слабых мест по версии служб
и ОС), `exploit` (доступ к удалённой машине), `crack` (подбор пароля по словарю), `privesc` (повышение прав),
`tracks` (подозрение `heat`, след источника, чистка журналов; `decay` зовётся из `World::tick`), `proxy` (цепочка).
Команды — `shell/cmds_hack.rs` (группа «Взлом»), функции языка — `net.*` в `script_host.rs`. Состояние: `Player.owned`,
`Player.found_creds`, `Player.heat/heat_peak/traced`; таблица уязвимостей — `net::services::VULNS` (столбцы доступа и области).

## Слежка (`src/hack/watch.rs`, третий этап)

Расследование центров безопасности и киберполиции. `Investigation` в `World` (стадия, прогресс, ведущий орган, следимые
узлы, рейды, `locked_until`); `sweep` из `World::tick` двигает дело от улик (`tracks::evidence`) и подозрения, шлёт письма
игроку и пишет в `/var/log/alerts.log` органа; на пике — рейд (блокировка наступательных команд, `Game::poll_surveillance`
обрывает цепочки). Команда `trace`, функция `net.watch()`. Блокировка — `watch::is_locked`, в командах — `cmds_hack::locked_guard`.

## Связка со скриптами (`src/script_host.rs`)

`script_host::call(game, session, pid, "модуль.функция", args_json)` — все функции модулей `sys`, `fs`, `net`, `shell`
(и `time.now`) из `docs/ADDER.md`. Язык живёт в крейте `blackwire-lang` и ядра не знает: оболочка Tauri получает
`Host::call` от исполнителя, переводит значения в JSON и зовёт сюда под замком игры. `sys.open`/`sys.notify` и окна из
`shell.exec` складываются в `Game::pending_ui`, откуда их забирает оболочка.
