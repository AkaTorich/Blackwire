//! Свои сценарии: игрок сам задаёт задание (цель, условие, награда) в окне-
//! конструкторе, сохраняет его файлом и проходит. Здесь — модель сценария и
//! проверка выполнения по состоянию мира (как у подработок). Файлами сценариев
//! заведует оболочка (`app`); ядро хранит только активный и проверяет его.

use serde::{Deserialize, Serialize};

use crate::net::machine::MailFolder;
use crate::world::World;

/// Условие прохождения сценария.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum ScenarioGoal {
    /// Получить доступ к узлу по имени.
    OwnHost { host: String },
    /// Прислать (на любой адрес из «Отправленных») текст с этой строкой.
    Deliver { needle: String },
    /// Накопить столько ₵.
    Money { amount: i64 },
}

/// Один сценарий.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Scenario {
    pub name: String,
    pub brief: String,
    pub goal: ScenarioGoal,
    pub reward: i64,
}

/// Активный сценарий и его состояние (в мире).
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ScenarioState {
    pub active: Option<Scenario>,
    pub done: bool,
    #[serde(skip)]
    pub pending: Option<ScenarioEvent>,
}

#[derive(Clone, Debug)]
pub struct ScenarioEvent {
    pub title: String,
    pub text: String,
}

pub fn take_pending(world: &mut World) -> Option<ScenarioEvent> {
    world.scenarios.pending.take()
}

/// Человекочитаемое условие цели.
pub fn goal_text(goal: &ScenarioGoal) -> String {
    match goal {
        ScenarioGoal::OwnHost { host } => format!("получить доступ к {}", host),
        ScenarioGoal::Deliver { needle } => format!("прислать текст со строкой «{}»", needle),
        ScenarioGoal::Money { amount } => format!("накопить {} ₵", amount),
    }
}

/// Запустить сценарий (сбрасывает отметку прохождения).
pub fn start(world: &mut World, s: Scenario) {
    world.scenarios.active = Some(s);
    world.scenarios.done = false;
}

/// Снять активный сценарий.
pub fn stop(world: &mut World) {
    world.scenarios.active = None;
    world.scenarios.done = false;
}

fn goal_met(world: &World, goal: &ScenarioGoal) -> bool {
    match goal {
        ScenarioGoal::OwnHost { host } => match world.find_by_hostname(host) {
            Some(id) => world.player.owned.contains(&id),
            None => false,
        },
        ScenarioGoal::Deliver { needle } => world.machines[0]
            .mail
            .iter()
            .any(|m| m.folder == MailFolder::Sent && m.body.contains(needle.as_str())),
        ScenarioGoal::Money { amount } => world.player.money >= *amount,
    }
}

/// Проверка активного сценария. Зовётся из `World::tick`.
pub fn check(world: &mut World) {
    if world.scenarios.done {
        return;
    }
    let s = match &world.scenarios.active {
        Some(s) => s.clone(),
        None => return,
    };
    if goal_met(world, &s.goal) {
        world.scenarios.done = true;
        world.player.money += s.reward;
        world.scenarios.pending = Some(ScenarioEvent {
            title: "Сценарий пройден".to_string(),
            text: format!("«{}» — выполнено (+{} ₵).", s.name, s.reward),
        });
    }
}

/// Имена узлов для конструктора: публичные включённые машины, по алфавиту.
pub fn targets(world: &World) -> Vec<String> {
    let mut v: Vec<String> = world
        .machines
        .iter()
        .filter(|m| m.id != 0 && m.up && m.is_public())
        .map(|m| m.hostname.clone())
        .collect();
    v.sort();
    v.dedup();
    v
}
