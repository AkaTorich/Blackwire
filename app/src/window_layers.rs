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
    let focused=app.state::<AppState>().focused_window.lock().ok().and_then(|f|f.clone());
    let layers=app.webview_windows().into_iter().filter(|(label,_)|labels.contains(label)).map(|(label,w)|Layer{
        focused:window_focused(&w,&label,focused.as_deref()),minimized:w.is_minimized().unwrap_or(true),label
    }).collect();
    let (active,labels)=plan(layers);
    if std::env::args().any(|a|a=="--window-system-test"){eprintln!("window layers: active={active} labels={labels:?}");}
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
// GTK focus-in/out can describe focus moving into WebKit children rather than
// top-level activation. Observe the native active property instead.
#[cfg(target_os="linux")]
pub fn observe_focus(w:&tauri::WebviewWindow){
    use gtk::prelude::*;
    if let Ok(native)=w.gtk_window(){
        let app=w.app_handle().clone();let label=w.label().to_string();
        native.connect_is_active_notify(move|window|focus_changed(&app,&label,window.is_active()));
    }
}
#[cfg(not(target_os="linux"))]
pub fn observe_focus(_w:&tauri::WebviewWindow){}
#[cfg(target_os="linux")]
fn window_focused(w:&tauri::WebviewWindow,_label:&str,_focused:Option<&str>)->bool{w.is_focused().unwrap_or(false)}
#[cfg(not(target_os="linux"))]
fn window_focused(_w:&tauri::WebviewWindow,label:&str,focused:Option<&str>)->bool{focused==Some(label)}
#[cfg(target_os="linux")]
fn set_level(w:&tauri::WebviewWindow,active:bool){
    use gtk::prelude::*;
    if let Ok(native)=w.gtk_window(){
        native.set_keep_below(!active);
        native.set_keep_above(active);
        // Raise without present()/activation, in the planner's background-first order.
        if active{if let Some(surface)=native.window(){surface.raise();}}
        if std::env::args().any(|a|a=="--window-system-test"){eprintln!("GTK level: {} active={active} native_active={} backend={} state={:?}",w.label(),native.is_active(),native.display().type_().name(),native.window().map(|surface|surface.state()));}
    }
}
#[cfg(not(any(windows,target_os="linux")))]
fn set_level(w:&tauri::WebviewWindow,active:bool){
    if active {
        let _=w.set_always_on_bottom(false);
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

pub fn focus_changed(app:&AppHandle,label:&str,focused:bool){
    let state=app.state::<AppState>();
    if let Ok(mut current)=state.focused_window.lock(){
        if focused {*current=Some(label.into());}
        else if current.as_deref()==Some(label){*current=None;}
    }
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
