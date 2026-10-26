//! Цепочка подключений: через какие машины идёт игрок и какой адрес видит цель.

use serde::{Deserialize, Serialize};

use crate::net::machine::MachineId;
use crate::session::Session;
use crate::world::World;

use super::tracks;

/// Сведения о цепочке.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ProxyInfo {
    /// (машина, пользователь) по hop'ам; [0] — своя машина.
    pub chain: Vec<(MachineId, String)>,
    pub exit_ip: String,
    pub exit_host: String,
    /// Цепочка длиннее своей машины — реальный адрес скрыт.
    pub hidden: bool,
    /// Hop'ы под наблюдением (риск).
    pub monitored_hops: Vec<MachineId>,
}

pub fn proxy_info(world: &World, session: &Session) -> ProxyInfo {
    let chain: Vec<(MachineId, String)> = session.hops.iter().map(|h| (h.machine, h.user.clone())).collect();
    let exit_mid = session.machine_id();
    let exit_ip = world.machine(exit_mid).ip.clone();
    let exit_host = world.machine(exit_mid).hostname.clone();
    let hidden = session.hops.len() > 1;
    let monitored_hops: Vec<MachineId> = session.hops.iter().map(|h| h.machine).filter(|id| tracks::is_monitored(world, *id)).collect();
    ProxyInfo { chain, exit_ip, exit_host, hidden, monitored_hops }
}
