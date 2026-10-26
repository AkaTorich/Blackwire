//! Деньги и долг: кредит, выплаты, магазин, эффекты покупок, сбыт. Над моделью
//! игровой сети.

use blackwire_core::money;
use blackwire_core::net::machine::FoundCred;
use blackwire_core::session::Session;
use blackwire_core::world::World;

fn world() -> World {
    World::new(7, "ghost", "secret")
}

fn session(w: &World) -> Session {
    let home = w.machines[0].users.home_of("ghost");
    Session::new(1, w.machines[0].id, "ghost", &home)
}

#[test]
fn debt_is_set_on_new_world() {
    let w = world();
    assert!(w.finance.started);
    assert!(w.finance.debt > 0, "у игрока есть долг");
    assert!(!w.finance.creditor.is_empty(), "кредитор назван");
    assert!(!w.finance.cleared);
}

#[test]
fn pay_reduces_debt_and_writes_receipt() {
    let mut w = world();
    w.player.money = 1000;
    let before = w.finance.debt;
    let mail_before = w.machines[0].mail.len();
    let (paid, rest) = money::pay(&mut w, 500).expect("платёж");
    assert_eq!(paid, 500);
    assert_eq!(rest, before - 500);
    assert_eq!(w.player.money, 500);
    assert!(w.machines[0].mail.len() > mail_before, "банк прислал квитанцию");
}

#[test]
fn pay_all_clears_debt() {
    let mut w = world();
    let debt = w.finance.debt;
    w.player.money = debt + 100;
    let (_, rest) = money::pay(&mut w, debt).expect("платёж");
    assert_eq!(rest, 0);
    assert!(w.finance.cleared, "долг закрыт");
    assert_eq!(w.player.flags.get("debt_cleared").map(|s| s.as_str()), Some("1"));
    assert!(w.finance.pending.is_some(), "есть уведомление о свободе");
}

#[test]
fn pay_more_than_money_is_capped() {
    let mut w = world();
    w.player.money = 100;
    let (paid, _) = money::pay(&mut w, 10_000).expect("платёж");
    assert_eq!(paid, 100, "нельзя заплатить больше, чем есть");
    assert_eq!(w.player.money, 0);
}

#[test]
fn buy_deducts_and_applies() {
    let mut w = world();
    w.player.money = 2000;
    let home = w.machines[0].users.home_of("ghost");
    let wl_before = w.machines[0].fs.read_text(&format!("{}/wordlist.txt", home)).map(|s| s.lines().count()).unwrap_or(0);
    money::buy(&mut w, "wordlist").expect("покупка");
    assert!(w.player.upgrades.contains("wordlist"));
    assert_eq!(w.player.money, 1700);
    let wl_after = w.machines[0].fs.read_text(&format!("{}/wordlist.txt", home)).map(|s| s.lines().count()).unwrap_or(0);
    assert!(wl_after > wl_before, "словарь пополнился");
    // повторно нельзя
    assert!(money::buy(&mut w, "wordlist").is_err());
}

#[test]
fn buy_fails_without_money() {
    let mut w = world();
    w.player.money = 10;
    assert!(money::buy(&mut w, "rootkit").is_err(), "не хватает денег");
    assert!(!w.player.upgrades.contains("rootkit"));
}

#[test]
fn vpn_halves_heat() {
    // без туннеля
    let mut a = world();
    let sa = session(&a);
    let target = a.machines.iter().find(|m| m.id != 0 && m.up && blackwire_core::hack::is_monitored(&a, m.id)).map(|m| m.id).unwrap();
    let h_plain = blackwire_core::hack::tracks::attack_heat(&mut a, &sa, target);
    // с туннелем
    let mut b = world();
    b.player.upgrades.insert("vpn".to_string());
    let sb = session(&b);
    let h_vpn = blackwire_core::hack::tracks::attack_heat(&mut b, &sb, target);
    assert!(h_vpn < h_plain, "с туннелем подозрение растёт тише ({} < {})", h_vpn, h_plain);
}

#[test]
fn rootkit_makes_privesc_always_work() {
    let mut w = world();
    w.player.upgrades.insert("rootkit".to_string());
    // зайдём на какую-нибудь машину обычным пользователем
    let id = w.machines.iter().find(|m| m.id != 0 && m.up).map(|m| m.id).unwrap();
    let mut s = session(&w);
    s.push_hop(id, "user", "/home/user");
    let r = blackwire_core::hack::privesc(&mut w, &mut s).expect("privesc");
    assert!(r.ok, "с руткитом повышение прав удаётся всегда");
    assert_eq!(s.user(), "root");
}

#[test]
fn sell_creds_earns_money() {
    let mut w = world();
    w.player.found_creds.push(FoundCred { machine: 1, user: "root".into(), password: "x".into(), note: "".into() });
    w.player.found_creds.push(FoundCred { machine: 2, user: "admin".into(), password: "y".into(), note: "".into() });
    let money0 = w.player.money;
    let (n, earn) = money::sell_creds(&mut w);
    assert_eq!(n, 2);
    assert!(earn > 0);
    assert_eq!(w.player.money, money0 + earn);
    assert!(w.player.found_creds.is_empty(), "логины сбыты");
}

#[test]
fn shop_page_lists_items() {
    let w = world();
    let html = money::shop_html(&w);
    assert!(html.contains("Руткит") && html.contains("Туннель"), "в магазине есть товары");
}
