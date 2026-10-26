//! Окна игры: каждое приложение — отдельное окно Windows без системной рамки.
//! Подложка рабочего стола — окно во весь монитор, всегда позади остальных.

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize, State, WebviewUrl, WebviewWindowBuilder};

use crate::state::{AppState, WinInfo};

/// Начальные размеры окон по приложению (логические пиксели): ширина, высота, мин. ширина, мин. высота.
fn default_size(app: &str) -> (f64, f64, f64, f64) {
    match app {
        "terminal" => (900.0, 560.0, 520.0, 320.0),
        "editor" => (1000.0, 680.0, 640.0, 400.0),
        "files" => (960.0, 620.0, 640.0, 400.0),
        "browser" => (1100.0, 760.0, 640.0, 420.0),
        "monitor" => (960.0, 640.0, 640.0, 420.0),
        "map" => (1000.0, 700.0, 600.0, 420.0),
        "mail" => (1060.0, 680.0, 720.0, 420.0),
        "settings" => (860.0, 640.0, 620.0, 460.0),
        "help" => (980.0, 700.0, 620.0, 420.0),
        "finale" => (760.0, 780.0, 560.0, 560.0),
        "scenario" => (980.0, 720.0, 640.0, 460.0),
        "duel" => (900.0, 700.0, 620.0, 480.0),
        "login" => (520.0, 640.0, 520.0, 640.0),
        _ => (900.0, 600.0, 520.0, 320.0),
    }
}

fn app_title(app: &str) -> &'static str {
    match app {
        "terminal" => "Терминал",
        "editor" => "Редактор",
        "files" => "Файлы",
        "browser" => "Браузер",
        "monitor" => "Монитор",
        "map" => "Карта сети",
        "mail" => "Почта",
        "plugin_panel" => "Приложение плагина",
        "plugins" => "Плагины",
        "settings" => "Настройки",
        "help" => "Справка",
        "finale" => "hello-world",
        "scenario" => "Редактор сценариев",
        "duel" => "Сетевая игра",
        "login" => "Blackwire",
        "desktop" => "Blackwire — рабочий стол",
        _ => "Blackwire",
    }
}

fn encode(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => out.push(b as char),
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}

#[derive(Serialize, Clone)]
pub struct MonitorInfo {
    pub index: u32,
    pub name: String,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub scale: f64,
    pub primary: bool,
}

fn monitor_list(app: &AppHandle) -> Vec<MonitorInfo> {
    let primary = app.primary_monitor().ok().flatten();
    let mut out = Vec::new();
    if let Ok(list) = app.available_monitors() {
        for (i, m) in list.iter().enumerate() {
            let is_primary = primary
                .as_ref()
                .map(|p| p.position() == m.position() && p.size() == m.size())
                .unwrap_or(i == 0);
            out.push(MonitorInfo {
                index: i as u32,
                name: m.name().cloned().unwrap_or_else(|| format!("Монитор {}", i + 1)),
                x: m.position().x,
                y: m.position().y,
                width: m.size().width,
                height: m.size().height,
                scale: m.scale_factor(),
                primary: is_primary,
            });
        }
    }
    out
}

fn monitor_or_default(app: &AppHandle, st: &AppState, monitor: Option<u32>) -> Option<MonitorInfo> {
    let list = monitor_list(app);
    let want = monitor.or_else(|| st.game().ok().map(|g| g.world.settings.default_monitor as u32)).unwrap_or(0);
    list.get(want as usize).cloned().or_else(|| list.into_iter().find(|m| m.primary))
}

/// Создать окно приложения. Возвращает метку окна.
pub fn create_app_window(app: &AppHandle, st: &AppState, app_id: &str, arg: Option<&str>, monitor: Option<u32>) -> Result<String, String> {
    let label = st.next_label();
    let mut url = format!("index.html?app={}&win={}", encode(app_id), label);
    if let Some(a) = arg {
        url.push_str("&arg=");
        url.push_str(&encode(a));
    }
    let (mut w, mut h, min_w, min_h) = default_size(app_id);
    if let Some((fw, fh)) = st.forced_size.lock().ok().and_then(|f| *f) {
        w = fw.max(min_w);
        h = fh.max(min_h);
    }
    let mon = monitor_or_default(app, st, monitor);

    let mut builder = WebviewWindowBuilder::new(app, &label, WebviewUrl::App(url.into()))
        .title(app_title(app_id))
        .decorations(false)
        .transparent(true)
        .shadow(true)
        .resizable(true)
        .inner_size(w, h)
        .min_inner_size(min_w, min_h);

    if let Some(m) = &mon {
        // каскад: каждое новое окно чуть ниже и правее, в пределах монитора
        let n = (st.next_win.load(std::sync::atomic::Ordering::Relaxed) % 8) as f64;
        let mw = m.width as f64 / m.scale;
        let mh = m.height as f64 / m.scale;
        let x = m.x as f64 / m.scale + ((mw - w) / 2.0 + n * 28.0 - 100.0).max(0.0);
        let y = m.y as f64 / m.scale + ((mh - h) / 2.0 + n * 24.0 - 80.0).max(0.0);
        builder = builder.position(x, y);
    } else {
        builder = builder.center();
    }

    // окна-приложения — дочерние к рабочему столу этого монитора: они всегда над
    // фоном, а щелчок по кнопке «Blackwire» выводит вперёд весь набор вместе.
    if let Some(m) = &mon {
        if let Some(parent) = app.get_webview_window(&format!("desktop{}", m.index)) {
            builder = builder.parent(&parent).map_err(|e| format!("родитель окна: {e}"))?;
        }
    }

    let win = builder.build().map_err(|e| format!("не удалось открыть окно: {e}"))?;
    let _ = win.set_focus();

    if let Ok(mut list) = st.windows.lock() {
        list.push(WinInfo { label: label.clone(), app: app_id.to_string(), arg: arg.map(|s| s.to_string()) });
    }
    emit_windows(app, st);
    Ok(label)
}

/// Окно входа — первое окно игры.
pub fn open_login(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let st = app.state::<AppState>();
    let (w, h, _, _) = default_size("login");
    let label = "login";
    WebviewWindowBuilder::new(app, label, WebviewUrl::App("index.html?app=login&win=login".into()))
        .title("Blackwire")
        .decorations(false)
        .transparent(true)
        .shadow(true)
        .resizable(false)
        .inner_size(w, h)
        .center()
        .build()?;
    if let Ok(mut list) = st.windows.lock() {
        list.push(WinInfo { label: label.into(), app: "login".into(), arg: None });
    }
    Ok(())
}

/// Подложка рабочего стола на мониторе: во весь экран, без рамки, всегда позади.
pub fn open_desktop(app: &AppHandle, st: &AppState, monitor: u32) -> Result<(), String> {
    if st.desktops.lock().map(|d| d.contains_key(&monitor)).unwrap_or(false) {
        return Ok(());
    }
    let list = monitor_list(app);
    let m = list.get(monitor as usize).cloned().ok_or_else(|| "нет такого монитора".to_string())?;
    let label = format!("desktop{}", monitor);
    let url = format!("index.html?app=desktop&win={}&arg={}", label, monitor);
    // Первый рабочий стол — главное окно игры: у него есть кнопка в панели задач,
    // чтобы игру всегда можно было вернуть, даже закрыв все окна-приложения.
    // Подложки на других мониторах кнопку не получают.
    let is_primary = st.desktops.lock().map(|d| d.is_empty()).unwrap_or(true);
    let win = WebviewWindowBuilder::new(app, &label, WebviewUrl::App(url.into()))
        .title(if is_primary { "Blackwire" } else { app_title("desktop") })
        .decorations(false)
        .transparent(true)
        .shadow(false)
        .resizable(false)
        .skip_taskbar(!is_primary)
        .focused(false)
        .build()
        .map_err(|e| format!("не удалось открыть рабочий стол: {e}"))?;
    let _ = win.set_position(PhysicalPosition::new(m.x, m.y));
    let _ = win.set_size(PhysicalSize::new(m.width, m.height));
    if let Ok(mut d) = st.desktops.lock() {
        d.insert(monitor, label.clone());
    }
    if let Ok(mut list) = st.windows.lock() {
        list.push(WinInfo { label, app: "desktop".into(), arg: Some(monitor.to_string()) });
    }
    Ok(())
}

pub fn on_window_destroyed(app: &AppHandle, label: &str) {
    let st = app.state::<AppState>();
    if let Ok(mut list) = st.windows.lock() {
        list.retain(|w| w.label != label);
    }
    if let Ok(mut d) = st.desktops.lock() {
        d.retain(|_, l| l != label);
    }
    emit_windows(app, &st);
}

#[derive(Serialize, Clone)]
pub struct WinState {
    pub label: String,
    pub app: String,
    pub title: String,
    pub minimized: bool,
    pub focused: bool,
}

fn window_states(app: &AppHandle, st: &AppState) -> Vec<WinState> {
    let list = st.windows.lock().map(|l| l.clone()).unwrap_or_default();
    list.into_iter()
        .filter(|w| w.app != "desktop" && w.app != "login")
        .map(|w| {
            let win = app.get_webview_window(&w.label);
            WinState {
                title: win.as_ref().and_then(|x| x.title().ok()).unwrap_or_else(|| app_title(&w.app).to_string()),
                minimized: win.as_ref().and_then(|x| x.is_minimized().ok()).unwrap_or(false),
                focused: win.as_ref().and_then(|x| x.is_focused().ok()).unwrap_or(false),
                label: w.label,
                app: w.app,
            }
        })
        .collect()
}

pub fn emit_windows(app: &AppHandle, st: &AppState) {
    let states = window_states(app, st);
    let _ = app.emit("windows_changed", serde_json::json!({ "windows": states }));
}

// ---------- команды ----------

#[tauri::command]
pub async fn open_app(handle: AppHandle, st: State<'_, AppState>, app: String, arg: Option<String>, monitor: Option<u32>) -> Result<String, String> {
    create_app_window(&handle, &st, &app, arg.as_deref(), monitor)
}

#[tauri::command]
pub async fn focus_window(app: AppHandle, label: String) -> Result<(), String> {
    let win = app.get_webview_window(&label).ok_or_else(|| "окно уже закрыто".to_string())?;
    if win.is_minimized().unwrap_or(false) {
        let _ = win.unminimize();
    }
    win.set_focus().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn close_window(app: AppHandle, label: String) -> Result<(), String> {
    if let Some(win) = app.get_webview_window(&label) {
        win.close().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn list_windows(app: AppHandle, st: State<'_, AppState>) -> Vec<WinState> {
    window_states(&app, &st)
}

#[tauri::command]
pub fn monitors(app: AppHandle) -> Vec<MonitorInfo> {
    monitor_list(&app)
}

/// Разложить открытые окна приложений на мониторе: сетка, колонки, ряды или каскад.
#[tauri::command]
pub async fn arrange_windows(app: AppHandle, st: State<'_, AppState>, monitor: Option<u32>, layout: Option<String>) -> Result<(), String> {
    arrange_all(&app, &st, monitor, layout.as_deref().unwrap_or("grid"))
}

pub fn arrange_all(app: &AppHandle, st: &AppState, monitor: Option<u32>, layout: &str) -> Result<(), String> {
    let m = monitor_or_default(app, st, monitor).ok_or_else(|| "монитор не найден".to_string())?;
    let labels: Vec<String> = st
        .windows
        .lock()
        .map(|l| l.iter().filter(|w| w.app != "desktop" && w.app != "login").map(|w| w.label.clone()).collect())
        .unwrap_or_default();
    let n = labels.len();
    if n == 0 {
        return Ok(());
    }
    // рабочая область монитора: сверху полоса состояния стола, снизу док и панель задач Windows
    let top = (44.0 * m.scale) as u32;
    let bottom = (140.0 * m.scale) as u32;
    let (ax, ay, aw, ah) = (m.x, m.y + top as i32, m.width, m.height.saturating_sub(top + bottom));
    // проверочный режим `--size`: окна не растягивать, а расставить рядами как есть
    if let Some((fw, fh)) = st.forced_size.lock().ok().and_then(|f| *f) {
        let (pw, ph) = ((fw * m.scale) as i32, (fh * m.scale) as i32);
        let per_row = ((aw as i32) / pw.max(1)).max(1) as usize;
        for (i, label) in labels.iter().enumerate() {
            let Some(win) = app.get_webview_window(label) else { continue };
            let c = (i % per_row) as i32;
            let r = (i / per_row) as i32;
            let _ = win.set_position(PhysicalPosition::new(ax + c * pw, ay + r * ph));
        }
        return Ok(());
    }
    let (cols, rows) = match layout {
        "columns" => (n, 1),
        "rows" => (1, n),
        "cascade" => (1, 1),
        _ => {
            let cols = (n as f64).sqrt().ceil() as usize;
            let rows = (n + cols - 1) / cols;
            (cols, rows)
        }
    };
    for (i, label) in labels.iter().enumerate() {
        let Some(win) = app.get_webview_window(label) else { continue };
        let _ = win.unminimize();
        let _ = win.unmaximize();
        let (x, y, w, h) = if layout == "cascade" {
            let step = (36.0 * m.scale) as i32;
            let w = (aw as f64 * 0.62) as u32;
            let h = (ah as f64 * 0.62) as u32;
            (ax + step * i as i32, ay + step * i as i32, w, h)
        } else {
            let cw = aw / cols as u32;
            let ch = ah / rows as u32;
            let c = (i % cols) as i32;
            let r = (i / cols) as i32;
            (ax + c * cw as i32, ay + r * ch as i32, cw, ch)
        };
        let _ = win.set_position(PhysicalPosition::new(x, y));
        let _ = win.set_size(PhysicalSize::new(w.max(200), h.max(150)));
        let _ = win.set_focus();
    }
    Ok(())
}

/// Включить или выключить подложку рабочего стола на мониторе.
#[tauri::command]
pub async fn desktop_on(app: AppHandle, st: State<'_, AppState>, monitor: u32, on: bool) -> Result<(), String> {
    if on {
        open_desktop(&app, &st, monitor)
    } else {
        let label = st.desktops.lock().ok().and_then(|d| d.get(&monitor).cloned());
        if let Some(label) = label {
            if let Some(win) = app.get_webview_window(&label) {
                let _ = win.close();
            }
        }
        Ok(())
    }
}

/// Перенести окно на монитор (сохранив размер) — для меню рабочего стола.
#[allow(dead_code)]
pub fn move_to_monitor(app: &AppHandle, label: &str, m: &MonitorInfo) {
    if let Some(win) = app.get_webview_window(label) {
        let size = win.inner_size().unwrap_or(PhysicalSize::new(900, 600));
        let x = m.x + ((m.width as i64 - size.width as i64) / 2).max(0) as i32;
        let y = m.y + ((m.height as i64 - size.height as i64) / 2).max(0) as i32;
        let _ = win.set_position(PhysicalPosition::new(x, y));
    }
}
