//! Имена людей разных народов (уважительно, на равных), логины, слабые пароли
//! (тот же список, что в словаре игрока) и генератор сильных паролей.

use crate::rng::Rng;

/// Человек: латинский логин, имя и фамилия (для файлов и подписей).
pub struct Person {
    pub login: &'static str,
    pub first: &'static str,
    pub last: &'static str,
    pub first_en: &'static str,
    pub last_en: &'static str,
}

impl Person {
    /// Имя на языке игры.
    pub fn first_name(&self) -> &'static str {
        if crate::i18n::is_en() { self.first_en } else { self.first }
    }
    /// Фамилия на языке игры.
    pub fn last_name(&self) -> &'static str {
        if crate::i18n::is_en() { self.last_en } else { self.last }
    }
}

const fn p(
    login: &'static str,
    first: &'static str,
    last: &'static str,
    first_en: &'static str,
    last_en: &'static str,
) -> Person {
    Person { login, first, last, first_en, last_en }
}

/// Пул людей: имена и фамилии разных культур, вперемешку.
pub const PEOPLE: &[Person] = &[
    p("marat", "Марат", "Сафин", "Marat", "Safin"),
    p("leyla", "Лейла", "Гаспарян", "Leyla", "Gasparyan"),
    p("einar", "Эйнар", "Торвальдсен", "Einar", "Thorvaldsen"),
    p("solveyg", "Сольвейг", "Ларсен", "Solveig", "Larsen"),
    p("temuchin", "Тэмуджин", "Батбаяр", "Temujin", "Batbayar"),
    p("yadviga", "Ядвига", "Ковальчик", "Jadwiga", "Kowalczyk"),
    p("kasym", "Касым", "Абдуллаев", "Kasym", "Abdullaev"),
    p("nanna", "Нанна", "Йенсен", "Nanna", "Jensen"),
    p("rashid", "Рашид", "Хаккани", "Rashid", "Haqqani"),
    p("bjorn", "Бьёрн", "Эриксон", "Bjorn", "Eriksson"),
    p("zuhra", "Зухра", "Ниязова", "Zuhra", "Niyazova"),
    p("osvald", "Освальд", "Крамер", "Oswald", "Kramer"),
    p("ingrid", "Ингрид", "Халворсен", "Ingrid", "Halvorsen"),
    p("farid", "Фарид", "Мансуров", "Farid", "Mansurov"),
    p("otto", "Отто", "Шнайдер", "Otto", "Schneider"),
    p("dilnoza", "Дильноза", "Юсупова", "Dilnoza", "Yusupova"),
    p("hugo", "Гуго", "Бертран", "Hugo", "Bertrand"),
    p("aiko", "Айко", "Танака", "Aiko", "Tanaka"),
    p("miro", "Миро", "Новак", "Miro", "Novak"),
    p("galina", "Галина", "Орлова", "Galina", "Orlova"),
    p("tariq", "Тарик", "Насер", "Tariq", "Nasser"),
    p("sanna", "Санна", "Виртанен", "Sanna", "Virtanen"),
    p("liang", "Лян", "Чэнь", "Liang", "Chen"),
    p("amara", "Амара", "Диалло", "Amara", "Diallo"),
    p("viktor", "Виктор", "Петров", "Viktor", "Petrov"),
    p("elda", "Эльда", "Росси", "Elda", "Rossi"),
    p("yusuf", "Юсуф", "Карими", "Yusuf", "Karimi"),
    p("hilde", "Хильда", "Фишер", "Hilde", "Fischer"),
    p("ramon", "Рамон", "Гарсия", "Ramon", "Garcia"),
    p("oksana", "Оксана", "Шевченко", "Oksana", "Shevchenko"),
    p("dawit", "Давит", "Гебре", "Dawit", "Gebre"),
    p("meret", "Мерет", "Келлер", "Meret", "Keller"),
    p("ildar", "Ильдар", "Хакимов", "Ildar", "Khakimov"),
    p("astrid", "Астрид", "Бергман", "Astrid", "Bergman"),
    p("noah", "Ной", "Леви", "Noah", "Levi"),
    p("parvati", "Парвати", "Шарма", "Parvati", "Sharma"),
    p("goran", "Горан", "Ковачевич", "Goran", "Kovacevic"),
    p("fatima", "Фатима", "аль-Захра", "Fatima", "al-Zahra"),
    p("soren", "Сёрен", "Мадсен", "Soren", "Madsen"),
    p("mei", "Мэй", "Ван", "Mei", "Wang"),
];

/// Начало строки словаря игрока — ровно 200 частых паролей. Слабые пароли мира
/// берутся отсюда же, поэтому подбор по словарю их находит.
pub const COMMON_PASSWORDS: &[&str] = &[
    "123456", "password", "12345678", "qwerty", "123456789", "12345", "1234", "111111", "1234567",
    "dragon", "123123", "baseball", "abc123", "football", "monkey", "letmein", "696969", "shadow",
    "master", "666666", "qwertyuiop", "123321", "mustang", "1234567890", "michael", "654321",
    "superman", "1qaz2wsx", "7777777", "121212", "000000", "qazwsx", "123qwe", "killer", "trustno1",
    "jordan", "jennifer", "zxcvbnm", "asdfgh", "hunter", "buster", "soccer", "harley", "batman",
    "andrew", "tigger", "sunshine", "iloveyou", "2000", "charlie", "robert", "thomas", "hockey",
    "ranger", "daniel", "starwars", "klaster", "112233", "george", "computer", "michelle", "jessica",
    "pepper", "1111", "zxcvbn", "555555", "11111111", "131313", "freedom", "777777", "pass",
    "maggie", "159753", "aaaaaa", "ginger", "princess", "joshua", "cheese", "amanda", "summer",
    "love", "ashley", "nicole", "chelsea", "biteme", "matthew", "access", "yankees", "987654321",
    "dallas", "austin", "thunder", "taylor", "matrix", "william", "corvette", "hello", "martin",
    "heather", "secret", "merlin", "diamond", "1234qwer", "gfhjkm", "hammer", "silver", "222222",
    "88888888", "anthony", "justin", "test", "bailey", "q1w2e3r4t5", "patrick", "internet", "scooter",
    "orange", "11111", "golfer", "cookie", "richard", "samantha", "bigdog", "guitar", "jackson",
    "whatever", "mickey", "chicken", "sparky", "snoopy", "maverick", "phoenix", "camaro", "sexy",
    "peanut", "morgan", "welcome", "falcon", "cowboy", "ferrari", "samsung", "andrea", "smokey",
    "steelers", "joseph", "mercedes", "dakota", "arsenal", "eagles", "melissa", "boomer", "booboo",
    "spider", "nascar", "monster", "tigers", "yellow", "xxxxxx", "123123123", "gateway", "marina",
    "diablo", "bulldog", "qwer1234", "compaq", "purple", "hardcore", "banana", "junior", "hannah",
    "123654", "porsche", "lakers", "iceman", "money", "cowboys", "987654", "london", "tennis",
    "999999", "ncc1701", "coffee", "scooby", "0000", "miller", "boston", "q1w2e3r4", "fuckoff",
    "brandon", "yamaha", "chester", "mother", "forever", "johnny", "edward", "333333", "oliver",
    "redsox", "player", "nikita", "knight", "fender", "barney", "midnight", "please", "brandy",
    "chicago", "badboy", "iwantu", "slayer", "rangers", "charles", "angel", "flower", "bigdaddy",
];

/// Список слов словаря в виде строк (копия `COMMON_PASSWORDS`).
pub fn wordlist() -> Vec<String> {
    COMMON_PASSWORDS.iter().map(|s| s.to_string()).collect()
}

/// Слабый пароль: иногда из словаря, иногда имя/фирма с числом (но так, чтобы
/// при желании попадал в словарь — для надёжности подбора берём чистое слово
/// из словаря в большинстве случаев).
pub fn weak_password(rng: &mut Rng, hint_tag: &str, year: i64) -> String {
    match rng.range(0, 10) {
        0..=6 => (*rng.pick(COMMON_PASSWORDS)).to_string(),
        7 => format!("{}{}", hint_tag, year),
        8 => format!("{}123", hint_tag),
        _ => {
            let base = *rng.pick(COMMON_PASSWORDS);
            format!("{}{}", base, rng.range(10, 100))
        }
    }
}

const SYLLABLES: &[&str] = &[
    "qa", "zx", "vb", "kr", "lo", "tu", "mi", "ph", "gw", "nd", "rk", "xy", "pz", "jv", "wm", "cf",
];

/// Сильный пароль: слоги, заглавные, цифры и символ — в словарь не попадает.
pub fn strong_password(rng: &mut Rng) -> String {
    let mut s = String::new();
    let parts = rng.range(3, 5);
    for i in 0..parts {
        let syl = *rng.pick(SYLLABLES);
        if i == 0 {
            let mut c = syl.chars();
            if let Some(first) = c.next() {
                s.extend(first.to_uppercase());
                s.push_str(c.as_str());
            }
        } else {
            s.push_str(syl);
        }
    }
    s.push_str(&format!("{}", rng.range(100, 1000)));
    let sym = *rng.pick(&["!", "#", "$", "%", "&", "@", "*"]);
    s.push_str(sym);
    s
}

/// Код приглашения из 8 шестнадцатеричных символов — детерминированно от зерна.
pub fn invite_code(rng: &mut Rng) -> String {
    let n = rng.next_u64();
    format!("{:08x}", (n & 0xFFFF_FFFF) as u32)
}
