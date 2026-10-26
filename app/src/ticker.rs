//! Часы мира: раз в секунду двигаем время и фон процессов, рассылаем окнам
//! `world_tick`; раз в минуту — автосохранение; следим за новыми письмами.

use std::time::Duration;

use tauri::{AppHandle, Emitter, Manager};

use crate::state::AppState;

pub fn start(app: AppHandle) {
    std::thread::Builder::new()
        .name("world-ticker".into())
        .spawn(move || {
            let mut seconds: u64 = 0;
            let mut last_unread: Option<usize> = None;
            loop {
                std::thread::sleep(Duration::from_secs(1));
                seconds += 1;
                let st = app.state::<AppState>();
                let payload = {
                    let Ok(mut g) = st.game() else { continue };
                    g.tick(1000);
                    let surveil = g.poll_surveillance();
                    let mission = g.poll_missions().or_else(|| g.poll_finance()).or_else(|| g.poll_gigs()).or_else(|| g.poll_defense()).or_else(|| g.poll_base()).or_else(|| g.poll_achievement()).or_else(|| g.poll_scenario());
                    let ending = g.poll_ending();
                    let s = g.status();
                    let unread = g.mail_unread_count();
                    let mail_new = last_unread.map(|u| unread > u).unwrap_or(false);
                    last_unread = Some(unread);
                    (
                        serde_json::json!({
                            "time": g.world.clock.epoch_secs,
                            "time_text": g.world.clock.format_now(),
                            "cpu": s.cpu, "mem": s.mem_used, "mem_total": s.mem_total,
                            "net_in": s.net_in, "net_out": s.net_out, "uptime": s.uptime,
                            "mail_unread": unread,
                        }),
                        mail_new,
                        unread,
                        surveil,
                        mission,
                        ending,
                    )
                };
                let _ = app.emit("world_tick", payload.0);
                if let Some((title, text, _raided)) = payload.3 {
                    let _ = app.emit("notify", serde_json::json!({ "title": title, "text": text }));
                }
                if let Some((title, text)) = payload.4 {
                    let _ = app.emit("notify", serde_json::json!({ "title": title, "text": text }));
                }
                if let Some((title, text)) = payload.5 {
                    // победа: уведомление и окно финала (один раз). Игра продолжается.
                    let _ = app.emit("notify", serde_json::json!({ "title": title, "text": text }));
                    let _ = crate::windows::create_app_window(&app, &st, "finale", None, None);
                }
                if payload.1 {
                    let _ = app.emit("mail_new", serde_json::json!({ "count": payload.2 }));
                    let _ = app.emit("notify", serde_json::json!({ "title": "Почта", "text": "Новое письмо" }));
                }
                crate::cmd_shell::flush_pending(&app, &st);
                crate::netplay::pump(&app, &st);
                if seconds % 60 == 0 {
                    let _ = st.save();
                }
            }
        })
        .expect("поток часов мира");
}
