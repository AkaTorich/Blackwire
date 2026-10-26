//! `Game` — мир плюс сеансы терминалов; точка входа для оболочки Tauri.
//! Все методы, которые зовёт мост `js/bridge.js`, собраны здесь: оболочка,
//! файлы для окон, показатели, карта сети, браузер, почта, настройки.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::net::machine::{LogKind, MachineId, MachineKind, MailFolder, MailMessage};
use crate::procs::ProcKind;
use crate::session::{PendingScript, Session};
use crate::shell;
use crate::sysinfo::{self, Status};
use crate::types::{FsEntry, Prompt, ShellResult, UiAction};
use crate::world::{HttpPage, Settings, World};

/// Узел карты сети.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct NetNode {
    pub id: MachineId,
    pub hostname: String,
    pub ip: String,
    pub kind: String,
    pub known: bool,
    pub up: bool,
    pub x: f32,
    pub y: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct NetLink {
    pub a: MachineId,
    pub b: MachineId,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct NetMap {
    pub nodes: Vec<NetNode>,
    pub links: Vec<NetLink>,
}

pub struct Game {
    pub world: World,
    pub sessions: HashMap<u32, Session>,
    next_session: u32,
    /// Действия для интерфейса, накопленные скриптами (sys.open, sys.notify).
    pub pending_ui: Vec<UiAction>,
    /// Заявки на запуск скриптов (команда `adder`).
    pub pending_scripts: Vec<PendingScript>,
}

impl Game {
    pub fn new(world: World) -> Game {
        Game { world, sessions: HashMap::new(), next_session: 1, pending_ui: Vec::new(), pending_scripts: Vec::new() }
    }

    /// Загрузить игру из JSON мира.
    pub fn from_json(s: &str) -> Result<Game, String> {
        Ok(Game::new(World::from_json(s)?))
    }

    pub fn save_json(&self) -> String {
        self.world.to_json()
    }

    pub fn tick(&mut self, dt_ms: u64) {
        self.world.tick(dt_ms);
    }

    /// Пароль своей учётной записи.
    pub fn login(&self, password: &str) -> bool {
        let handle = self.world.player.handle.clone();
        self.world.machines[0].users.check_password(&handle, password)
    }

    pub fn handle(&self) -> &str {
        &self.world.player.handle
    }

    // ---------- сеансы и оболочка ----------

    pub fn open_session(&mut self) -> u32 {
        let id = self.next_session;
        self.next_session += 1;
        let handle = self.world.player.handle.clone();
        let home = self.world.machines[0].users.home_of(&handle);
        self.sessions.insert(id, Session::new(id, self.world.machines[0].id, &handle, &home));
        id
    }

    pub fn close_session(&mut self, id: u32) {
        if self.sessions.remove(&id).is_some() {
            for m in &mut self.world.machines {
                m.procs.remove_session_scripts(id);
            }
        }
    }

    pub fn exec(&mut self, session: u32, line: &str) -> ShellResult {
        let s = match self.sessions.get_mut(&session) {
            Some(s) => s,
            None => {
                return ShellResult {
                    lines: vec![crate::types::OutLine::error(format!("нет сеанса {}", session))],
                    ..ShellResult::default()
                }
            }
        };
        let res = shell::exec(&mut self.world, s, line);
        self.pending_scripts.append(&mut s.pending_scripts);
        res
    }

    pub fn complete(&self, session: u32, line: &str) -> Vec<String> {
        match self.sessions.get(&session) {
            Some(s) => shell::complete(&self.world, s, line),
            None => Vec::new(),
        }
    }

    pub fn prompt(&self, session: u32) -> Prompt {
        match self.sessions.get(&session) {
            Some(s) => shell::prompt(&self.world, s),
            None => Prompt::default(),
        }
    }

    pub fn history(&self, session: u32) -> Vec<String> {
        self.sessions.get(&session).map(|s| s.history.clone()).unwrap_or_default()
    }

    pub fn shell_doc(&self) -> String {
        shell::shell_doc()
    }

    pub fn take_pending_ui(&mut self) -> Vec<UiAction> {
        std::mem::take(&mut self.pending_ui)
    }

    pub fn take_pending_scripts(&mut self) -> Vec<PendingScript> {
        std::mem::take(&mut self.pending_scripts)
    }

    pub fn notify(&mut self, title: &str, text: &str) {
        self.pending_ui.push(UiAction::Notify { title: title.to_string(), text: text.to_string() });
    }

    /// Событие слежки: забрать уведомление расследования; при рейде оборвать все
    /// цепочки подключений (сеансы возвращаются на свою машину). Возвращает
    /// `(заголовок, текст, рейд?)` для уведомления оболочки.
    pub fn poll_surveillance(&mut self) -> Option<(String, String, bool)> {
        let ev = crate::hack::watch::take_pending(&mut self.world)?;
        if ev.raided {
            let home = self.world.machines[0].users.home_of(&self.world.player.handle);
            for s in self.sessions.values_mut() {
                if s.hops.len() > 1 {
                    s.hops.truncate(1);
                    s.cwd = home.clone();
                }
            }
        }
        Some((ev.title, ev.text, ev.raided))
    }

    /// Событие заданий: забрать уведомление о выполненной миссии.
    pub fn poll_missions(&mut self) -> Option<(String, String)> {
        let ev = crate::missions::take_pending(&mut self.world)?;
        Some((ev.title, ev.text))
    }

    /// Событие финансов: уведомление (например, долг погашен).
    pub fn poll_finance(&mut self) -> Option<(String, String)> {
        let ev = crate::money::take_pending(&mut self.world)?;
        Some((ev.title, ev.text))
    }

    /// Событие концовки: победа (цепочка пройдена + долг закрыт). Игра продолжается.
    pub fn poll_ending(&mut self) -> Option<(String, String)> {
        let ev = crate::ending::take_pending(&mut self.world)?;
        Some((ev.title, ev.text))
    }

    /// Сводка пути — для окна финала и команды `ending`.
    pub fn ending_summary(&self) -> crate::ending::EndingSummary {
        crate::ending::summary(&self.world)
    }

    /// Событие подработок: заказ оплачен.
    pub fn poll_gigs(&mut self) -> Option<(String, String)> {
        let ev = crate::gigs::take_pending(&mut self.world)?;
        Some((ev.title, ev.text))
    }

    /// Событие защиты: ответный удар врага (закладка, штраф, отражение).
    pub fn poll_defense(&mut self) -> Option<(String, String)> {
        let ev = crate::hack::counter::take_pending(&mut self.world)?;
        Some((ev.title, ev.text))
    }

    /// Событие базы: релей забрали за неуплату.
    pub fn poll_base(&mut self) -> Option<(String, String)> {
        let ev = crate::base::take_pending(&mut self.world)?;
        Some((ev.title, ev.text))
    }

    /// Открылось новое достижение.
    pub fn poll_achievement(&mut self) -> Option<(String, String)> {
        let ev = crate::achievements::take_pending(&mut self.world)?;
        Some(("Достижение открыто".to_string(), ev.name))
    }

    /// Свой сценарий пройден.
    pub fn poll_scenario(&mut self) -> Option<(String, String)> {
        let ev = crate::scenarios::take_pending(&mut self.world)?;
        Some((ev.title, ev.text))
    }

    // ---------- скрипты как процессы ----------

    /// Машина и пользователь сеанса (None → своя машина и игрок).
    pub(crate) fn target(&self, session: Option<u32>) -> (MachineId, String) {
        match session.and_then(|id| self.sessions.get(&id)) {
            Some(s) => (s.machine_id(), s.user().to_string()),
            None => (self.world.machines[0].id, self.world.player.handle.clone()),
        }
    }

    /// Машина, где живёт скрипт с таким pid: сперва своя, затем остальные.
    fn script_machine(&self, pid: u32) -> Option<MachineId> {
        let own = &self.world.machines[0];
        if own.procs.get(pid).is_some() {
            return Some(own.id);
        }
        self.world
            .machines
            .iter()
            .find(|m| m.procs.get(pid).map(|p| matches!(p.kind, ProcKind::Script { .. })).unwrap_or(false))
            .map(|m| m.id)
    }

    /// Снять процесс (монитор, `sys.kill`). Скрипт только помечается — поток остановится сам.
    pub fn kill(&mut self, pid: u32) -> Result<(), String> {
        let id = self.script_machine(pid).ok_or_else(|| format!("нет процесса с pid {}", pid))?;
        let m = self.world.machine_mut(id);
        let is_script = m.procs.get(pid).map(|p| matches!(p.kind, ProcKind::Script { .. })).unwrap_or(false);
        if is_script {
            m.procs.mark_killed(pid);
            Ok(())
        } else {
            m.procs.kill(pid)
        }
    }

    /// Скрипту велели остановиться (или его уже нет).
    pub fn is_killed(&self, pid: u32) -> bool {
        match self.script_machine(pid) {
            Some(id) => self.world.machine(id).procs.is_killed(pid),
            None => true,
        }
    }

    /// Поток скрипта завершился — убрать процесс из таблицы.
    pub fn script_finished(&mut self, pid: u32) {
        if let Some(id) = self.script_machine(pid) {
            self.world.machine_mut(id).procs.remove(pid);
        }
    }

    // ---------- файлы для окон ----------

    pub(crate) fn fs_path(&self, session: Option<u32>, path: &str) -> String {
        let cwd = session.and_then(|id| self.sessions.get(&id)).map(|s| s.cwd.clone()).unwrap_or_else(|| "/".to_string());
        let (mid, user) = self.target(session);
        let home = self.world.machine(mid).users.home_of(&user);
        let p = if path == "~" {
            home
        } else if let Some(rest) = path.strip_prefix("~/") {
            format!("{}/{}", home, rest)
        } else {
            path.to_string()
        };
        crate::vfs::resolve(&cwd, &p)
    }

    pub fn fs_list(&self, session: Option<u32>, path: &str) -> Result<Vec<FsEntry>, String> {
        let (mid, user) = self.target(session);
        let p = self.fs_path(session, path);
        let mut v = self.world.machine(mid).fs.list(&p, &user).map_err(|e| e.to_string())?;
        v.sort_by(|a, b| b.is_dir.cmp(&a.is_dir).then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase())));
        Ok(v)
    }

    pub fn fs_read(&self, session: Option<u32>, path: &str) -> Result<String, String> {
        let (mid, user) = self.target(session);
        let p = self.fs_path(session, path);
        self.world.machine(mid).fs.read(&p, &user).map_err(|e| e.to_string())
    }

    pub fn fs_write(&mut self, session: Option<u32>, path: &str, text: &str) -> Result<(), String> {
        let (mid, user) = self.target(session);
        let p = self.fs_path(session, path);
        self.world.machine_mut(mid).fs.write(&p, text, &user).map_err(|e| e.to_string())
    }

    pub fn fs_mkdir(&mut self, session: Option<u32>, path: &str) -> Result<(), String> {
        let (mid, user) = self.target(session);
        let p = self.fs_path(session, path);
        self.world.machine_mut(mid).fs.mkdir(&p, &user, true).map_err(|e| e.to_string())
    }

    pub fn fs_delete(&mut self, session: Option<u32>, path: &str) -> Result<(), String> {
        let (mid, user) = self.target(session);
        let p = self.fs_path(session, path);
        self.world.machine_mut(mid).fs.remove(&p, &user, true).map_err(|e| e.to_string())?;
        self.world.log(mid, LogKind::FileChange, &format!("{} удалил {}", user, p));
        Ok(())
    }

    pub fn fs_rename(&mut self, session: Option<u32>, from: &str, to: &str) -> Result<(), String> {
        let (mid, user) = self.target(session);
        let (f, t) = (self.fs_path(session, from), self.fs_path(session, to));
        self.world.machine_mut(mid).fs.rename(&f, &t, &user).map(|_| ()).map_err(|e| e.to_string())
    }

    pub fn fs_copy(&mut self, session: Option<u32>, from: &str, to: &str) -> Result<(), String> {
        let (mid, user) = self.target(session);
        let (f, t) = (self.fs_path(session, from), self.fs_path(session, to));
        self.world.machine_mut(mid).fs.copy(&f, &t, &user, true).map(|_| ()).map_err(|e| e.to_string())
    }

    pub fn fs_exists(&self, session: Option<u32>, path: &str) -> bool {
        let (mid, _) = self.target(session);
        let p = self.fs_path(session, path);
        self.world.machine(mid).fs.exists(&p)
    }

    // ---------- показатели, карта, браузер ----------

    pub fn status(&self) -> Status {
        sysinfo::status(&self.world, self.world.machines[0].id)
    }

    pub fn status_of(&self, session: u32) -> Status {
        let (mid, _) = self.target(Some(session));
        sysinfo::status(&self.world, mid)
    }

    pub fn net_map(&self) -> NetMap {
        let mut map = NetMap::default();
        for m in &self.world.machines {
            map.nodes.push(NetNode {
                id: m.id,
                hostname: m.hostname.clone(),
                ip: m.ip.clone(),
                kind: format!("{:?}", m.kind).to_lowercase(),
                known: self.world.is_known(m.id),
                up: m.up,
                x: m.x,
                y: m.y,
            });
        }
        let isp = self.world.machines.iter().find(|m| m.kind == MachineKind::Isp).map(|m| m.id);
        let push = |a: MachineId, b: MachineId, links: &mut Vec<NetLink>| {
            if a == b {
                return;
            }
            let (x, y) = if a < b { (a, b) } else { (b, a) };
            if !links.iter().any(|l| l.a == x && l.b == y) {
                links.push(NetLink { a: x, b: y });
            }
        };
        for m in &self.world.machines {
            if let Some(r) = m.router {
                push(m.id, r, &mut map.links);
            }
        }
        for org in &self.world.orgs {
            for member in &org.members {
                push(*member, org.gateway, &mut map.links);
            }
            if let Some(isp) = isp {
                push(org.gateway, isp, &mut map.links);
            }
        }
        if let Some(isp) = isp {
            for m in &self.world.machines {
                let in_org = m.org.is_some();
                if !in_org && m.router.is_none() && m.id != isp {
                    push(m.id, isp, &mut map.links);
                }
                if m.kind == MachineKind::Router {
                    push(m.id, isp, &mut map.links);
                }
            }
        }
        map
    }

    pub fn http(&mut self, url: &str) -> HttpPage {
        let page = self.world.http(self.world.machines[0].id, url);
        if page.status == 200 {
            if let Some(host) = url.split("://").nth(1).and_then(|r| r.split('/').next()).map(|h| h.split(':').next().unwrap_or(h)) {
                if let Some(id) = self.world.resolve(host).and_then(|ip| self.world.find_by_ip(&ip)) {
                    self.world.mark_known(id);
                }
            }
        }
        page
    }

    // ---------- настройки ----------

    pub fn settings(&self) -> Settings {
        self.world.settings.clone()
    }

    pub fn set_settings(&mut self, s: Settings) {
        self.world.settings = s;
    }

    // ---------- почта ----------

    /// Адрес игрока: `<handle>@home.net`.
    pub fn mail_address(&self) -> String {
        format!("{}@home.net", self.world.player.handle)
    }

    /// Письма игрока: `inbox` или `sent`, новые сверху.
    pub fn mail_list(&self, folder: &str) -> Vec<MailMessage> {
        let addr = self.mail_address();
        let want = if folder.eq_ignore_ascii_case("sent") { MailFolder::Sent } else { MailFolder::Inbox };
        let mut v: Vec<MailMessage> = self.world.machines[0]
            .mail
            .iter()
            .filter(|m| m.folder == want && (if want == MailFolder::Inbox { m.to == addr } else { m.from == addr }))
            .cloned()
            .collect();
        v.sort_by(|a, b| b.time.cmp(&a.time).then_with(|| b.id.cmp(&a.id)));
        v
    }

    pub fn mail_read(&mut self, id: u32) -> Result<MailMessage, String> {
        let m = self.world.machines[0].mail.iter_mut().find(|m| m.id == id).ok_or_else(|| format!("нет письма с номером {}", id))?;
        m.read = true;
        Ok(m.clone())
    }

    pub fn mail_delete(&mut self, id: u32) -> Result<(), String> {
        let mail = &mut self.world.machines[0].mail;
        let before = mail.len();
        mail.retain(|m| m.id != id);
        if mail.len() == before {
            Err(format!("нет письма с номером {}", id))
        } else {
            Ok(())
        }
    }

    pub fn mail_unread_count(&self) -> usize {
        let addr = self.mail_address();
        self.world.machines[0].mail.iter().filter(|m| m.folder == MailFolder::Inbox && m.to == addr && !m.read).count()
    }

    /// Машина-получатель письма по адресу `user@domain`.
    fn mail_target(&self, to: &str) -> Result<Option<MachineId>, String> {
        let (user, domain) = to.split_once('@').ok_or_else(|| format!("адрес не найден: {}", to))?;
        if user.is_empty() || domain.is_empty() {
            return Err(format!("адрес не найден: {}", to));
        }
        let domain_l = domain.to_ascii_lowercase();
        if domain_l == "home.net" {
            let found = self
                .world
                .machines
                .iter()
                .find(|m| matches!(m.kind, MachineKind::Player | MachineKind::HomePc) && m.users.exists(user))
                .map(|m| m.id);
            return found.map(Some).ok_or_else(|| format!("адрес не найден: {}", to));
        }
        if domain_l == "hw" {
            let gate = self.world.machines.iter().find(|m| m.hostname.eq_ignore_ascii_case("gate.hw") || m.domain.as_deref() == Some("hw")).map(|m| m.id);
            if user == "null" {
                return Ok(gate);
            }
            return gate.map(Some).ok_or_else(|| format!("адрес не найден: {}", to));
        }
        if let Some(m) = self.world.machines.iter().find(|m| m.domain.as_deref().map(|d| d.eq_ignore_ascii_case(&domain_l)).unwrap_or(false)) {
            return Ok(Some(m.id));
        }
        if let Some(org) = self.world.orgs.iter().find(|o| o.domain.eq_ignore_ascii_case(&domain_l)) {
            let found = org.members.iter().copied().find(|id| self.world.has_machine(*id) && self.world.machine(*id).users.exists(user));
            return found.map(Some).ok_or_else(|| format!("адрес не найден: {}", to));
        }
        Err(format!("адрес не найден: {}", to))
    }

    /// Отправить письмо от игрока: копия в «Отправленные», доставка адресату.
    pub fn mail_send(&mut self, to: &str, subject: &str, body: &str) -> Result<(), String> {
        let target = self.mail_target(to.trim())?;
        let now = self.world.now();
        let from = self.mail_address();
        let msg = MailMessage {
            id: 0,
            from: from.clone(),
            to: to.trim().to_string(),
            subject: subject.to_string(),
            body: body.to_string(),
            time: now,
            read: false,
            folder: MailFolder::Inbox,
        };
        if let Some(id) = target {
            self.world.machine_mut(id).push_mail(msg.clone());
            self.world.log(id, LogKind::Net, &format!("почта: письмо от {} для {}", from, to.trim()));
        }
        let mut sent = msg;
        sent.read = true;
        sent.folder = MailFolder::Sent;
        self.world.machines[0].push_mail(sent);
        Ok(())
    }
}
