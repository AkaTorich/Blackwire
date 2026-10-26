//! Деньги и долг (содержание). Банк из сюжета забрал у игрока половину дома за
//! непогашенный кредит; остаток долга висит и медленно растёт на проценты.
//! Награды за задания и сбыт добычи дают ₵; ими гасят кредит и покупают в
//! магазине улучшения (`shop`). Погашенный долг — маленькая победа.

pub mod shop;

use serde::{Deserialize, Serialize};

use crate::net::machine::{MailFolder, MailMessage};
use crate::net::orgs::OrgKind;
use crate::world::World;

/// Начальный долг и ставка.
const START_DEBT: i64 = 3000;
const RATE_PER_DAY: f32 = 2.0; // процентов в день
const SELL_PRICE: i64 = 50; // ₵ за один сбытый логин

/// Состояние финансов (в мире).
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Finance {
    pub started: bool,
    pub debt: i64,
    pub rate_per_day: f32,
    pub accrued: f64,
    pub last_tick: u64,
    pub paid_total: i64,
    pub cleared: bool,
    pub creditor: String,
    pub creditor_mail: String,
    #[serde(skip)]
    pub pending: Option<FinanceEvent>,
}

/// Событие — для уведомления оболочки.
#[derive(Clone, Debug)]
pub struct FinanceEvent {
    pub title: String,
    pub text: String,
}

pub fn take_pending(world: &mut World) -> Option<FinanceEvent> {
    world.finance.pending.take()
}

pub fn money(world: &World) -> i64 {
    world.player.money
}

pub fn debt(world: &World) -> i64 {
    world.finance.debt
}

/// Завести кредит при создании мира.
pub fn ensure_init(world: &mut World) {
    if world.finance.started {
        return;
    }
    let (name, domain) = world
        .orgs
        .iter()
        .find(|o| matches!(o.kind, OrgKind::Bank))
        .map(|o| (o.name.clone(), o.domain.clone()))
        .unwrap_or_else(|| ("Банк".to_string(), "bank.net".to_string()));
    world.finance = Finance {
        started: true,
        debt: START_DEBT,
        rate_per_day: RATE_PER_DAY,
        accrued: 0.0,
        last_tick: world.now(),
        paid_total: 0,
        cleared: false,
        creditor: name,
        creditor_mail: format!("credit@{}", domain),
        pending: None,
    };
}

/// Набегают проценты на долг. Зовётся из `World::tick`.
pub fn tick(world: &mut World, dt: u64) {
    ensure_init(world);
    if world.finance.cleared || world.finance.debt <= 0 {
        return;
    }
    let add = world.finance.debt as f64 * (world.finance.rate_per_day as f64 / 100.0 / 86_400.0) * dt as f64;
    world.finance.accrued += add;
    let whole = world.finance.accrued.floor();
    if whole >= 1.0 {
        world.finance.debt += whole as i64;
        world.finance.accrued -= whole;
    }
    world.finance.last_tick = world.now();
}

fn push_bank_mail(world: &mut World, subject: &str, body: &str) {
    let to = format!("{}@home.net", world.player.handle);
    let from = world.finance.creditor_mail.clone();
    let now = world.now();
    world.machines[0].push_mail(MailMessage {
        id: 0,
        from,
        to,
        subject: subject.to_string(),
        body: body.to_string(),
        time: now,
        read: false,
        folder: MailFolder::Inbox,
    });
}

/// Погасить часть долга. `amount` — сколько хочет заплатить (уже число).
pub fn pay(world: &mut World, amount: i64) -> Result<(i64, i64), String> {
    if world.finance.cleared || world.finance.debt <= 0 {
        return Err("долг уже закрыт".to_string());
    }
    let pay = amount.min(world.player.money).min(world.finance.debt);
    if pay <= 0 {
        if world.player.money <= 0 {
            return Err("на счету пусто".to_string());
        }
        return Err("нечего платить".to_string());
    }
    world.player.money -= pay;
    world.finance.debt -= pay;
    world.finance.paid_total += pay;
    let remaining = world.finance.debt;
    let creditor = world.finance.creditor.clone();
    if remaining <= 0 {
        world.finance.debt = 0;
        world.finance.cleared = true;
        push_bank_mail(
            world,
            "Кредит закрыт",
            &format!("{}: ваш долг погашен полностью. Удержание имущества снято. Счёт закрыт.", creditor),
        );
        world.player.flags.insert("debt_cleared".to_string(), "1".to_string());
        world.finance.pending = Some(FinanceEvent { title: creditor, text: "Долг погашен — ты свободен.".to_string() });
    } else {
        push_bank_mail(world, "Платёж принят", &format!("{}: получено {} ₵. Остаток долга: {} ₵.", creditor, pay, remaining));
    }
    Ok((pay, world.finance.debt))
}

/// Купить улучшение по id.
pub fn buy(world: &mut World, id: &str) -> Result<String, String> {
    let item = shop::item(id).ok_or_else(|| format!("нет такого товара: {}", id))?;
    if world.player.upgrades.contains(id) {
        return Err(format!("«{}» уже куплено", item.name));
    }
    if world.player.money < item.price {
        return Err(format!("не хватает: нужно {} ₵, есть {} ₵", item.price, world.player.money));
    }
    world.player.money -= item.price;
    world.player.upgrades.insert(id.to_string());
    let note = shop::apply(world, id);
    Ok(format!("Куплено: {} (−{} ₵). {}", item.name, item.price, note))
}

/// Сбыть найденные логины за ₵.
pub fn sell_creds(world: &mut World) -> (usize, i64) {
    let n = world.player.found_creds.len();
    let earn = n as i64 * SELL_PRICE;
    world.player.found_creds.clear();
    world.player.money += earn;
    (n, earn)
}

/// Страница магазина (HTML) — узел `gate.hw/shop`.
pub fn shop_html(world: &World) -> String {
    let mut s = String::from("<header><h1>Барахолка</h1></header><nav><a href=\"/board\">Доска</a> <a href=\"/manifesto\">Манифест</a></nav><main>");
    s.push_str(&format!(
        "<p>На счету: <b>{} ₵</b> · Долг: <b>{} ₵</b></p><p><small>Покупка — в терминале: <code>buy id</code>. Оплата долга: <code>pay сумма</code>.</small></p><ul>",
        world.player.money, world.finance.debt
    ));
    for it in shop::ITEMS {
        let owned = world.player.upgrades.contains(it.id);
        let mark = if owned { " — <i>куплено</i>" } else { "" };
        s.push_str(&format!("<li><b>{}</b> ({}) — {} ₵ · {}{}</li>", it.name, it.id, it.price, it.desc, mark));
    }
    s.push_str("</ul></main>");
    s
}
