//! Своя база: игрок арендует в сети прокси-релеи — настоящие машины, которыми
//! владеет полностью (root). Через них ходят, чтобы прятать настоящий адрес, и
//! держат на них инструменты. Аренда стоит ₵ сразу и понемногу съедает деньги
//! со временем; перестал платить — релей забирают.

use serde::{Deserialize, Serialize};

use crate::net::machine::{LogKind, Machine, MachineId, MachineKind};
use crate::net::services::Service;
use crate::procs::ProcTable;
use crate::users::UserDb;
use crate::vfs::Vfs;
use crate::world::World;

const RENT_PRICE: i64 = 500;
const RENT_PER_DAY: i64 = 50;

/// Арендованный релей.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RentedRelay {
    pub machine: MachineId,
    pub host: String,
    pub ip: String,
    pub password: String,
}

/// Состояние базы.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct BaseState {
    pub rented: Vec<RentedRelay>,
    pub rent_acc: f64,
    #[serde(skip)]
    pub pending: Option<BaseEvent>,
}

#[derive(Clone, Debug)]
pub struct BaseEvent {
    pub title: String,
    pub text: String,
}

pub fn take_pending(world: &mut World) -> Option<BaseEvent> {
    world.base.pending.take()
}

fn next_id(world: &World) -> MachineId {
    world.machines.iter().map(|m| m.id).max().unwrap_or(0) + 1
}

fn unique_ip(world: &mut World) -> String {
    for _ in 0..64 {
        let ip = format!("185.{}.{}.{}", world.rng.range(10, 240), world.rng.range(0, 255), world.rng.range(2, 254));
        if world.find_by_ip(&ip).is_none() {
            return ip;
        }
    }
    format!("185.200.{}.{}", world.rng.range(0, 255), world.rng.range(2, 254))
}

fn ssh_service() -> Service {
    Service {
        port: 22,
        name: "ssh".to_string(),
        version: "OpenSSH_9.6p1".to_string(),
        running: true,
        banner: "SSH-2.0-OpenSSH_9.6p1".to_string(),
        pages: Vec::new(),
    }
}

/// Арендовать новый релей. Игрок получает его адрес и пароль root.
pub fn rent(world: &mut World) -> Result<RentedRelay, String> {
    if world.player.money < RENT_PRICE {
        return Err(format!("аренда стоит {} ₵, а на счету {} ₵", RENT_PRICE, world.player.money));
    }
    world.player.money -= RENT_PRICE;
    let id = next_id(world);
    let n = world.base.rented.len() + 1;
    let host = format!("relay-{}", n);
    let ip = unique_ip(world);
    let password = crate::net::names::strong_password(&mut world.rng);
    let now = world.now();

    let mut fs = Vfs::new_unix("root");
    fs.set_clock(now);
    let _ = fs.write_text("/etc/motd", "Личный релей. Только для своих.\n", "root");
    let mut procs = ProcTable::new();
    procs.set_clock(now);

    let x = world.rng.range(5, 95) as f32 / 100.0;
    let y = world.rng.range(5, 95) as f32 / 100.0;
    let m = Machine {
        device_type: None, custom_vulns: Vec::new(),
        id,
        hostname: host.clone(),
        ip: ip.clone(),
        lan_ip: None,
        domain: None,
        os: "Debian 12".to_string(),
        cpu: "VPS 2 vCPU".to_string(),
        ram_mb: 2048,
        disk_mb: 20_000,
        fs,
        users: UserDb::new_root(&password),
        procs,
        services: vec![ssh_service()],
        logs: Vec::new(),
        mail: Vec::new(),
        kind: MachineKind::Server,
        org: None,
        router: None,
        up: true,
        x,
        y,
    };
    world.machines.push(m);
    world.player.owned.push(id);
    world.mark_known(id);
    world.log(id, LogKind::Service, "релей запущен");

    let relay = RentedRelay { machine: id, host, ip, password };
    world.base.rented.push(relay.clone());
    Ok(relay)
}

/// Отказаться от релея по адресу.
pub fn release(world: &mut World, addr: &str) -> Result<String, String> {
    let pos = world.base.rented.iter().position(|r| r.ip == addr || r.host == addr);
    let pos = pos.ok_or_else(|| "нет такого релея".to_string())?;
    let r = world.base.rented.remove(pos);
    world.machines.retain(|m| m.id != r.machine);
    world.player.owned.retain(|id| *id != r.machine);
    Ok(r.host)
}

fn reclaim_one(world: &mut World) {
    if let Some(r) = world.base.rented.pop() {
        world.machines.retain(|m| m.id != r.machine);
        world.player.owned.retain(|id| *id != r.machine);
        world.base.pending = Some(BaseEvent { title: "Аренда".to_string(), text: format!("Нечем платить — релей {} забрали.", r.host) });
    }
}

/// Плата за аренду со временем. Зовётся из `World::tick`.
pub fn tick(world: &mut World, dt: u64) {
    if world.base.rented.is_empty() {
        return;
    }
    world.base.rent_acc += world.base.rented.len() as f64 * (RENT_PER_DAY as f64 / 86_400.0) * dt as f64;
    let whole = world.base.rent_acc.floor();
    if whole >= 1.0 {
        world.base.rent_acc -= whole;
        let owed = whole as i64;
        if world.player.money >= owed {
            world.player.money -= owed;
        } else {
            world.player.money = 0;
            reclaim_one(world);
        }
    }
}

pub fn rent_price() -> i64 {
    RENT_PRICE
}
pub fn rent_per_day() -> i64 {
    RENT_PER_DAY
}
