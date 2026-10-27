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
fn install_example_files(root: &std::path::Path, name: &str) -> Result<(), String> {
    let files: &[(&str, &str)] = match name {
        "device_lab" => &[
            ("plugin.json", include_str!("../../sdk/examples/device_lab/plugin.json")),
            ("main.adr", include_str!("../../sdk/examples/device_lab/main.adr")),
            ("camera_payload.adr", include_str!("../../sdk/examples/device_lab/camera_payload.adr")),
        ],
        "district" => &[
            ("plugin.json", include_str!("../../sdk/examples/district/plugin.json")),
            ("main.adr", include_str!("../../sdk/examples/district/main.adr")),
        ],
        "custom_vuln_type" => &[
            ("plugin.json", include_str!("../../sdk/examples/custom_vuln_type/plugin.json")),
            ("main.adr", include_str!("../../sdk/examples/custom_vuln_type/main.adr")),
            ("signal_solver.adr", include_str!("../../sdk/examples/custom_vuln_type/signal_solver.adr")),
        ],
        "access_lab" => &[
            ("plugin.json", include_str!("../../sdk/examples/access_lab/plugin.json")),
            ("main.adr", include_str!("../../sdk/examples/access_lab/main.adr")),
        ],
        _ => return Err("нет такого примера".into()),
    };
    let dir = root.join(name);
    if dir.exists() { return Err("пример уже установлен; его файлы сохранены".into()); }
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    for (file, source) in files {
        std::fs::write(dir.join(file), source).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn plugins_example(st:State<'_,AppState>,name:Option<String>)->Result<serde_json::Value,String>{
    let name=name.unwrap_or_else(||"device_lab".into());
    install_example_files(&st.plugins_dir, &name)?;
    let mut g=st.game()?;g.load_plugins(&st.plugins_dir);Ok(g.plugin_list())
}

#[cfg(test)]
mod tests {
    #[test]
    fn installed_device_example_loads_its_paid_exploit_asset_and_preserves_edits() {
        let temporary = std::env::temp_dir();
        let root = temporary.join(format!("blackwire-example-{}-{}", std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        super::install_example_files(&root, "device_lab").unwrap();
        for name in ["district","custom_vuln_type","access_lab"] { super::install_example_files(&root,name).unwrap(); }
        let mut game = blackwire_core::Game::new(blackwire_core::World::new(42, "example_test", ""));
        game.load_plugins(&root);
        assert!(game.plugins.errors.is_empty(), "{:?}", game.plugins.errors);
        assert!(game.plugins.loaded.values().all(|p|p.error.is_none()));
        assert!(game.world.plugins.exploit_types.contains_key("custom_vuln_type:signal_lock"));
        assert!(game.world.plugins.vulnerabilities.contains_key("access_lab:partial_file"));
        assert!(game.plugins.loaded["device_lab"].assets.contains_key("camera_payload.adr"));
        assert!(game.world.plugins.catalogs["market"].contains_key("device_lab:camera_exploit"));
        let main = root.join("device_lab/main.adr");
        std::fs::write(&main, "# my edited example").unwrap();
        assert!(super::install_example_files(&root, "device_lab").is_err());
        assert_eq!(std::fs::read_to_string(&main).unwrap(), "# my edited example");
        let resolved = root.canonicalize().unwrap();
        assert!(resolved.starts_with(temporary.canonicalize().unwrap()));
        assert!(root.file_name().unwrap().to_string_lossy().starts_with("blackwire-example-"));
        std::fs::remove_dir_all(resolved).unwrap();
    }
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
