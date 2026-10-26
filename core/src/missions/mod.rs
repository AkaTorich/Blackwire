//! Задания группы hello-world — цепочка миссий (третий этап содержания).
//! Движок находит в мире цели по ролям, формирует брифинги с настоящими
//! именами и адресами, проверяет выполнение по состоянию мира, платит награду
//! и шлёт ответ группы со следующим заданием. Содержание — `data.rs`.

pub mod data;

use serde::{Deserialize, Serialize};

use crate::net::machine::{MachineId, MachineKind, MailFolder, MailMessage};
use crate::net::orgs::OrgKind;
use crate::world::World;

use data::{ActionKind, TargetRole, CHAIN, DROP};

/// Проверяемое условие выполнения.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Goal {
    /// Прислать содержимое файла цели на адрес группы (ищем в «Отправленных»).
    ReadDeliver { target: MachineId, path: String, to: String, needle: String },
    /// Получить доступ к машине.
    Own { target: MachineId },
    /// Создать на машине файл с меткой.
    Plant { target: MachineId, path: String, needle: String },
    /// Замести следы: подозрение ниже порога и улик нет.
    CoverTracks { heat_below: f32 },
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MissionStatus {
    Locked,
    Active,
    Done,
}

/// Задание с привязкой к конкретной машине и готовыми текстами.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MissionRun {
    pub code: String,
    pub title: String,
    pub objective: String,
    pub brief: String,
    pub reply: String,
    pub reward: i64,
    pub goal: Goal,
    pub status: MissionStatus,
}

/// Состояние цепочки (в мире).
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct MissionState {
    pub initialized: bool,
    /// Индекс текущего задания (== длине, когда всё пройдено).
    pub active: usize,
    pub missions: Vec<MissionRun>,
    pub money_earned: i64,
    #[serde(skip)]
    pub pending: Option<MissionEvent>,
}

/// Событие — для уведомления оболочки.
#[derive(Clone, Debug)]
pub struct MissionEvent {
    pub title: String,
    pub text: String,
}

pub fn take_pending(world: &mut World) -> Option<MissionEvent> {
    world.missions.pending.take()
}

pub fn money(world: &World) -> i64 {
    world.player.money
}

// ---------------------------------------------------------------- поиск целей

fn is_public(world: &World, id: MachineId) -> bool {
    world.machine(id).is_public()
}

fn vulnerable(world: &World, id: MachineId) -> bool {
    !crate::hack::remote_vulns(world.machine(id)).is_empty()
}

fn org_kind(world: &World, id: MachineId) -> Option<OrgKind> {
    world.org_of(id).map(|o| o.kind)
}

/// Подобрать читаемый файл на машине и характерную строку из него.
fn pick_file(world: &World, id: MachineId, roots: &[&str], names: &[&str]) -> Option<(String, String)> {
    let fs = &world.machine(id).fs;
    for root in roots {
        for pat in names {
            for p in fs.find(root, pat) {
                if let Ok(txt) = fs.read_text(&p) {
                    if let Some(n) = needle_of(&txt) {
                        return Some((p, n));
                    }
                }
            }
        }
    }
    None
}

/// Читаемый личный файл домашней машины (есть только если на ней живёт человек).
fn home_file(world: &World, id: MachineId) -> Option<(String, String)> {
    pick_file(world, id, &["/home", "/root"], &["notes.txt", "passwords.txt", "todo.txt", "contract_01.txt", "*.txt", ".bash_history"])
}

/// Характерная строка файла: самая длинная в разумных пределах.
fn needle_of(text: &str) -> Option<String> {
    let mut best: Option<&str> = None;
    for line in text.lines() {
        let t = line.trim();
        if t.len() < 12 || t.len() > 120 {
            continue;
        }
        if best.map(|b| t.len() > b.len()).unwrap_or(true) {
            best = Some(t);
        }
    }
    best.or_else(|| text.lines().map(|l| l.trim()).find(|l| !l.is_empty() && l.len() <= 120))
        .map(|s| s.to_string())
}

/// Первая машина, удовлетворяющая условию (по порядку — детерминированно).
fn first_machine(world: &World, pred: impl Fn(&World, MachineId) -> bool) -> Option<MachineId> {
    world.machines.iter().map(|m| m.id).find(|id| *id != 0 && world.machine(*id).up && pred(world, *id))
}

fn resolve_target(world: &World, role: TargetRole) -> Option<MachineId> {
    let target = match role {
        TargetRole::None => None,
        TargetRole::HomePc => first_machine(world, |w, id| {
            matches!(w.machine(id).kind, MachineKind::HomePc) && is_public(w, id) && vulnerable(w, id) && home_file(w, id).is_some()
        })
        .or_else(|| first_machine(world, |w, id| matches!(w.machine(id).kind, MachineKind::HomePc) && is_public(w, id) && vulnerable(w, id)))
        .or_else(|| {
            first_machine(world, |w, id| {
                is_public(w, id) && vulnerable(w, id) && !matches!(w.machine(id).kind, MachineKind::Router | MachineKind::Gateway)
            })
        }),
        TargetRole::BankLedger => first_machine(world, |w, id| {
            org_kind(w, id) == Some(OrgKind::Bank)
                && (w.machine(id).service_named("ledger").is_some() || matches!(w.machine(id).kind, MachineKind::Database))
        }),
        TargetRole::InternalServer => first_machine(world, |w, id| {
            !is_public(w, id)
                && vulnerable(w, id)
                && w.org_of(id).map(|o| o.gateway != id).unwrap_or(false)
                && matches!(w.machine(id).kind, MachineKind::FileServer | MachineKind::Database | MachineKind::Workstation)
        }),
        TargetRole::HospitalRecords => first_machine(world, |w, id| {
            org_kind(w, id) == Some(OrgKind::Hospital)
                && matches!(w.machine(id).kind, MachineKind::Database | MachineKind::FileServer | MachineKind::MedicalDevice)
        }),
        TargetRole::PublicWeb => first_machine(world, |w, id| {
            is_public(w, id)
                && w.machine(id).service_named("http").is_some()
                && vulnerable(w, id)
                && matches!(org_kind(w, id), Some(OrgKind::Media | OrgKind::Government | OrgKind::Shop | OrgKind::Hosting | OrgKind::University))
        }),
        TargetRole::Watchdog => first_machine(world, |w, id| {
            matches!(org_kind(w, id), Some(OrgKind::CyberPolice | OrgKind::SecurityCenter)) && is_public(w, id) && vulnerable(w, id)
        })
        .or_else(|| {
            // хотя бы шлюз наблюдателей
            world
                .orgs
                .iter()
                .find(|o| matches!(o.kind, OrgKind::CyberPolice | OrgKind::SecurityCenter))
                .map(|o| o.gateway)
        }),
    };
    // запасной вариант: любая публичная машина со слабым местом
    target.or_else(|| {
        if role == TargetRole::None {
            None
        } else {
            first_machine(world, |w, id| is_public(w, id) && vulnerable(w, id))
        }
    })
}

/// Файл и путь для действия ReadDeliver по роли.
fn deliver_file(world: &World, role: TargetRole, id: MachineId) -> (String, String) {
    let found = match role {
        TargetRole::BankLedger => pick_file(world, id, &["/var", "/home", "/root"], &["transactions.log", "*.log", "ledger*"]),
        TargetRole::HospitalRecords => pick_file(world, id, &["/var", "/srv", "/home", "/root"], &["patients.csv", "*.csv", "records*"]),
        _ => home_file(world, id).or_else(|| pick_file(world, id, &["/var/log", "/etc", "/var"], &["auth.log", "*.log", "motd"])),
    };
    found.unwrap_or_else(|| ("/etc/hostname".to_string(), world.machine(id).hostname.clone()))
}

// ---------------------------------------------------------------- построение

fn build_goal(world: &World, def: &data::MissionDef, id: Option<MachineId>) -> (Goal, String, String) {
    // возвращает (цель, путь-для-текста, адрес-доставки)
    match def.action {
        ActionKind::ReadDeliver => {
            let id = id.unwrap_or(0);
            let (path, needle) = deliver_file(world, def.role, id);
            (Goal::ReadDeliver { target: id, path: path.clone(), to: DROP.to_string(), needle }, path, DROP.to_string())
        }
        ActionKind::Own => {
            let id = id.unwrap_or(0);
            (Goal::Own { target: id }, String::new(), String::new())
        }
        ActionKind::Plant => {
            let id = id.unwrap_or(0);
            let path = "/var/www/html/hello-world.html".to_string();
            (Goal::Plant { target: id, path: path.clone(), needle: "hello-world".to_string() }, path, String::new())
        }
        ActionKind::CoverTracks => (Goal::CoverTracks { heat_below: 15.0 }, String::new(), String::new()),
    }
}

fn fill(text: &str, host: &str, ip: &str, path: &str, to: &str) -> String {
    text.replace("{host}", host).replace("{ip}", ip).replace("{path}", path).replace("{to}", to)
}

/// Построить цепочку, если ещё не построена и мир готов.
pub fn ensure_init(world: &mut World) {
    if world.missions.initialized || world.orgs.is_empty() {
        return;
    }
    let mut runs = Vec::new();
    for (i, def) in CHAIN.iter().enumerate() {
        let id = resolve_target(world, def.role);
        let (goal, path, to) = build_goal(world, def, id);
        let (host, ip) = match id {
            Some(id) => (world.machine(id).hostname.clone(), world.machine(id).ip.clone()),
            None => (String::new(), String::new()),
        };
        runs.push(MissionRun {
            code: def.code.to_string(),
            title: def.title.to_string(),
            objective: fill(def.objective, &host, &ip, &path, &to),
            brief: fill(def.brief, &host, &ip, &path, &to),
            reply: def.reply.to_string(),
            reward: def.reward,
            goal,
            status: if i == 0 { MissionStatus::Active } else { MissionStatus::Locked },
        });
    }
    world.missions.missions = runs;
    world.missions.active = 0;
    world.missions.initialized = true;
    // первое задание — письмом от группы
    send_brief_email(world, 0);
}

fn send_brief_email(world: &mut World, idx: usize) {
    let (title, brief) = match world.missions.missions.get(idx) {
        Some(m) => (m.title.clone(), m.brief.clone()),
        None => return,
    };
    push_group_mail(world, &format!("Задание: {}", title), &brief);
}

fn push_group_mail(world: &mut World, subject: &str, body: &str) {
    let to = format!("{}@home.net", world.player.handle);
    let now = world.now();
    world.machines[0].push_mail(MailMessage {
        id: 0,
        from: DROP.to_string(),
        to,
        subject: subject.to_string(),
        body: body.to_string(),
        time: now,
        read: false,
        folder: MailFolder::Inbox,
    });
}

// ---------------------------------------------------------------- проверка

fn goal_met(world: &World, goal: &Goal) -> bool {
    match goal {
        Goal::Own { target } => world.player.owned.contains(target),
        Goal::Plant { target, path, needle } => {
            world.has_machine(*target) && world.machine(*target).fs.read_text(path).map(|s| s.contains(needle.as_str())).unwrap_or(false)
        }
        Goal::CoverTracks { heat_below } => world.player.heat < *heat_below && crate::hack::tracks::evidence(world).is_empty(),
        Goal::ReadDeliver { to, needle, .. } => world.machines[0]
            .mail
            .iter()
            .any(|m| m.folder == MailFolder::Sent && m.to.contains(to.as_str()) && m.body.contains(needle.as_str())),
    }
}

/// Шаг цепочки: зовётся из `World::tick`. Строит цепочку и проверяет текущее
/// задание; при выполнении платит, шлёт ответ и открывает следующее.
pub fn check(world: &mut World) {
    ensure_init(world);
    let idx = world.missions.active;
    let goal = match world.missions.missions.get(idx) {
        Some(m) if m.status == MissionStatus::Active => m.goal.clone(),
        _ => return,
    };
    if goal_met(world, &goal) {
        complete(world, idx);
    }
}

fn complete(world: &mut World, idx: usize) {
    let (title, reply, reward) = {
        let m = &world.missions.missions[idx];
        (m.title.clone(), m.reply.clone(), m.reward)
    };
    world.player.money += reward;
    world.missions.money_earned += reward;
    world.missions.missions[idx].status = MissionStatus::Done;

    let next = idx + 1;
    let next_brief = world.missions.missions.get(next).map(|m| (m.title.clone(), m.brief.clone()));
    if next < world.missions.missions.len() {
        world.missions.missions[next].status = MissionStatus::Active;
    }
    world.missions.active = next;

    // ответ группы + следующее задание одним письмом
    let mut body = format!("{}\n\nНачислено: {} ₵.", reply, reward);
    if let Some((nt, nb)) = &next_brief {
        body.push_str(&format!("\n\n— — —\n\nСледующее: {}\n\n{}", nt, nb));
    } else {
        body.push_str("\n\nЭто было последнее. Дальше — сам. Мы на связи.");
    }
    let subject = format!("Принято: {}", title);
    push_group_mail(world, &subject, &body);

    let notify = if next_brief.is_some() {
        format!("«{}» выполнено (+{} ₵). Есть новое задание.", title, reward)
    } else {
        format!("«{}» выполнено (+{} ₵). Цепочка пройдена.", title, reward)
    };
    world.missions.pending = Some(MissionEvent { title: "hello-world".to_string(), text: notify });
}

// ---------------------------------------------------------------- вывод

/// Доска заданий (HTML) для браузера — узлы `gate.hw/board` и `board.hw`.
pub fn board_html(world: &World) -> String {
    let mut s = String::from("<header><h1>Доска заданий</h1></header><nav><a href=\"/manifesto\">Манифест</a></nav><main>");
    if world.missions.missions.is_empty() {
        s.push_str("<p>Пусто. Загляни позже.</p></main>");
        return s;
    }
    s.push_str(&format!("<p>Заработано: <b>{} ₵</b></p><ol>", world.missions.money_earned));
    for m in &world.missions.missions {
        let (mark, cls) = match m.status {
            MissionStatus::Done => ("✓ сделано", "done"),
            MissionStatus::Active => ("● сейчас", "active"),
            MissionStatus::Locked => ("· закрыто", "locked"),
        };
        if m.status == MissionStatus::Locked {
            s.push_str(&format!("<li class=\"{}\"><b>{}</b> — <i>{}</i></li>", cls, m.title, mark));
        } else {
            s.push_str(&format!(
                "<li class=\"{}\"><b>{}</b> — {} · награда {} ₵<br><small>{}</small>",
                cls, m.title, mark, m.reward, m.objective
            ));
            if m.status == MissionStatus::Active {
                s.push_str(&format!("<p>{}</p>", m.brief.replace('\n', "<br>")));
            }
            s.push_str("</li>");
        }
    }
    s.push_str("</ol><h2>Задания расширений</h2><ul>");
    for q in world.plugins.quests.values(){fn esc(s:&str)->String{s.replace('&',"&amp;").replace('<',"&lt;").replace('>',"&gt;")}s.push_str(&format!("<li><b>{}</b> — {} · {} ₵<br>{}</li>",esc(&q.title),if q.done{"выполнено"}else{"ожидает"},q.reward,esc(&q.objective)));}
    s.push_str("</ul></main>");
    s
}

/// Краткие строки для команды `missions`: (метка, заголовок, статус, цель, награда).
pub struct MissionRow {
    pub code: String,
    pub title: String,
    pub status: MissionStatus,
    pub objective: String,
    pub reward: i64,
}

pub fn rows(world: &World) -> Vec<MissionRow> {
    world
        .missions
        .missions
        .iter()
        .map(|m| MissionRow { code: m.code.clone(), title: m.title.clone(), status: m.status, objective: m.objective.clone(), reward: m.reward })
        .collect()
}

/// Брифинг задания по номеру (1-based) или текущего (None).
pub fn brief(world: &World, n: Option<usize>) -> Option<(String, String)> {
    let idx = match n {
        Some(k) if k >= 1 => k - 1,
        _ => world.missions.active,
    };
    world.missions.missions.get(idx).map(|m| (m.title.clone(), m.brief.clone()))
}
