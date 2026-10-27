//! SDK плагинов Adder. Код исполняется заново для каждого события; состояние — JSON в сохранении.
mod host;
mod extensions;
mod network;
pub(crate) mod events;
mod validate;
pub mod catalog;
mod content;
pub(crate) mod exploit_types;
#[cfg(test)] mod tests;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use crate::{Game, ShellResult, OutLine};

pub const API_VERSION: u32 = 1;
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct SavedPlugins {
    #[serde(default)] pub catalogs: BTreeMap<String, BTreeMap<String, catalog::CatalogItem>>,
    pub quests: BTreeMap<String, extensions::Quest>,
    pub timers: BTreeMap<String, BTreeMap<String, extensions::Timer>>,
    pub windows: BTreeMap<String, Value>,
    pub disabled: BTreeSet<String>,
    pub data: BTreeMap<String, BTreeMap<String, Value>>,
    pub device_types: BTreeMap<String, Value>,
    pub vulnerabilities: BTreeMap<String, Value>,
    pub exploit_types: BTreeMap<String,Value>,
    pub host_activity: BTreeMap<u32,crate::hack::access::Activity>,
    pub npc_logins: BTreeMap<u32,BTreeMap<String,u64>>,
    #[serde(skip)] pub(crate) exploit_presence: BTreeMap<u32,crate::hack::access::Activity>,
    #[serde(skip)] pub(crate) exploit_grants: BTreeMap<String,crate::hack::access::Grant>,
    #[serde(skip)] pub(crate) exploit_grant_seq: u64,
    #[serde(skip)]
    pub(crate) exploit_runtime: BTreeMap<String,std::sync::Arc<Plugin>>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub api: u32,
    pub entry: String,
    #[serde(default)] pub libraries: Vec<String>,
    #[serde(default)] pub assets: Vec<String>,
    #[serde(default)] pub requires: Vec<String>,
    #[serde(default)] pub permissions: BTreeSet<String>,
    #[serde(default)] pub events: BTreeMap<String, String>,
    #[serde(default)] pub commands: BTreeMap<String, String>,
}
#[derive(Clone, Debug)]
pub struct Plugin { pub manifest: Manifest, pub source: String, pub assets: BTreeMap<String,String>, pub error: Option<String> }
#[derive(Default)]
pub struct Runtime { pub sessions: BTreeMap<String,u32>, pub order: Vec<String>, pub network: Value, pub outbox: Vec<Value>, pub loaded: BTreeMap<String, Plugin>, pub errors: Vec<String>, pub busy: bool }
fn ident(s: &str) -> bool { !s.is_empty() && s.len() <= 64 && s.bytes().enumerate().all(|(i,b)| b.is_ascii_alphabetic() || b == b'_' || (i > 0 && b.is_ascii_digit())) }
impl Manifest {
    fn validate(&self) -> Result<(), String> {
        if !ident(&self.id) || self.name.is_empty() || self.version.is_empty() { return Err("неверные id, name или version".into()); }
        if self.api != API_VERSION { return Err(format!("нужен API {}, заявлен {}", API_VERSION, self.api)); }
        if self.requires.iter().any(|s|!ident(s)||s==&self.id){return Err("неверная зависимость плагина".into());}
        for event in self.events.keys() { if !event.starts_with("custom.") && !["on_load","on_tick","on_command","on_mission_done","on_mail","on_hack","on_ui","on_purchase","on_achievement","on_gig_done","on_investigation","on_scenario_done","on_world","on_save","on_unload","on_file","on_network"].contains(&event.as_str()) { return Err(format!("неизвестное событие {event}")); } }
        for handler in self.events.values().chain(self.commands.values()) { if !ident(handler) { return Err("неверное имя обработчика".into()); } }
        for command in self.commands.keys() { if !ident(command) || crate::shell::table::COMMANDS.iter().any(|c| c.name == command) || command == "plugins" { return Err(format!("недопустимая команда {command}")); } }
        for permission in &self.permissions { if !["content","sys","fs","net","shell","time","network"].contains(&permission.as_str()) { return Err(format!("неизвестное разрешение {permission}")); } }
        Ok(())
    }
}
impl Game {
    fn plugin_ready(&self,id:&str)->bool{
        let mut pending=vec![id.to_string()];let mut seen=BTreeSet::new();
        while let Some(id)=pending.pop(){if !seen.insert(id.clone()){continue;}let Some(p)=self.plugins.loaded.get(&id)else{return false};if p.error.is_some()||self.world.plugins.disabled.contains(&id){return false;}pending.extend(p.manifest.requires.clone());}true
    }
    pub fn load_plugins(&mut self, root: &Path) {
        let old_sessions:Vec<_>=self.plugins.sessions.values().copied().collect();for id in old_sessions{self.close_session(id);}
        let mut runtime = Runtime::default();
        let mut dirs: Vec<_> = if root.join("plugin.json").is_file(){vec![root.to_path_buf()]}else{match std::fs::read_dir(root) { Ok(d) => d.flatten().map(|d|d.path()).filter(|d|d.is_dir()).collect(), Err(_) => Vec::new() }};
        dirs.sort();
        let canonical_root = root.canonicalize().ok();
        for dir in dirs.into_iter().take(128) {
            let result = (|| -> Result<Plugin, String> {
                let manifest_path = dir.join("plugin.json").canonicalize().map_err(|e|e.to_string())?;
                if !canonical_root.as_ref().map(|r|manifest_path.starts_with(r)).unwrap_or(false) { return Err("манифест за пределами plugins".into()); }
                if std::fs::metadata(&manifest_path).map_err(|e|e.to_string())?.len() > 65536 { return Err("слишком большой манифест".into()); }
                let manifest: Manifest = serde_json::from_str(&std::fs::read_to_string(manifest_path).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;
                manifest.validate()?;
                let entry = dir.join(&manifest.entry).canonicalize().map_err(|e|e.to_string())?;
                if !entry.starts_with(dir.canonicalize().map_err(|e|e.to_string())?) { return Err("entry выходит за папку плагина".into()); }
                if std::fs::metadata(&entry).map_err(|e|e.to_string())?.len() > 262144 { return Err("исходник больше 256 КБ".into()); }
                let entry_source = std::fs::read_to_string(entry).map_err(|e|e.to_string())?;
                let mut source=String::new();let mut assets=BTreeMap::new();
                for (is_library,relative) in manifest.libraries.iter().map(|s|(true,s)).chain(manifest.assets.iter().map(|s|(false,s))){
                    let path=dir.join(relative).canonicalize().map_err(|e|e.to_string())?;
                    if !path.starts_with(dir.canonicalize().map_err(|e|e.to_string())?){return Err("ресурс выходит за папку плагина".into());}
                    if std::fs::metadata(&path).map_err(|e|e.to_string())?.len()>262144{return Err("ресурс больше 256 КБ".into());}
                    let text=std::fs::read_to_string(path).map_err(|e|e.to_string())?;
                    if is_library{source.push_str(&text);source.push('\n');}else{assets.insert(relative.clone(),text);}
                }
                source.push_str(&entry_source);if source.len()>1048576||assets.values().map(String::len).sum::<usize>()>2097152{return Err("слишком большой пакет плагина".into());}
                blackwire_lang::parse(&source).map_err(|e|e.to_string())?;
                if runtime.loaded.contains_key(&manifest.id) { return Err("повтор id плагина".into()); }
                for c in manifest.commands.keys() { if runtime.loaded.values().any(|p: &Plugin|p.manifest.commands.contains_key(c)) {return Err(format!("повтор команды {c}"));} }
                Ok(Plugin { manifest, source, assets, error: None })
            })();
            match result { Ok(p) => {runtime.loaded.insert(p.manifest.id.clone(),p);}, Err(e) => runtime.errors.push(format!("{}: {e}",dir.display())) }
        }
        let mut remaining:BTreeSet<_>=runtime.loaded.keys().cloned().collect();
        loop {let ready:Vec<_>=remaining.iter().filter(|id|runtime.loaded[*id].manifest.requires.iter().all(|dep|runtime.order.contains(dep))).cloned().collect();if ready.is_empty(){break;}for id in ready{remaining.remove(&id);runtime.order.push(id);}}
        for id in remaining {runtime.loaded.get_mut(&id).unwrap().error=Some("нет зависимости или найден цикл зависимостей".into());}
        self.world.plugins.exploit_runtime=runtime.loaded.iter().filter(|(_,p)|p.error.is_none()).map(|(id,p)|(id.clone(),std::sync::Arc::new(p.clone()))).collect();
        self.plugins = runtime;
        self.plugin_event("on_load", json!({}));
    }
    pub fn plugin_list(&self) -> Value {
        json!({"api": API_VERSION, "errors":self.plugins.errors, "plugins":self.plugins.loaded.iter().map(|(id,p)|json!({"id":id,"name":p.manifest.name,"version":p.manifest.version,"permissions":p.manifest.permissions,"commands":p.manifest.commands.keys().collect::<Vec<_>>(),"windows":self.world.plugins.windows.iter().filter(|(key,_)|key.starts_with(&format!("{id}:"))).map(|(key,v)|json!({"id":key,"title":v["title"]})).collect::<Vec<_>>(),"enabled":!self.world.plugins.disabled.contains(id),"error":p.error})).collect::<Vec<_>>()})
    }
    pub fn plugin_enable(&mut self, id: &str, enabled: bool) -> Result<(),String> {
        if !self.plugins.loaded.contains_key(id) {return Err("плагин не найден".into());}
        if enabled {for dep in &self.plugins.loaded[id].manifest.requires{if !self.plugin_ready(dep){return Err(format!("недоступна зависимость {dep}"));}}}
        if !enabled {if let Some(p)=self.plugins.loaded.get(id).cloned(){if let Some(h)=p.manifest.events.get("on_unload"){self.run_plugin(&p,h,json!({}),None);}}}
        if enabled { self.world.plugins.disabled.remove(id); self.plugins.loaded.get_mut(id).unwrap().error=None;
            self.world.plugins.exploit_runtime.insert(id.into(),std::sync::Arc::new(self.plugins.loaded[id].clone()));
            if let Some(p)=self.plugins.loaded.get(id).cloned() {if let Some(h)=p.manifest.events.get("on_load"){self.run_plugin(&p,h,json!({}),None);}}
        } else {self.world.plugins.disabled.insert(id.into());self.plugins.outbox.retain(|packet|packet["plugin"]!=json!(id));if let Some(sid)=self.plugins.sessions.remove(id){self.close_session(sid);}}
        Ok(())
    }
    pub fn plugin_event(&mut self, event: &str, payload: Value) {
        if self.plugins.busy { return; }
        if event=="on_hack" {
            if let Some(name)=payload["vuln"]["name"].as_str(){if let Some((owner,_))=name.split_once(':'){
                let handler=self.world.plugins.vulnerabilities.get(name).and_then(|s|s["handler"].as_str()).map(String::from);
                if let (Some(handler),Some(p))=(handler,self.plugins.loaded.get(owner).cloned()){if p.error.is_none()&&!self.world.plugins.disabled.contains(owner){self.run_plugin(&p,&handler,payload.clone(),None);}}
            }}
        }
        let order:Vec<_>=if self.plugins.order.is_empty(){self.plugins.loaded.keys().cloned().collect()}else{self.plugins.order.clone()};
        let plugins: Vec<_> = order.iter().filter_map(|id|self.plugins.loaded.get(id)).filter(|p|p.error.is_none() && !self.world.plugins.disabled.contains(&p.manifest.id) && p.manifest.events.contains_key(event)).cloned().collect();
        for p in plugins {if !self.plugin_ready(&p.manifest.id){continue;}self.run_plugin(&p,&p.manifest.events[event],payload.clone(),None);}
    }
    fn run_plugin(&mut self, p: &Plugin, handler: &str, event: Value, session: Option<u32>) -> Vec<OutLine> {
        if self.plugins.busy {return vec![OutLine::error("вложенный запуск плагина запрещён")];}
        let previous_world=self.world.clone();
        let pending_scripts=self.pending_scripts.clone();
        let outbox_len=self.plugins.outbox.len();
        let previous_sessions=self.sessions.clone();
        let ui_len=self.pending_ui.len();
        self.plugins.busy=true;
        let internal = session.is_none();
        let sid=if let Some(sid)=session{sid}else if let Some(sid)=self.plugins.sessions.get(&p.manifest.id).copied().filter(|sid|self.sessions.contains_key(sid)){sid}else{let sid=self.open_session();self.plugins.sessions.insert(p.manifest.id.clone(),sid);sid};
        let source=format!("{}\n\n{}(plugin.event())\n",p.source,handler);
        let result=(|| {
            let program=blackwire_lang::parse(&source).map_err(|e|e.to_string())?;
            let mut vm=blackwire_lang::interp::Interpreter::new(vec![p.manifest.id.clone()]);
            vm.set_step_limit(50000);
            let mut host=host::PluginHost::new(self,p,sid,event);
            let result=vm.run(&program,&mut host).map_err(|e|e.to_string()).and_then(|v| match v {blackwire_lang::Value::Int(n) if n!=0=>Err(format!("exit({n})")),_=>Ok(v)});
            Ok::<_,String>((result,host.lines))
        })();
        let (status,mut lines)=match result {Ok(x)=>x,Err(e)=>(Err(e),Vec::new())};
        let _=internal;
        self.plugins.busy=false;
        if let Err(e)=status { self.world=previous_world;self.sessions=previous_sessions;if !self.sessions.contains_key(&sid){self.plugins.sessions.remove(&p.manifest.id);}self.pending_scripts=pending_scripts;self.plugins.outbox.truncate(outbox_len);self.pending_ui.truncate(ui_len);if let Some(loaded)=self.plugins.loaded.get_mut(&p.manifest.id){loaded.error=Some(e.clone());} self.world.plugins.exploit_runtime.remove(&p.manifest.id); lines.push(OutLine::error(format!("{}: {e}",p.manifest.name)));}
        lines
    }
    pub fn plugin_command(&mut self, session: u32, line: &str) -> Option<ShellResult> {
        if self.plugins.busy || !self.sessions.contains_key(&session) {return None;}
        let jobs=crate::shell::parse::parse(line).ok()?;
        if jobs.len()!=1 || jobs[0].pipeline.cmds.len()!=1 {return None;}
        let cmd=&jobs[0].pipeline.cmds[0];
        let words:Vec<_>=cmd.words.iter().map(|w|w.raw()).collect();
        let name=words.first()?;
        let p=self.plugins.loaded.values().find(|p|p.manifest.commands.contains_key(name) && !self.world.plugins.disabled.contains(&p.manifest.id)).cloned()?;
        let lines=if !self.plugin_ready(&p.manifest.id) {vec![OutLine::error(format!("плагин или его зависимость не работает: {}",p.error.as_deref().unwrap_or("зависимость выключена")))]} else if let Some(e)=&p.error {vec![OutLine::error(format!("плагин остановлен: {e}"))]} else if cmd.redirs.stdout.is_some() || cmd.redirs.stderr.is_some() || cmd.redirs.stdin.is_some() {vec![OutLine::error("команды плагинов API 1 запускаются отдельно, без перенаправления")]} else {self.run_plugin(&p,&p.manifest.commands[name],json!({"args":&words[1..],"line":line,"session":session}),Some(session))};
        if let Some(s)=self.sessions.get_mut(&session){s.history.push(line.into());s.last_status=if lines.iter().any(|l|l.kind==crate::OutKind::Error){1}else{0};}
        Some(ShellResult{lines,prompt:self.prompt(session),..Default::default()})
    }
}
