//! Хакерский маркет: все скрипты валидны на Adder, покупка ставит файл в ~/scripts.

use blackwire_core::market::{self, MARKET};
use blackwire_core::world::World;

fn world() -> World {
    World::new(7, "ghost", "secret")
}

#[test]
fn all_market_scripts_parse() {
    for it in MARKET {
        let diags = blackwire_lang::check(it.code);
        assert!(diags.is_empty(), "скрипт {} должен разбираться: {:?}", it.id, diags);
    }
}

#[test]
fn buy_installs_script() {
    let mut w = world();
    w.player.money = 1000;
    let it = &MARKET[0];
    let money0 = w.player.money;
    market::buy(&mut w, it.id).expect("покупка");
    assert_eq!(w.player.money, money0 - it.price, "₵ списаны");
    assert!(w.market.bought.contains(it.id));
    let path = format!("/home/ghost/scripts/{}", it.file);
    let text = w.machines[0].fs.read_text(&path).expect("скрипт установлен");
    assert_eq!(text, it.code);
    // повторная покупка не списывает
    let m1 = w.player.money;
    market::buy(&mut w, it.id).expect("переустановка");
    assert_eq!(w.player.money, m1, "второй раз бесплатно");
}

#[test]
fn buy_fails_without_money() {
    let mut w = world();
    w.player.money = 10;
    assert!(market::buy(&mut w, "autocrack").is_err());
    assert!(!w.market.bought.contains("autocrack"));
}

#[test]
fn market_page_lists_scripts() {
    let w = world();
    let html = market::market_html(&w);
    assert!(html.contains("Маркет") && html.contains("Сканер сети"));
}
