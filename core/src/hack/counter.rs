//! Ответный удар. Когда расследование заходит далеко, наблюдатели бьют в ответ
//! по машине игрока: подсаживают закладки, выписывают штрафы. Игрок защищается —
//! свой файрвол отражает атаки, а закладки надо находить и вычищать, иначе они
//! вредят (тянут на тебя следствие, крадут ₵).

use serde::{Deserialize, Serialize};

use crate::net::machine::{LogKind, MailFolder, MailMessage};
use crate::net::orgs::OrgKind;
use crate::world::World;

/// Вид закладки.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ImplantKind {
    /// Маяк — ускоряет расследование.
    Tracker,
    /// Чёрный ход — через него можно подсадить ещё, минуя файрвол.
    Backdoor,
    /// Майнер — тихо тянет деньги.
    Miner,
}

impl ImplantKind {
    pub fn title(self) -> &'static str {
        match self {
            ImplantKind::Tracker => "маяк (тянет следствие)",
            ImplantKind::Backdoor => "чёрный ход",
            ImplantKind::Miner => "майнер (крадёт ₵)",
        }
    }
}

/// Закладка на машине игрока.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Implant {
    pub id: u32,
    pub path: String,
    pub kind: ImplantKind,
    pub planted: u64,
}

/// Защита игрока.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct PlayerDefense {
    /// Свой файрвол поднят.
    pub shield: bool,
    pub implants: Vec<Implant>,
    pub next_id: u32,
    pub last_counter: u64,
    pub fines_total: i64,
    pub mine_acc: f64,
    #[serde(skip)]
    pub pending: Option<DefenseEvent>,
}

#[derive(Clone, Debug)]
pub struct DefenseEvent {
    pub title: String,
    pub text: String,
}

pub fn take_pending(world: &mut World) -> Option<DefenseEvent> {
    world.defense.pending.take()
}

fn has_backdoor(world: &World) -> bool {
    world.defense.implants.iter().any(|i| i.kind == ImplantKind::Backdoor)
}

/// Уровень угрозы: насколько далеко зашло расследование.
fn under_threat(world: &World) -> bool {
    use crate::hack::watch::InvStage;
    world.player.traced
        || world.player.flags.contains_key("trapped")
        || matches!(world.investigation.stage, InvStage::Investigating | InvStage::Tracing)
}

/// Шаг ответного удара. Зовётся из `World::tick` после слежки.
pub fn tick(world: &mut World, dt: u64) {
    apply_implants(world, dt);

    let now = world.now();
    if now.saturating_sub(world.defense.last_counter) < 40 {
        return;
    }
    if !under_threat(world) {
        return;
    }
    // вероятность контратаки растёт со стадией
    use crate::hack::watch::InvStage;
    let base = match world.investigation.stage {
        InvStage::Tracing => 0.05,
        InvStage::Investigating => 0.025,
        _ => 0.015,
    };
    if !world.rng.chance(base * dt as f64) {
        return;
    }
    world.defense.last_counter = now;

    // файрвол отражает (если нет чёрного хода)
    if world.defense.shield && !has_backdoor(world) {
        world.machines[0].log(now, LogKind::Warn, "входящая атака отражена файрволом");
        world.defense.pending = Some(DefenseEvent { title: "Защита".to_string(), text: "Атаку на твою машину отразил файрвол.".to_string() });
        return;
    }

    let roll = world.rng.range(0, 100);
    if roll < 70 {
        plant_implant(world);
    } else {
        fine(world);
    }
}

fn plant_implant(world: &mut World) {
    let now = world.now();
    let id = world.defense.next_id + 1;
    world.defense.next_id = id;
    let kind = match world.rng.range(0, 3) {
        0 => ImplantKind::Tracker,
        1 => ImplantKind::Backdoor,
        _ => ImplantKind::Miner,
    };
    let path = format!("/tmp/.sys{}", id);
    let _ = world.machines[0].fs.write_text(&path, "watcher implant\n", "root");
    world.machines[0].log(now, LogKind::Warn, "на машину подсажена закладка");
    world.defense.implants.push(Implant { id, path: path.clone(), kind, planted: now });
    world.defense.pending = Some(DefenseEvent {
        title: "Внимание".to_string(),
        text: "Тебе подсадили закладку на машину. Проверь: implants, вычисти: clean.".to_string(),
    });
}

fn fine(world: &mut World) {
    let amount = 100 + world.rng.range(0, 200);
    let pay = amount.min(world.player.money.max(0));
    world.player.money -= pay;
    world.defense.fines_total += pay;
    let domain = world
        .orgs
        .iter()
        .find(|o| matches!(o.kind, OrgKind::Government | OrgKind::CyberPolice))
        .map(|o| o.domain.clone())
        .unwrap_or_else(|| "gov.net".to_string());
    let to = format!("{}@home.net", world.player.handle);
    let now = world.now();
    world.machines[0].push_mail(MailMessage {
        id: 0,
        from: format!("court@{}", domain),
        to,
        subject: "Судебное взыскание".to_string(),
        body: format!("По результатам проверки с вашего счёта списано {} ₵. Прекратите противоправную активность.", pay),
        time: now,
        read: false,
        folder: MailFolder::Inbox,
    });
    world.defense.pending = Some(DefenseEvent { title: "Суд".to_string(), text: format!("Штраф: списано {} ₵.", pay) });
}

/// Эффекты активных закладок (каждую секунду).
fn apply_implants(world: &mut World, dt: u64) {
    if world.defense.implants.is_empty() {
        return;
    }
    let trackers = world.defense.implants.iter().filter(|i| i.kind == ImplantKind::Tracker).count();
    let miners = world.defense.implants.iter().filter(|i| i.kind == ImplantKind::Miner).count();
    if trackers > 0 {
        // маяк подтягивает следствие
        world.investigation.progress = (world.investigation.progress + trackers as f32 * 0.03 * dt as f32).min(100.0);
    }
    if miners > 0 {
        world.defense.mine_acc += miners as f64 * 0.2 * dt as f64;
        let whole = world.defense.mine_acc.floor();
        if whole >= 1.0 && world.player.money > 0 {
            world.defense.mine_acc -= whole;
            world.player.money = (world.player.money - whole as i64).max(0);
        }
    }
}

// ---------------------------------------------------------------- для команд

/// Поднять/опустить свой файрвол.
pub fn set_shield(world: &mut World, on: bool) {
    world.defense.shield = on;
    let now = world.now();
    world.machines[0].log(now, LogKind::Service, if on { "файрвол поднят" } else { "файрвол снят" });
}

/// Убрать закладку по пути (или все). Возвращает число убранных.
pub fn clean(world: &mut World, which: Option<&str>) -> usize {
    let targets: Vec<(u32, String)> = world
        .defense
        .implants
        .iter()
        .filter(|i| which.map(|w| i.path == w || w == "all" || w == "все").unwrap_or(true))
        .map(|i| (i.id, i.path.clone()))
        .collect();
    for (_, path) in &targets {
        let _ = world.machines[0].fs.remove(path, "root", false);
    }
    let ids: Vec<u32> = targets.iter().map(|(id, _)| *id).collect();
    world.defense.implants.retain(|i| !ids.contains(&i.id));
    if !targets.is_empty() {
        world.player.flags.insert("cleaned".to_string(), "1".to_string());
    }
    targets.len()
}
