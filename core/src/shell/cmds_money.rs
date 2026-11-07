//! Деньги и долг: `wallet` (счёт), `debt` (кредит), `pay` (погасить),
//! `shop` (магазин), `buy` (купить), `sell` (сбыть добычу). Логика — `crate::money`.

use super::{CmdOutput, Ctx};
use crate::money;
use crate::types::{OutKind, OutLine};

pub fn wallet(ctx: &mut Ctx<'_>, _args: &[String]) -> CmdOutput {
    let mut out = CmdOutput::ok();
    out.push_kind(trf!("На счету: {} ₵", "Balance: {} ₵", money::money(ctx.world)), OutKind::Heading);
    out.push(trf!("Заработано на заданиях: {} ₵", "Earned on missions: {} ₵", ctx.world.missions.money_earned));
    let d = money::debt(ctx.world);
    if ctx.world.finance.cleared {
        out.push_kind(tr!("Кредит закрыт.", "Loan paid off."), OutKind::Success);
    } else {
        out.lines.push(OutLine::new(trf!("Долг банку: {} ₵", "Debt to the bank: {} ₵", d), OutKind::Warn));
    }
    out
}

pub fn debt(ctx: &mut Ctx<'_>, _args: &[String]) -> CmdOutput {
    let f = &ctx.world.finance;
    let mut out = CmdOutput::ok();
    out.push_kind(tr!("Кредит:", "Loan:"), OutKind::Heading);
    out.push(trf!("  Кредитор: {}", "  Lender: {}", f.creditor));
    if f.cleared {
        out.push_kind(tr!("  Долг погашен. Имущество больше не удерживают.", "  Debt repaid. Your property is no longer held."), OutKind::Success);
    } else {
        out.lines.push(OutLine::new(trf!("  Остаток: {} ₵  (ставка {:.0}%/день)", "  Remaining: {} ₵  (rate {:.0}%/day)", f.debt, f.rate_per_day), OutKind::Warn));
    }
    out.push(trf!("  Выплачено всего: {} ₵ · на счету: {} ₵", "  Paid in total: {} ₵ · balance: {} ₵", f.paid_total, ctx.world.player.money));
    if !f.cleared {
        out.push_kind(tr!("  Погасить: pay сумма  (или pay all)", "  Repay: pay amount  (or pay all)"), OutKind::Dim);
    }
    out
}

pub fn pay(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    let arg = match args.iter().find(|a| !a.starts_with('-')) {
        Some(a) => a.clone(),
        None => return CmdOutput::err(tr!("pay: сколько платить? pay сумма | pay all", "pay: how much? pay amount | pay all")),
    };
    let amount = if arg.eq_ignore_ascii_case("all") || arg == "всё" || arg == "все" {
        ctx.world.player.money.min(ctx.world.finance.debt)
    } else {
        match arg.parse::<i64>() {
            Ok(n) if n > 0 => n,
            _ => return CmdOutput::err(tr!("pay: укажите положительную сумму или all", "pay: specify a positive amount or all")),
        }
    };
    match money::pay(ctx.world, amount) {
        Ok((paid, rest)) => {
            let mut out = CmdOutput::ok();
            out.lines.push(OutLine::success(trf!("Уплачено {} ₵.", "Paid {} ₵.", paid)).after(200));
            if ctx.world.finance.cleared {
                out.push_kind(tr!("Кредит закрыт. Ты больше им ничего не должен.", "Loan paid off. You owe them nothing anymore."), OutKind::Success);
            } else {
                out.push(trf!("Остаток долга: {} ₵.", "Debt remaining: {} ₵.", rest));
            }
            out
        }
        Err(e) => CmdOutput::err(format!("pay: {}", e)),
    }
}

pub fn shop(ctx: &mut Ctx<'_>, _args: &[String]) -> CmdOutput {
    let mut out = CmdOutput::ok();
    out.push_kind(tr!("Барахолка — улучшения за ₵:", "Flea market — upgrades for ₵:"), OutKind::Heading);
    for (id,it) in crate::plugins::catalog::entries(ctx.world,"shop"){out.push(format!("{}  {} ₵  {} — {}",id,it.price,it.name,it.desc));}
    for it in money::shop::ITEMS {
        let owned = ctx.world.player.upgrades.contains(it.id);
        let kind = if owned { OutKind::Dim } else { OutKind::Normal };
        let mark = if owned { tr!("  [куплено]", "  [owned]") } else { "" };
        out.lines.push(OutLine::new(format!("  {:<9} {:>5} ₵  {}{}", it.id, it.price, it.name(), mark), kind));
        out.lines.push(OutLine::new(format!("            {}", it.desc()), OutKind::Dim));
    }
    out.push_kind(trf!("На счету: {} ₵ · долг: {} ₵", "Balance: {} ₵ · debt: {} ₵", ctx.world.player.money, ctx.world.finance.debt), OutKind::Heading);
    out.push_kind(tr!("Купить: buy id · сбыть логины: sell", "Buy: buy id · sell logins: sell"), OutKind::Dim);
    out
}

pub fn buy(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    let id = match args.iter().find(|a| !a.starts_with('-')) {
        Some(a) => a.clone(),
        None => return CmdOutput::err(tr!("buy: что купить? список — shop", "buy: buy what? list — shop")),
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
            None => return CmdOutput::err(tr!("market: что купить? market buy id (список — market)", "market: buy what? market buy id (list — market)")),
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
    out.push_kind(tr!("Маркет скриптов (сеть hw):", "Script market (hw network):"), OutKind::Heading);
    for (id,it) in crate::plugins::catalog::entries(ctx.world,"market"){out.push(format!("{}  {} ₵  {} — {}",id,it.price,it.name,it.desc));}
    for it in crate::market::MARKET {
        let owned = ctx.world.market.bought.contains(it.id);
        let mark = if owned { tr!("  [куплено]", "  [owned]") } else { "" };
        let kind = if owned { OutKind::Dim } else { OutKind::Normal };
        out.lines.push(OutLine::new(format!("  {:<10} {:>5} ₵  {}{}", it.id, it.price, it.name(), mark), kind));
        out.lines.push(OutLine::new(format!("            {}", it.desc()), OutKind::Dim));
    }
    out.push_kind(trf!("На счету: {} ₵ · купить: market buy id", "Balance: {} ₵ · buy: market buy id", ctx.world.player.money), OutKind::Heading);
    out.push_kind(tr!("Купленное — в ~/scripts, запуск: adder scripts/имя.adr", "Purchases go to ~/scripts, run: adder scripts/name.adr"), OutKind::Dim);
    out
}

pub fn sell(ctx: &mut Ctx<'_>, _args: &[String]) -> CmdOutput {
    let (n, earn) = money::sell_creds(ctx.world);
    let mut out = CmdOutput::ok();
    if n == 0 {
        out.push_kind(tr!("Сбывать нечего — найденных логинов нет.", "Nothing to sell — no logins found."), OutKind::Dim);
    } else {
        out.lines.push(OutLine::success(trf!("Сбыто логинов: {} за {} ₵.", "Logins sold: {} for {} ₵.", n, earn)).after(200));
        out.push(trf!("На счету: {} ₵.", "Balance: {} ₵.", ctx.world.player.money));
    }
    out
}
