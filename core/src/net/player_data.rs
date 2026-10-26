//! Тексты для компьютера игрока: приветствие, готовые скрипты Adder, словарь,
//! `/etc/hosts`, `/etc/motd`. Скрипты написаны по `docs/ADDER.md`.

/// Приветствие `readme.txt` — по-русски, без детского тона.
pub fn readme(handle: &str) -> String {
    format!(
        "Привет, {h}.\n\nЭто твой компьютер. Почти всё, что было в доме, забрали за долги — остался он, старый\nмаршрутизатор и счёт, который не закрыть. Начинать придётся отсюда.\n\nС чего начать:\n  - открой почту (приложение «Почта» или команда open mail) — там несколько писем;\n  - посмотри, кто вокруг: ping и scan по адресам, которые уже знаешь (команда net_map\n    покажет карту);\n  - в папке scripts лежат готовые примеры на языке Adder — запусти hello.adr, затем\n    scan.adr, разберись, как они устроены;\n  - wordlist.txt — список частых паролей, он нужен скрипту bruteforce_demo.adr.\n\nСписок известных узлов — в /etc/hosts. Документация по языку — команда adder --help или\nприложение «Помощь».\n\nНе торопись. Сначала осмотрись.\n",
        h = handle
    )
}

/// `hello.adr` — самый простой скрипт.
pub fn script_hello() -> &'static str {
    "# hello.adr — первый скрипт: кто я и где я\n\
print(\"Привет, \" + sys.whoami() + \"!\")\n\
print(\"Машина: \" + sys.hostname())\n\
print(\"Адрес:  \" + net.my_ip())\n\
print(\"Время:  \" + time.format(time.now()))\n"
}

/// `scan.adr` — пинг и сканирование портов (пример из ADDER.md).
pub fn script_scan() -> &'static str {
    "# scan.adr — найти машины с открытым ssh\n\
targets = sys.args[1:]\n\
if not targets:\n\
\u{20}   print(\"Использование: scan.adr ip [ip ...]\")\n\
\u{20}   exit(1)\n\
\n\
for ip in targets:\n\
\u{20}   ms = net.ping(ip)\n\
\u{20}   if ms == null:\n\
\u{20}       print(f\"{ip:16} недоступен\")\n\
\u{20}       continue\n\
\u{20}   ports = net.scan(ip)\n\
\u{20}   open_ssh = [p for p in ports if p[\"port\"] == 22]\n\
\u{20}   mark = \"ssh\" if open_ssh else \"---\"\n\
\u{20}   print(f\"{ip:16} {ms:4} мс  {mark}  {len(ports)} портов\")\n"
}

/// `bruteforce_demo.adr` — перебор паролей по словарю через `net.connect`.
pub fn script_bruteforce(home: &str) -> String {
    format!(
        "# bruteforce_demo.adr — подбор пароля по словарю\n\
# Осторожно: неудачные попытки видны в журнале чужой машины.\n\
args = sys.args[1:]\n\
if len(args) < 2:\n\
\u{20}   print(\"Использование: bruteforce_demo.adr <ip> <пользователь>\")\n\
\u{20}   exit(1)\n\
\n\
ip = args[0]\n\
user = args[1]\n\
words = fs.read(\"{home}/wordlist.txt\").split(\"\\n\")\n\
print(f\"Подбор пароля для {{user}}@{{ip}} по словарю ({{len(words)}} слов)\")\n\
\n\
tries = 0\n\
for pw in words:\n\
\u{20}   if pw == \"\":\n\
\u{20}       continue\n\
\u{20}   tries += 1\n\
\u{20}   try:\n\
\u{20}       if net.connect(ip, user, pw):\n\
\u{20}           print(f\"Пароль найден после {{tries}} попыток: {{pw}}\")\n\
\u{20}           net.disconnect()\n\
\u{20}           exit(0)\n\
\u{20}   except as e:\n\
\u{20}       print(f\"Нет связи: {{e}}\")\n\
\u{20}       exit(1)\n\
\n\
print(\"Пароль не найден в словаре.\")\n",
        home = home
    )
}

/// `backup.adr` — простая резервная копия важных файлов.
pub fn script_backup(home: &str) -> String {
    format!(
        "# backup.adr — собрать важные файлы в один архив\n\
home = \"{home}\"\n\
dst = home + \"/backup.tar\"\n\
items = [\"notes.txt\", \"wordlist.txt\", \"scripts\"]\n\
present = [home + \"/\" + x for x in items if fs.exists(home + \"/\" + x)]\n\
if not present:\n\
\u{20}   print(\"Нечего сохранять.\")\n\
\u{20}   exit(1)\n\
\n\
line = \"tar -c -f \" + dst + \" \" + \" \".join(present)\n\
res = shell.exec(line)\n\
if res[\"ok\"]:\n\
\u{20}   print(\"Резервная копия готова: \" + dst)\n\
else:\n\
\u{20}   print(\"Не удалось: \" + res[\"out\"])\n",
        home = home
    )
}

/// `/etc/motd` компьютера игрока.
pub fn motd() -> &'static str {
    "Добро пожаловать. Система готова. Введите help для списка команд.\n"
}
