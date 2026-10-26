# Blackwire — интерфейс (`app/ui`)

Чистый HTML/CSS/JS без Node, сборщиков и библиотек. Договор с ядром и вторым помощником — `docs/UI-CONTRACT.md`.

## Что где лежит

| Файл | Назначение |
|---|---|
| `index.html` | Единственная страница для всех окон. Inline-скрипт в `<head>` читает `localStorage.bw_theme`, `bw_glass`, `bw_font` и выставляет `data-theme`, `.no-glass`, `--font-size` до загрузки стилей; `data-app` — имя приложения из `?app=`. Подключает все стили и скрипты, включая `js/apps/*` второго помощника (terminal, editor_highlight + editor, files, browser, mail, monitor, map). |
| `js/bridge.js` | Глобальный `BW`: `app`, `arg`, `label` из адреса; `invoke`, `listen`, `emit`; `settings`, `ready`, `onSettings`, `saveSettings`; `sound(name)` — синтез WebAudio (`click open close error key notify`); обёртка для каждой команды контракта (`BW.shell_exec(sessionId, line)` и т. д., ключи camelCase), почта `mail_list/mail_read/mail_send/mail_delete/mail_unread_count`. Без `window.__TAURI__` — заглушка с правдоподобными данными, тиками `world_tick` и одним уведомлением. |
| `js/icons.js` | `Icons.get(name, size)` — SVG-значки одним стилем (линии 1.6, `currentColor`, viewBox 24). Неизвестное имя — пустой кружок. `Icons.has`, `Icons.names`. |
| `js/win.js` | `Win.init({title, icon, minWidth, minHeight})` — рамка окна по разметке контракта, возвращает `.win-body`; кнопки окна через `getCurrentWindow()`; двойной щелчок по заголовку; клавиши F11 / Esc / Ctrl+W и глобальные Ctrl+Shift+T/E/F/M (`Win.bindKeys`); `setTitle`, `setModified`, `close`, `confirm`, `prompt`, `toast`, `menu(items, x, y)` с подменю (`children`), `closeMenus`, `current`. |
| `js/theme.js` | `Theme.apply(settings)` — `data-theme`, `--font-size`, `--term-opacity`, класс `no-glass`, дублирование в `localStorage`; `Theme.color('--accent')` → `{r,g,b}`, `Theme.rgba`. Слушает `BW.onSettings`. |
| `js/markdown.js` | `Markdown.render(text)` → HTML (заголовки с якорями, абзацы, списки с вложенностью, таблицы, код, цитаты, жирный/курсив, ссылки, HTML экранируется); `Markdown.headings(text)` — оглавление с теми же якорями. |
| `js/fx.js` | `FX.start(canvas, settings)` / `update` / `stop` / `pause` / `resume`. Проволочные куб, икосаэдр, додекаэдр с перспективой и свечением, «пакеты» по рёбрам, поток шестнадцатеричных цифр, сканлайны и бегущая полоса, радиальная глубина. Учитывает `devicePixelRatio`, `ResizeObserver`, пауза при скрытой вкладке, не чаще ~60 кадров. |
| `js/main.js` | Реестр `Apps.register(id, {title, icon, minWidth, minHeight, mount})`, `Apps.openOrFocus(app, arg)`. На `DOMContentLoaded` после `BW.ready` запускает приложение из `?app=`: обычным — рамка `Win.init`, `desktop`/`login` — прямо в `#root`. Неизвестное — страница «Неизвестное приложение». Гасит системное контекстное меню. |
| `js/apps/desktop.js` + `css/desktop.css` | Рабочий стол: canvas FX, полоса состояния (имя системы, `user@host`, часы из `world_tick`, графики CPU/память/сеть), док (приложения, разделитель, открытые окна из `list_windows`/`windows_changed`, точка непрочитанных у почты по `mail_unread_count`/`mail_new`), уведомления (`notify`, 6 с), контекстное меню фона (терминал, редактор, файлы, разложить окна → раскладки, настройки, сменить монитор подложки → мониторы, выйти). |
| `js/apps/login.js` + `css/login.css` | Вход: журнал загрузки, полоса, логотип с глитчем; затем карточка — пароль (`login`) или «Новая личность» (`new_game(handle, seed, password)`); после успеха `start_desktop` и `Win.close()`. |
| `js/apps/settings.js` + `css/settings.css` | Настройки: плитки тем с предпросмотром (`.theme-preview[data-theme]`), выключатели эффектов, ползунки шрифта и прозрачности, звук, монитор по умолчанию, раскладка окон, сохранить/загрузить/новая личность, о программе. Сохранение через `BW.saveSettings` → `settings_set`. |
| `js/apps/help.js` + `css/help.css` | Справка: две книги (`lang_doc`, `shell_doc`), оглавление, якоря, подсветка текущего раздела, поиск с подсветкой и переходами (Enter / Shift+Enter, Ctrl+F). |
| `css/base.css` | Сброс, шрифты, кнопки (`.btn .btn-primary .btn-danger .btn-icon`), поля (`.input`), переключатель (`.toggle`), ползунок (`.slider`), таблица (`.table`), тонкая полоса прокрутки, заслонка и диалог, меню, тосты, анимации. |
| `css/themes.css` | 12 тем: swordfish, ice, amber, phosphor, violet, crimson, paper, midnight, steel, gold, aurora, ember. |
| `css/win.css` | Рамка окна: прозрачные `html, body`, стекло `.win`, заголовок 36 px, кнопки, общие `.toolbar` и `.statusbar`. |

## Как устроены окна

Каждое окно Tauri грузит `index.html?app=<id>&arg=<...>&win=<label>`. `main.js` находит приложение в реестре `Apps` и
вызывает `Win.init` (рамка: `.win > header.win-title[data-tauri-drag-region] + main.win-body`), затем `mount(body, arg)`.
Окна `desktop` и `login` рамки не получают и рисуют всё сами. Состояние живёт в Rust: окна зовут команды через `BW.*`
и слушают события `BW.listen(...)`. Диалоги, меню и тосты (`Win.confirm/prompt/menu/toast`) — свои, в стиле темы.

## Как устроены темы

Тема — набор CSS-переменных на `:root[data-theme="..."]` (и `.theme-preview[data-theme]` для плиток): `--bg --bg2 --panel
--panel-border --fg --fg-dim --accent --accent2 --glow --ok --warn --err --sel --shadow --term-bg --mono --sans --radius`,
плюс `--accent-rgb --fg-rgb --bg-rgb` для полупрозрачных оттенков через `rgba(var(--accent-rgb), .12)`. Смена темы:
настройки → `settings_set` → Rust рассылает `theme_changed` → `theme.js` ставит `data-theme` во всех окнах и пишет
`localStorage.bw_theme`, чтобы новое окно открывалось сразу в нужных цветах. Размер шрифта — `--font-size` на `html`,
стекло — класс `no-glass`, плотность фона терминала — `--term-opacity` (используется в `--term-bg`).

## Договорённости кода

Обычные `<script>` без модулей, `'use strict'`, один глобальный объект на файл, идентификаторы английские, комментарии
по-русски, без inline-стилей в JS кроме вычисляемых (ширина полосы, позиция меню, `--fill` ползунка).
