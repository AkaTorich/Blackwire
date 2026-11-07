//! Справка: подробные тексты `--help`/`man`, таблица `help`, Markdown для окна справки.

use super::table::{by_group, CmdInfo, Group, COMMANDS};
use super::util::table;
use super::CmdOutput;
use crate::types::OutKind;

/// Подробное пояснение к команде (если нет — берётся краткое из таблицы).
pub fn long_help(name: &str) -> Option<&'static str> {
    let text = match name {
        "ls" => tr!("Показывает содержимое папки (без аргумента — текущей).\n  -l  подробно: права, владелец, размер, время\n  -a  вместе со скрытыми файлами (имена с точки)\n  -h  размеры по-человечески", "Shows the contents of a directory (no argument — the current one).\n  -l  long format: permissions, owner, size, time\n  -a  include hidden files (names starting with a dot)\n  -h  human-readable sizes"),
        "cd" => tr!("Меняет текущую папку. Без аргумента ведёт домой, «-» — в предыдущую, «..» — на уровень выше.", "Changes the current directory. No argument goes home, '-' to the previous one, '..' one level up."),
        "cat" => tr!("Печатает файлы подряд. С -n нумерует строки. Без файлов берёт вход конвейера.", "Prints files one after another. With -n numbers the lines. With no files reads the pipeline input."),
        "mkdir" => tr!("Создаёт папку. С -p создаёт и недостающие папки по пути.", "Creates a directory. With -p also creates missing directories along the path."),
        "rm" => tr!("Удаляет файлы. Папки — только с -r. -f молчит, если файла нет. Удаление пишется в журнал.", "Removes files. Directories — only with -r. -f stays quiet if the file is missing. Deletions are logged."),
        "cp" => tr!("Копирует файлы. Папки — только с -r.", "Copies files. Directories — only with -r."),
        "echo" => tr!("Печатает аргументы через пробел. -e раскрывает \\n и \\t. Переменные вида $USER, ${HOME}.", "Prints the arguments separated by spaces. -e expands \\n and \\t. Variables like $USER, ${HOME}."),
        "grep" => tr!("Ищет строки по подстановке (или по шаблону с *). -i без учёта регистра, -n номера, -v наоборот, -c счёт, -r по дереву.", "Searches lines by substring (or by a pattern with *). -i ignore case, -n line numbers, -v invert, -c count, -r recursive."),
        "find" => tr!("Ищет файлы под путём. -name шаблон (можно * и ?), -type f или d.", "Searches for files under a path. -name pattern (* and ? allowed), -type f or d."),
        "chmod" => tr!("Меняет права: число (755), символы (u+x, go-w, a=r) или rwxr-xr-x. Меняет владелец или root.", "Changes permissions: a number (755), symbols (u+x, go-w, a=r) or rwxr-xr-x. Only the owner or root can do it."),
        "chown" => tr!("Меняет владельца файла. Только root.", "Changes a file's owner. Root only."),
        "ps" => tr!("Список процессов. С -a — процессы всех пользователей.", "Process list. With -a — processes of all users."),
        "kill" => tr!("Снимает процесс по pid. Скрипту Adder велит остановиться.", "Kills a process by pid. Tells an Adder script to stop."),
        "su" => tr!("Меняет пользователя: su пользователь пароль (без имени — root). Пишет вход в журнал.", "Switches user: su user password (no name — root). The login is logged."),
        "sudo" => tr!("Выполняет команду от root. Доступно администраторам.", "Runs a command as root. Available to administrators."),
        "passwd" => tr!("Меняет пароль: passwd новый_пароль (свой) или passwd пользователь пароль (root).", "Changes a password: passwd new_password (your own) or passwd user password (root)."),
        "useradd" => tr!("Добавляет пользователя: useradd имя пароль. С -a — администратор. Нужны права администратора.", "Adds a user: useradd name password. With -a — administrator. Requires administrator rights."),
        "ssh" => tr!("Подключается к машине: ssh пользователь@адрес пароль. Добавляет шаг в цепочку, exit снимает.\nУспех и неудача пишутся в журнал той машины.", "Connects to a machine: ssh user@address password. Adds a hop to the chain, exit removes it.\nSuccess and failure are written to that machine's log."),
        "scp" => tr!("Копирует файл между машинами: scp [user@ip:]откуда [user@ip:]куда пароль.", "Copies a file between machines: scp [user@ip:]src [user@ip:]dest password."),
        "research" => tr!("Исследует конкретную уязвимость: research адрес id. id показывает probe. -h 1..4 раскрывает подсказки. Показывает поля виртуального payload и совместимый товар маркета.", "Researches a specific vulnerability: research address id. probe shows the id. -h 1..4 reveals hints. Shows the virtual payload fields and the compatible market item."),
        "exploit" => tr!("Одна команда больше не выдаёт готовый взлом. Сначала probe адрес, затем research адрес id. Создай файл: exploit new ~/scripts/my_exploit.adr адрес id. Редактор откроется автоматически. Заполни payload, проверь net.probe и примени net.exploit. Запуск файла: exploit -f ~/scripts/my_exploit.adr [-s id] адрес. Готовый редактируемый код: market buy exp_<механика>. Неверная версия, firewall, маршрут или payload не дают доступа.", "A single command no longer hands you a ready-made hack. First probe address, then research address id. Create a file: exploit new ~/scripts/my_exploit.adr address id. The editor opens automatically. Fill in the payload, check net.probe and apply net.exploit. Run the file: exploit -f ~/scripts/my_exploit.adr [-s id] address. Ready editable code: market buy exp_<mechanic>. A wrong version, firewall, route or payload gives no access."),
        "privesc" => tr!("Для повышения прав нужен собственный или купленный локальный эксплоит: privesc ~/scripts/exp_race.adr id. Сначала войди на машину, выполни probe текущий_хост и research текущий_хост id. Сам скрипт вызывает net.privesc(payload, id).", "Privilege escalation needs your own or a purchased local exploit: privesc ~/scripts/exp_race.adr id. First log in to the machine, run probe current_host and research current_host id. The script itself calls net.privesc(payload, id)."),
        "scan" => tr!("Показывает открытые порты, службы и версии машины (строки идут с задержкой). То же — nmap.", "Shows the machine's open ports, services and versions (lines arrive with a delay). Same as nmap."),
        "ping" => tr!("Проверяет связь: 4 пакета с задержками. Выключенная или недоступная машина не отвечает.", "Checks connectivity: 4 packets with latencies. A machine that is off or unreachable does not reply."),
        "logs" => tr!("Журнал текущей машины: вход, неверный пароль, запуск, изменения. -n сколько строк, -f auth|system.", "Log of the current machine: logins, wrong passwords, launches, changes. -n number of lines, -f auth|system."),
        "adder" => tr!("Запускает скрипт Adder процессом: adder файл.adr [аргументы]. Также adder -c \"код\" и adder --check файл.", "Runs an Adder script as a process: adder file.adr [args]. Also adder -c \"code\" and adder --check file."),
        "wget" => tr!("Скачивает веб-страницу игровой сети в файл. -O имя задаёт имя файла.", "Downloads a web page from the game network to a file. -O name sets the file name."),
        "tar" => tr!("Архив как папка в одном файле: tar -c архив.tar файлы… пакует, -x распаковывает, -t показывает.", "An archive is a directory in a single file: tar -c archive.tar files… packs, -x unpacks, -t lists."),
        "mail" => tr!("Открывает окно почты. С -l печатает входящие письма списком.", "Opens the mail window. With -l prints the Inbox as a list."),
        "bc" => tr!("Калькулятор: + - * / % ^, скобки, sqrt/abs/floor/ceil/round/ln/log, pi, e.", "Calculator: + - * / % ^, parentheses, sqrt/abs/floor/ceil/round/ln/log, pi, e."),
        "banner" => tr!("Рисует текст крупными буквами. Кириллица переводится в латиницу.", "Draws text in big letters. Cyrillic is transliterated to Latin."),
        "neofetch" => tr!("Карточка машины с ASCII-логотипом: система, процессор, память, адрес.", "Machine card with an ASCII logo: system, CPU, memory, address."),
        _ => return None,
    };
    Some(text)
}

/// Ответ на `команда --help`, `help команда`, `man команда`.
pub fn command_help(info: &CmdInfo) -> CmdOutput {
    let mut out = CmdOutput::ok();
    out.push_kind(info.name.to_string(), OutKind::Heading);
    out.push_kind(trf!("Использование: {}", "Usage: {}", info.usage_text()), OutKind::Accent);
    out.push("");
    out.push(info.short_text().to_string());
    if let Some(long) = long_help(info.name) {
        out.push("");
        out.push(long.to_string());
    }
    out
}

/// Таблица команд для `help` без аргументов (по группам).
pub fn help_table() -> Vec<String> {
    let mut lines = Vec::new();
    lines.push(tr!("Команды Blackwire. Подробнее: help <команда> или <команда> --help.", "Blackwire commands. More: help <command> or <command> --help.").to_string());
    for group in Group::all() {
        let cmds = by_group(group);
        if cmds.is_empty() {
            continue;
        }
        lines.push(format!("== {}", group.title()));
        let rows: Vec<Vec<String>> = cmds.iter().map(|c| vec![c.name.to_string(), c.short_text().to_string()]).collect();
        lines.extend(table(&rows, &[false, false]).into_iter().map(|l| format!("  {}", l)));
    }
    lines.push(format!("== {}", tr!("Оболочка", "Shell")));
    lines.push(tr!("  Конвейер a | b, перенаправления > >> 2> <, разделители ; && ||", "  Pipeline a | b, redirections > >> 2> <, separators ; && ||").to_string());
    lines.push(tr!("  Переменные $ИМЯ, ~ — домашняя папка, * и ? в именах файлов", "  Variables $NAME, ~ — home directory, * and ? in file names").to_string());
    lines
}

/// Markdown всех команд — для окна «Справка».
pub fn shell_doc() -> String {
    let mut s = String::from(tr!("# Команды терминала Blackwire\n\n", "# Blackwire terminal commands\n\n"));
    s.push_str(tr!("Оболочка понимает конвейеры (`a | b`), перенаправления (`>`, `>>`, `2>`, `<`), ", "The shell understands pipelines (`a | b`), redirections (`>`, `>>`, `2>`, `<`), "));
    s.push_str(tr!("разделители (`;`, `&&`, `||`), переменные (`$ИМЯ`, `${ИМЯ}`), домашнюю папку (`~`) ", "separators (`;`, `&&`, `||`), variables (`$NAME`, `${NAME}`), the home directory (`~`) "));
    s.push_str(tr!("и подстановку `*`/`?` в именах файлов. У каждой команды есть `--help`.\n\n", "and `*`/`?` wildcards in file names. Every command has `--help`.\n\n"));
    for group in Group::all() {
        let cmds = by_group(group);
        if cmds.is_empty() {
            continue;
        }
        s.push_str(&format!("## {}\n\n", group.title()));
        s.push_str(tr!("| Команда | Использование | Назначение |\n|---|---|---|\n", "| Command | Usage | Purpose |\n|---|---|---|\n"));
        for c in cmds {
            s.push_str(&format!("| `{}` | `{}` | {} |\n", c.name, c.usage_text(), c.short_text()));
        }
        s.push('\n');
    }
    s.push_str(&trf!("\nВсего команд: {}.\n", "\nTotal commands: {}.\n", COMMANDS.len()));
    s
}
