//! Таблица процессов одной машины: система, службы, окна приложений и скрипты Adder.
//! Нагрузка процессора у каждого процесса слегка колеблется на каждом тике,
//! чтобы монитор показывал живые графики.

use serde::{Deserialize, Serialize};

use crate::rng::Rng;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ProcKind {
    System,
    Daemon,
    /// Открытое окно игровой программы; память освобождается при закрытии.
    Application { window: String },
    /// Скрипт Adder, запущенный из сеанса терминала.
    Script { session: u32 },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Proc {
    pub pid: u32,
    pub name: String,
    pub user: String,
    /// Текущая нагрузка, проценты.
    pub cpu: f32,
    /// Средняя нагрузка, вокруг которой идёт колебание.
    pub base_cpu: f32,
    pub mem_mb: u32,
    /// Игровое время запуска, секунды.
    pub started: u64,
    pub kind: ProcKind,
    /// Скрипту велели остановиться (`kill`); поток увидит и завершится.
    #[serde(default)]
    pub killed: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ProcTable {
    next_pid: u32,
    #[serde(serialize_with="save_processes")]
    procs: Vec<Proc>,
    /// Игровое время, которым помечаются новые процессы.
    clock: u64,
}

// GUI processes are runtime state, so older saves and reopening never retain phantom windows.
fn save_processes<S:serde::Serializer>(procs:&[Proc],serializer:S)->Result<S::Ok,S::Error>{
    procs.iter().filter(|p|!matches!(p.kind,ProcKind::Application{..})).collect::<Vec<_>>().serialize(serializer)
}

impl Default for ProcTable {
    fn default() -> Self {
        ProcTable::new()
    }
}

impl ProcTable {
    /// Таблица с процессом init (pid 1).
    pub fn new() -> ProcTable {
        let mut t = ProcTable { next_pid: 1, procs: Vec::new(), clock: 0 };
        t.spawn("init", "root", 4, 0.3, ProcKind::System);
        t.spawn("kworker", "root", 2, 0.8, ProcKind::System);
        t
    }

    pub fn set_clock(&mut self, t: u64) {
        self.clock = t;
    }

    fn spawn(&mut self, name: &str, user: &str, mem_mb: u32, base_cpu: f32, kind: ProcKind) -> u32 {
        let pid = self.next_pid;
        self.next_pid += 1;
        self.procs.push(Proc {
            pid,
            name: name.to_string(),
            user: user.to_string(),
            cpu: base_cpu,
            base_cpu,
            mem_mb,
            started: self.clock,
            kind,
            killed: false,
        });
        pid
    }

    /// Демон службы.
    pub fn spawn_daemon(&mut self, name: &str, user: &str, mem_mb: u32) -> u32 {
        let base = 0.2 + (mem_mb as f32 / 64.0).min(3.0);
        self.spawn(name, user, mem_mb, base, ProcKind::Daemon)
    }

    /// Скрипт Adder как процесс сеанса.
    pub fn spawn_script(&mut self, name: &str, user: &str, session: u32) -> u32 {
        self.spawn(name, user, 12, 4.0, ProcKind::Script { session })
    }

    /// Учёт памяти открытого окна. Повторная регистрация не создаёт второй процесс.
    pub fn spawn_application(&mut self, window: &str, app: &str, user: &str) -> u32 {
        if let Some(p)=self.procs.iter().find(|p|matches!(&p.kind,ProcKind::Application{window:w} if w==window)){return p.pid;}
        let mem=match app {
            "terminal"=>24, "editor"=>48, "files"=>40, "browser"=>96,
            "monitor"=>32, "map"=>64, "mail"=>48, "help"=>40,
            "scenario"=>64, "plugins"|"plugin_panel"=>48, _=>32,
        };
        self.spawn(app,user,mem,0.4,ProcKind::Application{window:window.into()})
    }

    pub fn remove_application(&mut self, window:&str) {
        self.procs.retain(|p|!matches!(&p.kind,ProcKind::Application{window:w} if w==window));
    }

    /// Окна не восстанавливаются из таблицы процессов сохранения.
    pub fn remove_applications(&mut self) {
        self.procs.retain(|p|!matches!(p.kind,ProcKind::Application{..}));
    }

    pub fn stopped_applications(&self)->Vec<String> {
        self.procs.iter().filter_map(|p|match &p.kind {ProcKind::Application{window} if p.killed=>Some(window.clone()),_=>None}).collect()
    }

    /// Снять процесс; для приложения сначала запросить закрытие его окна. init снять нельзя.
    pub fn kill(&mut self, pid: u32) -> Result<(), String> {
        if pid == 1 {
            return Err("нельзя снять процесс init (pid 1)".to_string());
        }
        if let Some(p)=self.get_mut(pid) {
            if matches!(p.kind,ProcKind::Application{..}) {p.killed=true;return Ok(());}
        }
        let before = self.procs.len();
        self.procs.retain(|p| p.pid != pid);
        if self.procs.len() == before {
            Err(format!("нет процесса с pid {}", pid))
        } else {
            Ok(())
        }
    }

    /// Снять процесс от имени пользователя: чужие процессы снимает только root.
    pub fn kill_as(&mut self, pid: u32, who: &str) -> Result<(), String> {
        let p = self.get(pid).ok_or_else(|| format!("нет процесса с pid {}", pid))?;
        if who != "root" && p.user != who {
            return Err(format!("отказано в доступе: процесс {} принадлежит {}", pid, p.user));
        }
        self.kill(pid)
    }

    /// Пометить скрипт остановленным, не убирая из таблицы (уберёт `remove`).
    pub fn mark_killed(&mut self, pid: u32) -> bool {
        match self.get_mut(pid) {
            Some(p) => {
                p.killed = true;
                true
            }
            None => false,
        }
    }

    pub fn is_killed(&self, pid: u32) -> bool {
        self.get(pid).map(|p| p.killed).unwrap_or(true)
    }

    /// Убрать процесс без проверок (скрипт завершился).
    pub fn remove(&mut self, pid: u32) -> bool {
        let before = self.procs.len();
        self.procs.retain(|p| p.pid != pid);
        self.procs.len() != before
    }

    /// Убрать все скрипты сеанса (сеанс закрыт).
    pub fn remove_session_scripts(&mut self, session: u32) {
        self.procs.retain(|p| !matches!(p.kind, ProcKind::Script { session: s } if s == session));
    }

    pub fn list(&self) -> &[Proc] {
        &self.procs
    }

    pub fn get(&self, pid: u32) -> Option<&Proc> {
        self.procs.iter().find(|p| p.pid == pid)
    }

    pub fn get_mut(&mut self, pid: u32) -> Option<&mut Proc> {
        self.procs.iter_mut().find(|p| p.pid == pid)
    }

    pub fn len(&self) -> usize {
        self.procs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.procs.is_empty()
    }

    /// Колебание нагрузки: раз в секунду.
    pub fn tick(&mut self, rng: &mut Rng) {
        for p in &mut self.procs {
            let spread = (p.base_cpu * 0.6).max(0.2);
            let v = rng.jitter(p.base_cpu, spread);
            p.cpu = v.clamp(0.0, 100.0);
        }
    }

    /// Суммарная нагрузка процессора, проценты (не больше 100).
    pub fn total_cpu(&self) -> f32 {
        self.procs.iter().map(|p| p.cpu).sum::<f32>().min(100.0)
    }

    /// Суммарная память процессов, МБ.
    pub fn total_mem(&self) -> u32 {
        self.procs.iter().map(|p| p.mem_mb).sum()
    }
}
