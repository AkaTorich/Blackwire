//! Подбор пароля по словарю через ssh. Перебирает слова по порядку, сверяя с
//! паролем пользователя на машине. Шумно: цель записывает попытки, у
//! наблюдаемой растёт подозрение. Найденный логин кладётся в `found_creds`.

use serde::{Deserialize, Serialize};

use crate::net::machine::{FoundCred, LogKind, MachineId};
use crate::session::Session;
use crate::world::World;

use super::tracks;

/// Отчёт о подборе.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CrackReport {
    pub target: MachineId,
    pub target_host: String,
    pub user: String,
    pub tried: usize,
    pub found: Option<String>,
    /// Слова по порядку до результата включительно — для анимации прогресса.
    pub attempts: Vec<String>,
    pub source_ip: String,
    pub monitored: bool,
    pub heat_added: f32,
}

/// Подобрать пароль пользователя `user` на машине `target` по словарю `wordlist`.
pub fn crack(world: &mut World, session: &Session, target: MachineId, user: &str, wordlist: &[String]) -> Result<CrackReport, String> {
    let m = world.machine(target);
    if !m.up {
        return Err("узел не отвечает".to_string());
    }
    if m.service_named("ssh").is_none() {
        return Err(format!("{}: порт 22 закрыт", m.hostname));
    }
    if super::defense::firewall_up(world, target) {
        return Err("узел закрыт файрволом — сначала обойди его: firewall <адрес>".to_string());
    }
    let host = m.hostname.clone();
    // пароль пользователя (если пользователя нет — не найдём ничего, но шум оставим)
    let real = m.users.get(user).map(|u| u.password.clone());
    let source_ip = tracks::source_ip(world, session);
    let monitored = tracks::is_monitored(world, target);

    let mut attempts = Vec::new();
    let mut found = None;
    for w in wordlist {
        attempts.push(w.clone());
        if real.as_deref() == Some(w.as_str()) {
            found = Some(w.clone());
            break;
        }
    }
    let tried = attempts.len();

    if let Some(pw) = &found {
        world.add_found_cred(FoundCred {
            machine: target,
            user: user.to_string(),
            password: pw.clone(),
            note: format!("подбор ({})", host),
        });
        world.log(target, LogKind::Login, &format!("подбор пароля: успех для {} с {}", user, source_ip));
    } else {
        world.log(target, LogKind::LoginFailed, &format!("подбор пароля: {} неудачных попыток для {} с {}", tried, user, source_ip));
    }

    // шум подбора поднимает подозрение заметнее обычного действия
    let heat_added = if monitored {
        let base = tracks::attack_heat(world, session, target);
        let extra = (tried as f32 / 80.0).min(8.0);
        tracks::add_heat(world, extra);
        base + extra
    } else {
        0.0
    };

    Ok(CrackReport { target, target_host: host, user: user.to_string(), tried, found, attempts, source_ip, monitored, heat_added })
}
