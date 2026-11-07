//! Список всех команд оболочки: имя, группа, строка использования, краткое
//! описание и функция. По этой таблице работают `help`, `man`, Tab и `shell_doc`.

use super::{cmds_base, cmds_defense, cmds_fs, cmds_hack, cmds_jobs, cmds_misc, cmds_money, cmds_net, cmds_sys, cmds_text, CmdOutput, Ctx};

pub type CmdFn = fn(&mut Ctx<'_>, &[String]) -> CmdOutput;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Group {
    Files,
    Text,
    System,
    Net,
    Hack,
    Defense,
    Base,
    Jobs,
    Money,
    Adder,
    Apps,
    Misc,
}

impl Group {
    pub fn title(self) -> &'static str {
        match self {
            Group::Files => tr!("Файлы и каталоги", "Files and directories"),
            Group::Text => tr!("Текст и архивы", "Text and archives"),
            Group::System => tr!("Система и пользователи", "System and users"),
            Group::Net => tr!("Сеть", "Network"),
            Group::Hack => tr!("Взлом", "Hacking"),
            Group::Defense => tr!("Защита", "Defense"),
            Group::Base => tr!("База", "Base"),
            Group::Jobs => tr!("Задания", "Missions"),
            Group::Money => tr!("Деньги", "Money"),
            Group::Adder => tr!("Скрипты Adder", "Adder scripts"),
            Group::Apps => tr!("Окна приложений", "App windows"),
            Group::Misc => tr!("Прочее", "Misc"),
        }
    }

    pub fn all() -> [Group; 12] {
        [Group::Files, Group::Text, Group::System, Group::Net, Group::Hack, Group::Defense, Group::Base, Group::Jobs, Group::Money, Group::Adder, Group::Apps, Group::Misc]
    }
}

pub struct CmdInfo {
    pub name: &'static str,
    pub group: Group,
    pub usage: &'static str,
    pub short: &'static str,
    pub usage_en: &'static str,
    pub short_en: &'static str,
    pub run: CmdFn,
}

impl CmdInfo {
    /// Строка использования на выбранном языке.
    pub fn usage_text(&self) -> &'static str {
        tr!(self.usage, self.usage_en)
    }

    /// Краткое описание на выбранном языке.
    pub fn short_text(&self) -> &'static str {
        tr!(self.short, self.short_en)
    }
}

macro_rules! cmd {
    ($name:expr, $group:ident, $usage:expr, $short:expr, $usage_en:expr, $short_en:expr, $f:path) => {
        CmdInfo { name: $name, group: Group::$group, usage: $usage, short: $short, usage_en: $usage_en, short_en: $short_en, run: $f }
    };
}

pub static COMMANDS: &[CmdInfo] = &[
    // ---- файлы
    cmd!("ls", Files, "ls [-l] [-a] [-h] [путь…]", "список файлов", "ls [-l] [-a] [-h] [path…]", "list files", cmds_fs::ls),
    cmd!("cd", Files, "cd [путь]", "сменить папку (без аргумента — домой, «-» — назад)", "cd [path]", "change directory (no argument — home, '-' — back)", cmds_fs::cd),
    cmd!("pwd", Files, "pwd", "текущая папка", "pwd", "current directory", cmds_fs::pwd),
    cmd!("cat", Files, "cat [-n] [файл…]", "вывести файл", "cat [-n] [file…]", "print a file", cmds_fs::cat),
    cmd!("less", Files, "less файл", "вывести файл (то же, что cat)", "less file", "print a file (same as cat)", cmds_fs::cat),
    cmd!("more", Files, "more файл", "вывести файл (то же, что cat)", "more file", "print a file (same as cat)", cmds_fs::cat),
    cmd!("touch", Files, "touch файл…", "создать пустой файл или обновить время", "touch file…", "create an empty file or update its time", cmds_fs::touch),
    cmd!("mkdir", Files, "mkdir [-p] папка…", "создать папку", "mkdir [-p] dir…", "create a directory", cmds_fs::mkdir),
    cmd!("rm", Files, "rm [-r] [-f] путь…", "удалить файлы и папки", "rm [-r] [-f] path…", "remove files and directories", cmds_fs::rm),
    cmd!("rmdir", Files, "rmdir папка…", "удалить пустую папку", "rmdir dir…", "remove an empty directory", cmds_fs::rmdir),
    cmd!("mv", Files, "mv откуда… куда", "переместить или переименовать", "mv src… dest", "move or rename", cmds_fs::mv),
    cmd!("cp", Files, "cp [-r] откуда… куда", "скопировать", "cp [-r] src… dest", "copy", cmds_fs::cp),
    cmd!("echo", Files, "echo [-n] [-e] текст…", "напечатать текст", "echo [-n] [-e] text…", "print text", cmds_fs::echo),
    cmd!("printf", Files, "printf формат [аргументы…]", "напечатать текст (как echo -e)", "printf format [args…]", "print text (like echo -e)", cmds_fs::echo),
    cmd!("find", Files, "find [путь] [-name шаблон] [-type f|d]", "искать файлы", "find [path] [-name pattern] [-type f|d]", "search for files", cmds_fs::find),
    cmd!("du", Files, "du [-h] [-s] [путь]", "размер папок", "du [-h] [-s] [path]", "directory sizes", cmds_fs::du),
    cmd!("df", Files, "df [-h]", "место на диске", "df [-h]", "disk space", cmds_fs::df),
    cmd!("chmod", Files, "chmod режим файл…", "сменить права (755, u+x, rwxr-xr-x)", "chmod mode file…", "change permissions (755, u+x, rwxr-xr-x)", cmds_fs::chmod),
    cmd!("chown", Files, "chown пользователь файл…", "сменить владельца (root)", "chown user file…", "change owner (root)", cmds_fs::chown),
    cmd!("file", Files, "file путь…", "определить вид файла", "file path…", "identify file type", cmds_fs::file),
    cmd!("stat", Files, "stat путь…", "сведения о файле", "stat path…", "file details", cmds_fs::stat),
    cmd!("tree", Files, "tree [-a] [-L глубина] [путь]", "дерево папок", "tree [-a] [-L depth] [path]", "directory tree", cmds_fs::tree),
    cmd!("ln", Files, "ln [-s] цель имя", "создать копию под другим именем", "ln [-s] target name", "create a copy under another name", cmds_fs::ln),
    // ---- текст и архивы
    cmd!("head", Text, "head [-n N] [файл…]", "первые строки", "head [-n N] [file…]", "first lines", cmds_text::head),
    cmd!("tail", Text, "tail [-n N] [файл…]", "последние строки", "tail [-n N] [file…]", "last lines", cmds_text::tail),
    cmd!("grep", Text, "grep [-i] [-n] [-v] [-c] [-r] шаблон [файл…]", "искать строки", "grep [-i] [-n] [-v] [-c] [-r] pattern [file…]", "search for lines", cmds_text::grep),
    cmd!("wc", Text, "wc [-l] [-w] [-c] [файл…]", "посчитать строки, слова, байты", "wc [-l] [-w] [-c] [file…]", "count lines, words, bytes", cmds_text::wc),
    cmd!("sort", Text, "sort [-r] [-n] [-u] [файл…]", "отсортировать строки", "sort [-r] [-n] [-u] [file…]", "sort lines", cmds_text::sort),
    cmd!("uniq", Text, "uniq [-c] [файл]", "убрать повторы соседних строк", "uniq [-c] [file]", "drop repeated adjacent lines", cmds_text::uniq),
    cmd!("cut", Text, "cut -d разделитель -f номера [файл…]", "вырезать поля", "cut -d delim -f fields [file…]", "cut out fields", cmds_text::cut),
    cmd!("tr", Text, "tr [-d] набор1 [набор2]", "заменить символы", "tr [-d] set1 [set2]", "translate characters", cmds_text::tr),
    cmd!("rev", Text, "rev [файл…]", "перевернуть строки", "rev [file…]", "reverse lines", cmds_text::rev),
    cmd!("tee", Text, "tee [-a] файл", "вывести и записать в файл", "tee [-a] file", "print and write to a file", cmds_text::tee),
    cmd!("diff", Text, "diff файл1 файл2", "различия двух файлов", "diff file1 file2", "differences between two files", cmds_text::diff),
    cmd!("md5", Text, "md5 [-s строка] [файл…]", "условный хеш", "md5 [-s string] [file…]", "simplified hash", cmds_text::md5),
    cmd!("md5sum", Text, "md5sum [файл…]", "условный хеш", "md5sum [file…]", "simplified hash", cmds_text::md5),
    cmd!("base64", Text, "base64 [-d] [файл]", "кодировать/раскодировать base64", "base64 [-d] [file]", "encode/decode base64", cmds_text::base64),
    cmd!("hexdump", Text, "hexdump [-n байт] файл", "байты файла в шестнадцатеричном виде", "hexdump [-n bytes] file", "file bytes in hex", cmds_text::hexdump),
    cmd!("xxd", Text, "xxd [-n байт] файл", "байты файла в шестнадцатеричном виде", "xxd [-n bytes] file", "file bytes in hex", cmds_text::hexdump),
    cmd!("tar", Text, "tar -c|-x|-t архив.tar [файлы…]", "упаковать/распаковать архив", "tar -c|-x|-t archive.tar [files…]", "pack/unpack an archive", cmds_text::tar),
    cmd!("zip", Text, "zip архив.zip файлы…", "упаковать в архив", "zip archive.zip files…", "pack into an archive", cmds_text::zip),
    cmd!("unzip", Text, "unzip архив.zip [папка]", "распаковать архив", "unzip archive.zip [dir]", "unpack an archive", cmds_text::unzip),
    // ---- система
    cmd!("whoami", System, "whoami", "имя пользователя", "whoami", "user name", cmds_sys::whoami),
    cmd!("id", System, "id", "пользователь и права", "id", "user and permissions", cmds_sys::id),
    cmd!("hostname", System, "hostname", "имя машины", "hostname", "machine name", cmds_sys::hostname),
    cmd!("uname", System, "uname [-a]", "сведения о системе", "uname [-a]", "system info", cmds_sys::uname),
    cmd!("date", System, "date", "дата и время мира", "date", "world date and time", cmds_sys::date),
    cmd!("uptime", System, "uptime", "время работы", "uptime", "uptime", cmds_sys::uptime),
    cmd!("ps", System, "ps [-a]", "процессы (с -a — всех пользователей)", "ps [-a]", "processes (with -a — all users)", cmds_sys::ps),
    cmd!("top", System, "top", "снимок нагрузки и процессов", "top", "load and process snapshot", cmds_sys::top),
    cmd!("kill", System, "kill pid…", "снять процесс", "kill pid…", "kill a process", cmds_sys::kill),
    cmd!("free", System, "free [-h]", "память", "free [-h]", "memory", cmds_sys::free),
    cmd!("env", System, "env", "переменные окружения", "env", "environment variables", cmds_sys::env),
    cmd!("export", System, "export ИМЯ=значение", "задать переменную", "export NAME=value", "set a variable", cmds_sys::export),
    cmd!("set", System, "set ИМЯ=значение", "задать переменную", "set NAME=value", "set a variable", cmds_sys::export),
    cmd!("unset", System, "unset ИМЯ…", "убрать переменную", "unset NAME…", "remove a variable", cmds_sys::unset),
    cmd!("alias", System, "alias [имя='команда']", "псевдонимы команд", "alias [name='command']", "command aliases", cmds_sys::alias),
    cmd!("unalias", System, "unalias имя…", "убрать псевдоним", "unalias name…", "remove an alias", cmds_sys::unalias),
    cmd!("history", System, "history [-c]", "история команд", "history [-c]", "command history", cmds_sys::history),
    cmd!("clear", System, "clear", "очистить экран", "clear", "clear the screen", cmds_sys::clear),
    cmd!("help", System, "help [команда] [-w]", "таблица команд; -w — окно справки", "help [command] [-w]", "command table; -w — help window", cmds_sys::help),
    cmd!("man", System, "man команда", "справка по команде", "man command", "help for a command", cmds_sys::man),
    cmd!("which", System, "which команда…", "где лежит команда", "which command…", "where a command lives", cmds_sys::which),
    cmd!("passwd", System, "passwd [пользователь] новый_пароль", "сменить пароль", "passwd [user] new_password", "change password", cmds_sys::passwd),
    cmd!("useradd", System, "useradd [-a] имя [пароль]", "добавить пользователя (-a — администратор)", "useradd [-a] name [password]", "add a user (-a — administrator)", cmds_sys::useradd),
    cmd!("userdel", System, "userdel имя", "удалить пользователя", "userdel name", "delete a user", cmds_sys::userdel),
    cmd!("users", System, "users", "список пользователей", "users", "list users", cmds_sys::users),
    cmd!("su", System, "su [пользователь] пароль", "сменить пользователя", "su [user] password", "switch user", cmds_sys::su),
    cmd!("sudo", System, "sudo команда…", "выполнить от root (администраторы)", "sudo command…", "run as root (administrators)", cmds_sys::sudo),
    cmd!("shutdown", System, "shutdown", "сохранить и выйти из игры; по SSH — выключить удалённую машину", "shutdown", "save and quit the game; over SSH — power off the remote machine", cmds_sys::shutdown),
    cmd!("reboot", System, "reboot", "перезагрузить машину", "reboot", "reboot the machine", cmds_sys::reboot),
    cmd!("logs", System, "logs [-n N] [-f auth|system]", "журнал машины", "logs [-n N] [-f auth|system]", "machine log", cmds_sys::logs),
    cmd!("exit", System, "exit", "отключиться / закрыть терминал", "exit", "disconnect / close the terminal", cmds_sys::exit),
    cmd!("logout", System, "logout", "то же, что exit", "logout", "same as exit", cmds_sys::exit),
    cmd!("true", System, "true", "ничего не делать, код 0", "true", "do nothing, exit code 0", cmds_sys::cmd_true),
    cmd!("false", System, "false", "ничего не делать, код 1", "false", "do nothing, exit code 1", cmds_sys::cmd_false),
    // ---- сеть
    cmd!("ifconfig", Net, "ifconfig", "сетевые интерфейсы", "ifconfig", "network interfaces", cmds_net::ifconfig),
    cmd!("ip", Net, "ip [addr|route]", "адреса и маршруты", "ip [addr|route]", "addresses and routes", cmds_net::ip),
    cmd!("ping", Net, "ping адрес|имя", "проверить связь", "ping address|name", "check connectivity", cmds_net::ping),
    cmd!("nslookup", Net, "nslookup имя", "узнать адрес по имени", "nslookup name", "resolve a name to an address", cmds_net::nslookup),
    cmd!("dig", Net, "dig имя", "узнать адрес по имени", "dig name", "resolve a name to an address", cmds_net::nslookup),
    cmd!("scan", Net, "scan адрес|имя", "порты и службы машины", "scan address|name", "machine ports and services", cmds_net::scan),
    cmd!("nmap", Net, "nmap адрес|имя", "порты и службы машины", "nmap address|name", "machine ports and services", cmds_net::scan),
    cmd!("ssh", Net, "ssh пользователь@адрес пароль", "подключиться к машине", "ssh user@address password", "connect to a machine", cmds_net::ssh),
    cmd!("scp", Net, "scp [user@ip:]откуда [user@ip:]куда пароль", "скопировать файл между машинами", "scp [user@ip:]src [user@ip:]dest password", "copy a file between machines", cmds_net::scp),
    cmd!("netstat", Net, "netstat", "соединения и открытые порты", "netstat", "connections and open ports", cmds_net::netstat),
    cmd!("whois", Net, "whois адрес|имя", "кому принадлежит адрес", "whois address|name", "who owns an address", cmds_net::whois),
    cmd!("traceroute", Net, "traceroute адрес|имя", "путь до машины", "traceroute address|name", "route to a machine", cmds_net::traceroute),
    cmd!("wget", Net, "wget url [-O файл]", "скачать страницу в файл", "wget url [-O file]", "download a page to a file", cmds_net::wget),
    cmd!("curl", Net, "curl url", "вывести страницу", "curl url", "print a page", cmds_net::curl),
    cmd!("arp", Net, "arp", "соседи в локальной сети", "arp", "neighbours on the local network", cmds_net::arp),
    cmd!("route", Net, "route", "таблица маршрутов", "route", "routing table", cmds_net::route),
    // ---- взлом
    cmd!("probe", Hack, "probe адрес", "разведка: службы, слабые места, пользователи", "probe address", "recon: services, weak spots, users", cmds_hack::probe),
    cmd!("research", Hack, "research [-h 1..4] адрес [id]", "исследование уязвимости и подсказки для своего эксплоита", "research [-h 1..4] address [id]", "research a vulnerability and get hints for your own exploit", cmds_hack::research),
    cmd!("exploit", Hack, "exploit -f файл.adr [-s id] адрес | exploit new файл.adr адрес [id]", "создать или запустить собственный эксплоит на Adder", "exploit -f file.adr [-s id] address | exploit new file.adr address [id]", "create or run your own Adder exploit", cmds_hack::exploit),
    cmd!("crack", Hack, "crack [-w словарь] пользователь@адрес", "подобрать пароль по словарю", "crack [-w wordlist] user@address", "crack a password with a wordlist", cmds_hack::crack),
    cmd!("privesc", Hack, "privesc файл.adr [id]", "запустить собственный локальный эксплоит", "privesc file.adr [id]", "run your own local exploit", cmds_hack::privesc),
    cmd!("clearlogs", Hack, "clearlogs [шаблон]", "убрать следы из журналов (нужны права)", "clearlogs [pattern]", "wipe traces from logs (needs permissions)", cmds_hack::clearlogs),
    cmd!("wipe", Hack, "wipe [шаблон]", "убрать следы из журналов (то же, что clearlogs)", "wipe [pattern]", "wipe traces from logs (same as clearlogs)", cmds_hack::clearlogs),
    cmd!("proxy", Hack, "proxy", "цепочка подключений и выходной адрес", "proxy", "connection chain and exit address", cmds_hack::proxy),
    cmd!("heat", Hack, "heat", "уровень подозрения и улики", "heat", "heat level and evidence", cmds_hack::heat),
    cmd!("trace", Hack, "trace", "состояние слежки: расследование центров безопасности", "trace", "surveillance status: security center investigations", cmds_hack::trace),
    cmd!("creds", Hack, "creds", "найденные логины", "creds", "found logins", cmds_hack::creds),
    cmd!("firewall", Hack, "firewall адрес", "обойти файрвол узла", "firewall address", "bypass a host's firewall", cmds_hack::firewall),
    cmd!("keylog", Hack, "keylog [plant адрес]", "перехватчик ввода: поставить / смотреть", "keylog [plant address]", "keylogger: plant / view", cmds_hack::keylog),
    cmd!("botnet", Hack, "botnet [add|remove адрес]", "ботнет из захваченных машин", "botnet [add|remove address]", "botnet of captured machines", cmds_hack::botnet),
    cmd!("encrypt", Hack, "encrypt файл ключ", "зашифровать файл", "encrypt file key", "encrypt a file", cmds_hack::encrypt),
    cmd!("decrypt", Hack, "decrypt файл ключ", "расшифровать файл", "decrypt file key", "decrypt a file", cmds_hack::decrypt),
    // ---- защита
    cmd!("shield", Defense, "shield [on|off]", "свой файрвол: отражать ответные атаки", "shield [on|off]", "your own firewall: repel counterattacks", cmds_defense::shield),
    cmd!("implants", Defense, "implants", "найти закладки на своей машине", "implants", "find implants on your machine", cmds_defense::implants),
    cmd!("clean", Defense, "clean [путь|all]", "вычистить закладки", "clean [path|all]", "clean out implants", cmds_defense::clean),
    // ---- база
    cmd!("base", Base, "base", "своя база и арендованные релеи", "base", "your base and rented relays", cmds_base::base),
    cmd!("rent", Base, "rent", "арендовать прокси-релей", "rent", "rent a proxy relay", cmds_base::rent),
    cmd!("release", Base, "release адрес", "отказаться от релея", "release address", "give up a relay", cmds_base::release),
    // ---- задания
    cmd!("missions", Jobs, "missions", "задания группы hello-world и счёт", "missions", "hello-world missions and score", cmds_jobs::missions),
    cmd!("jobs", Jobs, "jobs", "задания (то же, что missions)", "jobs", "missions (same as missions)", cmds_jobs::missions),
    cmd!("mission", Jobs, "mission [N]", "брифинг задания (без номера — текущего)", "mission [N]", "mission briefing (no number — current one)", cmds_jobs::mission),
    cmd!("ending", Jobs, "ending", "путь к свободе: прогресс и финал", "ending", "road to freedom: progress and ending", cmds_jobs::ending),
    cmd!("gigs", Jobs, "gigs", "подработки: открытые заказы и доход", "gigs", "gigs: open jobs and income", cmds_jobs::gigs),
    cmd!("scenario", Jobs, "scenario", "свой сценарий из редактора: цель и прогресс", "scenario", "your scenario from the editor: goal and progress", cmds_jobs::scenario),
    // ---- деньги
    cmd!("wallet", Money, "wallet", "счёт, заработок и долг", "wallet", "balance, earnings and debt", cmds_money::wallet),
    cmd!("debt", Money, "debt", "кредит: остаток, ставка, выплаты", "debt", "loan: balance, rate, payments", cmds_money::debt),
    cmd!("pay", Money, "pay сумма|all", "погасить часть долга", "pay amount|all", "pay off part of the debt", cmds_money::pay),
    cmd!("shop", Money, "shop", "магазин улучшений", "shop", "upgrade shop", cmds_money::shop),
    cmd!("buy", Money, "buy id", "купить улучшение", "buy id", "buy an upgrade", cmds_money::buy),
    cmd!("sell", Money, "sell", "сбыть найденные логины за ₵", "sell", "sell found logins for ₵", cmds_money::sell),
    cmd!("market", Money, "market [buy id]", "маркет скриптов в сети hw", "market [buy id]", "script market on the hw network", cmds_money::market),
    // ---- adder
    cmd!("adder", Adder, "adder файл.adr [аргументы] | -c \"код\" | --check файл", "запустить скрипт Adder", "adder file.adr [args] | -c \"code\" | --check file", "run an Adder script", cmds_misc::adder),
    // ---- окна
    cmd!("edit", Apps, "edit [файл]", "открыть редактор", "edit [file]", "open the editor", cmds_misc::edit),
    cmd!("nano", Apps, "nano [файл]", "открыть редактор", "nano [file]", "open the editor", cmds_misc::edit),
    cmd!("vim", Apps, "vim [файл]", "открыть редактор", "vim [file]", "open the editor", cmds_misc::edit),
    cmd!("open", Apps, "open приложение [аргумент]", "открыть окно приложения", "open app [arg]", "open an app window", cmds_misc::open),
    cmd!("files", Apps, "files [путь]", "окно файлов", "files [path]", "files window", cmds_misc::files),
    cmd!("browser", Apps, "browser [url]", "окно браузера", "browser [url]", "browser window", cmds_misc::browser),
    cmd!("map", Apps, "map", "карта сети", "map", "network map", cmds_misc::map),
    cmd!("monitor", Apps, "monitor", "монитор системы", "monitor", "system monitor", cmds_misc::monitor),
    cmd!("settings", Apps, "settings", "настройки", "settings", "settings", cmds_misc::settings),
    cmd!("mail", Apps, "mail [-l]", "почта (-l — список писем в терминале)", "mail [-l]", "mail (-l — list messages in the terminal)", cmds_misc::mail),
    // ---- прочее
    cmd!("banner", Misc, "banner текст", "крупные буквы", "banner text", "big letters", cmds_misc::banner),
    cmd!("matrix", Misc, "matrix [строк]", "дождь кода", "matrix [rows]", "code rain", cmds_misc::matrix),
    cmd!("fortune", Misc, "fortune", "случайное изречение", "fortune", "random saying", cmds_misc::fortune),
    cmd!("cal", Misc, "cal [месяц год]", "календарь", "cal [month year]", "calendar", cmds_misc::cal),
    cmd!("bc", Misc, "bc выражение", "калькулятор", "bc expression", "calculator", cmds_misc::bc),
    cmd!("yes", Misc, "yes [текст]", "повторить текст (20 строк)", "yes [text]", "repeat text (20 lines)", cmds_misc::yes),
    cmd!("sleep", Misc, "sleep секунды", "подождать", "sleep seconds", "wait", cmds_misc::sleep),
    cmd!("neofetch", Misc, "neofetch", "карточка машины", "neofetch", "machine card", cmds_misc::neofetch),
    cmd!("seq", Misc, "seq [от] до", "числа по порядку", "seq [from] to", "numbers in order", cmds_misc::seq),
    cmd!("news", Misc, "news", "городская лента событий", "news", "city news feed", cmds_misc::news),
    cmd!("achievements", Misc, "achievements", "достижения: открытые и ещё нет", "achievements", "achievements: unlocked and not yet", cmds_misc::achievements),
    cmd!("ach", Misc, "ach", "достижения (то же, что achievements)", "ach", "achievements (same as achievements)", cmds_misc::achievements),
];

pub fn lookup(name: &str) -> Option<&'static CmdInfo> {
    COMMANDS.iter().find(|c| c.name == name)
}

/// Команды группы в порядке таблицы.
pub fn by_group(group: Group) -> Vec<&'static CmdInfo> {
    COMMANDS.iter().filter(|c| c.group == group).collect()
}
