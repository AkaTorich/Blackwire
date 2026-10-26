//! Проверочный запуск без экрана входа — чтобы смотреть окна снимками:
//!
//! ```text
//! Blackwire.exe --new ghost --open terminal:neofetch --open editor:/home/ghost/scripts/scan.adr --size 1000x650
//! Blackwire.exe --load --open mail
//! ```
//! `--new имя [--seed N]` — новая личность без пароля (сохранение перезаписывается);
//! `--load` — продолжить сохранённую игру; `--open приложение[:аргумент]` — открыть окно (можно несколько);
//! `--size ШxВ` — размер всех открываемых окон; `--monitor N` — монитор; `--no-desktop` — без подложки;
//! `--arrange grid|columns|rows|cascade` — разложить открытые окна; `--dump файл` — записать список окон в JSON.
//! Без этих ключей игра стартует как обычно, с экрана входа.

use blackwire_core::world::World;
use blackwire_core::Game;
use tauri::{AppHandle, Manager};

use crate::state::AppState;
use crate::windows;

pub fn start_from_args(app: &AppHandle) -> Result<bool, Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        return Ok(false);
    }
    let st = app.state::<AppState>();
    let mut new_handle: Option<String> = None;
    let mut seed: Option<u64> = None;
    let mut load = false;
    let mut opens: Vec<(String, Option<String>)> = Vec::new();
    let mut monitor: Option<u32> = None;
    let mut desktop = true;
    let mut arrange: Option<String> = None;
    let mut dump: Option<String> = None;
    let mut duel: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        let a = args[i].as_str();
        let next = args.get(i + 1).cloned();
        match a {
            "--new" => {
                new_handle = next;
                i += 1;
            }
            "--seed" => {
                seed = next.and_then(|s| s.parse().ok());
                i += 1;
            }
            "--load" => load = true,
            "--open" => {
                if let Some(spec) = next {
                    let (app_id, arg) = match spec.split_once(':') {
                        Some((a, b)) => (a.to_string(), Some(b.to_string())),
                        None => (spec.clone(), None),
                    };
                    opens.push((app_id, arg));
                }
                i += 1;
            }
            "--size" => {
                if let Some(s) = next {
                    if let Some((w, h)) = s.split_once('x') {
                        if let (Ok(w), Ok(h)) = (w.parse::<f64>(), h.parse::<f64>()) {
                            if let Ok(mut f) = st.forced_size.lock() {
                                *f = Some((w, h));
                            }
                        }
                    }
                }
                i += 1;
            }
            "--monitor" => {
                monitor = next.and_then(|s| s.parse().ok());
                i += 1;
            }
            "--no-desktop" => desktop = false,
            "--arrange" => {
                arrange = next;
                i += 1;
            }
            "--dump" => {
                dump = next;
                i += 1;
            }
            "--duel" => {
                duel = next;
                i += 1;
            }
            _ => {}
        }
        i += 1;
    }

    if new_handle.is_none() && !load {
        return Ok(false);
    }
    if let Some(handle) = new_handle {
        let world = World::new(seed.unwrap_or(7), &handle, "");
        st.set_game(Game::new(world));
        st.save()?;
    } else {
        let text = std::fs::read_to_string(&st.save_path)?;
        st.set_game(Game::new(World::from_json(&text)?));
    }
    let mon = monitor.unwrap_or_else(|| st.game().map(|g| g.world.settings.default_monitor as u32).unwrap_or(0));
    if desktop {
        windows::open_desktop(app, &st, mon)?;
    }
    if opens.is_empty() {
        opens.push(("terminal".into(), None));
    }
    for (app_id, arg) in opens {
        let label=windows::create_app_window(app, &st, &app_id, arg.as_deref(), Some(mon))?;
        if args.iter().any(|a|a=="--sdk-ui-test")&&args.iter().any(|a|a=="--data-dir")&&["plugins","plugin_panel"].contains(&app_id.as_str()){crate::sdk_qa::inject(app,label); }
    }
    // проверочная сетевая игра: `--duel host[:team]` или `--duel join:адрес`
    if let Some(spec) = duel {
        let (kind, rest) = match spec.split_once(':') {
            Some((k, r)) => (k.to_string(), r.to_string()),
            None => (spec.clone(), String::new()),
        };
        match kind.as_str() {
            "host" => {
                let mode = if rest == "team" { "team" } else { "duel" };
                let _ = crate::netplay::do_host(app, &st, mode, 7777);
            }
            "join" => {
                let addr = if rest.is_empty() { "127.0.0.1:7777".to_string() } else { rest };
                let _ = crate::netplay::do_join(app, &st, &addr);
            }
            _ => {}
        }
    }
    if arrange.is_some() || dump.is_some() {
        // окнам нужно мгновение, чтобы появиться, иначе раскладка их не найдёт
        let handle = app.clone();
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(1500));
            let st = handle.state::<AppState>();
            if let Some(layout) = arrange {
                let _ = windows::arrange_all(&handle, &st, Some(mon), &layout);
            }
            if let Some(path) = dump {
                // `--dump файл` — список окон, каким его видит интерфейс (для проверки)
                std::thread::sleep(std::time::Duration::from_millis(1500));
                let states = windows::list_windows(handle.clone(), st.clone());
                let mons = windows::monitors(handle.clone());
                let geometry: Vec<serde_json::Value> = states
                    .iter()
                    .map(|w| {
                        let win = handle.get_webview_window(&w.label);
                        let pos = win.as_ref().and_then(|x| x.outer_position().ok()).map(|p| (p.x, p.y));
                        let size = win.as_ref().and_then(|x| x.inner_size().ok()).map(|s| (s.width, s.height));
                        let scale = win.as_ref().and_then(|x| x.scale_factor().ok());
                        serde_json::json!({ "label": w.label, "pos": pos, "size": size, "scale": scale })
                    })
                    .collect();
                let out = serde_json::json!({ "windows": states, "monitors": mons, "geometry": geometry });
                let _ = std::fs::write(path, serde_json::to_string_pretty(&out).unwrap_or_default());
            }
        });
    }
    Ok(true)
}
