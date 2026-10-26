//! Весь мир целиком: машины, организации, часы, игрок, настройки.
//! Строится из зерна через `net::gen`, сохраняется и загружается как JSON.
//! Здесь же — достижимость адресов, журналирование и http-запросы к машинам.

use std::collections::BTreeMap;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::net::dns;
use crate::net::gen;
use crate::net::machine::{LogEntry, LogKind, Machine, MachineId, MachineKind};
use crate::net::orgs::Org;
use crate::rng::Rng;

/// Начало игрового времени: 2031-03-14 03:00:00, секунды от 1970 года.
pub const START_EPOCH: u64 = 1_931_223_600;

// ---------------------------------------------------------------- часы

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GameClock {
    pub epoch_secs: u64,
    ms: u64,
}

impl Default for GameClock {
    fn default() -> Self {
        GameClock::new()
    }
}

impl GameClock {
    pub fn new() -> GameClock {
        GameClock { epoch_secs: START_EPOCH, ms: 0 }
    }

    pub fn tick(&mut self, dt_ms: u64) {
        self.ms += dt_ms;
        self.epoch_secs += self.ms / 1000;
        self.ms %= 1000;
    }

    pub fn now(&self) -> u64 {
        self.epoch_secs
    }

    /// Секунды от 1970 → (год, месяц, день, час, минута, секунда).
    pub fn civil(secs: u64) -> (i64, u32, u32, u32, u32, u32) {
        let days = (secs / 86_400) as i64;
        let rem = secs % 86_400;
        let z = days + 719_468;
        let era = z.div_euclid(146_097);
        let doe = z.rem_euclid(146_097);
        let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
        let y = yoe + era * 400;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
        let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
        let year = if m <= 2 { y + 1 } else { y };
        (year, m, d, (rem / 3600) as u32, ((rem % 3600) / 60) as u32, (rem % 60) as u32)
    }

    /// Дни от 1970-01-01 по григорианской дате.
    pub fn days_from_civil(y: i64, m: u32, d: u32) -> i64 {
        let y = if m <= 2 { y - 1 } else { y };
        let era = y.div_euclid(400);
        let yoe = y.rem_euclid(400);
        let mp = if m > 2 { m - 3 } else { m + 9 } as i64;
        let doy = (153 * mp + 2) / 5 + d as i64 - 1;
        let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
        era * 146_097 + doe - 719_468
    }

    /// День недели: 0 — понедельник … 6 — воскресенье.
    pub fn weekday(y: i64, m: u32, d: u32) -> u32 {
        let days = Self::days_from_civil(y, m, d);
        // 1970-01-01 — четверг (3)
        (days + 3).rem_euclid(7) as u32
    }

    pub fn days_in_month(y: i64, m: u32) -> u32 {
        match m {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            _ => {
                if (y % 4 == 0 && y % 100 != 0) || y % 400 == 0 {
                    29
                } else {
                    28
                }
            }
        }
    }

    pub fn month_name_ru(m: u32) -> &'static str {
        const M: [&str; 12] = [
            "январь", "февраль", "март", "апрель", "май", "июнь", "июль", "август", "сентябрь", "октябрь", "ноябрь",
            "декабрь",
        ];
        M[((m.max(1) - 1) % 12) as usize]
    }

    pub fn weekday_name_ru(wd: u32) -> &'static str {
        const W: [&str; 7] = ["понедельник", "вторник", "среда", "четверг", "пятница", "суббота", "воскресенье"];
        W[(wd % 7) as usize]
    }

    /// `2031-03-14 03:00:00`
    pub fn format_time(secs: u64) -> String {
        let (y, m, d, hh, mm, ss) = Self::civil(secs);
        format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02}", y, m, d, hh, mm, ss)
    }

    /// `2031-03-14 03:00`
    pub fn format_short(secs: u64) -> String {
        let (y, m, d, hh, mm, _) = Self::civil(secs);
        format!("{:04}-{:02}-{:02} {:02}:{:02}", y, m, d, hh, mm)
    }

    /// `03:00:00`
    pub fn format_hms(secs: u64) -> String {
        let (_, _, _, hh, mm, ss) = Self::civil(secs);
        format!("{:02}:{:02}:{:02}", hh, mm, ss)
    }

    /// Длительность: `3 дн. 02:15:07` или `02:15:07`.
    pub fn format_duration(secs: u64) -> String {
        let d = secs / 86_400;
        let rest = secs % 86_400;
        let hms = format!("{:02}:{:02}:{:02}", rest / 3600, (rest % 3600) / 60, rest % 60);
        if d > 0 {
            format!("{} дн. {}", d, hms)
        } else {
            hms
        }
    }

    pub fn format_now(&self) -> String {
        Self::format_time(self.epoch_secs)
    }
}

// ---------------------------------------------------------------- настройки и игрок

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Fx {
    pub wireframe: bool,
    pub hexrain: bool,
    pub scanlines: bool,
    pub glass: bool,
}

impl Default for Fx {
    fn default() -> Self {
        Fx { wireframe: true, hexrain: true, scanlines: true, glass: true }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct Settings {
    pub theme: String,
    pub fx: Fx,
    pub font_size: u32,
    pub terminal_opacity: f32,
    /// Непрозрачность фона окон (0.3..1.0): меньше — окна прозрачнее.
    pub window_opacity: f32,
    pub sound: bool,
    pub default_monitor: u32,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            theme: "swordfish".to_string(),
            fx: Fx::default(),
            font_size: 14,
            terminal_opacity: 0.85,
            window_opacity: 0.72,
            sound: true,
            default_monitor: 0,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
#[serde(default)]
pub struct Player {
    pub handle: String,
    pub money: i64,
    /// Машины, которые игрок уже видел (пинг, скан, ssh): они ярко на карте.
    pub known_machines: Vec<MachineId>,
    /// Машины, которые игрок сканировал: карта показывает их службы.
    pub scanned: Vec<MachineId>,
    pub notes: Vec<String>,
    /// Флаги сюжета (второй этап).
    pub flags: BTreeMap<String, String>,
    /// Машины, где получен доступ (shell или root).
    pub owned: Vec<MachineId>,
    /// Найденные логины (подбор, эксплойты, утечки).
    pub found_creds: Vec<crate::net::machine::FoundCred>,
    /// Подозрение к игроку, 0..100.
    pub heat: f32,
    /// Наибольшее достигнутое подозрение.
    pub heat_peak: f32,
    /// Игрока запеленговали (heat дошёл до 100).
    pub traced: bool,
    /// Купленные в магазине улучшения (id из каталога).
    pub upgrades: std::collections::BTreeSet<String>,
    /// Уже добытые логины (ключ «машина:пользователь») — чтобы один и тот же
    /// логин не приносил доход повторно при пере-взломе.
    pub harvested: std::collections::BTreeSet<String>,
}

// ---------------------------------------------------------------- сеть: ошибки и страницы

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum NetError {
    /// Имя не разрешается / адреса нет в сети.
    Unknown(String),
    /// Адрес внутри чужой подсети — нет маршрута.
    Unreachable(String),
    /// Машина выключена.
    Down(String),
}

impl fmt::Display for NetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NetError::Unknown(a) => write!(f, "неизвестный узел: {}", a),
            NetError::Unreachable(a) => write!(f, "узел недоступен: {} (нет маршрута)", a),
            NetError::Down(a) => write!(f, "узел не отвечает: {}", a),
        }
    }
}

/// Ответ веб-службы для браузера и `wget`/`curl`.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct HttpPage {
    pub status: u16,
    pub title: String,
    pub body: String,
}

impl HttpPage {
    fn error(status: u16, title: &str, text: &str) -> HttpPage {
        HttpPage {
            status,
            title: title.to_string(),
            body: format!("<h1>{} {}</h1><p>{}</p>", status, title, text),
        }
    }
}

/// Похоже ли на IPv4-адрес.
pub fn looks_like_ip(s: &str) -> bool {
    let parts: Vec<&str> = s.split('.').collect();
    parts.len() == 4 && parts.iter().all(|p| !p.is_empty() && p.len() <= 3 && p.chars().all(|c| c.is_ascii_digit()))
}

/// Частный адрес (10.x, 192.168.x, 172.16–31.x).
pub fn is_private_ip(ip: &str) -> bool {
    if ip.starts_with("10.") || ip.starts_with("192.168.") {
        return true;
    }
    if let Some(rest) = ip.strip_prefix("172.") {
        if let Some(second) = rest.split('.').next().and_then(|s| s.parse::<u32>().ok()) {
            return (16..=31).contains(&second);
        }
    }
    false
}

// ---------------------------------------------------------------- мир

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct World {
    pub seed: u64,
    pub clock: GameClock,
    /// machines[0] — компьютер игрока.
    pub machines: Vec<Machine>,
    pub orgs: Vec<Org>,
    pub player: Player,
    pub settings: Settings,
    pub rng: Rng,
    /// Игровое время включения своей машины (для uptime).
    pub boot_time: u64,
    tick_acc_ms: u64,
    /// Условный сетевой трафик своей машины, КБ/с.
    pub net_in: f32,
    pub net_out: f32,
    /// Выключенные машины и время, когда они снова поднимутся.
    pub down_until: Vec<(MachineId, u64)>,
    /// Расследование центров безопасности и киберполиции (слежка).
    pub investigation: crate::hack::watch::Investigation,
    /// Задания группы hello-world (цепочка миссий).
    pub missions: crate::missions::MissionState,
    /// Деньги и долг: кредит, выплаты, покупки.
    pub finance: crate::money::Finance,
    /// Концовка: условие победы и финал (игра продолжается и после).
    pub ending: crate::ending::EndingState,
    /// Живой мир: активность сети, новости, события.
    pub life: crate::life::LifeState,
    /// Подработки: случайные оплачиваемые заказы.
    pub gigs: crate::gigs::GigState,
    /// Файрволы, которые игрок уже обошёл (id машин).
    pub fw_down: std::collections::BTreeSet<MachineId>,
    /// Установленные перехватчики ввода (кейлоггеры).
    pub keyloggers: Vec<crate::hack::tools::Keylogger>,
    /// Машины, захваченные в сеть (ботнет).
    pub botnet: Vec<MachineId>,
    /// Намайнено ботнетом всего, ₵.
    pub botnet_mined: i64,
    /// Дробный остаток дохода ботнета.
    pub botnet_acc: f64,
    /// Защита своей машины и ответный удар врага.
    pub defense: crate::hack::counter::PlayerDefense,
    /// Хакерский маркет скриптов в сети hw.
    pub market: crate::market::MarketState,
    /// Своя база: арендованные прокси-релеи.
    pub base: crate::base::BaseState,
    /// Открытые достижения.
    pub achievements: crate::achievements::AchState,
    /// Свой активный сценарий (из редактора сценариев).
    pub scenarios: crate::scenarios::ScenarioState,
}

impl Default for World {
    fn default() -> Self {
        World {
            seed: 0,
            clock: GameClock::new(),
            machines: Vec::new(),
            orgs: Vec::new(),
            player: Player::default(),
            settings: Settings::default(),
            rng: Rng::new(0),
            boot_time: START_EPOCH,
            tick_acc_ms: 0,
            net_in: 0.0,
            net_out: 0.0,
            down_until: Vec::new(),
            investigation: crate::hack::watch::Investigation::default(),
            missions: crate::missions::MissionState::default(),
            finance: crate::money::Finance::default(),
            ending: crate::ending::EndingState::default(),
            life: crate::life::LifeState::default(),
            gigs: crate::gigs::GigState::default(),
            fw_down: std::collections::BTreeSet::new(),
            keyloggers: Vec::new(),
            botnet: Vec::new(),
            botnet_mined: 0,
            botnet_acc: 0.0,
            defense: crate::hack::counter::PlayerDefense::default(),
            market: crate::market::MarketState::default(),
            base: crate::base::BaseState::default(),
            achievements: crate::achievements::AchState::default(),
            scenarios: crate::scenarios::ScenarioState::default(),
        }
    }
}

impl World {
    /// Построить мир из зерна: сеть делает генератор `net::gen`.
    pub fn new(seed: u64, handle: &str, password: &str) -> World {
        let mut rng = Rng::new(seed);
        let g = gen::generate(&mut rng, handle, password);
        let mut w = World {
            seed,
            clock: GameClock::new(),
            machines: g.machines,
            orgs: g.orgs,
            player: Player {
                handle: handle.to_string(),
                money: 0,
                known_machines: g.player_known,
                ..Player::default()
            },
            settings: Settings::default(),
            rng,
            boot_time: START_EPOCH,
            ..World::default()
        };
        if w.machines.is_empty() {
            w.machines.push(fallback_player_machine(handle, password));
        }
        let now = w.clock.now();
        for m in &mut w.machines {
            m.fs.set_clock(now);
            m.procs.set_clock(now);
        }
        w.player.known_machines.retain(|id| *id != 0);
        // построить цепочку заданий сразу — первое письмо приходит с порога
        crate::missions::ensure_init(&mut w);
        // задать кредит (банк из сюжета забрал половину дома)
        crate::money::ensure_init(&mut w);
        // открыть первые подработки
        crate::gigs::ensure(&mut w);
        w
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|e| format!("{{\"error\":\"{}\"}}", e))
    }

    pub fn from_json(s: &str) -> Result<World, String> {
        let w: World = serde_json::from_str(s).map_err(|e| format!("не удалось прочитать сохранение: {}", e))?;
        if w.machines.is_empty() {
            return Err("сохранение пустое: нет машин".to_string());
        }
        Ok(w)
    }

    pub fn now(&self) -> u64 {
        self.clock.now()
    }

    /// Ход времени: часы, колебания процессов, сетевой шум, подъём выключенных машин.
    pub fn tick(&mut self, dt_ms: u64) {
        self.clock.tick(dt_ms);
        self.tick_acc_ms += dt_ms;
        while self.tick_acc_ms >= 1000 {
            self.tick_acc_ms -= 1000;
            self.second_tick();
        }
    }

    fn second_tick(&mut self) {
        let now = self.clock.now();
        for m in &mut self.machines {
            m.fs.set_clock(now);
            m.procs.set_clock(now);
            if m.up {
                m.procs.tick(&mut self.rng);
            }
        }
        let base_in = 2.0 + self.machines[0].procs.len() as f32 * 0.4;
        self.net_in = (self.net_in * 0.7 + self.rng.jitter(base_in, base_in) * 0.3).max(0.0);
        self.net_out = (self.net_out * 0.7 + self.rng.jitter(base_in * 0.4, base_in * 0.4) * 0.3).max(0.0);
        let mut wake = Vec::new();
        self.down_until.retain(|(id, t)| {
            if *t <= now {
                wake.push(*id);
                false
            } else {
                true
            }
        });
        for id in wake {
            if let Some(m) = self.machines.iter_mut().find(|m| m.id == id) {
                m.up = true;
                m.log(now, LogKind::Service, "система запущена");
            }
        }
        // подозрение к игроку медленно спадает со временем
        crate::hack::tracks::decay(self, 1);
        // слежка: центры безопасности и киберполиция ведут расследование
        crate::hack::watch::sweep(self, 1);
        // задания группы hello-world: проверка выполнения
        crate::missions::check(self);
        // деньги и долг: набегают проценты по кредиту
        crate::money::tick(self, 1);
        // концовка: долг закрыт + цепочка пройдена (игра продолжается и после)
        crate::ending::check(self);
        // живой мир: сеть не стоит на месте
        crate::life::tick(self, 1);
        // подработки: проверить выполнение и пополнить доску
        crate::gigs::check(self);
        // перехватчики ввода и ботнет
        crate::hack::tools::tick_keyloggers(self, 1);
        crate::hack::tools::tick_botnet(self, 1);
        // ответный удар наблюдателей по машине игрока
        crate::hack::counter::tick(self, 1);
        // плата за аренду релеев
        crate::base::tick(self, 1);
        // достижения: проверить, не открылось ли новое
        crate::achievements::check(self);
        // свой сценарий из редактора
        crate::scenarios::check(self);
    }

    // ---------- машины и организации ----------

    pub fn machine(&self, id: MachineId) -> &Machine {
        self.machines
            .iter()
            .find(|m| m.id == id)
            .unwrap_or_else(|| panic!("нет машины с id {}", id))
    }

    pub fn machine_mut(&mut self, id: MachineId) -> &mut Machine {
        self.machines
            .iter_mut()
            .find(|m| m.id == id)
            .unwrap_or_else(|| panic!("нет машины с id {}", id))
    }

    pub fn has_machine(&self, id: MachineId) -> bool {
        self.machines.iter().any(|m| m.id == id)
    }

    pub fn player_machine(&self) -> &Machine {
        &self.machines[0]
    }

    pub fn org(&self, id: u32) -> Option<&Org> {
        self.orgs.iter().find(|o| o.id == id)
    }

    pub fn org_of(&self, machine: MachineId) -> Option<&Org> {
        self.machine(machine).org.and_then(|o| self.org(o))
    }

    /// Машина по публичному адресу, затем по локальному.
    pub fn find_by_ip(&self, ip: &str) -> Option<MachineId> {
        self.machines
            .iter()
            .find(|m| m.ip == ip)
            .or_else(|| self.machines.iter().find(|m| m.lan_ip.as_deref() == Some(ip)))
            .map(|m| m.id)
    }

    pub fn find_by_hostname(&self, name: &str) -> Option<MachineId> {
        self.machines.iter().find(|m| m.hostname.eq_ignore_ascii_case(name)).map(|m| m.id)
    }

    /// DNS: имя машины или домен → адрес. IP возвращается как есть.
    pub fn resolve(&self, name: &str) -> Option<String> {
        if looks_like_ip(name) {
            return Some(name.to_string());
        }
        if name == "localhost" {
            return Some("127.0.0.1".to_string());
        }
        if let Some(ip) = dns::resolve(&self.machines, &self.orgs, name) {
            return Some(ip);
        }
        self.machines
            .iter()
            .find(|m| m.hostname.eq_ignore_ascii_case(name) || m.domain.as_deref().map(|d| d.eq_ignore_ascii_case(name)).unwrap_or(false))
            .map(|m| m.ip.clone())
    }

    /// Виден ли адрес `to` с машины `from`. Публичные адреса видны всем;
    /// локальный — изнутри той же организации, с её шлюза и из её LAN.
    pub fn reachable(&self, from: MachineId, to: MachineId, via_lan: bool) -> bool {
        if from == to {
            return true;
        }
        if !via_lan {
            return true;
        }
        let f = self.machine(from);
        let t = self.machine(to);
        if f.org.is_some() && f.org == t.org {
            return true;
        }
        if let Some(org) = t.org.and_then(|o| self.org(o)) {
            if org.gateway == from {
                return true;
            }
        }
        if f.router == Some(to) || t.router == Some(from) {
            return true;
        }
        if f.router.is_some() && f.router == t.router {
            return true;
        }
        false
    }

    /// Найти машину по адресу или имени с проверкой достижимости и включённости.
    pub fn lookup(&self, from: MachineId, addr: &str) -> Result<MachineId, NetError> {
        let ip = self.resolve(addr).ok_or_else(|| NetError::Unknown(addr.to_string()))?;
        if ip == "127.0.0.1" {
            return Ok(from);
        }
        let id = self.find_by_ip(&ip).ok_or_else(|| NetError::Unknown(addr.to_string()))?;
        let m = self.machine(id);
        let via_lan = is_private_ip(&ip) || m.ip != ip;
        if !self.reachable(from, id, via_lan) {
            return Err(NetError::Unreachable(ip));
        }
        if !m.up {
            return Err(NetError::Down(ip));
        }
        Ok(id)
    }

    /// Адрес машины, каким его видит `from` (локальный изнутри, иначе публичный).
    pub fn address_for(&self, from: MachineId, to: MachineId) -> String {
        let f = self.machine(from);
        let t = self.machine(to);
        if let Some(lan) = &t.lan_ip {
            if f.org.is_some() && f.org == t.org || f.router == Some(to) || t.router == Some(from) {
                return lan.clone();
            }
        }
        t.ip.clone()
    }

    pub fn is_known(&self, id: MachineId) -> bool {
        id == 0 || self.player.known_machines.contains(&id)
    }

    pub fn mark_known(&mut self, id: MachineId) {
        if id != 0 && !self.player.known_machines.contains(&id) {
            self.player.known_machines.push(id);
        }
    }

    pub fn mark_scanned(&mut self, id: MachineId) {
        self.mark_known(id);
        if !self.player.scanned.contains(&id) {
            self.player.scanned.push(id);
        }
    }

    /// Записать найденный логин. Один и тот же логин (машина+пользователь)
    /// добывается лишь единожды — иначе сбыт стал бы фермой. Возвращает true,
    /// если логин новый.
    pub fn add_found_cred(&mut self, cred: crate::net::machine::FoundCred) -> bool {
        let key = format!("{}:{}", cred.machine, cred.user);
        if self.player.harvested.contains(&key) {
            return false;
        }
        self.player.harvested.insert(key);
        self.player.found_creds.push(cred);
        true
    }

    /// Запись в журнал машины и в её файл журнала (/var/log/auth.log или system.log).
    pub fn log(&mut self, id: MachineId, kind: LogKind, text: &str) {
        let now = self.clock.now();
        let m = self.machine_mut(id);
        m.log(now, kind, text);
        let file = match kind {
            LogKind::Login | LogKind::LoginFailed | LogKind::Logout => "/var/log/auth.log",
            _ => "/var/log/system.log",
        };
        let tag = match kind {
            LogKind::Login => "login",
            LogKind::LoginFailed => "login-failed",
            LogKind::Logout => "logout",
            LogKind::Exec => "exec",
            LogKind::FileChange => "file",
            LogKind::Service => "service",
            LogKind::Net => "net",
            LogKind::Warn => "warn",
        };
        let line = format!("{} {} {}: {}\n", GameClock::format_time(now), m.hostname, tag, text);
        let owner = "root".to_string();
        m.fs.append_text(file, &line, &owner).ok();
    }

    /// Выключить машину на `secs` игровых секунд.
    pub fn shutdown_machine(&mut self, id: MachineId, secs: u64) {
        let until = self.clock.now() + secs;
        self.machine_mut(id).up = false;
        self.down_until.retain(|(m, _)| *m != id);
        self.down_until.push((id, until));
    }

    /// Перезапуск своей машины: отсчёт uptime с нуля.
    pub fn reboot_player(&mut self) {
        self.boot_time = self.clock.now();
    }

    pub fn uptime(&self) -> u64 {
        self.clock.now().saturating_sub(self.boot_time)
    }

    // ---------- http ----------

    /// Запрос к веб-службе: `http://host[:port]/path`, `home://` — сводка своей машины.
    pub fn http(&self, from: MachineId, url: &str) -> HttpPage {
        let url = url.trim();
        let (scheme, rest) = match url.split_once("://") {
            Some((s, r)) => (s.to_ascii_lowercase(), r),
            None => ("http".to_string(), url),
        };
        if scheme == "home" {
            return self.home_page();
        }
        if scheme != "http" && scheme != "https" {
            return HttpPage::error(400, "Неизвестная схема", &format!("Браузер понимает только http, https и home, а не «{}».", scheme));
        }
        let (hostport, path) = match rest.split_once('/') {
            Some((h, p)) => (h, format!("/{}", p)),
            None => (rest, "/".to_string()),
        };
        let default_port: u16 = if scheme == "https" { 443 } else { 80 };
        let (host, port) = match hostport.rsplit_once(':') {
            Some((h, p)) => (h, p.parse::<u16>().unwrap_or(default_port)),
            None => (hostport, default_port),
        };
        if host.is_empty() {
            return HttpPage::error(400, "Пустой адрес", "Укажите узел: http://имя или http://ip.");
        }
        let id = match self.lookup(from, host) {
            Ok(id) => id,
            Err(NetError::Unknown(a)) => return HttpPage::error(404, "Нет такого узла", &format!("Имя «{}» не разрешается.", a)),
            Err(NetError::Unreachable(a)) => return HttpPage::error(503, "Нет маршрута", &format!("Адрес {} находится в чужой подсети.", a)),
            Err(NetError::Down(a)) => return HttpPage::error(504, "Узел не отвечает", &format!("Машина {} выключена.", a)),
        };
        let m = self.machine(id);
        let svc = m.services.iter().find(|s| s.port == port && s.running);
        let svc = match svc {
            Some(s) => s,
            None => {
                return HttpPage::error(502, "Порт закрыт", &format!("На {} порт {} не отвечает.", host, port));
            }
        };
        if svc.name != "http" && svc.name != "https" {
            return HttpPage::error(502, "Не веб-служба", &format!("На порту {} работает {} — это не веб-сервер.", port, svc.name));
        }
        let want = crate::net::machine::normalize_path(&path);
        // доска заданий hello-world рисуется из состояния миссий
        if (m.hostname == "gate.hw" && want == "/board") || m.hostname == "board.hw" {
            return HttpPage { status: 200, title: "Доска заданий".to_string(), body: crate::missions::board_html(self) };
        }
        // барахолка группы — из состояния финансов
        if (m.hostname == "gate.hw" && want == "/shop") || m.hostname == "shop.hw" {
            return HttpPage { status: 200, title: "Барахолка".to_string(), body: crate::money::shop_html(self) };
        }
        // маркет скриптов
        if (m.hostname == "gate.hw" && want == "/market") || m.hostname == "market.hw" {
            return HttpPage { status: 200, title: "Маркет".to_string(), body: crate::market::market_html(self) };
        }
        let page = svc
            .pages
            .iter()
            .find(|p| crate::net::machine::normalize_path(&p.path) == want)
            .or_else(|| if want == "/" { svc.pages.first() } else { None });
        match page {
            Some(p) => HttpPage { status: 200, title: p.title.clone(), body: p.html.clone() },
            None => HttpPage {
                status: 404,
                title: "Страница не найдена".to_string(),
                body: format!(
                    "<h1>404</h1><p>На {} нет страницы <code>{}</code>.</p><p><a href=\"http://{}/\">На главную</a></p>",
                    m.hostname, want, host
                ),
            },
        }
    }

    fn home_page(&self) -> HttpPage {
        let m = self.player_machine();
        let mut body = format!(
            "<h1>{}</h1><p>Адрес: <code>{}</code> · Система: {} · Время: {}</p>",
            m.hostname,
            m.ip,
            m.os,
            GameClock::format_time(self.clock.now())
        );
        body.push_str("<h2>Известные узлы</h2><ul>");
        for id in &self.player.known_machines {
            if let Some(k) = self.machines.iter().find(|x| x.id == *id) {
                let has_web = k.services.iter().any(|s| s.running && (s.name == "http" || s.name == "https"));
                if has_web {
                    body.push_str(&format!("<li><a href=\"http://{0}/\">{1}</a> — {0}</li>", k.ip, k.hostname));
                } else {
                    body.push_str(&format!("<li>{} — {}</li>", k.hostname, k.ip));
                }
            }
        }
        body.push_str("</ul><p>Команды терминала: <code>scan ip</code>, <code>ssh user@ip пароль</code>, <code>help</code>.</p>");
        HttpPage { status: 200, title: "Домашняя страница".to_string(), body }
    }
}

/// Запасной компьютер игрока, если генератор ничего не вернул.
fn fallback_player_machine(handle: &str, password: &str) -> Machine {
    let mut users = crate::users::UserDb::new_root(password);
    users.add(handle, password, true);
    let mut fs = crate::vfs::Vfs::new_unix(handle);
    fs.write_text(&format!("/home/{}/readme.txt", handle), "Добро пожаловать в Blackwire. Начните с команды help.\n", handle).ok();
    let mut procs = crate::procs::ProcTable::new();
    procs.spawn_daemon("sshd", "root", 6);
    Machine {
        id: 0,
        hostname: format!("{}-pc", handle),
        ip: "87.240.12.44".to_string(),
        lan_ip: Some("192.168.0.10".to_string()),
        domain: None,
        os: "Blackwire OS 1.0".to_string(),
        cpu: "Quad 3.2 GHz".to_string(),
        ram_mb: 8192,
        disk_mb: 256_000,
        fs,
        users,
        procs,
        services: Vec::new(),
        logs: Vec::new(),
        mail: Vec::new(),
        kind: MachineKind::Player,
        org: None,
        router: None,
        up: true,
        x: 0.5,
        y: 0.5,
    }
}

/// Запись журнала в одну строку для команды `logs`.
pub fn format_log(e: &LogEntry) -> String {
    let tag = match e.kind {
        LogKind::Login => "вход",
        LogKind::LoginFailed => "ошибка входа",
        LogKind::Logout => "выход",
        LogKind::Exec => "запуск",
        LogKind::FileChange => "файлы",
        LogKind::Service => "служба",
        LogKind::Net => "сеть",
        LogKind::Warn => "внимание",
    };
    format!("{}  {:<13} {}", GameClock::format_time(e.time), tag, e.text)
}
