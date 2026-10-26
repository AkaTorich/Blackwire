//! Сеанс терминала: цепочка подключений (ssh), текущая папка, переменные,
//! история, псевдонимы. Сеансов может быть несколько — по одному на окно.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::net::machine::MachineId;
use crate::world::World;

/// Один шаг цепочки: машина и пользователь на ней.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Hop {
    pub machine: MachineId,
    pub user: String,
}

/// Заявка на запуск скрипта Adder: команда `adder` создаёт процесс и кладёт
/// её сюда, а оболочка Tauri забирает через `Game::take_pending_scripts`.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PendingScript {
    pub pid: u32,
    pub session: u32,
    pub machine: MachineId,
    pub user: String,
    /// Абсолютный путь к файлу или `<cmd>` для `adder -c`.
    pub path: String,
    /// Текст скрипта.
    pub src: String,
    /// Аргументы; args[0] — путь.
    pub args: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct Session {
    pub id: u32,
    /// hops[0] — своя машина; `ssh` добавляет, `exit` снимает.
    pub hops: Vec<Hop>,
    pub cwd: String,
    pub env: BTreeMap<String, String>,
    pub history: Vec<String>,
    pub last_status: i32,
    pub aliases: BTreeMap<String, String>,
    /// Скрипты, запущенные последней командой (забирает `Game`).
    pub pending_scripts: Vec<PendingScript>,
}

impl Default for Session {
    fn default() -> Self {
        Session::new(0, 0, "root", "/root")
    }
}

impl Session {
    pub fn new(id: u32, machine: MachineId, user: &str, home: &str) -> Session {
        let mut env = BTreeMap::new();
        env.insert("HOME".to_string(), home.to_string());
        env.insert("USER".to_string(), user.to_string());
        env.insert("PWD".to_string(), home.to_string());
        env.insert("SHELL".to_string(), "/bin/sh".to_string());
        env.insert("PATH".to_string(), "/usr/bin:/bin:/sbin".to_string());
        env.insert("TERM".to_string(), "blackwire".to_string());
        env.insert("LANG".to_string(), "ru_RU.UTF-8".to_string());
        Session {
            id,
            hops: vec![Hop { machine, user: user.to_string() }],
            cwd: home.to_string(),
            env,
            history: Vec::new(),
            last_status: 0,
            aliases: BTreeMap::new(),
            pending_scripts: Vec::new(),
        }
    }

    /// Текущий шаг цепочки (где мы сейчас).
    pub fn current(&self) -> &Hop {
        self.hops.last().expect("у сеанса всегда есть хотя бы своя машина")
    }

    pub fn current_mut(&mut self) -> &mut Hop {
        self.hops.last_mut().expect("у сеанса всегда есть хотя бы своя машина")
    }

    /// Мы на своей машине (цепочка длиной 1).
    pub fn at_home(&self) -> bool {
        self.hops.len() <= 1
    }

    pub fn machine_id(&self) -> MachineId {
        self.current().machine
    }

    pub fn user(&self) -> &str {
        &self.current().user
    }

    /// Подключиться к машине: новый шаг, папка — дом пользователя там.
    pub fn push_hop(&mut self, machine: MachineId, user: &str, home: &str) {
        self.hops.push(Hop { machine, user: user.to_string() });
        self.set_cwd(home);
        self.env.insert("USER".to_string(), user.to_string());
        self.env.insert("HOME".to_string(), home.to_string());
    }

    /// Отключиться: снять шаг. На своей машине ничего не делает и отдаёт None.
    pub fn pop_hop(&mut self, world: &World) -> Option<Hop> {
        if self.hops.len() <= 1 {
            return None;
        }
        let hop = self.hops.pop();
        let home = self.home(world);
        let user = self.user().to_string();
        self.set_cwd(&home);
        self.env.insert("USER".to_string(), user);
        self.env.insert("HOME".to_string(), home);
        hop
    }

    /// Домашняя папка текущего пользователя на текущей машине.
    pub fn home(&self, world: &World) -> String {
        let hop = self.current();
        world.machine(hop.machine).users.home_of(&hop.user)
    }

    pub fn set_cwd(&mut self, cwd: &str) {
        self.cwd = cwd.to_string();
        self.env.insert("PWD".to_string(), cwd.to_string());
    }
}
