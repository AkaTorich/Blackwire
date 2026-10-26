//! Автоматическая проверка UI разрешена только при явно указанном отдельном профиле.
use tauri::{State,AppHandle};
use crate::state::AppState;
fn allowed()->Result<(),String>{let a:Vec<_>=std::env::args().collect();if a.iter().any(|a|a=="--sdk-ui-test")&&a.iter().any(|a|a=="--data-dir"){Ok(())}else{Err("тестовый режим не включён".into())}}
#[tauri::command]
pub fn sdk_ui_report(st:State<'_,AppState>,stage:String,result:serde_json::Value)->Result<(),String>{
 allowed()?;if !["manager","panel"].contains(&stage.as_str())||result.to_string().len()>65536{return Err("неверный отчёт".into());}
 let path=st.save_path.with_file_name("sdk-ui-test-report.json");let mut report=std::fs::read_to_string(&path).ok().and_then(|s|serde_json::from_str::<serde_json::Value>(&s).ok()).unwrap_or_else(||serde_json::json!({}));report[stage]=result;std::fs::write(path,serde_json::to_string_pretty(&report).unwrap()).map_err(|e|e.to_string())?;st.save()
}
#[tauri::command]
pub fn sdk_ui_status(st:State<'_,AppState>)->Result<serde_json::Value,String>{allowed()?;Ok(std::fs::read_to_string(st.save_path.with_file_name("sdk-ui-test-report.json")).ok().and_then(|s|serde_json::from_str(&s).ok()).unwrap_or_else(||serde_json::json!({})))}
pub fn inject(app:&AppHandle,label:String){
 use tauri::Manager;
 let app=app.clone();std::thread::spawn(move||{std::thread::sleep(std::time::Duration::from_millis(1500));if let Some(win)=app.get_webview_window(&label){let _=win.eval(include_str!("../../tools/sdk-ui-test.js"));}});
}
