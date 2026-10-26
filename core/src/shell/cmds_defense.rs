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
            out.lines.push(OutLine::success("Файрвол поднят — входящие атаки будут отражаться.").after(200));
        }
        Some("off") | Some("выкл") => {
            crate::hack::counter::set_shield(ctx.world, false);
            out.push_kind("Файрвол снят.", OutKind::Warn);
        }
        _ => {
            let on = ctx.world.defense.shield;
            let n = ctx.world.defense.implants.len();
            out.push_kind("Защита своей машины:", OutKind::Heading);
            out.lines.push(OutLine::new(format!("  Файрвол: {}", if on { "поднят" } else { "снят" }), if on { OutKind::Success } else { OutKind::Warn }));
            out.push(format!("  Закладок на машине: {}", n));
            out.push_kind("  Поднять/снять: shield on | shield off · закладки: implants, clean", OutKind::Dim);
        }
    }
    out
}

pub fn implants(ctx: &mut Ctx<'_>, _args: &[String]) -> CmdOutput {
    let list = ctx.world.defense.implants.clone();
    let mut out = CmdOutput::ok();
    out.push_kind("Закладки на твоей машине:", OutKind::Heading);
    if list.is_empty() {
        out.push_kind("  Чисто.", OutKind::Success);
        return out;
    }
    for im in &list {
        out.lines.push(OutLine::new(format!("  {:<14} {}  (с {})", im.path, im.kind.title(), GameClock::format_short(im.planted)), OutKind::Error));
    }
    out.push_kind("  Убрать: clean путь · всё: clean all", OutKind::Dim);
    out
}

pub fn clean(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    let which = args.iter().find(|a| !a.starts_with('-')).map(|s| s.as_str());
    let n = crate::hack::counter::clean(ctx.world, which.or(Some("all")));
    let mut out = CmdOutput::ok();
    if n == 0 {
        out.push_kind("Убирать нечего.", OutKind::Dim);
    } else {
        out.lines.push(OutLine::success(format!("Убрано закладок: {}.", n)).after(200));
    }
    out
}
