//! Условные показатели машины для монитора и полосы состояния: процессор,
//! память, сеть, время работы, список процессов. Значения живые — считаются
//! из таблицы процессов, которая колеблется на каждом тике мира.

use serde::{Deserialize, Serialize};

use crate::net::machine::MachineId;
use crate::world::World;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ProcInfo {
    pub pid: u32,
    pub name: String,
    pub user: String,
    pub cpu: f32,
    pub mem: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Status {
    pub hostname: String,
    pub ip: String,
    /// Секунды с включения.
    pub uptime: u64,
    /// Загрузка процессора, проценты.
    pub cpu: f32,
    /// Память, МБ.
    pub mem_used: u32,
    pub mem_total: u32,
    /// Сеть, КБ/с.
    pub net_in: f32,
    pub net_out: f32,
    pub procs: Vec<ProcInfo>,
}

/// Память, занятая ядром и кэшем помимо процессов, МБ.
const BASE_MEM_MB: u32 = 380;

/// Показатели машины `id` (для своей машины — с uptime и сетью мира).
pub fn status(world: &World, id: MachineId) -> Status {
    let m = world.machine(id);
    let procs: Vec<ProcInfo> = m
        .procs
        .list()
        .iter()
        .map(|p| ProcInfo { pid: p.pid, name: p.name.clone(), user: p.user.clone(), cpu: p.cpu, mem: p.mem_mb })
        .collect();
    let mem_used = (BASE_MEM_MB + m.procs.total_mem()).min(m.ram_mb);
    let (net_in, net_out) = if id == 0 { (world.net_in, world.net_out) } else { (world.net_in * 0.3, world.net_out * 0.3) };
    Status {
        hostname: m.hostname.clone(),
        ip: m.ip.clone(),
        uptime: if id == 0 { world.uptime() } else { world.now().saturating_sub(crate::world::START_EPOCH) + 86_400 * 3 },
        cpu: m.procs.total_cpu(),
        mem_used,
        mem_total: m.ram_mb,
        net_in,
        net_out,
        procs,
    }
}
