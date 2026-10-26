//! Слежка: центры безопасности и киберполиция ведут расследование против
//! игрока. Раз в такт `sweep` смотрит на улики (журналы с реальным адресом
//! игрока) и подозрение (`heat`), двигает прогресс дела по стадиям, шлёт
//! игроку письма-предупреждения и на пике устраивает рейд: обрывает каналы и
//! временно запрещает наступательные действия. Заметание следов (`clearlogs`)
//! убирает улики — дело стихает.

use serde::{Deserialize, Serialize};

use crate::net::machine::{MachineId, MailFolder, MailMessage};
use crate::net::orgs::OrgKind;
use crate::world::World;

/// Стадия расследования.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InvStage {
    /// Затишье — за игроком не следят.
    Dormant,
    /// Замечена активность.
    Noticed,
    /// Открыто дело.
    Investigating,
    /// Идёт установление источника.
    Tracing,
    /// Рейд (мгновенное событие).
    Raid,
}

impl Default for InvStage {
    fn default() -> Self {
        InvStage::Dormant
    }
}

impl InvStage {
    pub fn title(self) -> &'static str {
        match self {
            InvStage::Dormant => "затишье",
            InvStage::Noticed => "замечена активность",
            InvStage::Investigating => "открыто дело",
            InvStage::Tracing => "идёт пеленг",
            InvStage::Raid => "рейд",
        }
    }
    fn rank(self) -> u8 {
        match self {
            InvStage::Dormant => 0,
            InvStage::Noticed => 1,
            InvStage::Investigating => 2,
            InvStage::Tracing => 3,
            InvStage::Raid => 4,
        }
    }
}

/// Состояние расследования (хранится в мире).
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Investigation {
    pub stage: InvStage,
    /// Прогресс дела, 0..100.
    pub progress: f32,
    /// Ведущий орган (id организации) — киберполиция или центр безопасности.
    pub lead_org: Option<u32>,
    /// Машины, где следствие видит след игрока.
    pub watched: Vec<MachineId>,
    pub last_sweep: u64,
    pub raids: u32,
    /// Игровое время, до которого наступательные действия заблокированы.
    pub locked_until: u64,
    /// Highest warning already sent for this investigation; None supports old saves.
    #[serde(default)]
    pub notified_stage: Option<InvStage>,
    /// Событие для оболочки (не сохраняется).
    #[serde(skip)]
    pub pending: Option<InvEvent>,
}

/// Событие расследования — для уведомления игрока.
#[derive(Clone, Debug)]
pub struct InvEvent {
    pub stage: InvStage,
    pub title: String,
    pub text: String,
    pub raided: bool,
}

/// Идёт ли сейчас блокировка каналов (последствие рейда).
pub fn is_locked(world: &World) -> bool {
    world.now() < world.investigation.locked_until
}

/// Сколько секунд осталось до снятия блокировки.
pub fn lock_remaining(world: &World) -> u64 {
    world.investigation.locked_until.saturating_sub(world.now())
}

/// Забрать событие расследования (обрабатывает оболочка Tauri).
pub fn take_pending(world: &mut World) -> Option<InvEvent> {
    world.investigation.pending.take()
}

/// Орган, ведущий дело: сначала киберполиция, затем центр безопасности.
fn lead_org(world: &World) -> Option<u32> {
    world
        .orgs
        .iter()
        .find(|o| matches!(o.kind, OrgKind::CyberPolice))
        .or_else(|| world.orgs.iter().find(|o| matches!(o.kind, OrgKind::SecurityCenter)))
        .map(|o| o.id)
}

fn stage_for(progress: f32) -> InvStage {
    if progress >= 100.0 {
        InvStage::Raid
    } else if progress >= 70.0 {
        InvStage::Tracing
    } else if progress >= 40.0 {
        InvStage::Investigating
    } else if progress >= 15.0 {
        InvStage::Noticed
    } else {
        InvStage::Dormant
    }
}

/// Один шаг расследования. Зовётся из `World::tick` раз в игровую секунду.
pub fn sweep(world: &mut World, dt: u64) {
    let lead = lead_org(world);
    world.investigation.lead_org = lead;

    let now = world.now();
    let heat = world.player.heat;
    let ev = crate::hack::tracks::evidence(world);
    let total_ev: usize = ev.iter().map(|(_, n)| n).sum();
    world.investigation.watched = ev.iter().map(|(id, _)| *id).collect();
    world.investigation.last_sweep = now;

    let notified = world.investigation.notified_stage.unwrap_or(world.investigation.stage);
    world.investigation.notified_stage = Some(notified);

    // дело растёт от улик и подозрения; без них — стихает
    let active = lead.is_some() && (total_ev > 0 || heat > 20.0);
    let delta = if active {
        heat * 0.004 + total_ev as f32 * 0.02
    } else {
        -0.08
    } * dt as f32;
    let mut progress = (world.investigation.progress + delta).clamp(0.0, 100.0);

    let mut raided = false;
    // Leave two minutes to recover after the two-minute lock expires.
    let raid_ready = world.investigation.locked_until == 0
        || now >= world.investigation.locked_until.saturating_add(120);
    if progress >= 100.0 && lead.is_some() && raid_ready {
        raided = true;
        progress = 55.0;
        world.investigation.raids += 1;
        world.investigation.locked_until = now + 120;
    }
    // A running raid must expire; repeated evidence cannot extend its deadline.
    if !raided && !raid_ready {
        progress = progress.min(99.0);
    }
    world.investigation.progress = progress;
    let new_stage = stage_for(progress);
    world.investigation.stage = new_stage;

    if progress == 0.0 {
        world.investigation.notified_stage = Some(InvStage::Dormant);
    }
    if raided {
        world.investigation.notified_stage = Some(InvStage::Raid);
        emit_raid(world, lead);
    } else if new_stage.rank() > notified.rank() && lead.is_some() {
        world.investigation.notified_stage = Some(new_stage);
        emit_stage(world, lead, new_stage);
    }
}

fn sender_for(kind: OrgKind, domain: &str) -> String {
    match kind {
        OrgKind::CyberPolice => format!("otdel-k@{}", domain),
        OrgKind::SecurityCenter => format!("soc@{}", domain),
        _ => format!("security@{}", domain),
    }
}

fn stage_mail(stage: InvStage, org_name: &str) -> (String, String) {
    match stage {
        InvStage::Noticed => (
            "Замечена подозрительная активность".to_string(),
            format!(
                "{} фиксирует необычные обращения к защищённым узлам с вашего направления.\n\
                 Это предупреждение. Прекратите — и дела не будет.",
                org_name
            ),
        ),
        InvStage::Investigating => (
            "Открыто дело".to_string(),
            format!(
                "По фактам вторжений {} открыл расследование. Мы собираем следы в журналах узлов.\n\
                 Пока источник не установлен. Советуем остановиться и замести следы.",
                org_name
            ),
        ),
        InvStage::Tracing => (
            "Устанавливаем источник".to_string(),
            format!(
                "{}: круг сужен. Мы идём по цепочке ваших подключений от узла к узлу.\n\
                 Ещё немного — и адрес будет наш.",
                org_name
            ),
        ),
        _ => ("Уведомление".to_string(), String::new()),
    }
}

fn emit_stage(world: &mut World, lead: Option<u32>, stage: InvStage) {
    let (sender, org_name) = match lead.and_then(|id| world.org(id)) {
        Some(o) => (sender_for(o.kind, &o.domain), o.name.clone()),
        None => return,
    };
    let (subject, body) = stage_mail(stage, &org_name);
    push_mail_to_player(world, &sender, &subject, &body);
    alert(world, lead, &format!("дело по источнику: {}", stage.title()));
    world.investigation.pending = Some(InvEvent {
        stage,
        title: org_name,
        text: subject,
        raided: false,
    });
}

fn emit_raid(world: &mut World, lead: Option<u32>) {
    let (sender, org_name) = match lead.and_then(|id| world.org(id)) {
        Some(o) => (sender_for(o.kind, &o.domain), o.name.clone()),
        None => return,
    };
    let subject = "Канал заблокирован".to_string();
    let body = format!(
        "{}: источник вычислен. Ваши соединения разорваны, исходящий доступ временно ограничен.\n\
         Это не конец, но впредь работайте через цепочку и подчищайте журналы.",
        org_name
    );
    push_mail_to_player(world, &sender, &subject, &body);
    alert(world, lead, "источник вычислен — рейд");
    world.investigation.pending = Some(InvEvent {
        stage: InvStage::Raid,
        title: org_name,
        text: "Вас вычислили — рейд".to_string(),
        raided: true,
    });
}

fn push_mail_to_player(world: &mut World, from: &str, subject: &str, body: &str) {
    let to = format!("{}@home.net", world.player.handle);
    let now = world.now();
    // Keep repeated system alerts in one inbox message, including across saves.
    if let Some(mail) = world.machines[0].mail.iter_mut().rev().find(|m| {
        m.folder == MailFolder::Inbox && m.from == from && m.subject == subject && m.body == body
    }) {
        mail.time = now;
        mail.read = false;
        return;
    }
    world.machines[0].push_mail(MailMessage {
        id: 0,
        from: from.to_string(),
        to,
        subject: subject.to_string(),
        body: body.to_string(),
        time: now,
        read: false,
        folder: MailFolder::Inbox,
    });
}

/// Отметка в журнале тревог машин ведущего органа (кто вломится — прочтёт дело о себе).
fn alert(world: &mut World, lead: Option<u32>, text: &str) {
    let now = world.now();
    let members: Vec<MachineId> = match lead.and_then(|id| world.org(id)) {
        Some(o) => o.members.clone(),
        None => return,
    };
    let stamp = crate::world::GameClock::format_hms(now);
    for id in members {
        let has_monitor = world.machine(id).service_named("monitor").is_some();
        if !has_monitor {
            continue;
        }
        let m = world.machine_mut(id);
        let line = format!("[{}] {}\n", stamp, text);
        let prev = m.fs.read_text("/var/log/alerts.log").unwrap_or_default();
        let _ = m.fs.write_text("/var/log/alerts.log", &format!("{}{}", prev, line), "root");
    }
}
