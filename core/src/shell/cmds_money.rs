//! Деньги и долг: `wallet` (счёт), `debt` (кредит), `pay` (погасить),
//! `shop` (магазин), `buy` (купить), `sell` (сбыть добычу). Логика — `crate::money`.

use super::{CmdOutput, Ctx};
use crate::money;
use crate::types::{OutKind, OutLine};

pub fn wallet(ctx: &mut Ctx<'_>, _args: &[String]) -> CmdOutput {
    let mut out = CmdOutput::ok();
    out.push_kind(format!("На счету: {} ₵", money::money(ctx.world)), OutKind::Heading);
    out.push(format!("Заработано на заданиях: {} ₵", ctx.world.missions.money_earned));
    let d = money::debt(ctx.world);
    if ctx.world.finance.cleared {
        out.push_kind("Кредит закрыт.", OutKind::Success);
    } else {
        out.lines.push(OutLine::new(format!("Долг банку: {} ₵", d), OutKind::Warn));
    }
    out
}

pub fn debt(ctx: &mut Ctx<'_>, _args: &[String]) -> CmdOutput {
    let f = &ctx.world.finance;
    let mut out = CmdOutput::ok();
    out.push_kind("Кредит:", OutKind::Heading);
    out.push(format!("  Кредитор: {}", f.creditor));
    if f.cleared {
        out.push_kind("  Долг погашен. Имущество больше не удерживают.", OutKind::Success);
    } else {
        out.lines.push(OutLine::new(format!("  Остаток: {} ₵  (ставка {:.0}%/день)", f.debt, f.rate_per_day), OutKind::Warn));
    }
    out.push(format!("  Выплачено всего: {} ₵ · на счету: {} ₵", f.paid_total, ctx.world.player.money));
    if !f.cleared {
        out.push_kind("  Погасить: pay сумма  (или pay all)", OutKind::Dim);
    }
    out
}

pub fn pay(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    let arg = match args.iter().find(|a| !a.starts_with('-')) {
        Some(a) => a.clone(),
        None => return CmdOutput::err("pay: сколько платить? pay сумма | pay all"),
    };
    let amount = if arg.eq_ignore_ascii_case("all") || arg == "всё" || arg == "все" {
        ctx.world.player.money.min(ctx.world.finance.debt)
    } else {
        match arg.parse::<i64>() {
            Ok(n) if n > 0 => n,
            _ => return CmdOutput::err("pay: укажите положительную сумму или all"),
        }
    };
    match money::pay(ctx.world, amount) {
        Ok((paid, rest)) => {
            let mut out = CmdOutput::ok();
            out.lines.push(OutLine::success(format!("Уплачено {} ₵.", paid)).after(200));
            if ctx.world.finance.cleared {
                out.push_kind("Кредит закрыт. Ты больше им ничего не должен.", OutKind::Success);
            } else {
                out.push(format!("Остаток долга: {} ₵.", rest));
            }
            out
        }
        Err(e) => CmdOutput::err(format!("pay: {}", e)),
    }
}

pub fn shop(ctx: &mut Ctx<'_>, _args: &[String]) -> CmdOutput {
    let mut out = CmdOutput::ok();
    out.push_kind("Барахолка — улучшения за ₵:", OutKind::Heading);
    for it in money::shop::ITEMS {
        let owned = ctx.world.player.upgrades.contains(it.id);
        let kind = if owned { OutKind::Dim } else { OutKind::Normal };
        let mark = if owned { "  [куплено]" } else { "" };
        out.lines.push(OutLine::new(format!("  {:<9} {:>5} ₵  {}{}", it.id, it.price, it.name, mark), kind));
        out.lines.push(OutLine::new(format!("            {}", it.desc), OutKind::Dim));
    }
    out.push_kind(format!("На счету: {} ₵ · долг: {} ₵", ctx.world.player.money, ctx.world.finance.debt), OutKind::Heading);
    out.push_kind("Купить: buy id · сбыть логины: sell", OutKind::Dim);
    out
}

pub fn buy(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    let id = match args.iter().find(|a| !a.starts_with('-')) {
        Some(a) => a.clone(),
        None => return CmdOutput::err("buy: что купить? список — shop"),
    };
    match money::buy(ctx.world, &id) {
        Ok(msg) => {
            let mut out = CmdOutput::ok();
            out.lines.push(OutLine::success(msg).after(200));
            out
        }
        Err(e) => CmdOutput::err(format!("buy: {}", e)),
    }
}

pub fn market(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    let pos: Vec<&String> = args.iter().filter(|a| !a.starts_with('-')).collect();
    if pos.first().map(|s| s.as_str()) == Some("buy") {
        let id = match pos.get(1) {
            Some(a) => a.to_string(),
            None => return CmdOutput::err("market: что купить? market buy id (список — market)"),
        };
        return match crate::market::buy(ctx.world, &id) {
            Ok(msg) => {
                let mut out = CmdOutput::ok();
                out.lines.push(OutLine::success(msg).after(200));
                out
            }
            Err(e) => CmdOutput::err(format!("market: {}", e)),
        };
    }
    let mut out = CmdOutput::ok();
    out.push_kind("Маркет скриптов (сеть hw):", OutKind::Heading);
    for it in crate::market::MARKET {
        let owned = ctx.world.market.bought.contains(it.id);
        let mark = if owned { "  [куплено]" } else { "" };
        let kind = if owned { OutKind::Dim } else { OutKind::Normal };
        out.lines.push(OutLine::new(format!("  {:<10} {:>5} ₵  {}{}", it.id, it.price, it.name, mark), kind));
        out.lines.push(OutLine::new(format!("            {}", it.desc), OutKind::Dim));
    }
    out.push_kind(format!("На счету: {} ₵ · купить: market buy id", ctx.world.player.money), OutKind::Heading);
    out.push_kind("Купленное — в ~/scripts, запуск: adder scripts/имя.adr", OutKind::Dim);
    out
}

pub fn sell(ctx: &mut Ctx<'_>, _args: &[String]) -> CmdOutput {
    let (n, earn) = money::sell_creds(ctx.world);
    let mut out = CmdOutput::ok();
    if n == 0 {
        out.push_kind("Сбывать нечего — найденных логинов нет.", OutKind::Dim);
    } else {
        out.lines.push(OutLine::success(format!("Сбыто логинов: {} за {} ₵.", n, earn)).after(200));
        out.push(format!("На счету: {} ₵.", ctx.world.player.money));
    }
    out
}
