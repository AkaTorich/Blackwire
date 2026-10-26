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
            Group::Files => "Файлы и каталоги",
            Group::Text => "Текст и архивы",
            Group::System => "Система и пользователи",
            Group::Net => "Сеть",
            Group::Hack => "Взлом",
            Group::Defense => "Защита",
            Group::Base => "База",
            Group::Jobs => "Задания",
            Group::Money => "Деньги",
            Group::Adder => "Скрипты Adder",
            Group::Apps => "Окна приложений",
            Group::Misc => "Прочее",
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
    pub run: CmdFn,
}

macro_rules! cmd {
    ($name:expr, $group:ident, $usage:expr, $short:expr, $f:path) => {
        CmdInfo { name: $name, group: Group::$group, usage: $usage, short: $short, run: $f }
    };
}

pub static COMMANDS: &[CmdInfo] = &[
    // ---- файлы
    cmd!("ls", Files, "ls [-l] [-a] [-h] [путь…]", "список файлов", cmds_fs::ls),
    cmd!("cd", Files, "cd [путь]", "сменить папку (без аргумента — домой, «-» — назад)", cmds_fs::cd),
    cmd!("pwd", Files, "pwd", "текущая папка", cmds_fs::pwd),
    cmd!("cat", Files, "cat [-n] [файл…]", "вывести файл", cmds_fs::cat),
    cmd!("less", Files, "less файл", "вывести файл (то же, что cat)", cmds_fs::cat),
    cmd!("more", Files, "more файл", "вывести файл (то же, что cat)", cmds_fs::cat),
    cmd!("touch", Files, "touch файл…", "создать пустой файл или обновить время", cmds_fs::touch),
    cmd!("mkdir", Files, "mkdir [-p] папка…", "создать папку", cmds_fs::mkdir),
    cmd!("rm", Files, "rm [-r] [-f] путь…", "удалить файлы и папки", cmds_fs::rm),
    cmd!("rmdir", Files, "rmdir папка…", "удалить пустую папку", cmds_fs::rmdir),
    cmd!("mv", Files, "mv откуда… куда", "переместить или переименовать", cmds_fs::mv),
    cmd!("cp", Files, "cp [-r] откуда… куда", "скопировать", cmds_fs::cp),
    cmd!("echo", Files, "echo [-n] [-e] текст…", "напечатать текст", cmds_fs::echo),
    cmd!("printf", Files, "printf формат [аргументы…]", "напечатать текст (как echo -e)", cmds_fs::echo),
    cmd!("find", Files, "find [путь] [-name шаблон] [-type f|d]", "искать файлы", cmds_fs::find),
    cmd!("du", Files, "du [-h] [-s] [путь]", "размер папок", cmds_fs::du),
    cmd!("df", Files, "df [-h]", "место на диске", cmds_fs::df),
    cmd!("chmod", Files, "chmod режим файл…", "сменить права (755, u+x, rwxr-xr-x)", cmds_fs::chmod),
    cmd!("chown", Files, "chown пользователь файл…", "сменить владельца (root)", cmds_fs::chown),
    cmd!("file", Files, "file путь…", "определить вид файла", cmds_fs::file),
    cmd!("stat", Files, "stat путь…", "сведения о файле", cmds_fs::stat),
    cmd!("tree", Files, "tree [-a] [-L глубина] [путь]", "дерево папок", cmds_fs::tree),
    cmd!("ln", Files, "ln [-s] цель имя", "создать копию под другим именем", cmds_fs::ln),
    // ---- текст и архивы
    cmd!("head", Text, "head [-n N] [файл…]", "первые строки", cmds_text::head),
    cmd!("tail", Text, "tail [-n N] [файл…]", "последние строки", cmds_text::tail),
    cmd!("grep", Text, "grep [-i] [-n] [-v] [-c] [-r] шаблон [файл…]", "искать строки", cmds_text::grep),
    cmd!("wc", Text, "wc [-l] [-w] [-c] [файл…]", "посчитать строки, слова, байты", cmds_text::wc),
    cmd!("sort", Text, "sort [-r] [-n] [-u] [файл…]", "отсортировать строки", cmds_text::sort),
    cmd!("uniq", Text, "uniq [-c] [файл]", "убрать повторы соседних строк", cmds_text::uniq),
    cmd!("cut", Text, "cut -d разделитель -f номера [файл…]", "вырезать поля", cmds_text::cut),
    cmd!("tr", Text, "tr [-d] набор1 [набор2]", "заменить символы", cmds_text::tr),
    cmd!("rev", Text, "rev [файл…]", "перевернуть строки", cmds_text::rev),
    cmd!("tee", Text, "tee [-a] файл", "вывести и записать в файл", cmds_text::tee),
    cmd!("diff", Text, "diff файл1 файл2", "различия двух файлов", cmds_text::diff),
    cmd!("md5", Text, "md5 [-s строка] [файл…]", "условный хеш", cmds_text::md5),
    cmd!("md5sum", Text, "md5sum [файл…]", "условный хеш", cmds_text::md5),
    cmd!("base64", Text, "base64 [-d] [файл]", "кодировать/раскодировать base64", cmds_text::base64),
    cmd!("hexdump", Text, "hexdump [-n байт] файл", "байты файла в шестнадцатеричном виде", cmds_text::hexdump),
    cmd!("xxd", Text, "xxd [-n байт] файл", "байты файла в шестнадцатеричном виде", cmds_text::hexdump),
    cmd!("tar", Text, "tar -c|-x|-t архив.tar [файлы…]", "упаковать/распаковать архив", cmds_text::tar),
    cmd!("zip", Text, "zip архив.zip файлы…", "упаковать в архив", cmds_text::zip),
    cmd!("unzip", Text, "unzip архив.zip [папка]", "распаковать архив", cmds_text::unzip),
    // ---- система
    cmd!("whoami", System, "whoami", "имя пользователя", cmds_sys::whoami),
    cmd!("id", System, "id", "пользователь и права", cmds_sys::id),
    cmd!("hostname", System, "hostname", "имя машины", cmds_sys::hostname),
    cmd!("uname", System, "uname [-a]", "сведения о системе", cmds_sys::uname),
    cmd!("date", System, "date", "дата и время мира", cmds_sys::date),
    cmd!("uptime", System, "uptime", "время работы", cmds_sys::uptime),
    cmd!("ps", System, "ps [-a]", "процессы (с -a — всех пользователей)", cmds_sys::ps),
    cmd!("top", System, "top", "снимок нагрузки и процессов", cmds_sys::top),
    cmd!("kill", System, "kill pid…", "снять процесс", cmds_sys::kill),
    cmd!("free", System, "free [-h]", "память", cmds_sys::free),
    cmd!("env", System, "env", "переменные окружения", cmds_sys::env),
    cmd!("export", System, "export ИМЯ=значение", "задать переменную", cmds_sys::export),
    cmd!("set", System, "set ИМЯ=значение", "задать переменную", cmds_sys::export),
    cmd!("unset", System, "unset ИМЯ…", "убрать переменную", cmds_sys::unset),
    cmd!("alias", System, "alias [имя='команда']", "псевдонимы команд", cmds_sys::alias),
    cmd!("unalias", System, "unalias имя…", "убрать псевдоним", cmds_sys::unalias),
    cmd!("history", System, "history [-c]", "история команд", cmds_sys::history),
    cmd!("clear", System, "clear", "очистить экран", cmds_sys::clear),
    cmd!("help", System, "help [команда] [-w]", "таблица команд; -w — окно справки", cmds_sys::help),
    cmd!("man", System, "man команда", "справка по команде", cmds_sys::man),
    cmd!("which", System, "which команда…", "где лежит команда", cmds_sys::which),
    cmd!("passwd", System, "passwd [пользователь] новый_пароль", "сменить пароль", cmds_sys::passwd),
    cmd!("useradd", System, "useradd [-a] имя [пароль]", "добавить пользователя (-a — администратор)", cmds_sys::useradd),
    cmd!("userdel", System, "userdel имя", "удалить пользователя", cmds_sys::userdel),
    cmd!("users", System, "users", "список пользователей", cmds_sys::users),
    cmd!("su", System, "su [пользователь] пароль", "сменить пользователя", cmds_sys::su),
    cmd!("sudo", System, "sudo команда…", "выполнить от root (администраторы)", cmds_sys::sudo),
    cmd!("shutdown", System, "shutdown", "сохранить и выйти из игры; по SSH — выключить удалённую машину", cmds_sys::shutdown),
    cmd!("reboot", System, "reboot", "перезагрузить машину", cmds_sys::reboot),
    cmd!("logs", System, "logs [-n N] [-f auth|system]", "журнал машины", cmds_sys::logs),
    cmd!("exit", System, "exit", "отключиться / закрыть терминал", cmds_sys::exit),
    cmd!("logout", System, "logout", "то же, что exit", cmds_sys::exit),
    cmd!("true", System, "true", "ничего не делать, код 0", cmds_sys::cmd_true),
    cmd!("false", System, "false", "ничего не делать, код 1", cmds_sys::cmd_false),
    // ---- сеть
    cmd!("ifconfig", Net, "ifconfig", "сетевые интерфейсы", cmds_net::ifconfig),
    cmd!("ip", Net, "ip [addr|route]", "адреса и маршруты", cmds_net::ip),
    cmd!("ping", Net, "ping адрес|имя", "проверить связь", cmds_net::ping),
    cmd!("nslookup", Net, "nslookup имя", "узнать адрес по имени", cmds_net::nslookup),
    cmd!("dig", Net, "dig имя", "узнать адрес по имени", cmds_net::nslookup),
    cmd!("scan", Net, "scan адрес|имя", "порты и службы машины", cmds_net::scan),
    cmd!("nmap", Net, "nmap адрес|имя", "порты и службы машины", cmds_net::scan),
    cmd!("ssh", Net, "ssh пользователь@адрес пароль", "подключиться к машине", cmds_net::ssh),
    cmd!("scp", Net, "scp [user@ip:]откуда [user@ip:]куда пароль", "скопировать файл между машинами", cmds_net::scp),
    cmd!("netstat", Net, "netstat", "соединения и открытые порты", cmds_net::netstat),
    cmd!("whois", Net, "whois адрес|имя", "кому принадлежит адрес", cmds_net::whois),
    cmd!("traceroute", Net, "traceroute адрес|имя", "путь до машины", cmds_net::traceroute),
    cmd!("wget", Net, "wget url [-O файл]", "скачать страницу в файл", cmds_net::wget),
    cmd!("curl", Net, "curl url", "вывести страницу", cmds_net::curl),
    cmd!("arp", Net, "arp", "соседи в локальной сети", cmds_net::arp),
    cmd!("route", Net, "route", "таблица маршрутов", cmds_net::route),
    // ---- взлом
    cmd!("probe", Hack, "probe адрес", "разведка: службы, слабые места, пользователи", cmds_hack::probe),
    cmd!("exploit", Hack, "exploit [-s служба] адрес", "воспользоваться слабым местом службы", cmds_hack::exploit),
    cmd!("crack", Hack, "crack [-w словарь] пользователь@адрес", "подобрать пароль по словарю", cmds_hack::crack),
    cmd!("privesc", Hack, "privesc", "повысить права на текущей машине до root", cmds_hack::privesc),
    cmd!("clearlogs", Hack, "clearlogs [шаблон]", "убрать следы из журналов (нужны права)", cmds_hack::clearlogs),
    cmd!("wipe", Hack, "wipe [шаблон]", "убрать следы из журналов (то же, что clearlogs)", cmds_hack::clearlogs),
    cmd!("proxy", Hack, "proxy", "цепочка подключений и выходной адрес", cmds_hack::proxy),
    cmd!("heat", Hack, "heat", "уровень подозрения и улики", cmds_hack::heat),
    cmd!("trace", Hack, "trace", "состояние слежки: расследование центров безопасности", cmds_hack::trace),
    cmd!("creds", Hack, "creds", "найденные логины", cmds_hack::creds),
    cmd!("firewall", Hack, "firewall адрес", "обойти файрвол узла", cmds_hack::firewall),
    cmd!("keylog", Hack, "keylog [plant адрес]", "перехватчик ввода: поставить / смотреть", cmds_hack::keylog),
    cmd!("botnet", Hack, "botnet [add|remove адрес]", "ботнет из захваченных машин", cmds_hack::botnet),
    cmd!("encrypt", Hack, "encrypt файл ключ", "зашифровать файл", cmds_hack::encrypt),
    cmd!("decrypt", Hack, "decrypt файл ключ", "расшифровать файл", cmds_hack::decrypt),
    // ---- защита
    cmd!("shield", Defense, "shield [on|off]", "свой файрвол: отражать ответные атаки", cmds_defense::shield),
    cmd!("implants", Defense, "implants", "найти закладки на своей машине", cmds_defense::implants),
    cmd!("clean", Defense, "clean [путь|all]", "вычистить закладки", cmds_defense::clean),
    // ---- база
    cmd!("base", Base, "base", "своя база и арендованные релеи", cmds_base::base),
    cmd!("rent", Base, "rent", "арендовать прокси-релей", cmds_base::rent),
    cmd!("release", Base, "release адрес", "отказаться от релея", cmds_base::release),
    // ---- задания
    cmd!("missions", Jobs, "missions", "задания группы hello-world и счёт", cmds_jobs::missions),
    cmd!("jobs", Jobs, "jobs", "задания (то же, что missions)", cmds_jobs::missions),
    cmd!("mission", Jobs, "mission [N]", "брифинг задания (без номера — текущего)", cmds_jobs::mission),
    cmd!("ending", Jobs, "ending", "путь к свободе: прогресс и финал", cmds_jobs::ending),
    cmd!("gigs", Jobs, "gigs", "подработки: открытые заказы и доход", cmds_jobs::gigs),
    cmd!("scenario", Jobs, "scenario", "свой сценарий из редактора: цель и прогресс", cmds_jobs::scenario),
    // ---- деньги
    cmd!("wallet", Money, "wallet", "счёт, заработок и долг", cmds_money::wallet),
    cmd!("debt", Money, "debt", "кредит: остаток, ставка, выплаты", cmds_money::debt),
    cmd!("pay", Money, "pay сумма|all", "погасить часть долга", cmds_money::pay),
    cmd!("shop", Money, "shop", "магазин улучшений", cmds_money::shop),
    cmd!("buy", Money, "buy id", "купить улучшение", cmds_money::buy),
    cmd!("sell", Money, "sell", "сбыть найденные логины за ₵", cmds_money::sell),
    cmd!("market", Money, "market [buy id]", "маркет скриптов в сети hw", cmds_money::market),
    // ---- adder
    cmd!("adder", Adder, "adder файл.adr [аргументы] | -c \"код\" | --check файл", "запустить скрипт Adder", cmds_misc::adder),
    // ---- окна
    cmd!("edit", Apps, "edit [файл]", "открыть редактор", cmds_misc::edit),
    cmd!("nano", Apps, "nano [файл]", "открыть редактор", cmds_misc::edit),
    cmd!("vim", Apps, "vim [файл]", "открыть редактор", cmds_misc::edit),
    cmd!("open", Apps, "open приложение [аргумент]", "открыть окно приложения", cmds_misc::open),
    cmd!("files", Apps, "files [путь]", "окно файлов", cmds_misc::files),
    cmd!("browser", Apps, "browser [url]", "окно браузера", cmds_misc::browser),
    cmd!("map", Apps, "map", "карта сети", cmds_misc::map),
    cmd!("monitor", Apps, "monitor", "монитор системы", cmds_misc::monitor),
    cmd!("settings", Apps, "settings", "настройки", cmds_misc::settings),
    cmd!("mail", Apps, "mail [-l]", "почта (-l — список писем в терминале)", cmds_misc::mail),
    // ---- прочее
    cmd!("banner", Misc, "banner текст", "крупные буквы", cmds_misc::banner),
    cmd!("matrix", Misc, "matrix [строк]", "дождь кода", cmds_misc::matrix),
    cmd!("fortune", Misc, "fortune", "случайное изречение", cmds_misc::fortune),
    cmd!("cal", Misc, "cal [месяц год]", "календарь", cmds_misc::cal),
    cmd!("bc", Misc, "bc выражение", "калькулятор", cmds_misc::bc),
    cmd!("yes", Misc, "yes [текст]", "повторить текст (20 строк)", cmds_misc::yes),
    cmd!("sleep", Misc, "sleep секунды", "подождать", cmds_misc::sleep),
    cmd!("neofetch", Misc, "neofetch", "карточка машины", cmds_misc::neofetch),
    cmd!("seq", Misc, "seq [от] до", "числа по порядку", cmds_misc::seq),
    cmd!("news", Misc, "news", "городская лента событий", cmds_misc::news),
    cmd!("achievements", Misc, "achievements", "достижения: открытые и ещё нет", cmds_misc::achievements),
    cmd!("ach", Misc, "ach", "достижения (то же, что achievements)", cmds_misc::achievements),
];

pub fn lookup(name: &str) -> Option<&'static CmdInfo> {
    COMMANDS.iter().find(|c| c.name == name)
}

/// Команды группы в порядке таблицы.
pub fn by_group(group: Group) -> Vec<&'static CmdInfo> {
    COMMANDS.iter().filter(|c| c.group == group).collect()
}
