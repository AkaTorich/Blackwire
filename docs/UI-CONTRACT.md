# Интерфейс (папка `app/ui`) — договорённости

Чистый HTML/CSS/JS, без Node и сборщиков. Tauri 2, `withGlobalTauri: true` → `window.__TAURI__`.
Каждое приложение игры — **отдельное настоящее окно** Windows (своя кнопка на панели задач, сворачивание, разворачивание,
перетаскивание на любой монитор). Все окна грузят один `index.html?app=<id>&arg=<...>&win=<label>`; `js/main.js` читает `app`
и запускает нужный модуль из `js/apps/<id>.js`. Состояние мира — только в Rust; окна — виды на него, узнают о переменах по событиям.

## Окна

Все окна без системной рамки (`decorations: false`, `transparent: true`, `shadow: true`). Рамку рисуем сами:
```
<div class="win">
  <header class="win-title" data-tauri-drag-region>   <!-- перетаскивание -->
    <span class="win-icon">…svg…</span><span class="win-name">Терминал — ghost@ghost-pc</span>
    <span class="win-btns"><button class="min">—</button><button class="max">▢</button><button class="close">✕</button></span>
  </header>
  <main class="win-body">…приложение…</main>
</div>
```
Кнопки: `getCurrentWindow().minimize() / toggleMaximize() / close()`. Изменение размера за края — делает Tauri (окно `resizable`).
Двойной щелчок по заголовку — развернуть. Клавиши: F11 — во весь экран, Esc — выйти из полного экрана, Ctrl+W — закрыть окно,
Ctrl+Shift+T — новый терминал, Ctrl+Shift+E — редактор, Ctrl+Shift+F — файлы.

Виды окон (`app`): `desktop` (подложка-рабочий стол), `terminal`, `editor`, `files`, `browser`, `monitor`, `map`, `settings`, `help`,
`login` (экран входа при запуске), `notify` — не окно, уведомления рисует desktop.

`desktop` — окно во весь экран выбранного монитора, **всегда позади остальных** (`set_always_on_bottom(true)`), без кнопок закрытия:
фон-анимация (canvas: медленно вращающиеся проволочные многогранники, поток шестнадцатеричных цифр, редкие вспышки «пакетов»,
тонкие сканлайны — каждый эффект отключаем в настройках), сверху полоса состояния (имя системы и машины, время мира, CPU/память/сеть
маленькими графиками), снизу по центру **док** со светящимися значками приложений и списком открытых окон (щелчок — поднять окно,
`focus_window`), правый край — уведомления. Правый щелчок по фону — меню: новый терминал, редактор, файлы, разложить окна, настройки,
сменить монитор, выйти.

## Мост в Rust (`js/bridge.js` — обёртки над `invoke`)

```
boot()                      -> { has_save: bool, handle: string|null, os_name, version, settings, theme_list: [{id, name, accent}] }
new_game(handle, seed?)     -> BootInfo           // новая личность
load_game()                 -> BootInfo
save_game()                 -> ok
login(password)             -> bool               // пароль своей учётки (пустой — без пароля)

open_app(app, arg?, monitor?) -> label            // создать окно (Rust считает окна, задаёт позицию каскадом на нужном мониторе)
focus_window(label), close_window(label), list_windows() -> [{label, app, title, minimized}]
monitors()                  -> [{index, name, x, y, width, height, scale, primary}]
arrange_windows(monitor, layout)   // layout: "grid" | "columns" | "rows" | "cascade"
desktop_on(monitor, on: bool)      // подложка на мониторе

shell_open()                -> session_id
shell_close(session_id)
shell_exec(session_id, line)      -> ShellResult   { lines:[{text, kind, delay_ms}], actions:[UiAction], prompt }
shell_complete(session_id, line)  -> [string]
shell_prompt(session_id)          -> Prompt { user, host, cwd, is_root, hops }
shell_history(session_id)         -> [string]

script_run(session_id, path, args[]) -> pid        // события: script_out {pid, text}, script_done {pid, ok, error, line}
script_input(pid, text)           // ответ на input()
script_kill(pid)

fs_list(session_id|null, path)    -> [FsEntry {name, path, is_dir, size, modified, owner, mode}]
fs_read(session_id|null, path)    -> string
fs_write(session_id|null, path, text)
fs_mkdir / fs_delete / fs_rename(from, to) / fs_copy
lang_check(src)                   -> [Diagnostic {line, col, message}]
lang_doc()                        -> markdown (docs/ADDER.md вшит в exe), shell_doc() -> markdown команд

sys_status()                      -> { hostname, ip, uptime, cpu, mem_used, mem_total, net_in, net_out, procs:[{pid,name,user,cpu,mem}] }
sys_kill(pid)
net_map()                         -> { nodes:[{id, hostname, ip, kind, known, x, y}], links:[{a, b}] }
web_get(url)                      -> { status, title, body }   // игровая сеть, html-строка для браузера
settings_get() -> Settings ; settings_set(Settings)   // { theme, fx:{wireframe,hexrain,scanlines,glass}, font_size, terminal_opacity, sound, default_monitor }

mail_list(folder)                 -> [MailMessage {id, from, to, subject, body, time, read, folder}]   // folder: "inbox" | "sent"
mail_read(id)                     -> MailMessage (помечает прочитанным)
mail_send(to, subject, body)      // ошибка строкой, если адрес не найден
mail_delete(id) ; mail_unread_count() -> n
start_desktop()                   // после входа: подложка + первый терминал, окно входа закрывается
quit()                            // сохранить и выйти
```
События почты: `mail_new {count}` (пришло новое), `mail_changed {}` (отправлено/удалено — обновить списки).
Приложение `mail` — «Почта» (см. раздел «Приложения»). Легенда и начальные письма — `docs/STORY.md`.

События (`listen`): `theme_changed {settings}`, `script_out`, `script_done`, `script_input_wanted {pid, prompt}`, `notify {title, text}`,
`windows_changed {windows}`, `world_tick {time, cpu, mem, net_in, net_out}` (раз в секунду), `fs_changed {path}` (чтобы файлы/редактор обновились),
`open_app {app, arg}` (ядро просит открыть окно — например `edit файл` из терминала; обрабатывает Rust сам, окнам только для сведения).

## Вид (стиль «Рыба-меч»)

Тёмная глубина (почти чёрный с синевой), холодный циан/электрик как акцент, тонкие светящиеся линии в 1 px, стекло
(полупрозрачные панели с размытием — `backdrop-filter`, отключаемо), моноширинный шрифт для данных (Cascadia Mono / Consolas),
Segoe UI для надписей. Анимации: появление окна (масштаб 0.96→1 + проявление 180 мс), мерцание курсора, «печатание» строк терминала с
задержками `delay_ms`, бегущие огни на графиках. Никаких картинок из интернета — только CSS, SVG и canvas. Все значки — свои SVG
(`js/icons.js`), один стиль линий.

Темы — CSS-переменные на `:root[data-theme=...]` в `css/themes.css`; переключение на лету во всех окнах (событие `theme_changed`),
выбранная тема хранится в настройках (Rust) и дублируется в `localStorage`, чтобы окно открывалось сразу в нужных цветах.
Обязательный набор тем (не меньше восьми): `swordfish` (циан на чёрно-синем, по умолчанию), `ice` (бело-голубой), `amber` (янтарь на
графите), `phosphor` (зелёный терминал), `violet`, `crimson`, `paper` (светлая, чернила на бумаге), `midnight` (глубокий синий без неона),
`steel`, `gold`. Переменные: `--bg --bg2 --panel --panel-border --fg --fg-dim --accent --accent2 --glow --ok --warn --err --mono --sans
--radius`.

Шрифты и размеры масштабируются от `settings.font_size`. Каждое окно обязано хорошо выглядеть в трёх размерах: минимальном
(терминал 520×320, редактор 640×400, остальные по смыслу), среднем (1000×650) и большом (во весь экран 1920×1080 и 3840×2160 при масштабе 150 %).

## Приложения

- **Терминал** (`apps/terminal.js`): свой вывод (строки по видам `kind`), строка ввода с мигающим курсором, история ↑/↓, Tab — подсказка
  (`shell_complete`, общий префикс и список вариантов), Ctrl+L, Ctrl+C (прервать скрипт / очистить ввод), выделение и копирование мышью,
  вставка Ctrl+V, прокрутка, печать с паузами `delay_ms`, события скриптов по pid этого сеанса, ответ на `input()` прямо в строке ввода,
  заголовок окна = `user@host: cwd`. Команда `exit` дома закрывает окно.
- **Редактор** (`apps/editor.js`): вкладки файлов, номера строк, подсветка Adder (ключевые слова, строки, числа, комментарии, f-строки,
  имена модулей `sys fs net shell math json time random`), подсветка `.txt/.log/.conf` по минимуму, автоотступ по Enter, Tab = 4 пробела,
  Ctrl+S сохранить, F5 запустить (открывает терминал с `adder файл`), проверка ошибок на лету (`lang_check` с задержкой 400 мс, отметка в
  поле номеров и строка состояния), поиск Ctrl+F, Ctrl+G к строке, «изменено» в заголовке. Реализация — `textarea` поверх подсвеченного
  `pre` (синхронная прокрутка), без сторонних библиотек.
- **Файлы** (`apps/files.js`): дерево слева, список/плитка справа, путь-«крошки», двойной щелчок — папка или открыть в редакторе, кнопки
  новая папка/файл, переименовать (F2), удалить (Del, с подтверждением), копировать/вставить, свойства (размер, владелец, права, время),
  скрытые файлы по Ctrl+H, адресная строка, обновление по `fs_changed`.
- **Браузер** (`apps/browser.js`): адресная строка, назад/вперёд/обновить, страницы из `web_get` рисуются в `iframe srcdoc` с нашим
  стилем, ссылки внутри перехватываются и идут снова через `web_get`, стартовая страница `home://` (сводка системы и ссылки), ошибки
  («нет такого узла», «порт закрыт») — своя страница.
- **Монитор** (`apps/monitor.js`): графики CPU/память/сеть на canvas (история 120 с, неоновые линии со свечением), таблица процессов с
  сортировкой и кнопкой «снять», карточка машины.
- **Карта сети** (`apps/map.js`): узлы и связи на canvas, известные — ярко, остальные — тускло/не показаны, бегущие импульсы по связям,
  колесо — масштаб, перетаскивание — панорама, щелчок по узлу — панель (имя, ip, вид, службы если сканировали), кнопка «терминал → ssh».
- **Настройки** (`apps/settings.js`): темы плитками с живым предпросмотром, выключатели эффектов, размер шрифта, прозрачность
  терминала, звук (короткие синтезированные щелчки WebAudio, без файлов), монитор по умолчанию, «разложить окна», сохранение/загрузка игры,
  новая личность (с подтверждением), о программе.
- **Справка** (`apps/help.js`): слева оглавление, справа Markdown (`lang_doc`, `shell_doc`) → HTML своим мини-парсером (заголовки, списки,
  таблицы, код), поиск по тексту.
- **Вход** (`apps/login.js`): при запуске — короткая «загрузка системы» (бегущие строки журнала, полоса), затем поле пароля на фоне
  анимации; первый запуск — «новая личность»: выбрать имя (handle) и пароль, кнопка «Войти» строит мир (`new_game`) и открывает desktop +
  терминал.
