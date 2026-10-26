//! Своя база: `base` (список релеев), `rent` (арендовать прокси), `release`
//! (отказаться). Логика — `crate::base`.

use super::{CmdOutput, Ctx};
use crate::types::{OutKind, OutLine};

pub fn base(ctx: &mut Ctx<'_>, _args: &[String]) -> CmdOutput {
    let rented = ctx.world.base.rented.clone();
    let mut out = CmdOutput::ok();
    out.push_kind("Своя база:", OutKind::Heading);
    out.push(format!("  Своя машина: {} ({})", ctx.world.machines[0].hostname, ctx.world.machines[0].ip));
    if rented.is_empty() {
        out.push_kind("  Релеев нет. Арендовать прокси: rent", OutKind::Dim);
    } else {
        out.push_kind("  Арендованные релеи (ходи через них — прячут адрес):", OutKind::Heading);
        for r in &rented {
            out.lines.push(OutLine::new(format!("    {} ({}) · root / {}", r.host, r.ip, r.password), OutKind::Normal));
            out.lines.push(OutLine::new(format!("      вход: ssh root@{} {}", r.ip, r.password), OutKind::Dim));
        }
    }
    out.push_kind(
        format!("  Аренда: {} ₵ сразу + {} ₵/день · на счету {} ₵", crate::base::rent_price(), crate::base::rent_per_day(), ctx.world.player.money),
        OutKind::Dim,
    );
    out
}

pub fn rent(ctx: &mut Ctx<'_>, _args: &[String]) -> CmdOutput {
    match crate::base::rent(ctx.world) {
        Ok(r) => {
            let mut out = CmdOutput::ok();
            out.lines.push(OutLine::success(format!("Арендован релей {} ({}).", r.host, r.ip)).after(300));
            out.push(format!("  Вход: ssh root@{} {}", r.ip, r.password));
            out.push_kind("  Зайди на него и атакуй оттуда — цепочка спрячет твой адрес (proxy).", OutKind::Dim);
            out
        }
        Err(e) => CmdOutput::err(format!("rent: {}", e)),
    }
}

pub fn release(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    let addr = match args.iter().find(|a| !a.starts_with('-')) {
        Some(a) => a.clone(),
        None => return CmdOutput::err("release: укажите адрес релея"),
    };
    match crate::base::release(ctx.world, &addr) {
        Ok(host) => {
            let mut out = CmdOutput::ok();
            out.push_kind(format!("Релей {} отпущен — плата прекращена.", host), OutKind::Normal);
            out
        }
        Err(e) => CmdOutput::err(format!("release: {}", e)),
    }
}
