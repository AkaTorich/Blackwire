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

#[cfg(test)]
mod tests {
    use crate::{Game,world::World};

    #[test]
    fn application_memory_tracks_each_window_and_is_released_on_close(){
        let mut game=Game::new(World::new(7,"memory_test",""));
        let baseline=game.status().mem_used;
        let remote=super::status(&game.world,game.world.machines[1].id).mem_used;
        let browser=game.world.machines[0].procs.spawn_application("w1","browser","memory_test");
        assert_eq!(game.status().mem_used,baseline+96);
        let same=game.world.machines[0].procs.spawn_application("w1","browser","memory_test");
        assert_eq!(same,browser);
        game.world.machines[0].procs.spawn_application("w2","editor","memory_test");
        assert_eq!(game.status().mem_used,baseline+144);
        assert_eq!(super::status(&game.world,game.world.machines[1].id).mem_used,remote);
        assert!(game.status().procs.iter().any(|p|p.pid==browser&&p.mem==96));
        game.world.machines[0].procs.remove_application("w1");
        assert_eq!(game.status().mem_used,baseline+48);
        game.world.machines[0].procs.remove_application("w2");
        assert_eq!(game.status().mem_used,baseline);
    }

    #[test]
    fn killed_app_waits_for_its_window_and_saved_games_have_no_phantom_ram(){
        let mut game=Game::new(World::new(7,"memory_test",""));
        let baseline=game.status().mem_used;
        let pid=game.world.machines[0].procs.spawn_application("w1","editor","memory_test");
        let save=game.save_json();
        assert!(!save.contains("\"type\":\"application\""));
        let restored=Game::from_json(&save).unwrap();
        assert_eq!(restored.status().mem_used,baseline);
        assert!(restored.world.machines[0].procs.get(pid).is_none());
        assert!(game.world.machines[0].procs.kill_as(pid,"another_player").is_err());
        game.kill(pid).unwrap();
        assert_eq!(game.world.machines[0].procs.stopped_applications(),vec!["w1"]);
        assert_eq!(game.status().mem_used,baseline+48);
        game.world.machines[0].procs.remove_application("w1");
        assert_eq!(game.status().mem_used,baseline);
    }
}
