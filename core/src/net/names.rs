//! Имена людей разных народов (уважительно, на равных), логины, слабые пароли
//! (тот же список, что в словаре игрока) и генератор сильных паролей.

use crate::rng::Rng;

/// Человек: латинский логин, имя и фамилия (для файлов и подписей).
pub struct Person {
    pub login: &'static str,
    pub first: &'static str,
    pub last: &'static str,
}

const fn p(login: &'static str, first: &'static str, last: &'static str) -> Person {
    Person { login, first, last }
}

/// Пул людей: имена и фамилии разных культур, вперемешку.
pub const PEOPLE: &[Person] = &[
    p("marat", "Марат", "Сафин"),
    p("leyla", "Лейла", "Гаспарян"),
    p("einar", "Эйнар", "Торвальдсен"),
    p("solveyg", "Сольвейг", "Ларсен"),
    p("temuchin", "Тэмуджин", "Батбаяр"),
    p("yadviga", "Ядвига", "Ковальчик"),
    p("kasym", "Касым", "Абдуллаев"),
    p("nanna", "Нанна", "Йенсен"),
    p("rashid", "Рашид", "Хаккани"),
    p("bjorn", "Бьёрн", "Эриксон"),
    p("zuhra", "Зухра", "Ниязова"),
    p("osvald", "Освальд", "Крамер"),
    p("ingrid", "Ингрид", "Халворсен"),
    p("farid", "Фарид", "Мансуров"),
    p("otto", "Отто", "Шнайдер"),
    p("dilnoza", "Дильноза", "Юсупова"),
    p("hugo", "Гуго", "Бертран"),
    p("aiko", "Айко", "Танака"),
    p("miro", "Миро", "Новак"),
    p("galina", "Галина", "Орлова"),
    p("tariq", "Тарик", "Насер"),
    p("sanna", "Санна", "Виртанен"),
    p("liang", "Лян", "Чэнь"),
    p("amara", "Амара", "Диалло"),
    p("viktor", "Виктор", "Петров"),
    p("elda", "Эльда", "Росси"),
    p("yusuf", "Юсуф", "Карими"),
    p("hilde", "Хильда", "Фишер"),
    p("ramon", "Рамон", "Гарсия"),
    p("oksana", "Оксана", "Шевченко"),
    p("dawit", "Давит", "Гебре"),
    p("meret", "Мерет", "Келлер"),
    p("ildar", "Ильдар", "Хакимов"),
    p("astrid", "Астрид", "Бергман"),
    p("noah", "Ной", "Леви"),
    p("parvati", "Парвати", "Шарма"),
    p("goran", "Горан", "Ковачевич"),
    p("fatima", "Фатима", "аль-Захра"),
    p("soren", "Сёрен", "Мадсен"),
    p("mei", "Мэй", "Ван"),
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
