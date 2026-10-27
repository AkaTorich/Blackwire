//! Native-window regression checks in an explicitly separate temporary profile.
use std::{collections::BTreeMap,sync::{Arc,Mutex},time::{Duration,Instant}};
use serde_json::{json,Value};
use tauri::{AppHandle,Listener,Manager,WebviewUrl,WebviewWindowBuilder};
use crate::{state::AppState,windows,window_layers};

fn wait(mut condition:impl FnMut()->bool,message:&str)->Result<(),String>{
    let start=Instant::now();
    while start.elapsed()<Duration::from_secs(8){if condition(){return Ok(())}std::thread::sleep(Duration::from_millis(100));}
    Err(message.into())
}
fn window(app:&AppHandle,label:&str)->Result<tauri::WebviewWindow,String>{app.get_webview_window(label).ok_or_else(||format!("нет окна {label}"))}
fn minimized(app:&AppHandle,label:&str)->bool{window(app,label).and_then(|w|w.is_minimized().map_err(|e|e.to_string())).unwrap_or(false)}
fn click(app:&AppHandle,label:&str,button:&str)->Result<(),String>{
    let script=format!("(function(){{var b=document.querySelector('.win-btns button.{}');if(b)b.click();}})()",button);
    window(app,label)?.eval(&script).map_err(|e|e.to_string())
}
#[cfg(windows)]
fn task_switcher_eligible(app:&AppHandle,label:&str)->Result<(),String>{
    use ::windows::Win32::UI::WindowsAndMessaging::{GetWindow,GetWindowLongW,GW_OWNER,GWL_EXSTYLE,WS_EX_TOOLWINDOW};
    let hwnd=window(app,label)?.hwnd().map_err(|e|e.to_string())?;
    unsafe{
        if !GetWindow(hwnd,GW_OWNER).unwrap_or_default().0.is_null(){return Err(format!("{label}: dependent/owned window"));}
        if GetWindowLongW(hwnd,GWL_EXSTYLE) as u32 & WS_EX_TOOLWINDOW.0!=0{return Err(format!("{label}: tool window excluded from Alt+Tab"));}
    }
    Ok(())
}
#[cfg(not(windows))]
fn task_switcher_eligible(_app:&AppHandle,_label:&str)->Result<(),String>{Ok(())}

pub fn start(app:&AppHandle)->Result<(),Box<dyn std::error::Error>>{
    let args:Vec<_>=std::env::args().collect();
    if !args.iter().any(|a|a=="--data-dir"){return Err("window QA requires an isolated --data-dir".into());}
    let st=app.state::<AppState>();if st.save_path.exists(){return Err("window QA refuses an existing save".into());}
    st.set_game(blackwire_core::Game::new(blackwire_core::World::new(7,"window_qa","")));
    let captured:Arc<Mutex<BTreeMap<String,usize>>>=Arc::new(Mutex::new(BTreeMap::new()));
    let events=captured.clone();app.listen("blackwire-window-controls-qa",move|event|{
        if let Ok(v)=serde_json::from_str::<Value>(event.payload()){
            if let (Some(label),Some(count))=(v["label"].as_str(),v["count"].as_u64()){if let Ok(mut map)=events.lock(){map.insert(label.into(),count as usize);}}
        }
    });
    windows::open_desktop(app,&st,0)?;
    let editor_path="/home/window_qa/scripts/window-check.adr";
    let editor_initial="# Подсветка Adder\ndef example(value):\n    return net.research(\"узел\", 42) if true else null\n";
    {let mut game=st.game()?;let fs=&mut game.world.machines[0].fs;fs.mkdir_all("/home/window_qa/scripts","window_qa")?;fs.write_text(editor_path,editor_initial,"window_qa")?;}
    let editor_expected=format!("{}\n# Сохранённая правка\n",editor_initial);
    let first=windows::create_app_window(app,&st,"terminal",None,Some(0))?;
    let second=windows::create_app_window(app,&st,"editor",Some(editor_path),Some(0))?;
    windows::open_login(app)?;
    let labels=vec!["desktop0".to_string(),first.clone(),second.clone(),"login".to_string()];
    // A normal independent window excluded from the game registry simulates another app.
    WebviewWindowBuilder::new(app,"qa-other-app",WebviewUrl::App("index.html?app=qa-neutral".into()))
        .title("Blackwire window test — other app").inner_size(300.0,200.0).focused(false).build()?;
    let report=st.save_path.with_file_name("native-window-report.json");let app=app.clone();
    std::thread::spawn(move||{
        let result=(||->Result<(),String>{
            std::thread::sleep(Duration::from_secs(2));
            for label in &labels{
                task_switcher_eligible(&app,label)?;
                let label_json=serde_json::to_string(label).unwrap();
                let script=format!("(function poll(n){{var b=document.querySelectorAll('.win-btns button');if(b.length===3||n===0){{window.__TAURI__.event.emit('blackwire-window-controls-qa',{{label:{label_json},count:b.length}});}}else{{setTimeout(function(){{poll(n-1);}},100);}}}})(50)");
                window(&app,label)?.eval(&script).map_err(|e|e.to_string())?;
            }
            wait(||captured.lock().is_ok_and(|m|labels.iter().all(|l|m.get(l)==Some(&3))),"not all windows expose three in-game buttons")?;
            let editor_script=r#"(function verify(n){
              var ta=document.querySelector('.ed-ta');
              if(!ta||ta.value!==$INITIAL){if(n>0)setTimeout(function(){verify(n-1);},100);return;}
              var source=$SOURCE;
              ta.value=source;ta.setSelectionRange(4,9);ta.dispatchEvent(new Event('input',{bubbles:true}));
              var pre=document.querySelector('.ed-pre'), kw=pre.querySelector('.tk-kw'), str=pre.querySelector('.tk-str'), num=pre.querySelector('.tk-num');
              var colors=kw&&str&&num&&new Set([getComputedStyle(kw).color,getComputedStyle(str).color,getComputedStyle(num).color]).size===3;
              var good=colors&&ta.value===source&&ta.selectionStart===4&&ta.selectionEnd===9&&pre.textContent.startsWith(source);
              window.__TAURI__.event.emit('blackwire-window-controls-qa',{label:'editor-syntax',count:good?1:0});
              document.querySelector('.ed-tools button[title^="Сохранить"]').click();
            })(30)"#.replace("$INITIAL",&serde_json::to_string(editor_initial).unwrap()).replace("$SOURCE",&serde_json::to_string(&editor_expected).unwrap());
            window(&app,&second)?.eval(&editor_script).map_err(|e|e.to_string())?;
            wait(||captured.lock().is_ok_and(|m|m.get("editor-syntax")==Some(&1))&&app.state::<AppState>().game().is_ok_and(|g|g.world.machines[0].fs.read_text(editor_path).is_ok_and(|text|text==editor_expected)),"editor syntax, selection or save round-trip failed")?;
            window(&app,&first)?.set_focus().map_err(|e|e.to_string())?;
            wait(||labels.iter().all(|l|window(&app,l).is_ok_and(|w|window_layers::native_topmost(&w))),"background and app windows did not rise together")?;
            // Exercise actual UI buttons and native minimized/maximized states.
            click(&app,&first,"max")?;
            wait(||window(&app,&first).is_ok_and(|w|w.is_maximized().unwrap_or(false)),"maximize button failed")?;
            window(&app,"qa-other-app")?.set_focus().map_err(|e|e.to_string())?;
            wait(||window(&app,"qa-other-app").is_ok_and(|w|w.is_focused().unwrap_or(false))&&labels.iter().all(|l|window(&app,l).is_ok_and(|w|!window_layers::native_topmost(&w))),"game stole focus or stayed above another app")?;
            window(&app,&first)?.set_focus().map_err(|e|e.to_string())?;
            wait(||window(&app,&first).is_ok_and(|w|w.is_focused().unwrap_or(false)),"failed to focus the game again")?;
            for label in &labels{
                click(&app,label,"min")?;
                wait(||minimized(&app,label),&format!("{label}: minimize button failed"))?;
            }
            std::thread::sleep(Duration::from_millis(500));
            if !labels.iter().all(|l|minimized(&app,l)){return Err("focus handling restored a minimized window".into());}
            let restored=window(&app,&second)?;restored.unminimize().map_err(|e|e.to_string())?;restored.set_focus().map_err(|e|e.to_string())?;
            wait(||!minimized(&app,&second),"restore failed")?;
            std::thread::sleep(Duration::from_millis(500));
            if labels.iter().filter(|l|**l!=second).any(|l|!minimized(&app,l)){return Err("restoring one window restored other windows".into());}
            // Closing one application must not destroy its siblings or the background.
            click(&app,&second,"close")?;
            wait(||app.get_webview_window(&second).is_none(),"close button failed")?;
            if labels.iter().filter(|l|**l!=second).any(|l|app.get_webview_window(l).is_none()){return Err("closing an app destroyed another window".into());}
            Ok(())
        })();
        let value=json!({"ok":result.is_ok(),"platform":std::env::consts::OS,"checks":["four windows with custom controls","editor syntax, selection and saving","independent native windows","maximize","focus and topmost group","minimize all","restore one without restoring others","close one without closing others"],"windows":app.webview_windows().into_iter().map(|(label,w)|json!({"label":label,"minimized":w.is_minimized().ok(),"focused":w.is_focused().ok(),"topmost":window_layers::native_topmost(&w)})).collect::<Vec<_>>(),"focus_event":app.state::<AppState>().focused_window.lock().ok().and_then(|f|f.clone()),"error":result.as_ref().err()});
        let _=std::fs::write(report,serde_json::to_vec_pretty(&value).unwrap());app.exit(if result.is_ok(){0}else{1});
    });
    Ok(())
}
