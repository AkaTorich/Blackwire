use tauri::State;
use crate::state::AppState;
#[tauri::command]
pub fn plugins_doc()->&'static str{include_str!("../../docs/SDK.md")}
#[tauri::command]
pub fn game_doc()->&'static str{include_str!("../../docs/GAME.md")}
#[tauri::command]
pub fn plugins_list(st:State<'_,AppState>)->Result<serde_json::Value,String>{
    let mut info=st.game()?.plugin_list();info["folder"]=serde_json::json!(st.plugins_dir.to_string_lossy());Ok(info)
}
#[tauri::command]
pub fn plugins_reload(st:State<'_,AppState>)->Result<serde_json::Value,String>{let mut g=st.game()?;g.load_plugins(&st.plugins_dir);Ok(g.plugin_list())}
#[tauri::command]
pub fn plugins_enable(st:State<'_,AppState>,id:String,enabled:bool)->Result<serde_json::Value,String>{let mut g=st.game()?;g.plugin_enable(&id,enabled)?;drop(g);st.save()?;let g=st.game()?;Ok(g.plugin_list())}
#[tauri::command]
pub fn plugins_run(st:State<'_,AppState>,command:String)->Result<Vec<blackwire_core::OutLine>,String>{
    let mut g=st.game()?;let sid=g.open_session();
    let result=g.plugin_command(sid,&command);g.close_session(sid);
    result.map(|r|r.lines).ok_or_else(||"команда плагина не найдена".into())
}
#[tauri::command]
pub fn plugins_folder(st:State<'_,AppState>)->Result<(),String>{
    std::fs::create_dir_all(&st.plugins_dir).map_err(|e|e.to_string())?;
    #[cfg(target_os="windows")] let command="explorer.exe";
    #[cfg(target_os="linux")] let command="xdg-open";
    #[cfg(target_os="macos")] let command="open";
    std::process::Command::new(command).arg(&st.plugins_dir).spawn().map_err(|e|e.to_string())?;Ok(())
}
#[tauri::command]
pub fn plugins_example(st:State<'_,AppState>,name:Option<String>)->Result<serde_json::Value,String>{
    let name=name.unwrap_or_else(||"device_lab".into());
    let (manifest,source)=match name.as_str(){"device_lab"=>(include_str!("../../sdk/examples/device_lab/plugin.json"),include_str!("../../sdk/examples/device_lab/main.adr")),"district"=>(include_str!("../../sdk/examples/district/plugin.json"),include_str!("../../sdk/examples/district/main.adr")),_=>return Err("нет такого примера".into())};
    let dir=st.plugins_dir.join(name);
    if dir.exists(){return Err("пример уже установлен; его файлы сохранены".into());}
    std::fs::create_dir_all(&dir).map_err(|e|e.to_string())?;
    std::fs::write(dir.join("plugin.json"),manifest).map_err(|e|e.to_string())?;
    std::fs::write(dir.join("main.adr"),source).map_err(|e|e.to_string())?;
    let mut g=st.game()?;g.load_plugins(&st.plugins_dir);Ok(g.plugin_list())
}

#[tauri::command]
pub fn plugins_panel(st:State<'_,AppState>,id:String)->Result<serde_json::Value,String>{
    let g=st.game()?;let(owner,_)=id.split_once(':').ok_or("неверный id окна")?;
    if g.world.plugins.disabled.contains(owner)||g.plugins.loaded.get(owner).map(|p|p.error.is_some()).unwrap_or(true){return Err("плагин не работает".into());}
    g.world.plugins.windows.get(&id).cloned().ok_or_else(||"окно не найдено".into())
}
#[tauri::command]
pub fn plugins_ui(st:State<'_,AppState>,id:String,action:String,values:serde_json::Value)->Result<Vec<blackwire_core::OutLine>,String>{
    if !values.is_object()||values.to_string().len()>65536{return Err("неверные данные формы".into());}
    st.game()?.plugin_ui(&id,&action,values)
}
