//! Fictional exploit conditions and scoped object grants. No host OS access.
use std::collections::BTreeSet;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use crate::{Game, World};
use crate::session::Session;
use super::{VulnInfo, VulnAccess, ExploitOutcome};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Activity { pub users: BTreeSet<String>, pub connections: u32, pub forwarded_ports: BTreeSet<u16> }
#[derive(Clone, Debug)]
pub struct Grant { pub session:u32, pub target:u32, pub expires:u64, pub definition:Value, pub rule:Value, pub origin:VulnInfo }

impl Game {
    pub(crate) fn sync_exploit_presence(&mut self) {
        self.world.plugins.exploit_presence.clear();
        for session in self.sessions.values() {
            for hop in &session.hops {
                let state=self.world.plugins.exploit_presence.entry(hop.machine).or_default();
                state.users.insert(hop.user.clone());
            }
            for hops in session.hops.windows(2) {
                let state=self.world.plugins.exploit_presence.entry(hops[1].machine).or_default();
                state.connections=state.connections.saturating_add(1);
            }
        }
    }
}
fn spec<'a>(world:&'a World,v:&VulnInfo)->Option<&'a Value>{world.plugins.vulnerabilities.get(&v.name)}
fn absolute(path:&str)->bool{path.starts_with('/') && path.len()<=1024 && !path.contains('\0') && crate::vfs::normalize(path)==path}
fn methods(kind:&str)->&'static [&'static str]{
    match kind.trim_start_matches("partial_") {
        "file"=>&["read","write","stat"],
        "router"=>&["hostname","services","routes","connections"],
        "computer"=>&["hostname","services","users","processes","read","write","list"],
        "shell"=>&["whoami","hostname","read","list"], _=>&[]
    }
}
pub fn validate_spec(spec:&Value)->Result<(),String>{
    if let Some(r)=spec.get("result"){
        let o=r.as_object().ok_or(tr!("result должен быть словарём", "result must be a dict"))?;
        if o.keys().any(|k|!["type","privilege","user","methods","path"].contains(&k.as_str())){return Err(tr!("неизвестное поле result", "unknown field in result").into());}
        let kind=r["type"].as_str().ok_or(tr!("нужен result.type", "result.type is required"))?;
        if !["shell","creds","read","info","computer","router","file","partial_shell","partial_computer","partial_router","partial_file"].contains(&kind){return Err(tr!("неизвестный result.type", "unknown result.type").into());}
        if spec["local"]==true && (kind!="shell" || r["privilege"]!="root"){return Err(tr!("локальный результат: shell с privilege=root", "local result: shell with privilege=root").into());}
        if let Some(p)=r.get("privilege"){if !p.as_str().is_some_and(|s|["guest","user","root"].contains(&s)){return Err(tr!("privilege: guest, user или root", "privilege: guest, user or root").into());}}
        if let Some(u)=r.get("user"){if !u.as_str().is_some_and(|s|!s.is_empty()&&s.len()<=64){return Err(tr!("неверный result.user", "invalid result.user").into());}}
        if let Some(p)=r.get("path"){if !p.as_str().is_some_and(absolute){return Err(tr!("result.path: нужен нормализованный абсолютный путь", "result.path: a normalized absolute path is required").into());}}
        if ["file","partial_file","read"].contains(&kind) && r["path"].as_str().is_none(){return Err(tr!("файловому результату нужен path", "a file result needs path").into());}
        if let Some(m)=r.get("methods"){
            let list=m.as_array().ok_or(tr!("methods должен быть списком", "methods must be a list"))?;
            if list.is_empty()||list.len()>8||list.iter().any(|m|!m.as_str().is_some_and(|m|methods(kind).contains(&m))){return Err(tr!("недопустимые методы для result.type", "methods not allowed for this result.type").into());}
        }
        if kind.starts_with("partial_")&&r.get("methods").is_none(){return Err(tr!("частичному объекту нужен явный список methods", "a partial object needs an explicit methods list").into());}
    }
    if let Some(list)=spec.get("conditions"){
        let list=list.as_array().ok_or(tr!("conditions должен быть списком", "conditions must be a list"))?;
        if list.len()>16{return Err(tr!("не более 16 условий", "at most 16 conditions").into());}
        for c in list {
            let o=c.as_object().ok_or(tr!("условие должно быть словарём", "a condition must be a dict"))?;
            let kind=c["type"].as_str().ok_or(tr!("нужен conditions.type", "conditions.type is required"))?;
            let allowed:&[&str]=match kind {
                "active_user"=>&["type","user"], "active_admin"|"same_lan"=>&["type"],
                "users_min"|"users_max"|"connections_min"|"connections_max"|"forwarded_ports_min"=>&["type","count"],
                "service_running"=>&["type","service"],"os_prefix"=>&["type","prefix"],"file_exists"=>&["type","path"],
                "time_window"=>&["type","start","end"],"plugin_state"=>&["type","key","equals"],_=>return Err(tr!("неизвестный тип условия", "unknown condition type").into())};
            if o.keys().any(|k|!allowed.contains(&k.as_str())){return Err(tr!("неизвестное поле условия", "unknown condition field").into());}
            if kind.ends_with("_min")||kind.ends_with("_max"){if !c["count"].as_u64().is_some_and(|n|n<=4096){return Err(tr!("count должен быть 0..4096", "count must be 0..4096").into());}}
            for key in ["service","prefix","key"]{if allowed.contains(&key)&&!c[key].as_str().is_some_and(|s|!s.is_empty()&&s.len()<=128){return Err(trf!("нужен {key}", "{key} is required"));}}
            if kind=="active_user"&&c.get("user").is_some_and(|u|!u.as_str().is_some_and(|s|!s.is_empty()&&s.len()<=64)){return Err(tr!("неверный user условия", "invalid condition user").into());}
            if kind=="file_exists"&&!c["path"].as_str().is_some_and(absolute){return Err(tr!("нужен абсолютный path", "an absolute path is required").into());}
            if kind=="time_window"&&["start","end"].iter().any(|k|!c[k].as_u64().is_some_and(|n|n<24)){return Err(tr!("часы должны быть 0..23", "hours must be 0..23").into());}
            if kind=="plugin_state"&&c.get("equals").is_none(){return Err(tr!("нужно equals для plugin_state", "plugin_state needs equals").into());}
        }
    }
    Ok(())
}
pub fn result(world:&World,v:&VulnInfo)->Value{
    if let Some(r)=spec(world,v).and_then(|s|s.get("result")){return r.clone();}
    // Native examples of scoped results, without replacing the older entry points.
    let kind=super::payload::kind(world,v);
    if !v.name.contains(':')&&!v.local {
        match kind.as_str(){
            "object_reference"|"xml_entity"=>return json!({"type":"partial_file","path":match &v.access{VulnAccess::Read(p)=>p.as_str(),_=>"/etc/passwd"},"methods":["read","stat"]}),
            "cache_poisoning"=>return json!({"type":"partial_router","methods":["hostname","routes"]}),
            "prototype_pollution"=>return json!({"type":"partial_computer","methods":["hostname","services"]}),
            "fragment_overlap"=>return json!({"type":"partial_shell","privilege":"user","methods":["whoami","hostname","list"]}),
            "token_confusion"=>return json!({"type":"shell","privilege":"guest"}),_=>{}
        }
    }
    match &v.access{
        VulnAccess::Root=>json!({"type":"shell","privilege":"root"}),VulnAccess::Shell=>json!({"type":"shell","privilege":"user"}),
        VulnAccess::Creds=>json!({"type":"creds"}),VulnAccess::Read(path)=>json!({"type":"read","path":path}),VulnAccess::Info=>json!({"type":"info"})}
}
fn requirements(world:&World,v:&VulnInfo)->Vec<Value>{
    if let Some(list)=spec(world,v).and_then(|s|s["conditions"].as_array()){return list.clone();}
    if !v.name.contains(':')&&!v.local {
        match super::payload::kind(world,v).as_str(){
            "token_confusion"=>return vec![json!({"type":"active_user"})],
            "unsafe_deserialization"=>return vec![json!({"type":"active_admin"})],
            "fragment_overlap"=>return vec![json!({"type":"connections_min","count":1})],
            "weak_rng"=>return vec![json!({"type":"users_min","count":2})],
            "request_smuggling"=>return vec![json!({"type":"service_running","service":v.service})],_=>{}
        }
    }
    vec![]
}
pub fn conditions(world:&World,target:u32,v:&VulnInfo,session:Option<&Session>)->Value{
    let m=world.machine(target);let mut active=world.plugins.host_activity.get(&target).cloned().unwrap_or_default();
    if let Some(p)=world.plugins.exploit_presence.get(&target){active.users.extend(p.users.clone());active.connections=active.connections.saturating_add(p.connections);}
    if let Some(s)=session {for h in &s.hops{if h.machine==target{active.users.insert(h.user.clone());}}}
    if let Some(users)=world.plugins.npc_logins.get(&target){
        let users:Vec<_>=users.iter().filter(|(user,expires)|**expires>world.now()&&m.users.exists(user)).map(|(user,_)|user.clone()).collect();
        active.connections=active.connections.saturating_add(users.len() as u32);active.users.extend(users);
    }
    active.users.retain(|u|m.users.exists(u));
    let owner=v.name.split_once(':').map(|x|x.0).unwrap_or("");
    let hour=(world.now()/3600)%24;
    let rows:Vec<Value>=requirements(world,v).into_iter().map(|c|{
        let kind=c["type"].as_str().unwrap_or("");let count=c["count"].as_u64().unwrap_or(0) as usize;
        let (ok,observed,hint)=match kind{
            "active_user"=>{let ok=if let Some(u)=c["user"].as_str(){active.users.contains(u)}else{active.users.iter().any(|u|m.users.get(u).is_some_and(|u|!u.is_admin))};(ok,json!(active.users),tr!("Нужен вошедший пользователь; дождись его входа или используй другой сеанс.", "A logged-in user is required; wait for a login or use another session."))},
            "active_admin"=>(active.users.iter().any(|u|m.users.get(u).is_some_and(|u|u.is_admin)),json!(active.users),tr!("Нужен активный администратор, системный процесс init не считается входом.", "An active administrator is required; the init system process does not count as a login.")),
            "users_min"=>(m.users.names().len()>=count,json!(m.users.names().len()),tr!("Нужно больше зарегистрированных учётных записей.", "More registered accounts are required.")),
            "users_max"=>(m.users.names().len()<=count,json!(m.users.names().len()),tr!("Слишком много зарегистрированных учётных записей.", "Too many registered accounts.")),
            "connections_min"=>(active.connections as usize>=count,json!(active.connections),tr!("Нужно больше активных входящих соединений.", "More active incoming connections are required.")),
            "connections_max"=>(active.connections as usize<=count,json!(active.connections),tr!("Слишком много активных входящих соединений.", "Too many active incoming connections.")),
            "forwarded_ports_min"=>(active.forwarded_ports.len()>=count,json!(active.forwarded_ports),tr!("Нужно больше проброшенных портов; открытые службы сами по себе не являются пробросом.", "More forwarded ports are required; open services by themselves are not forwarding.")),
            "service_running"=>(m.service_named(c["service"].as_str().unwrap_or("")).is_some(),json!(m.services.iter().filter(|s|s.running).map(|s|&s.name).collect::<Vec<_>>()),tr!("Нужна работающая служба из условия.", "The service named in the condition must be running.")),
            "os_prefix"=>(m.os.starts_with(c["prefix"].as_str().unwrap_or("\0")),json!(m.os),tr!("Нужна подходящая версия ОС.", "A matching OS version is required.")),
            "file_exists"=>(m.fs.exists(c["path"].as_str().unwrap_or("\0")),c["path"].clone(),tr!("На целевой машине должен существовать указанный файл.", "The specified file must exist on the target machine.")),
            "time_window"=>{let a=c["start"].as_u64().unwrap_or(24);let b=c["end"].as_u64().unwrap_or(24);(if a<=b{hour>=a&&hour<b}else{hour>=a||hour<b},json!(hour),tr!("Дождись нужного часа игрового времени; конец интервала не включён.", "Wait for the right hour of game time; the end of the interval is excluded."))},
            "same_lan"=>{let from=session.map(|s|world.machine(s.machine_id()));(from.is_some_and(|f|f.id==m.id||f.router==Some(m.id)||m.router==Some(f.id)||f.router.is_some()&&f.router==m.router),json!(session.map(|s|s.machine_id())),tr!("Запусти эксплоит из той же локальной сети.", "Run the exploit from the same local network."))},
            "plugin_state"=>{let value=world.plugins.data.get(owner).and_then(|d|d.get(c["key"].as_str().unwrap_or(""))).cloned().unwrap_or(Value::Null);(value==c["equals"],value,tr!("Выполни условие сценария плагина; значение хранится через plugin.set.", "Meet the plugin scenario condition; the value is stored via plugin.set."))},
            _=>(false,Value::Null,tr!("Неизвестное условие; обнови плагин.", "Unknown condition; update the plugin."))};
        json!({"requirement":c,"ok":ok,"observed":observed,"hint":hint})
    }).collect();json!(rows)
}
pub fn check(world:&World,session:&Session,target:u32,v:&VulnInfo)->Result<(),String>{
    for c in conditions(world,target,v,Some(session)).as_array().unwrap(){if c["ok"]!=true{return Err(trf!("Условие не выполнено: {}. {} Сейчас: {}", "Condition not met: {}. {} Now: {}",c["requirement"],c["hint"].as_str().unwrap_or(""),c["observed"]));}}
    Ok(())
}
fn shell_user(world:&World,target:u32,r:&Value)->Result<String,String>{
    let m=world.machine(target);
    if let Some(user)=r["user"].as_str(){if !m.users.exists(user){return Err(tr!("пользователь результата отсутствует", "the result user does not exist").into());}return Ok(user.into());}
    match r["privilege"].as_str().unwrap_or("user"){
        "root"=>Ok("root".into()),"guest"=>Ok("guest".into()),
        _=>m.users.list().iter().find(|u|!u.is_admin&&u.name!="guest").map(|u|u.name.clone()).map(Ok).unwrap_or_else(||Ok("guest".into()))}
}
pub fn apply(world:&mut World,session:&mut Session,target:u32,v:&VulnInfo)->Result<ExploitOutcome,String>{
    check(world,session,target,v)?;let r=result(world,v);let kind=r["type"].as_str().ok_or(tr!("нет типа результата", "result has no type"))?;
    match kind{
        "shell"=>{let user=shell_user(world,target,&r)?;
            if user=="guest"&&!world.machine(target).users.exists("guest"){let m=world.machine_mut(target);m.users.add("guest","!guest-disabled-login",false);m.fs.add_user_home("guest");}
            let home=world.machine(target).users.home_of(&user);session.push_hop(target,&user,&home);
            if !world.player.owned.contains(&target){world.player.owned.push(target);}
            Ok(ExploitOutcome::Shell{user})},
        "creds"=>{let user=r["user"].as_str().map(String::from).or_else(||world.machine(target).users.list().iter().find(|u|!u.is_admin).map(|u|u.name.clone())).unwrap_or_else(||"root".into());
            let password=world.machine(target).users.get(&user).ok_or(tr!("пользователь результата отсутствует", "the result user does not exist"))?.password.clone();
            world.add_found_cred(crate::net::machine::FoundCred{machine:target,user:user.clone(),password:password.clone(),note:trf!("эксплоит {}", "exploit {}", v.name)});Ok(ExploitOutcome::Creds{user,password})},
        "read"=>{let path=r["path"].as_str().ok_or(tr!("нет path", "no path"))?;let data=if path=="/etc/passwd"{world.machine(target).users.passwd_text()}else{world.machine(target).fs.read_text(path).unwrap_or_else(|_|tr!("(файл недоступен)", "(file unavailable)").into())};Ok(ExploitOutcome::Read{path:path.into(),data})},
        "info"=>Ok(ExploitOutcome::Read{path:tr!("(сведения)", "(info)").into(),data:world.machine(target).service_named(&v.service).map(|s|s.banner.clone()).unwrap_or_default()}),
        _=>{let mut definition=r.clone();let allowed=r["methods"].as_array().map(|a|a.iter().filter_map(Value::as_str).map(String::from).collect::<Vec<_>>()).unwrap_or_else(||methods(kind).iter().map(|s|s.to_string()).collect());
            definition["methods"]=json!(allowed);
            if kind=="partial_shell"{let user=shell_user(world,target,&r)?;if user=="guest"&&!world.machine(target).users.exists("guest"){let m=world.machine_mut(target);m.users.add("guest","!guest-disabled-login",false);m.fs.add_user_home("guest");}definition["user"]=json!(user);}
            let now=world.now();world.plugins.exploit_grants.retain(|_,g|g.expires>now);
            if world.plugins.exploit_grants.len()>=2048{return Err(tr!("слишком много объектов доступа; дождись истечения старых", "too many access objects; wait for old ones to expire").into());}
            world.plugins.exploit_grant_seq=world.plugins.exploit_grant_seq.checked_add(1).ok_or(tr!("исчерпаны номера объектов", "object numbers exhausted"))?;
            let epoch=std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d|d.as_nanos()).unwrap_or(0);
            let handle=format!("bw-{:x}-{}-{}-{}",epoch,session.id,target,world.plugins.exploit_grant_seq);
            world.plugins.exploit_grants.insert(handle.clone(),Grant{session:session.id,target,expires:now+300,definition,rule:r.clone(),origin:v.clone()});
            Ok(ExploitOutcome::Object{handle,kind:kind.into(),methods:allowed})}
    }
}
pub fn use_object(game:&mut Game,sid:u32,args:&[Value])->Result<Value,String>{
    let handle=args.first().and_then(Value::as_str).ok_or(tr!("net.access(handle, метод, ...)", "net.access(handle, method, ...)"))?;
    let method=args.get(1).and_then(Value::as_str).ok_or(tr!("нужен метод объекта", "object method required"))?;
    let grant=game.world.plugins.exploit_grants.get(handle).cloned().ok_or(tr!("объект доступа не найден; повтори взлом", "access object not found; repeat the hack"))?;
    if grant.session!=sid||!game.sessions.contains_key(&sid){return Err(tr!("объект принадлежит другому сеансу", "the object belongs to another session").into());}
    if game.world.now()>=grant.expires{return Err(tr!("объект доступа истёк; повтори взлом", "access object expired; repeat the hack").into());}
    let target=grant.target;
    let session=&game.sessions[&sid];super::payload::ensure_reachable(&game.world,session,target,&grant.origin)?;
    if !super::remote_vulns(game.world.machine(target)).contains(&grant.origin){return Err(tr!("служба или версия изменились; повтори взлом", "service or version changed; repeat the hack").into());}
    if result(&game.world,&grant.origin)!=grant.rule{return Err(tr!("правило доступа изменилось; повтори взлом", "access rule changed; repeat the hack").into());}
    if !grant.definition["methods"].as_array().unwrap().iter().any(|v|v==method){return Err(tr!("метод не разрешён этим объектом", "method not allowed by this object").into());}
    let r=&grant.definition;let is_file=r["type"].as_str().unwrap().ends_with("file");
    let path=if is_file{r["path"].as_str().unwrap_or("").to_string()}else{args.get(2).and_then(Value::as_str).unwrap_or("/").to_string()};
    if ["read","write","list","stat"].contains(&method)&&!absolute(&path){return Err(tr!("нужен нормализованный абсолютный путь", "a normalized absolute path is required").into());}
    let who=r["user"].as_str().unwrap_or("root");
    if !game.world.machine(target).users.exists(who){return Err(tr!("пользователь объекта удалён", "the object's user was deleted").into());}
    let m=game.world.machine(target);
    let value=match method{
        "hostname"=>json!(m.hostname),"whoami"=>json!(who),
        "services"=>json!(m.services),"users"=>json!(m.users.names()),
        "processes"=>json!(m.procs.list().iter().map(|p|json!({"pid":p.pid,"name":p.name,"user":p.user})).collect::<Vec<_>>()),
        "routes"=>json!(game.world.machines.iter().filter(|node|node.router==Some(target)).map(|node|json!({"hostname":node.hostname,"ip":node.lan_ip.as_ref().unwrap_or(&node.ip)})).collect::<Vec<_>>()),
        "connections"=>json!(game.world.plugins.exploit_presence.get(&target).map(|a|a.connections).unwrap_or(0)+game.world.plugins.host_activity.get(&target).map(|a|a.connections).unwrap_or(0)+game.world.plugins.npc_logins.get(&target).map(|users|users.iter().filter(|(user,end)|**end>game.world.now()&&m.users.exists(user)).count() as u32).unwrap_or(0)),
        "read"=>json!(m.fs.read(&path,who).map_err(|e|e.to_string())?),
        "stat"=>serde_json::to_value(m.fs.stat(&path).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?,
        "list"=>serde_json::to_value(m.fs.list(&path,who).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?,
        "write"=>{let text=args.get(if is_file{2}else{3}).and_then(Value::as_str).ok_or(tr!("нужен текст", "text required"))?;if text.len()>65536{return Err(tr!("текст больше 64 КиБ", "text exceeds 64 KiB").into());}game.world.machine_mut(target).fs.write(&path,text,who).map_err(|e|e.to_string())?;Value::Null},
        _=>return Err(tr!("неизвестный метод объекта", "unknown object method").into())};
    game.world.log(target,crate::net::machine::LogKind::Net,&trf!("объект {}: {}", "object {}: {}", grant.origin.name, method));
    if super::tracks::is_monitored(&game.world,target){super::tracks::attack_heat(&mut game.world,&game.sessions[&sid],target);}
    Ok(value)
}
