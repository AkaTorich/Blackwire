//! Shared window ordering for Windows, Linux and macOS.
//! Native top-level windows stay eligible for taskbar/Dock and Alt+Tab.
use tauri::{AppHandle, Manager};
use std::sync::atomic::Ordering;
use crate::state::AppState;

#[derive(Clone,Debug)]
struct Layer {
    label:String,
    minimized:bool,
    focused:bool,
}

fn plan(mut layers:Vec<Layer>)->(bool,Vec<String>){
    layers.retain(|w|!w.minimized);
    let active=layers.iter().any(|w|w.focused);
    // Backgrounds first, ordinary windows above them, focused ordinary window last.
    layers.sort_by_key(|w|(!w.label.starts_with("desktop"),w.focused,w.label.clone()));
    (active,layers.into_iter().map(|w|w.label).collect())
}

pub fn refresh(app:&AppHandle){
    let labels:std::collections::BTreeSet<_>=app.state::<AppState>().windows.lock().map(|list|list.iter().map(|w|w.label.clone()).collect()).unwrap_or_default();
    let layers=app.webview_windows().into_iter().filter(|(label,_)|labels.contains(label)).map(|(label,w)|Layer{
        label,minimized:w.is_minimized().unwrap_or(true),focused:w.is_focused().unwrap_or(false)
    }).collect();
    let (active,labels)=plan(layers);
    for label in labels {
        let Some(w)=app.get_webview_window(&label)else{continue};
        // Never restore or re-order an iconic window during a focus transition.
        if w.is_minimized().unwrap_or(true){continue;}
        set_level(&w,active);
    }
}

// Change Windows z-order without activating or restoring a maximized window.
#[cfg(windows)]
fn set_level(w:&tauri::WebviewWindow,active:bool){
    use ::windows::Win32::UI::WindowsAndMessaging::{SetWindowPos,HWND_TOPMOST,HWND_BOTTOM,SWP_NOACTIVATE,SWP_NOMOVE,SWP_NOSIZE};
    if let Ok(hwnd)=w.hwnd(){unsafe{let _=SetWindowPos(hwnd,Some(if active{HWND_TOPMOST}else{HWND_BOTTOM}),0,0,0,0,SWP_NOACTIVATE|SWP_NOMOVE|SWP_NOSIZE);}}
}
#[cfg(not(windows))]
fn set_level(w:&tauri::WebviewWindow,active:bool){
    if active {
        let _=w.set_always_on_bottom(false);
        let _=w.set_always_on_top(false);
        let _=w.set_always_on_top(true);
    }else{
        let _=w.set_always_on_top(false);
        let _=w.set_always_on_bottom(true);
    }
}
#[cfg(windows)]
pub fn native_topmost(w:&tauri::WebviewWindow)->bool{
    use ::windows::Win32::UI::WindowsAndMessaging::{GetWindowLongW,GWL_EXSTYLE,WS_EX_TOPMOST};
    w.hwnd().is_ok_and(|hwnd|unsafe{GetWindowLongW(hwnd,GWL_EXSTYLE) as u32 & WS_EX_TOPMOST.0!=0})
}
#[cfg(not(windows))]
pub fn native_topmost(w:&tauri::WebviewWindow)->bool{w.is_always_on_top().unwrap_or(false)}

pub fn focus_changed(app:&AppHandle){
    let revision=app.state::<AppState>().focus_revision.fetch_add(1,Ordering::Relaxed).wrapping_add(1);
    let app=app.clone();
    std::thread::spawn(move||{
        // Focus passes briefly through no window when switching inside the game.
        std::thread::sleep(std::time::Duration::from_millis(80));
        let handle=app.clone();
        let _=app.run_on_main_thread(move||{
            if handle.state::<AppState>().focus_revision.load(Ordering::Relaxed)==revision{
                refresh(&handle);
            }
        });
    });
}

#[cfg(test)]
mod tests{
    use super::*;
    fn layer(label:&str,minimized:bool,focused:bool)->Layer{Layer{label:label.into(),minimized,focused}}
    #[test]
    fn minimized_windows_are_excluded_even_during_stale_focus_notifications(){
        let (active,labels)=plan(vec![layer("desktop0",true,true),layer("app1",true,true),layer("app2",false,false)]);
        assert!(!active);assert_eq!(labels,vec!["app2"]);
    }
    #[test]
    fn active_order_keeps_all_backgrounds_under_apps_and_focused_app_last(){
        let (active,labels)=plan(vec![layer("app1",false,true),layer("desktop1",false,false),layer("app2",false,false),layer("desktop0",false,false)]);
        assert!(active);assert_eq!(labels,vec!["desktop0","desktop1","app2","app1"]);
    }
    #[test]
    fn focusing_background_never_puts_it_above_game_apps(){
        let (active,labels)=plan(vec![layer("app1",false,false),layer("desktop0",false,true)]);
        assert!(active);assert_eq!(labels,vec!["desktop0","app1"]);
    }
}
