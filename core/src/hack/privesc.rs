//! Повышение прав на машине текущего hop'а до root, если есть локальное слабое
//! место (уязвимое ядро) и пользователь ещё не root.

use serde::{Deserialize, Serialize};

use crate::net::machine::LogKind;
use crate::session::Session;
use crate::world::World;

use super::vuln::{local_vulns, VulnInfo};

/// Итог повышения прав.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PrivescReport {
    pub ok: bool,
    pub vuln: Option<VulnInfo>,
    pub host: String,
    pub was_user: String,
}

/// Повысить права до root на машине текущего hop'а.
pub fn privesc(world: &mut World, session: &mut Session) -> Result<PrivescReport, String> {
    let mid = session.machine_id();
    let host = world.machine(mid).hostname.clone();
    let was = session.user().to_string();
    if was == "root" {
        return Ok(PrivescReport { ok: false, vuln: None, host, was_user: was });
    }
    let v = match local_vulns(world.machine(mid)).into_iter().next() {
        Some(v) => Some(v),
        // «Руткит» из магазина — повышение прав удаётся всегда
        None if world.player.upgrades.contains("rootkit") => None,
        None => return Ok(PrivescReport { ok: false, vuln: None, host, was_user: was }),
    };
    apply_vulnerability(world, session, v)
}

pub fn apply_vulnerability(world: &mut World, session: &mut Session, v: Option<VulnInfo>) -> Result<PrivescReport, String> {
    let mid = session.machine_id();
    let host = world.machine(mid).hostname.clone();
    let was = session.user().to_string();
    if was == "root" { return Ok(PrivescReport { ok: false, vuln: None, host, was_user: was }); }
    if let Some(v)=&v { super::access::check(world,session,mid,v)?; }
    session.current_mut().user = "root".to_string();
    if !world.player.owned.contains(&mid) {
        world.player.owned.push(mid);
    }
    world.log(mid, LogKind::Warn, &format!("повышение прав до root (было: {})", was));
    if let Some(vuln) = &v {
        world.plugin_events.push(("on_hack".into(), serde_json::json!({
            "target": mid, "target_host": host, "vuln": vuln,
            "outcome": {"Shell": {"user": "root"}},
            "source_ip": world.machine(mid).ip, "monitored": false, "heat_added": 0.0
        })));
    }
    Ok(PrivescReport { ok: true, vuln: v, host, was_user: was })
}
