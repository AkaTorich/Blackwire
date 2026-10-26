//! Готовые скрипты игрока (scripts/*.adr) обязаны разбираться языком Adder.

use blackwire_core::world::World;

#[test]
fn player_scripts_parse() {
    let world = World::new(7, "ghost", "pass");
    let pc = &world.machines[0];
    let home = pc.users.home_of("ghost");
    let dir = format!("{}/scripts", home);
    let entries = pc.fs.list(&dir, "ghost").expect("папка scripts есть");
    assert!(!entries.is_empty(), "в scripts нет файлов");
    for e in entries.iter().filter(|e| e.name.ends_with(".adr")) {
        let src = pc.fs.read(&e.path, "ghost").expect("скрипт читается");
        let diags = blackwire_lang::check(&src);
        assert!(diags.is_empty(), "{}: {:?}", e.path, diags);
    }
}
