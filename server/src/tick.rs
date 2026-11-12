//! Часы общего мира: раз в секунду общий ход (сеть, живой мир), затем личный
//! ход каждого игрока в сети (сюжет, долг, подозрение…) и его события — те же,
//! что рассылают часы одиночной игры. Игроки не в сети стоят на паузе.

use std::sync::Arc;
use std::time::{Duration, Instant};

use serde_json::{json, Value};

use crate::hub::Hub;
use crate::scripts::{self, Pending};

struct Out {
    handle: String,
    events: Vec<(&'static str, Value)>,
    unread: usize,
    pending: Pending,
}

pub fn start(hub: Arc<Hub>) {
    std::thread::Builder::new()
        .name("world-ticker".into())
        .spawn(move || {
            let mut seconds: u64 = 0;
            let mut next = Instant::now() + Duration::from_secs(1);
            loop {
                let now = Instant::now();
                if next > now {
                    std::thread::sleep(next - now);
                }
                next += Duration::from_secs(1);
                seconds += 1;
                tick_once(&hub);
                if seconds % 60 == 0 {
                    hub.save();
                }
            }
        })
        .expect("поток часов");
}

fn tick_once(hub: &Arc<Hub>) {
    let online: Vec<(String, Option<usize>)> = hub
        .online
        .lock()
        .map(|o| o.iter().map(|(h, x)| (h.clone(), x.last_unread)).collect())
        .unwrap_or_default();
    let mut outs: Vec<Out> = Vec::new();
    {
        let Ok(mut st) = hub.state.lock() else { return };
        let secs = st.game.world.mmo_tick_shared(1000);
        drop(st);
        for (handle, last_unread) in online {
            let r = hub.with_player(&handle, |g| {
                for _ in 0..secs {
                    g.mmo_tick_player();
                }
                let mut events: Vec<(&'static str, Value)> = Vec::new();
                if let Some((title, text, _raided)) = g.poll_surveillance() {
                    events.push(("notify", json!({ "title": title, "text": text })));
                }
                let mission = g.poll_missions().or_else(|| g.poll_finance()).or_else(|| g.poll_gigs()).or_else(|| g.poll_defense()).or_else(|| g.poll_base()).or_else(|| g.poll_achievement()).or_else(|| g.poll_scenario());
                if let Some((title, text)) = mission {
                    events.push(("notify", json!({ "title": title, "text": text })));
                }
                if let Some((title, text)) = g.poll_ending() {
                    events.push(("notify", json!({ "title": title, "text": text })));
                    events.push(("open_app", json!({ "app": "finale", "arg": null })));
                }
                let s = g.status();
                let unread = g.mail_unread_count();
                events.insert(
                    0,
                    (
                        "world_tick",
                        json!({
                            "time": g.world.clock.epoch_secs,
                            "time_text": g.world.clock.format_now(),
                            "cpu": s.cpu, "mem": s.mem_used, "mem_total": s.mem_total,
                            "net_in": s.net_in, "net_out": s.net_out, "uptime": s.uptime,
                            "mail_unread": unread,
                        }),
                    ),
                );
                if last_unread.map(|u| unread > u).unwrap_or(false) {
                    events.push(("mail_new", json!({ "count": unread })));
                }
                (events, unread, scripts::take_pending(g))
            });
            if let Ok((events, unread, pending)) = r {
                outs.push(Out { handle, events, unread, pending });
            }
        }
    }
    for o in outs {
        if let Ok(mut on) = hub.online.lock() {
            if let Some(x) = on.get_mut(&o.handle) {
                x.last_unread = Some(o.unread);
            }
        }
        for (ev, data) in o.events {
            hub.send(&o.handle, ev, data);
        }
        scripts::deliver(hub, &o.handle, o.pending);
    }
}
