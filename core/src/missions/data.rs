//! Содержимое цепочки заданий hello-world: тексты и привязка к целям. Голос
//! группы — по `docs/STORY.md`: они годами в тени, вернулись, вступаются за
//! простых изгоев. Тексты с подстановками {host} {ip} {path} {to}.

/// Какую машину мира взять под задание.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TargetRole {
    /// Заброшенный домашний ПК с публичным адресом и слабым местом.
    HomePc,
    /// Сервер банковского ядра (проводки).
    BankLedger,
    /// Внутренняя машина за шлюзом — только через прокси.
    InternalServer,
    /// База больницы с данными пациентов.
    HospitalRecords,
    /// Публичный веб-сервер — для «голоса».
    PublicWeb,
    /// Машина наблюдателей: киберполиция или центр безопасности.
    Watchdog,
    /// Цель не нужна (действие над собой: заметание следов).
    None,
}

/// Что считается выполнением.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActionKind {
    /// Прочитать файл цели и прислать содержимое на адрес группы.
    ReadDeliver,
    /// Получить доступ к цели (owned).
    Own,
    /// Создать на цели файл с меткой.
    Plant,
    /// Замести следы: подозрение и улики убраны.
    CoverTracks,
}

/// Описание задания (без привязки к конкретной машине — её находит движок).
#[derive(Clone, Copy, Debug)]
pub struct MissionDef {
    pub code: &'static str,
    pub title: &'static str,
    pub role: TargetRole,
    pub action: ActionKind,
    pub reward: i64,
    pub objective: &'static str,
    pub brief: &'static str,
    pub reply: &'static str,
}

/// Адрес почты группы, куда шлют добычу.
pub const DROP: &str = "null@hw";

/// Цепочка заданий.
pub const CHAIN: &[MissionDef] = &[
    MissionDef {
        code: "m1",
        title: "Стук в дверь",
        role: TargetRole::HomePc,
        action: ActionKind::ReadDeliver,
        reward: 150,
        objective: "Прочитать файл {path} на {host} ({ip}) и прислать его содержимое на {to}",
        brief: "Ты нашёл дверь. Теперь покажи, что умеешь в неё входить.\n\n\
                На машине {host} ({ip}) лежит файл {path}. Войди как знаешь — слабое место там есть, — \
                прочитай его и пришли содержимое письмом на {to}.\n\n\
                Это не кража. Это проверка. Начни с probe {ip}.",
        reply: "Принято. Ты не растерялся у первой же двери — уже неплохо.\n\
                За работу платим: это не благотворительность, это дело.",
    },
    MissionDef {
        code: "m2",
        title: "Бумажка с печатью",
        role: TargetRole::BankLedger,
        action: ActionKind::ReadDeliver,
        reward: 400,
        objective: "Вытащить журнал проводок {path} с {host} ({ip}) и прислать на {to}",
        brief: "Тебя раздели из-за бумажки с печатью. Так работает весь их порядок.\n\n\
                На {host} ({ip}) — ядро банка. Там в {path} проводки, по которым видно, кто и как давит таких, как ты. \
                Достань их и пришли на {to}.\n\n\
                Банк сам слаб в старом углу — ищи устаревшую службу. Не забудь: ты наследишь, следи за heat.",
        reply: "Хорошо. Эти строки стоят дороже их печатей — мы знаем, куда их деть.\n\
                Дальше — тоньше. Со своего адреса больше не ходи.",
    },
    MissionDef {
        code: "m3",
        title: "Чужими руками",
        role: TargetRole::InternalServer,
        action: ActionKind::Own,
        reward: 350,
        objective: "Получить доступ к внутренней машине {host} ({ip}) — только через их шлюз",
        brief: "Запомни правило: к закрытому не ходят с порога.\n\n\
                Машина {host} ({ip}) сидит во внутренней сети — снаружи её не достать. \
                Сначала возьми их шлюз, а уже с него — проберись внутрь и получи доступ к {host}.\n\n\
                Так твой настоящий адрес остаётся в стороне. Проверь цепочку командой proxy.",
        reply: "Вот теперь видно руку. Ты прошёл через их же машину — и они винят её, не тебя.\n\
                Это и есть ремесло.",
    },
    MissionDef {
        code: "m4",
        title: "Истории болезни",
        role: TargetRole::HospitalRecords,
        action: ActionKind::ReadDeliver,
        reward: 500,
        objective: "Забрать {path} с {host} ({ip}) и прислать на {to}",
        brief: "Чью-то боль продали оптом. В {path} на {host} ({ip}) — записи, которые кто-то сливает на сторону.\n\n\
                Нам нужны не люди из этих списков, а тот, кто ими торгует. Достань файл и пришли на {to} — \
                дальше имя продавца найдём мы.\n\n\
                Иди через цепочку, как учили. И подчищай журналы.",
        reply: "Получили. Людей в обиду не дадим — а торговца вытащим на свет.\n\
                Ты, похоже, начинаешь понимать, за что мы.",
    },
    MissionDef {
        code: "m5",
        title: "Начисто",
        role: TargetRole::None,
        action: ActionKind::CoverTracks,
        reward: 250,
        objective: "Сбить подозрение (heat) ниже 15 и убрать улики со всех узлов",
        brief: "Ты нашумел. Отдел К уже принюхивается — посмотри trace.\n\n\
                Пройди по узлам, где остался твой след (команда heat покажет улики), зайди на каждый и clearlogs. \
                Добейся, чтобы подозрение упало ниже 15, а улик не осталось.\n\n\
                Хороший мастер не тот, кто вошёл, а тот, за кем не пришли.",
        reply: "Чисто. Теперь за тобой ничего не тянется.\n\
                Пора сказать громче, кто мы.",
    },
    MissionDef {
        code: "m6",
        title: "Голос",
        role: TargetRole::PublicWeb,
        action: ActionKind::Plant,
        reward: 600,
        objective: "Создать на {host} ({ip}) файл {path} со словом hello-world",
        brief: "Молчать больше незачем. Возьми публичный узел {host} ({ip}) и оставь на нём наш знак.\n\n\
                Получи права, создай файл {path} и впиши в него строку hello-world (и пару слов от себя, если хочешь). \
                Пусть увидят, что тень вернулась.\n\n\
                Нужны права root — вспомни privesc.",
        reply: "Теперь это их проблема. Утром половина города прочтёт одно слово — наше.\n\
                Осталось самое дерзкое.",
    },
    MissionDef {
        code: "m7",
        title: "Отдача",
        role: TargetRole::Watchdog,
        action: ActionKind::Own,
        reward: 1000,
        objective: "Проникнуть на машину наблюдателей {host} ({ip})",
        brief: "Те, кто вёл на тебя дело, сами не без дыр. {host} ({ip}) — узел тех, кто следит за всеми.\n\n\
                Войди к ним. Не ради мести — ради того, чтобы они знали: смотреть можно в обе стороны. \
                В их журналах ты найдёшь и дело о себе.\n\n\
                Иди тихо, через цепочку. Это выпускной.",
        reply: "Ты вошёл туда, куда входят немногие. Отныне ты — один из нас.\n\
                hello-world не выдумка. И ты теперь тоже.\n\nДобро пожаловать.",
    },
];

/// Тексты задания на английском (параллельно `CHAIN`, по тому же индексу).
#[derive(Clone, Copy, Debug)]
pub struct MissionTextEn {
    pub title: &'static str,
    pub objective: &'static str,
    pub brief: &'static str,
    pub reply: &'static str,
}

/// Английская цепочка — тот же порядок, что и `CHAIN`.
pub const CHAIN_EN: &[MissionTextEn] = &[
    MissionTextEn {
        title: "A Knock at the Door",
        objective: "Read the file {path} on {host} ({ip}) and send its contents to {to}",
        brief: "You found the door. Now show us you can get through it.\n\n\
                The machine {host} ({ip}) holds a file {path}. Get in however you can — there is a weak spot, — \
                read it and mail its contents to {to}.\n\n\
                This is not theft. It's a test. Start with probe {ip}.",
        reply: "Received. You didn't freeze at the very first door — that's a good start.\n\
                We pay for the work: this isn't charity, it's a cause.",
    },
    MissionTextEn {
        title: "A Paper with a Stamp",
        objective: "Pull the transaction log {path} from {host} ({ip}) and send it to {to}",
        brief: "They stripped you bare over a paper with a stamp. That's how their whole order works.\n\n\
                {host} ({ip}) is the bank's core. In {path} are the transactions that show who crushes people like you, and how. \
                Get them and send them to {to}.\n\n\
                The bank is weak in an old corner — look for an outdated service. Remember: you'll leave traces, watch your heat.",
        reply: "Good. These lines are worth more than their stamps — we know where to put them.\n\
                From here on it gets finer. Don't go out from your own address again.",
    },
    MissionTextEn {
        title: "Someone Else's Hands",
        objective: "Gain access to the internal machine {host} ({ip}) — only through their gateway",
        brief: "Remember the rule: you don't walk up to a locked place from the doorstep.\n\n\
                The machine {host} ({ip}) sits on the internal network — it can't be reached from outside. \
                Take their gateway first, then work your way inside from it and gain access to {host}.\n\n\
                That way your real address stays out of it. Check your chain with the proxy command.",
        reply: "Now we can see a steady hand. You went through their own machine — and they blame it, not you.\n\
                That is the craft.",
    },
    MissionTextEn {
        title: "Medical Records",
        objective: "Take {path} from {host} ({ip}) and send it to {to}",
        brief: "Someone's pain was sold wholesale. In {path} on {host} ({ip}) are records that someone is leaking to the side.\n\n\
                We don't need the people on those lists — we need whoever trades in them. Get the file and send it to {to} — \
                we'll find the seller's name ourselves.\n\n\
                Go through a chain, as you were taught. And clean up the logs.",
        reply: "Got it. We won't let those people be harmed — and we'll drag the trader into the light.\n\
                Looks like you're starting to understand what we stand for.",
    },
    MissionTextEn {
        title: "Clean Slate",
        objective: "Bring heat below 15 and remove evidence from every host",
        brief: "You made noise. Department K is already sniffing around — check trace.\n\n\
                Go through the hosts where your trail remains (the heat command shows the evidence), log in to each and clearlogs. \
                Get your heat below 15 and leave no evidence behind.\n\n\
                A good master isn't the one who got in, but the one nobody came for.",
        reply: "Clean. Nothing trails behind you now.\n\
                Time to say louder who we are.",
    },
    MissionTextEn {
        title: "The Voice",
        objective: "Create the file {path} on {host} ({ip}) containing the word hello-world",
        brief: "There's no reason to stay silent anymore. Take the public host {host} ({ip}) and leave our sign on it.\n\n\
                Get the rights, create the file {path} and write the line hello-world in it (and a few words of your own, if you like). \
                Let them see the shadow is back.\n\n\
                You'll need root access — remember privesc.",
        reply: "Now it's their problem. By morning half the city will read one word — ours.\n\
                Only the boldest part is left.",
    },
    MissionTextEn {
        title: "Blowback",
        objective: "Break into the watchers' machine {host} ({ip})",
        brief: "The ones who built a case against you have holes of their own. {host} ({ip}) is a host of those who watch everyone.\n\n\
                Get inside. Not for revenge — so they know that watching works both ways. \
                In their logs you'll find the case on yourself too.\n\n\
                Go quietly, through a chain. This is your graduation.",
        reply: "You got in where few ever do. From now on you are one of us.\n\
                hello-world is not a fiction. And neither are you now.\n\nWelcome.",
    },
];
