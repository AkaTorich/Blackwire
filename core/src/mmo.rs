//! Общий мир для онлайна. На сервере один мир на всех: организации, банки,
//! больницы, сеть — общие. У каждого игрока свой компьютер и роутер (обычные
//! машины общей сети — их видят и могут взломать другие) и своя часть
//! состояния: сюжет hello-world, долг, подработки, подозрение, достижения,
//! сеансы терминала.
//!
//! Вся одиночная логика написана для «игрока» и его компьютера `machines[0]`.
//! Чтобы её не переписывать, перед действием игрока сервер подставляет его
//! часть в мир (`Game::swap_player`), а после — убирает обратно тем же вызовом.
//! Компьютер игрока при этом меняется местами с заглушкой на месте 0, так что
//! правило «номер машины = её место в списке» сохраняется.

use std::collections::{BTreeSet, HashMap};

use serde::{Deserialize, Serialize};

use crate::game::Game;
use crate::net::machine::{LogKind, Machine, MachineId, MachineKind};
use crate::net::player::{build_player, build_router, Known};
use crate::session::{PendingScript, Session};
use crate::world::{Player, Settings, World};
use crate::UiAction;

/// Часть мира, принадлежащая одному игроку.
#[derive(Serialize, Deserialize, Default)]
pub struct PlayerCtx {
    /// Постоянный номер компьютера игрока в общем мире.
    pub home: MachineId,
    /// Номер его роутера.
    pub router: MachineId,
    /// Язык игрока: "ru" или "en".
    #[serde(default)]
    pub lang: String,
    pub player: Player,
    pub settings: Settings,
    pub boot_time: u64,
    pub investigation: crate::hack::watch::Investigation,
    pub missions: crate::missions::MissionState,
    pub finance: crate::money::Finance,
    pub ending: crate::ending::EndingState,
    pub gigs: crate::gigs::GigState,
    pub fw_down: BTreeSet<MachineId>,
    pub keyloggers: Vec<crate::hack::tools::Keylogger>,
    pub botnet: Vec<MachineId>,
    pub botnet_mined: i64,
    pub botnet_acc: f64,
    pub defense: crate::hack::counter::PlayerDefense,
    pub market: crate::market::MarketState,
    pub base: crate::base::BaseState,
    pub achievements: crate::achievements::AchState,
    pub scenarios: crate::scenarios::ScenarioState,
    pub plugins: crate::plugins::SavedPlugins,
    #[serde(skip)]
    pub net_in: f32,
    #[serde(skip)]
    pub net_out: f32,
    #[serde(skip)]
    pub plugin_events: Vec<(String, serde_json::Value)>,
    // часть `Game` — живёт только пока сервер работает
    #[serde(skip)]
    pub sessions: HashMap<u32, Session>,
    #[serde(skip)]
    pub next_session: u32,
    #[serde(skip)]
    pub pending_ui: Vec<UiAction>,
    #[serde(skip)]
    pub pending_scripts: Vec<PendingScript>,
}

impl World {
    /// Построить общий мир сервера: обычный мир из зерна, но без игрока —
    /// на месте 0 заглушка, личные поля пусты.
    pub fn mmo_new(seed: u64) -> World {
        let mut w = World::new(seed, "nobody", "");
        w.machines[0] = placeholder();
        // роутер одиночного игрока остаётся в сети ничьим модемом
        if let Some(r) = w.machines.get_mut(1) {
            if r.kind == MachineKind::Router {
                r.hostname = "dsl-modem".to_string();
            }
        }
        w.clear_player_part();
        w
    }

    /// Обнулить личную часть мира (после `mmo_new` и при загрузке сервера).
    fn clear_player_part(&mut self) {
        let mut empty = PlayerCtx::default();
        self.swap_fields(&mut empty);
    }

    /// Поменять местами личные поля мира и игрока (без машин).
    fn swap_fields(&mut self, c: &mut PlayerCtx) {
        use std::mem::swap;
        swap(&mut self.player, &mut c.player);
        swap(&mut self.settings, &mut c.settings);
        swap(&mut self.boot_time, &mut c.boot_time);
        swap(&mut self.investigation, &mut c.investigation);
        swap(&mut self.missions, &mut c.missions);
        swap(&mut self.finance, &mut c.finance);
        swap(&mut self.ending, &mut c.ending);
        swap(&mut self.gigs, &mut c.gigs);
        swap(&mut self.fw_down, &mut c.fw_down);
        swap(&mut self.keyloggers, &mut c.keyloggers);
        swap(&mut self.botnet, &mut c.botnet);
        swap(&mut self.botnet_mined, &mut c.botnet_mined);
        swap(&mut self.botnet_acc, &mut c.botnet_acc);
        swap(&mut self.defense, &mut c.defense);
        swap(&mut self.market, &mut c.market);
        swap(&mut self.base, &mut c.base);
        swap(&mut self.achievements, &mut c.achievements);
        swap(&mut self.scenarios, &mut c.scenarios);
        swap(&mut self.plugins, &mut c.plugins);
        swap(&mut self.net_in, &mut c.net_in);
        swap(&mut self.net_out, &mut c.net_out);
        swap(&mut self.plugin_events, &mut c.plugin_events);
    }

    /// Поставить компьютер `home` на место 0 (или вернуть его на своё место —
    /// вызов симметричен).
    fn swap_home(&mut self, home: MachineId) {
        if home == 0 {
            return;
        }
        if let Some(pos) = self.machines.iter().position(|m| m.id == home) {
            self.machines.swap(0, pos);
            self.machines[0].id = 0;
            self.machines[pos].id = home;
        }
    }

    /// Подставить игрока в мир или убрать его (повторный вызов возвращает всё).
    pub fn swap_player(&mut self, c: &mut PlayerCtx) {
        self.swap_fields(c);
        self.swap_home(c.home);
    }

    /// Номер для новой машины.
    fn mmo_next_id(&self) -> MachineId {
        self.machines.iter().map(|m| m.id).max().unwrap_or(0) + 1
    }

    /// Свободный домашний публичный адрес 87.x.x.x.
    fn mmo_home_ip(&mut self) -> String {
        loop {
            let ip = format!("87.{}.{}.{}", self.rng.range(0, 256), self.rng.range(0, 256), self.rng.range(1, 255));
            if self.find_by_ip(&ip).is_none() {
                return ip;
            }
        }
    }

    /// Завести нового игрока: компьютер, роутер, письма и сюжет. Мир должен
    /// быть без подставленного игрока. Возвращает его часть (уже убранную).
    /// `pc_password` — пароль его игрового компьютера (не пароль учётной записи).
    pub fn mmo_add_player(&mut self, handle: &str, pc_password: &str, lang: &str) -> PlayerCtx {
        let now = self.clock.now();
        let router_id = self.mmo_next_id();
        let home_id = router_id + 1;
        let pc_ip = self.mmo_home_ip();
        let router_ip = self.mmo_home_ip();
        // своя локальная сеть у каждого: 172.16+.N.1 — роутер, .2 — компьютер
        let k = self.machines.iter().filter(|m| m.kind == MachineKind::Player).count();
        let net = format!("172.{}.{}", 16 + k / 250, k % 250);
        let x = 0.03 + self.rng.range(0, 200) as f32 / 1000.0;
        let y = 0.76 + self.rng.range(0, 200) as f32 / 1000.0;

        // известные узлы — как у одиночного игрока: DNS провайдера, сайт, домашний ПК
        let dns = self.machines.iter().find(|m| m.kind == MachineKind::Dns).map(|m| m.id);
        let site = self.machines.iter().find(|m| m.kind == MachineKind::Web && m.is_public()).map(|m| m.id);
        let home_pc = self.machines.iter().find(|m| m.kind == MachineKind::HomePc).map(|m| m.id);
        let router_host = format!("{}-router", handle);
        let mut known = vec![Known { host: router_host.clone(), ip: router_ip.clone() }];
        let mut known_ids = vec![router_id];
        for id in [dns, site, home_pc].into_iter().flatten() {
            let m = self.machine(id);
            known.push(Known { host: m.hostname.clone(), ip: m.ip.clone() });
            known_ids.push(id);
        }

        let mut router = build_router(router_id, router_ip, format!("{}.1", net), true, x + 0.04, y - 0.06, now, &mut self.rng);
        router.hostname = router_host;
        let mut pc = build_player(home_id, handle, pc_password, pc_ip.clone(), format!("{}.2", net), Some(router_id), &known, x, y, now, &mut self.rng);

        // те же стартовые письма, что и в одиночной игре
        let bank = self.orgs.iter().find(|o| o.kind == crate::net::orgs::OrgKind::Bank);
        let corp = self.orgs.iter().find(|o| o.kind == crate::net::orgs::OrgKind::Corporation);
        let isp = self.orgs.iter().find(|o| o.kind == crate::net::orgs::OrgKind::Isp);
        let inbox = crate::net::mail::player_inbox(
            handle,
            &pc_ip,
            bank.map(|o| o.name.as_str()).unwrap_or(tr!("Банк", "Bank")),
            bank.map(|o| o.domain.as_str()).unwrap_or("bank.net"),
            corp.map(|o| o.name.as_str()).unwrap_or(tr!("Корпорация", "Corporation")),
            corp.map(|o| o.domain.as_str()).unwrap_or("corp.net"),
            isp.map(|o| o.domain.as_str()).unwrap_or("meridian.net"),
            &self.invite,
            now,
        );
        for msg in inbox {
            pc.push_mail(msg);
        }
        pc.log(now, LogKind::Service, tr!("компьютер подключён к сети", "computer connected to the network"));
        self.machines.push(router);
        self.machines.push(pc);

        let mut ctx = PlayerCtx {
            home: home_id,
            router: router_id,
            lang: lang.to_string(),
            player: Player { handle: handle.to_string(), known_machines: known_ids, ..Player::default() },
            boot_time: now,
            next_session: 1,
            ..PlayerCtx::default()
        };
        ctx.player.notes.push(trf!("Пароль твоего компьютера: {}", "Your computer's password: {}", pc_password));
        // сюжет, кредит и подработки — как в начале одиночной игры
        self.swap_player(&mut ctx);
        crate::missions::ensure_init(self);
        crate::money::ensure_init(self);
        crate::gigs::ensure(self);
        self.swap_player(&mut ctx);
        ctx
    }

    /// Общий ход времени сервера (без игрока): часы, процессы машин, подъём
    /// выключенных узлов, живой мир.
    pub fn mmo_tick_shared(&mut self, dt_ms: u64) -> u32 {
        self.clock.tick(dt_ms);
        self.tick_acc_ms += dt_ms;
        let mut seconds = 0;
        while self.tick_acc_ms >= 1000 {
            self.tick_acc_ms -= 1000;
            seconds += 1;
            let now = self.clock.now();
            for m in &mut self.machines {
                m.fs.set_clock(now);
                m.procs.set_clock(now);
                if m.up {
                    m.procs.tick(&mut self.rng);
                }
            }
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
                    m.log(now, LogKind::Service, tr!("система запущена", "system started"));
                }
            }
            crate::life::tick(self, 1);
        }
        seconds
    }

    /// Личный ход времени подставленного игрока (одна секунда).
    pub fn mmo_tick_player(&mut self) {
        let base_in = 2.0 + self.machines[0].procs.len() as f32 * 0.4;
        self.net_in = (self.net_in * 0.7 + self.rng.jitter(base_in, base_in) * 0.3).max(0.0);
        self.net_out = (self.net_out * 0.7 + self.rng.jitter(base_in * 0.4, base_in * 0.4) * 0.3).max(0.0);
        crate::hack::tracks::decay(self, 1);
        crate::hack::watch::sweep(self, 1);
        crate::missions::check(self);
        crate::money::tick(self, 1);
        crate::ending::check(self);
        crate::gigs::check(self);
        crate::hack::tools::tick_keyloggers(self, 1);
        crate::hack::tools::tick_botnet(self, 1);
        crate::hack::counter::tick(self, 1);
        crate::base::tick(self, 1);
        crate::achievements::check(self);
        crate::scenarios::check(self);
    }

    /// Все компьютеры игроков общей сети (кроме заглушки).
    pub fn mmo_player_machines(&self) -> impl Iterator<Item = &Machine> {
        self.machines.iter().filter(|m| m.kind == MachineKind::Player && m.id != 0)
    }
}

impl Game {
    /// Подставить игрока в игру или убрать его (симметрично).
    pub fn swap_player(&mut self, c: &mut PlayerCtx) {
        use std::mem::swap;
        swap(&mut self.sessions, &mut c.sessions);
        swap(&mut self.next_session, &mut c.next_session);
        swap(&mut self.pending_ui, &mut c.pending_ui);
        swap(&mut self.pending_scripts, &mut c.pending_scripts);
        self.world.swap_player(c);
    }

    /// Личный ход подставленного игрока: его часть мира и события плагинов.
    pub fn mmo_tick_player(&mut self) {
        let stamp = crate::plugins::events::Stamp::new(&self.world);
        self.world.mmo_tick_player();
        self.plugin_timers();
        self.plugin_event("on_tick", serde_json::json!({ "dt_ms": 1000 }));
        self.plugin_changes(stamp);
    }
}

/// Заглушка на месте 0, пока ни один игрок не подставлен: выключена, без
/// адреса, в сети не видна.
fn placeholder() -> Machine {
    let mut m = crate::world::fallback_player_machine("offline", "");
    m.hostname = "-".to_string();
    m.ip = "0.0.0.0".to_string();
    m.lan_ip = None;
    m.up = false;
    m.services.clear();
    m.mail.clear();
    m.x = -1.0;
    m.y = -1.0;
    m
}
