use super::*;
use crate::net::machine::Machine;
use crate::hack::{VulnInfo, access_from_str};
fn string<'a>(args: &'a [Value], i:usize)->Result<&'a str,String>{args.get(i).and_then(Value::as_str).ok_or_else(||"нужна строка".into())}
fn namespace(owner:&str,key:&str)->String{format!("{owner}:{key}")}
/// Применение нескольких разделов мира атомарно, без удаления машин и поломки активных сеансов.
pub(super) fn apply(game:&mut Game, patch:&Value)->Result<Value,String>{
    let mut value=serde_json::to_value(&game.world).map_err(|e|e.to_string())?;
    let object=patch.as_object().ok_or("нужен словарь разделов мира")?;
    for (key,v) in object {
        if !["machines","orgs","player","missions","finance","ending","life","gigs","market","base","achievements","scenarios","investigation","fw_down","keyloggers","botnet","botnet_mined","defense","down_until"].contains(&key.as_str()) {return Err(format!("раздел {key} недоступен для content.apply"));}
        value[key]=v.clone();
    }
    let decoded:crate::World=serde_json::from_value(value).map_err(|e|e.to_string())?;
    let mut next=game.world.clone();
    for key in object.keys() {match key.as_str(){
        "machines"=>next.machines=decoded.machines.clone(), "orgs"=>next.orgs=decoded.orgs.clone(),
        "player"=>next.player=decoded.player.clone(), "missions"=>next.missions=decoded.missions.clone(),
        "finance"=>next.finance=decoded.finance.clone(), "ending"=>next.ending=decoded.ending.clone(),
        "life"=>next.life=decoded.life.clone(), "gigs"=>next.gigs=decoded.gigs.clone(),
        "market"=>next.market=decoded.market.clone(), "base"=>next.base=decoded.base.clone(),
        "achievements"=>next.achievements=decoded.achievements.clone(), "scenarios"=>next.scenarios=decoded.scenarios.clone(),
        "investigation"=>next.investigation=decoded.investigation.clone(),"fw_down"=>next.fw_down=decoded.fw_down.clone(),
        "keyloggers"=>next.keyloggers=decoded.keyloggers.clone(),"botnet"=>next.botnet=decoded.botnet.clone(),
        "botnet_mined"=>next.botnet_mined=decoded.botnet_mined,"defense"=>next.defense=decoded.defense.clone(),"down_until"=>next.down_until=decoded.down_until.clone(),_=>{}
    }}
    if next.missions.active>next.missions.missions.len(){return Err("неверный индекс задания".into());}
    if next.machines.len()>4096 || next.machines.is_empty() {return Err("в мире должно быть 1..4096 устройств".into());}
    let ids:BTreeSet<_>=next.machines.iter().map(|m|m.id).collect();
    let orgs:BTreeSet<_>=next.orgs.iter().map(|o|o.id).collect();
    if ids.len()!=next.machines.len() || orgs.len()!=next.orgs.len(){return Err("повтор идентификаторов".into());}
    if game.world.machines.iter().any(|m|!ids.contains(&m.id)) || game.world.orgs.iter().any(|o|!orgs.contains(&o.id)) {return Err("нельзя удалять существующие устройства и организации; используйте up=false".into());}
    if next.machines[0].id!=game.world.machines[0].id || next.player.handle!=game.world.player.handle || !next.machines[0].users.exists(&next.player.handle) {return Err("нельзя подменять личность игрока и его компьютер".into());}
    let mut hosts=BTreeSet::new();let mut ips=BTreeSet::new();
    for m in &next.machines {
        if !hosts.insert(&m.hostname) || !ips.insert(&m.ip) || m.hostname.is_empty() || m.ip.parse::<std::net::Ipv4Addr>().is_err() {return Err("неверный или повторный hostname/ip".into());}
        if m.router.map(|r|!ids.contains(&r)).unwrap_or(false) || m.org.map(|o|!orgs.contains(&o)).unwrap_or(false) {return Err("устройство ссылается на неизвестный шлюз/организацию".into());}
    }
    for o in &next.orgs {if !ids.contains(&o.gateway) || o.members.iter().any(|id|!ids.contains(id)){return Err("неизвестное устройство в организации".into());}}
    for session in game.sessions.values(){for hop in &session.hops {if !next.machine(hop.machine).users.exists(&hop.user){return Err("нельзя удалять пользователя активного сеанса".into());}}}
    // Служебные очереди #[serde(skip)] сохраняем из исходного мира.
    next.missions.pending=game.world.missions.pending.clone();
    next.finance.pending=game.world.finance.pending.clone();
    next.ending.pending=game.world.ending.pending.clone();
    next.gigs.pending=game.world.gigs.pending.clone();
    next.scenarios.pending=game.world.scenarios.pending.clone();
    super::validate::world(game,&next)?;
    game.world=next;
    refresh(game)?;
    Ok(Value::Null)
}
pub fn refresh(game:&mut Game)->Result<(),String>{
    let defs:Vec<_>=game.world.plugins.vulnerabilities.values().cloned().collect();
    for spec in defs {
        let name=spec["name"].as_str().ok_or("нужно имя уязвимости")?;
        let service=spec["service"].as_str().unwrap_or("");
        let prefix=spec["version_prefix"].as_str().unwrap_or("");
        let local=spec["local"].as_bool().unwrap_or(false);
        for m in &mut game.world.machines {
            m.custom_vulns.retain(|v|v.name!=name);
            if spec.get("device_type").and_then(Value::as_str).map(|t|m.device_type.as_deref()!=Some(t)).unwrap_or(false) || spec.get("machine").and_then(Value::as_u64).map(|id|m.id as u64 !=id).unwrap_or(false){continue;}
            let access=access_from_str(spec["access"].as_str().unwrap_or("info"));
            if local {
                if m.os.starts_with(prefix){m.custom_vulns.push(VulnInfo{service:"kernel".into(),port:0,version:m.os.clone(),name:name.into(),access,local:true});}
            } else {
                for s in &m.services {if s.name==service && s.version.starts_with(prefix){m.custom_vulns.push(VulnInfo{service:s.name.clone(),port:s.port,version:s.version.clone(),name:name.into(),access:access.clone(),local:false});}}
            }
        }
    }
    Ok(())
}
pub fn call(game:&mut Game,owner:&str,name:&str,args:&[Value])->Result<Value,String>{
    if let Some(result)=super::extensions::call(game,owner,name,args){return result;}
    match name {
        "plugin.catalog"=>super::catalog::register(game,owner,args),
        "plugin.unlock"=>{let id=string(args,0)?;let item=game.world.plugins.catalogs.get("achievements").and_then(|c|c.get(id)).ok_or("достижение не найдено")?.clone();
            if game.world.achievements.unlocked.insert(id.into()){game.world.achievements.pending=Some(crate::achievements::AchEvent{name:item.name});}Ok(Value::Null)},
        "plugin.random"=>{let lo=args.first().and_then(Value::as_i64).ok_or("нужна нижняя граница")?;let hi=args.get(1).and_then(Value::as_i64).ok_or("нужна верхняя граница")?;if lo< -1_000_000_000 || hi>1_000_000_000 || hi<=lo{return Err("нужен диапазон -10^9..10^9, lo<hi".into());}Ok(json!(game.world.rng.range(lo,hi)))},
        "plugin.template"=>{
            let kind=serde_json::from_value(json!(string(args,0)?)).map_err(|e|e.to_string())?;
            let m=crate::net::gen_machine::build(crate::net::gen_machine::MachineSpec{id:0,hostname:"template".into(),ip:"198.18.0.1".into(),lan_ip:None,domain:None,kind,org:None,router:None,org_name:"".into(),care:crate::net::orgs::Care::Normal,x:0.5,y:0.5,people:Vec::new(),peer_ips:Vec::new(),epoch:game.world.now()},&mut game.world.rng);
            serde_json::to_value(m).map_err(|e|e.to_string())
        },
        "plugin.user"=>{let mid=args.first().and_then(Value::as_u64).filter(|n|*n<=u32::MAX as u64).ok_or("нужен id устройства")? as u32;if !game.world.has_machine(mid){return Err("нет устройства".into());}let name=string(args,1)?;if !super::ident(name){return Err("неверное имя пользователя".into());}let password=string(args,2)?;let admin=args.get(3).and_then(Value::as_bool).unwrap_or(false);let m=game.world.machine_mut(mid);m.users.add(name,password,admin);m.fs.add_user_home(name);Ok(Value::Null)},
        "plugin.write"=>{let mid=args.first().and_then(Value::as_u64).filter(|n|*n<=u32::MAX as u64).ok_or("нужен id устройства")? as u32;if !game.world.has_machine(mid){return Err("нет устройства".into());}let path=string(args,1)?;let text=string(args,2)?;let m=game.world.machine_mut(mid);m.fs.mkdir_all(&crate::vfs::dirname(path),"root").map_err(|e|e.to_string())?;m.fs.write_text(path,text,"root").map_err(|e|e.to_string())?;Ok(Value::Null)},
        "plugin.world"=>match args.first().and_then(Value::as_str) {
            None if args.is_empty()=>serde_json::to_value(&game.world).map_err(|e|e.to_string()),
            Some("overview")=>{
                let w=&game.world;
                let machines:Vec<_>=w.machines.iter().map(|m|json!({
                    "id":m.id,"hostname":m.hostname,"ip":m.ip,"lan_ip":m.lan_ip,
                    "router":m.router,"kind":m.kind,"os":m.os,"up":m.up,
                    "ram_mb":m.ram_mb,"disk_mb":m.disk_mb,"services":m.services,
                    "custom_vulns":m.custom_vulns,
                    "logs":m.logs.iter().rev().take(4).collect::<Vec<_>>(),
                    "mail":m.mail.iter().map(|mail|json!({"from":mail.from,"subject":mail.subject,"read":mail.read})).collect::<Vec<_>>()
                })).collect();
                let types:serde_json::Map<String,Value>=w.plugins.device_types.iter()
                    .map(|(key,value)|(key.clone(),json!({"kind":value["kind"],"os":value["os"]}))).collect();
                Ok(json!({"player":w.player,"machines":machines,"orgs":w.orgs,
                    "finance":w.finance,"investigation":w.investigation,
                    "plugins":{"device_types":types,"quests":w.plugins.quests,"catalogs":w.plugins.catalogs}}))
            },
            _=>Err("plugin.world: use no arguments or overview".into())
        },
        "plugin.apply"=>apply(game,args.first().ok_or("нужен словарь изменений")?),
        "plugin.device_type"=>{
            let key=namespace(owner,string(args,0)?);
            let template=args.get(1).ok_or("нужен шаблон устройства")?;
            serde_json::from_value::<Machine>(template.clone()).map_err(|e|e.to_string())?;
            game.world.plugins.device_types.insert(key.clone(),template.clone());Ok(json!(key))
        }
        "plugin.spawn"=>{
            let key=string(args,0)?;
            let mut template=game.world.plugins.device_types.get(key).cloned().ok_or("тип устройства не зарегистрирован")?;
            let overrides=args.get(1).and_then(Value::as_object).ok_or("нужны свойства нового устройства")?;
            for (k,v) in overrides {template[k]=v.clone();}
            let id=game.world.machines.iter().map(|m|m.id).max().unwrap_or(0).checked_add(1).ok_or("нет свободных id")?;
            template["id"]=json!(id);template["device_type"]=json!(key);
            let mut machines=serde_json::to_value(&game.world.machines).map_err(|e|e.to_string())?;
            machines.as_array_mut().unwrap().push(template);
            apply(game,&json!({"machines":machines}))?;Ok(json!(id))
        }
        "plugin.vulnerability"=>{
            let key=namespace(owner,string(args,0)?);
            let mut spec=args.get(1).cloned().ok_or("нужно описание уязвимости")?;
            if !spec.is_object(){return Err("нужен словарь уязвимости".into());}
            let access=spec["access"].as_str().ok_or("нужен access")?;
            if !["creds","shell","root","info"].contains(&access) && !access.starts_with("read:/"){return Err("неизвестный access".into());}
            if spec["local"]==json!(true)&&access!="root"{return Err("локальное правило privesc должно давать root".into());}
            if spec["local"]!=json!(true) && spec["service"].as_str().unwrap_or("").is_empty(){return Err("нужна служба".into());}
            if let Some(t)=spec.get("device_type").and_then(Value::as_str){if !game.world.plugins.device_types.contains_key(t){return Err("тип устройства не найден".into());}}
            if let Some(mid)=spec.get("machine").and_then(Value::as_u64){if mid>u32::MAX as u64 || !game.world.has_machine(mid as u32){return Err("устройство не найдено".into());}}
            if let Some(h)=spec.get("handler"){if !h.as_str().map(super::ident).unwrap_or(false){return Err("неверное имя обработчика уязвимости".into());}}
            if let Some(exploit) = spec.get("exploit") { crate::hack::payload::validate_spec(exploit)?; }
            spec["name"]=json!(key);
            game.world.plugins.vulnerabilities.insert(key.clone(),spec);refresh(game)?;Ok(json!(key))
        }
        _=>Err(format!("нет функции {name}"))
    }
}
