//! Английская версия: новый мир, справка и вывод команд без кириллицы.
//! Отдельный тестовый файл — свой процесс, поэтому переключение языка не задевает
//! остальные тесты (они идут на русском).

use blackwire_core::game::Game;
use blackwire_core::world::World;

fn has_cyrillic(s: &str) -> bool {
    s.chars().any(|c| ('\u{0400}'..='\u{04FF}').contains(&c))
}

/// Первые строки с кириллицей — чтобы при падении сразу было видно, что осталось.
fn cyrillic_lines(s: &str) -> Vec<String> {
    s.lines().filter(|l| has_cyrillic(l)).take(10).map(|l| l.to_string()).collect()
}

#[test]
fn english_world_commands_and_help() {
    blackwire_core::i18n::set("en");

    // Мир на английском: имена, организации, файлы, почта, страницы, журналы.
    let mut g = Game::new(World::new(7, "ghost", "pass"));
    for _ in 0..300 {
        g.tick(1000);
    }
    let save = g.save_json();
    assert!(!has_cyrillic(&save), "English world has Russian text: {:?}", cyrillic_lines(&save.replace("\\n", "\n")));

    // Справка по командам.
    let doc = blackwire_core::shell::shell_doc();
    assert!(!has_cyrillic(&doc), "shell doc: {:?}", cyrillic_lines(&doc));

    // Вывод частых команд.
    let s = g.open_session();
    let mut all = String::new();
    for cmd in [
        "help", "man ls", "ls -l", "cat readme.txt", "missions", "mission 1", "wallet", "debt", "shop", "market",
        "gigs", "news", "achievements", "base", "heat", "trace", "implants", "ps", "df", "free", "uptime",
        "neofetch", "date", "cal", "netstat", "nosuchcommand", "cat /nonexistent", "probe", "exploit",
    ] {
        let r = g.exec(s, cmd);
        for l in r.lines {
            all.push_str(&l.text);
            all.push('\n');
        }
    }
    assert!(!has_cyrillic(&all), "command output: {:?}", cyrillic_lines(&all));

    // Ошибка языка Adder.
    let diag = blackwire_lang::check("x = (1 +\n");
    assert!(diag.iter().all(|d| !has_cyrillic(&d.message)), "{:?}", diag);
}
