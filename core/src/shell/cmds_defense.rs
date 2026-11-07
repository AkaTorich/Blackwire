//! Защита своей машины: `shield` (свой файрвол), `implants` (найти закладки),
//! `clean` (вычистить). Логика — `crate::hack::counter`.

use super::{CmdOutput, Ctx};
use crate::types::{OutKind, OutLine};
use crate::world::GameClock;

pub fn shield(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    let a = args.iter().find(|a| !a.starts_with('-')).map(|s| s.as_str());
    let mut out = CmdOutput::ok();
    match a {
        Some("on") | Some("вкл") => {
            crate::hack::counter::set_shield(ctx.world, true);
            out.lines.push(OutLine::success(tr!("Файрвол поднят — входящие атаки будут отражаться.", "Firewall up — incoming attacks will be repelled.")).after(200));
        }
        Some("off") | Some("выкл") => {
            crate::hack::counter::set_shield(ctx.world, false);
            out.push_kind(tr!("Файрвол снят.", "Firewall down."), OutKind::Warn);
        }
        _ => {
            let on = ctx.world.defense.shield;
            let n = ctx.world.defense.implants.len();
            out.push_kind(tr!("Защита своей машины:", "Your machine's defense:"), OutKind::Heading);
            out.lines.push(OutLine::new(trf!("  Файрвол: {}", "  Firewall: {}", if on { tr!("поднят", "up") } else { tr!("снят", "down") }), if on { OutKind::Success } else { OutKind::Warn }));
            out.push(trf!("  Закладок на машине: {}", "  Implants on the machine: {}", n));
            out.push_kind(tr!("  Поднять/снять: shield on | shield off · закладки: implants, clean", "  Raise/lower: shield on | shield off · implants: implants, clean"), OutKind::Dim);
        }
    }
    out
}

pub fn implants(ctx: &mut Ctx<'_>, _args: &[String]) -> CmdOutput {
    let list = ctx.world.defense.implants.clone();
    let mut out = CmdOutput::ok();
    out.push_kind(tr!("Закладки на твоей машине:", "Implants on your machine:"), OutKind::Heading);
    if list.is_empty() {
        out.push_kind(tr!("  Чисто.", "  Clean."), OutKind::Success);
        return out;
    }
    for im in &list {
        out.lines.push(OutLine::new(trf!("  {:<14} {}  (с {})", "  {:<14} {}  (since {})", im.path, im.kind.title(), GameClock::format_short(im.planted)), OutKind::Error));
    }
    out.push_kind(tr!("  Убрать: clean путь · всё: clean all", "  Remove: clean path · everything: clean all"), OutKind::Dim);
    out
}

pub fn clean(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    let which = args.iter().find(|a| !a.starts_with('-')).map(|s| s.as_str());
    let n = crate::hack::counter::clean(ctx.world, which.or(Some("all")));
    let mut out = CmdOutput::ok();
    if n == 0 {
        out.push_kind(tr!("Убирать нечего.", "Nothing to remove."), OutKind::Dim);
    } else {
        out.lines.push(OutLine::success(trf!("Убрано закладок: {}.", "Implants removed: {}.", n)).after(200));
    }
    out
}
