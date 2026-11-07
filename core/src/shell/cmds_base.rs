//! Своя база: `base` (список релеев), `rent` (арендовать прокси), `release`
//! (отказаться). Логика — `crate::base`.

use super::{CmdOutput, Ctx};
use crate::types::{OutKind, OutLine};

pub fn base(ctx: &mut Ctx<'_>, _args: &[String]) -> CmdOutput {
    let rented = ctx.world.base.rented.clone();
    let mut out = CmdOutput::ok();
    out.push_kind(tr!("Своя база:", "Your base:"), OutKind::Heading);
    out.push(trf!("  Своя машина: {} ({})", "  Your machine: {} ({})", ctx.world.machines[0].hostname, ctx.world.machines[0].ip));
    if rented.is_empty() {
        out.push_kind(tr!("  Релеев нет. Арендовать прокси: rent", "  No relays. Rent a proxy: rent"), OutKind::Dim);
    } else {
        out.push_kind(tr!("  Арендованные релеи (ходи через них — прячут адрес):", "  Rented relays (route through them — they hide your address):"), OutKind::Heading);
        for r in &rented {
            out.lines.push(OutLine::new(format!("    {} ({}) · root / {}", r.host, r.ip, r.password), OutKind::Normal));
            out.lines.push(OutLine::new(trf!("      вход: ssh root@{} {}", "      login: ssh root@{} {}", r.ip, r.password), OutKind::Dim));
        }
    }
    out.push_kind(
        trf!("  Аренда: {} ₵ сразу + {} ₵/день · на счету {} ₵", "  Rent: {} ₵ upfront + {} ₵/day · balance {} ₵", crate::base::rent_price(), crate::base::rent_per_day(), ctx.world.player.money),
        OutKind::Dim,
    );
    out
}

pub fn rent(ctx: &mut Ctx<'_>, _args: &[String]) -> CmdOutput {
    match crate::base::rent(ctx.world) {
        Ok(r) => {
            let mut out = CmdOutput::ok();
            out.lines.push(OutLine::success(trf!("Арендован релей {} ({}).", "Relay {} ({}) rented.", r.host, r.ip)).after(300));
            out.push(trf!("  Вход: ssh root@{} {}", "  Login: ssh root@{} {}", r.ip, r.password));
            out.push_kind(tr!("  Зайди на него и атакуй оттуда — цепочка спрячет твой адрес (proxy).", "  Log in and attack from there — the chain hides your address (proxy)."), OutKind::Dim);
            out
        }
        Err(e) => CmdOutput::err(format!("rent: {}", e)),
    }
}

pub fn release(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    let addr = match args.iter().find(|a| !a.starts_with('-')) {
        Some(a) => a.clone(),
        None => return CmdOutput::err(tr!("release: укажите адрес релея", "release: specify the relay address")),
    };
    match crate::base::release(ctx.world, &addr) {
        Ok(host) => {
            let mut out = CmdOutput::ok();
            out.push_kind(trf!("Релей {} отпущен — плата прекращена.", "Relay {} released — rent stopped.", host), OutKind::Normal);
            out
        }
        Err(e) => CmdOutput::err(format!("release: {}", e)),
    }
}
