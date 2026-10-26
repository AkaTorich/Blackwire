//! Команды заданий группы hello-world: `missions` (список и деньги),
//! `mission N` (брифинг), `wallet` (счёт). Логика — в `crate::missions`.

use super::{CmdOutput, Ctx};
use crate::missions::{self, MissionStatus};
use crate::types::{OutKind, OutLine};

pub fn missions(ctx: &mut Ctx<'_>, _args: &[String]) -> CmdOutput {
    // освежить цепочку и сразу проверить выполнение текущего задания
    missions::check(ctx.world);
    let rows = missions::rows(ctx.world);
    let mut out = CmdOutput::ok();
    if rows.is_empty() {
        out.push_kind("Заданий пока нет. Загляни на http://gate.hw/board", OutKind::Dim);
        return out;
    }
    for q in ctx.world.plugins.quests.values(){out.push(format!("[{}] {} — {} ({} ₵)",if q.done{"x"}else{" "},q.title,q.objective,q.reward));}
    out.push_kind("Задания hello-world:", OutKind::Heading);
    for (i, r) in rows.iter().enumerate() {
        let (mark, kind) = match r.status {
            MissionStatus::Done => ("✓", OutKind::Success),
            MissionStatus::Active => ("●", OutKind::Accent),
            MissionStatus::Locked => ("·", OutKind::Dim),
        };
        let title = if r.status == MissionStatus::Locked { "· · · (закрыто)".to_string() } else { r.title.clone() };
        out.lines.push(OutLine::new(format!("  {} {}. {:<22} {} ₵", mark, i + 1, title, r.reward), kind));
        if r.status == MissionStatus::Active {
            out.lines.push(OutLine::new(format!("      цель: {}", r.objective), OutKind::Normal));
        }
    }
    out.push_kind(format!("Заработано: {} ₵ · на счету: {} ₵", ctx.world.missions.money_earned, missions::money(ctx.world)), OutKind::Heading);
    out.push_kind("Подробнее: mission N · доска: http://gate.hw/board", OutKind::Dim);
    out
}

pub fn gigs(ctx: &mut Ctx<'_>, _args: &[String]) -> CmdOutput {
    crate::gigs::ensure(ctx.world);
    let list = crate::gigs::list(ctx.world).to_vec();
    let mut out = CmdOutput::ok();
    out.push_kind("Подработки (скупщик, broker@hw):", OutKind::Heading);
    if list.is_empty() {
        out.push_kind("  Пока нет заказов.", OutKind::Dim);
        return out;
    }
    for g in &list {
        out.lines.push(OutLine::new(format!("  • {}", g.objective), OutKind::Normal));
    }
    out.push_kind(format!("Заработано на подработках: {} ₵ · выполнено заказов: {}", ctx.world.gigs.earned, ctx.world.gigs.done), OutKind::Heading);
    out.push_kind("Файл шлют на broker@hw; доступ засчитывается сам.", OutKind::Dim);
    out
}

pub fn ending(ctx: &mut Ctx<'_>, _args: &[String]) -> CmdOutput {
    let s = crate::ending::summary(ctx.world);
    let mut out = CmdOutput::ok();
    if s.won {
        out.push_kind("hello-world", OutKind::Heading);
        for line in s.text.lines() {
            if line.is_empty() {
                out.push("");
            } else {
                out.push(line.to_string());
            }
        }
        out.push("");
        out.push_kind("Итог пути:", OutKind::Heading);
        out.push(format!("  Задания: {}/{} · долг закрыт", s.missions_done, s.missions_total));
        out.push(format!("  Заработано: {} ₵ · выплачено по кредиту: {} ₵", s.money_earned, s.debt_paid));
        out.push(format!("  Взломано машин: {} · просканировано: {} · улучшений: {}", s.machines_owned, s.scanned, s.upgrades));
        out.push(format!("  Пик подозрения: {:.0} · пережито рейдов: {}", s.peak_heat, s.raids));
        out.push_kind("  Система твоя — оставайся сколько хочешь.", OutKind::Success);
    } else {
        out.push_kind("До свободы осталось:", OutKind::Heading);
        let m = if s.missions_done == s.missions_total && s.missions_total > 0 { "✓" } else { "·" };
        let d = if s.debt_cleared { "✓" } else { "·" };
        out.lines.push(OutLine::new(format!("  {} все задания ({}/{})", m, s.missions_done, s.missions_total), OutKind::Normal));
        out.lines.push(OutLine::new(format!("  {} закрыть кредит", d), OutKind::Normal));
        out.push_kind("  Сойдётся то и другое — и ты дома.", OutKind::Dim);
    }
    out
}

pub fn mission(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    missions::ensure_init(ctx.world);
    let n = args.iter().find(|a| !a.starts_with('-')).and_then(|s| s.parse::<usize>().ok());
    match missions::brief(ctx.world, n) {
        Some((title, brief)) => {
            let mut out = CmdOutput::ok();
            out.push_kind(format!("Задание: {}", title), OutKind::Heading);
            for line in brief.lines() {
                out.push(line.to_string());
            }
            out
        }
        None => CmdOutput::err("mission: нет такого задания"),
    }
}


pub fn scenario(ctx: &mut Ctx<'_>, _args: &[String]) -> CmdOutput {
    let mut out = CmdOutput::ok();
    match ctx.world.scenarios.active.clone() {
        None => {
            out.push_kind("Активного сценария нет.", OutKind::Dim);
            out.push_kind("Создай и запусти свой в окне «Редактор сценариев».", OutKind::Dim);
        }
        Some(s) => {
            out.push_kind(format!("Сценарий: {}", s.name), OutKind::Heading);
            for line in s.brief.lines() {
                out.push(line.to_string());
            }
            out.push("");
            out.push(format!("Цель: {} · награда {} ₵", crate::scenarios::goal_text(&s.goal), s.reward));
            if ctx.world.scenarios.done {
                out.push_kind("Пройдено.", OutKind::Success);
            } else {
                out.push_kind("В работе.", OutKind::Accent);
            }
        }
    }
    out
}
