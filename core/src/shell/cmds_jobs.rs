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
        out.push_kind(tr!("Заданий пока нет. Загляни на http://gate.hw/board", "No missions yet. Check http://gate.hw/board"), OutKind::Dim);
        return out;
    }
    for q in ctx.world.plugins.quests.values(){out.push(format!("[{}] {} — {} ({} ₵)",if q.done{"x"}else{" "},q.title,q.objective,q.reward));}
    out.push_kind(tr!("Задания hello-world:", "hello-world missions:"), OutKind::Heading);
    for (i, r) in rows.iter().enumerate() {
        let (mark, kind) = match r.status {
            MissionStatus::Done => ("✓", OutKind::Success),
            MissionStatus::Active => ("●", OutKind::Accent),
            MissionStatus::Locked => ("·", OutKind::Dim),
        };
        let title = if r.status == MissionStatus::Locked { tr!("· · · (закрыто)", "· · · (locked)").to_string() } else { r.title.clone() };
        out.lines.push(OutLine::new(format!("  {} {}. {:<22} {} ₵", mark, i + 1, title, r.reward), kind));
        if r.status == MissionStatus::Active {
            out.lines.push(OutLine::new(trf!("      цель: {}", "      goal: {}", r.objective), OutKind::Normal));
        }
    }
    out.push_kind(trf!("Заработано: {} ₵ · на счету: {} ₵", "Earned: {} ₵ · balance: {} ₵", ctx.world.missions.money_earned, missions::money(ctx.world)), OutKind::Heading);
    out.push_kind(tr!("Подробнее: mission N · доска: http://gate.hw/board", "Details: mission N · board: http://gate.hw/board"), OutKind::Dim);
    out
}

pub fn gigs(ctx: &mut Ctx<'_>, _args: &[String]) -> CmdOutput {
    crate::gigs::ensure(ctx.world);
    let list = crate::gigs::list(ctx.world).to_vec();
    let mut out = CmdOutput::ok();
    out.push_kind(tr!("Подработки (скупщик, broker@hw):", "Gigs (fence, broker@hw):"), OutKind::Heading);
    if list.is_empty() {
        out.push_kind(tr!("  Пока нет заказов.", "  No jobs yet."), OutKind::Dim);
        return out;
    }
    for g in &list {
        out.lines.push(OutLine::new(format!("  • {}", g.objective), OutKind::Normal));
    }
    out.push_kind(trf!("Заработано на подработках: {} ₵ · выполнено заказов: {}", "Earned on gigs: {} ₵ · jobs done: {}", ctx.world.gigs.earned, ctx.world.gigs.done), OutKind::Heading);
    out.push_kind(tr!("Файл шлют на broker@hw; доступ засчитывается сам.", "Send files to broker@hw; access is counted automatically."), OutKind::Dim);
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
        out.push_kind(tr!("Итог пути:", "Summary:"), OutKind::Heading);
        out.push(trf!("  Задания: {}/{} · долг закрыт", "  Missions: {}/{} · debt cleared", s.missions_done, s.missions_total));
        out.push(trf!("  Заработано: {} ₵ · выплачено по кредиту: {} ₵", "  Earned: {} ₵ · paid on the loan: {} ₵", s.money_earned, s.debt_paid));
        out.push(trf!("  Взломано машин: {} · просканировано: {} · улучшений: {}", "  Machines hacked: {} · scanned: {} · upgrades: {}", s.machines_owned, s.scanned, s.upgrades));
        out.push(trf!("  Пик подозрения: {:.0} · пережито рейдов: {}", "  Peak heat: {:.0} · raids survived: {}", s.peak_heat, s.raids));
        out.push_kind(tr!("  Система твоя — оставайся сколько хочешь.", "  The system is yours — stay as long as you like."), OutKind::Success);
    } else {
        out.push_kind(tr!("До свободы осталось:", "Left until freedom:"), OutKind::Heading);
        let m = if s.missions_done == s.missions_total && s.missions_total > 0 { "✓" } else { "·" };
        let d = if s.debt_cleared { "✓" } else { "·" };
        out.lines.push(OutLine::new(trf!("  {} все задания ({}/{})", "  {} all missions ({}/{})", m, s.missions_done, s.missions_total), OutKind::Normal));
        out.lines.push(OutLine::new(trf!("  {} закрыть кредит", "  {} pay off the loan", d), OutKind::Normal));
        out.push_kind(tr!("  Сойдётся то и другое — и ты дома.", "  Get both done — and you're home."), OutKind::Dim);
    }
    out
}

pub fn mission(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    missions::ensure_init(ctx.world);
    let n = args.iter().find(|a| !a.starts_with('-')).and_then(|s| s.parse::<usize>().ok());
    match missions::brief(ctx.world, n) {
        Some((title, brief)) => {
            let mut out = CmdOutput::ok();
            out.push_kind(trf!("Задание: {}", "Mission: {}", title), OutKind::Heading);
            for line in brief.lines() {
                out.push(line.to_string());
            }
            out
        }
        None => CmdOutput::err(tr!("mission: нет такого задания", "mission: no such mission")),
    }
}


pub fn scenario(ctx: &mut Ctx<'_>, _args: &[String]) -> CmdOutput {
    let mut out = CmdOutput::ok();
    match ctx.world.scenarios.active.clone() {
        None => {
            out.push_kind(tr!("Активного сценария нет.", "No active scenario."), OutKind::Dim);
            out.push_kind(tr!("Создай и запусти свой в окне «Редактор сценариев».", "Create and run your own in the Scenario Editor window."), OutKind::Dim);
        }
        Some(s) => {
            out.push_kind(trf!("Сценарий: {}", "Scenario: {}", s.name), OutKind::Heading);
            for line in s.brief.lines() {
                out.push(line.to_string());
            }
            out.push("");
            out.push(trf!("Цель: {} · награда {} ₵", "Goal: {} · reward {} ₵", crate::scenarios::goal_text(&s.goal), s.reward));
            if ctx.world.scenarios.done {
                out.push_kind(tr!("Пройдено.", "Completed."), OutKind::Success);
            } else {
                out.push_kind(tr!("В работе.", "In progress."), OutKind::Accent);
            }
        }
    }
    out
}
