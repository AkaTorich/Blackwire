use super::*;
use crate::net::machine::Machine;
use crate::hack::{VulnInfo, access_from_str};
fn string<'a>(args: &'a [Value], i:usize)->Result<&'a str,String>{args.get(i).and_then(Value::as_str).ok_or_else(||tr!("нужна строка", "string required").into())}
fn namespace(owner:&str,key:&str)->String{format!("{owner}:{key}")}
/// Применение нескольких разделов мира атомарно, без удаления машин и поломки активных сеансов.
pub(super) fn apply(game:&mut Game, patch:&Value)->Result<Value,String>{
    let mut value=serde_json::to_value(&game.world).map_err(|e|e.to_string())?;
    let object=patch.as_object().ok_or(tr!("нужен словарь разделов мира", "dictionary of world sections required"))?;
    for (key,v) in object {
        if !["machines","orgs","player","missions","finance","ending","life","gigs","market","base","achievements","scenarios","investigation","fw_down","keyloggers","botnet","botnet_mined","defense","down_until"].contains(&key.as_str()) {return Err(trf!("раздел {key} недоступен для content.apply", "section {key} is not available to content.apply"));}
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
    if next.missions.active>next.missions.missions.len(){return Err(tr!("неверный индекс задания", "invalid mission index").into());}
    if next.machines.len()>4096 || next.machines.is_empty() {return Err(tr!("в мире должно быть 1..4096 устройств", "the world must have 1..4096 devices").into());}
    let ids:BTreeSet<_>=next.machines.iter().map(|m|m.id).collect();
    let orgs:BTreeSet<_>=next.orgs.iter().map(|o|o.id).collect();
    if ids.len()!=next.machines.len() || orgs.len()!=next.orgs.len(){return Err(tr!("повтор идентификаторов", "duplicate ids").into());}
    if game.world.machines.iter().any(|m|!ids.contains(&m.id)) || game.world.orgs.iter().any(|o|!orgs.contains(&o.id)) {return Err(tr!("нельзя удалять существующие устройства и организации; используйте up=false", "existing devices and organizations cannot be removed; use up=false").into());}
    if next.machines[0].id!=game.world.machines[0].id || next.player.handle!=game.world.player.handle || !next.machines[0].users.exists(&next.player.handle) {return Err(tr!("нельзя подменять личность игрока и его компьютер", "the player's identity and computer cannot be replaced").into());}
    let mut hosts=BTreeSet::new();let mut ips=BTreeSet::new();
    for m in &next.machines {
        if !hosts.insert(&m.hostname) || !ips.insert(&m.ip) || m.hostname.is_empty() || m.ip.parse::<std::net::Ipv4Addr>().is_err() {return Err(tr!("неверный или повторный hostname/ip", "invalid or duplicate hostname/ip").into());}
        if m.router.map(|r|!ids.contains(&r)).unwrap_or(false) || m.org.map(|o|!orgs.contains(&o)).unwrap_or(false) {return Err(tr!("устройство ссылается на неизвестный шлюз/организацию", "device refers to an unknown gateway/organization").into());}
    }
    for o in &next.orgs {if !ids.contains(&o.gateway) || o.members.iter().any(|id|!ids.contains(id)){return Err(tr!("неизвестное устройство в организации", "unknown device in organization").into());}}
    for session in game.sessions.values(){for hop in &session.hops {if !next.machine(hop.machine).users.exists(&hop.user){return Err(tr!("нельзя удалять пользователя активного сеанса", "a user of an active session cannot be removed").into());}}}
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
        let name=spec["name"].as_str().ok_or(tr!("нужно имя уязвимости", "vulnerability name required"))?;
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
        "plugin.unlock"=>{let id=string(args,0)?;let item=game.world.plugins.catalogs.get("achievements").and_then(|c|c.get(id)).ok_or(tr!("достижение не найдено", "achievement not found"))?.clone();
            if game.world.achievements.unlocked.insert(id.into()){game.world.achievements.pending=Some(crate::achievements::AchEvent{name:item.name});}Ok(Value::Null)},
        "plugin.random"=>{let lo=args.first().and_then(Value::as_i64).ok_or(tr!("нужна нижняя граница", "lower bound required"))?;let hi=args.get(1).and_then(Value::as_i64).ok_or(tr!("нужна верхняя граница", "upper bound required"))?;if lo< -1_000_000_000 || hi>1_000_000_000 || hi<=lo{return Err(tr!("нужен диапазон -10^9..10^9, lo<hi", "range -10^9..10^9 with lo<hi required").into());}Ok(json!(game.world.rng.range(lo,hi)))},
        "plugin.template"=>{
            let kind=serde_json::from_value(json!(string(args,0)?)).map_err(|e|e.to_string())?;
            let m=crate::net::gen_machine::build(crate::net::gen_machine::MachineSpec{id:0,hostname:"template".into(),ip:"198.18.0.1".into(),lan_ip:None,domain:None,kind,org:None,router:None,org_name:"".into(),care:crate::net::orgs::Care::Normal,x:0.5,y:0.5,people:Vec::new(),peer_ips:Vec::new(),epoch:game.world.now()},&mut game.world.rng);
            serde_json::to_value(m).map_err(|e|e.to_string())
        },
        "plugin.user"=>{let mid=args.first().and_then(Value::as_u64).filter(|n|*n<=u32::MAX as u64).ok_or(tr!("нужен id устройства", "device id required"))? as u32;if !game.world.has_machine(mid){return Err(tr!("нет устройства", "no such device").into());}let name=string(args,1)?;if !super::ident(name){return Err(tr!("неверное имя пользователя", "invalid user name").into());}let password=string(args,2)?;let admin=args.get(3).and_then(Value::as_bool).unwrap_or(false);let m=game.world.machine_mut(mid);m.users.add(name,password,admin);m.fs.add_user_home(name);Ok(Value::Null)},
        "plugin.write"=>{let mid=args.first().and_then(Value::as_u64).filter(|n|*n<=u32::MAX as u64).ok_or(tr!("нужен id устройства", "device id required"))? as u32;if !game.world.has_machine(mid){return Err(tr!("нет устройства", "no such device").into());}let path=string(args,1)?;let text=string(args,2)?;let m=game.world.machine_mut(mid);m.fs.mkdir_all(&crate::vfs::dirname(path),"root").map_err(|e|e.to_string())?;m.fs.write_text(path,text,"root").map_err(|e|e.to_string())?;Ok(Value::Null)},
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
        "plugin.apply"=>apply(game,args.first().ok_or(tr!("нужен словарь изменений", "dictionary of changes required"))?),
        "plugin.device_type"=>{
            let key=namespace(owner,string(args,0)?);
            let template=args.get(1).ok_or(tr!("нужен шаблон устройства", "device template required"))?;
            serde_json::from_value::<Machine>(template.clone()).map_err(|e|e.to_string())?;
            game.world.plugins.device_types.insert(key.clone(),template.clone());Ok(json!(key))
        }
        "plugin.spawn"=>{
            let key=string(args,0)?;
            let mut template=game.world.plugins.device_types.get(key).cloned().ok_or(tr!("тип устройства не зарегистрирован", "device type not registered"))?;
            let overrides=args.get(1).and_then(Value::as_object).ok_or(tr!("нужны свойства нового устройства", "properties of the new device required"))?;
            for (k,v) in overrides {template[k]=v.clone();}
            let id=game.world.machines.iter().map(|m|m.id).max().unwrap_or(0).checked_add(1).ok_or(tr!("нет свободных id", "no free ids"))?;
            template["id"]=json!(id);template["device_type"]=json!(key);
            let mut machines=serde_json::to_value(&game.world.machines).map_err(|e|e.to_string())?;
            machines.as_array_mut().unwrap().push(template);
            apply(game,&json!({"machines":machines}))?;Ok(json!(id))
        }
        "plugin.host_activity"=>{
            let mid=args.first().and_then(Value::as_u64).filter(|id|*id<=u32::MAX as u64).ok_or(tr!("нужен id устройства", "device id required"))? as u32;
            if !game.world.has_machine(mid){return Err(tr!("нет устройства", "no such device").into());}
            if args.len()==1{return serde_json::to_value(game.world.plugins.host_activity.get(&mid).cloned().unwrap_or_default()).map_err(|e|e.to_string());}
            let state:crate::hack::access::Activity=serde_json::from_value(args[1].clone()).map_err(|e|e.to_string())?;
            if state.connections>4096||state.users.len()>128||state.forwarded_ports.len()>128||state.forwarded_ports.contains(&0)||state.users.iter().any(|u|!game.world.machine(mid).users.exists(u)){return Err(tr!("неверная активность устройства", "invalid device activity").into());}
            game.world.plugins.host_activity.insert(mid,state);Ok(Value::Null)
        },
        "plugin.exploit_type"=>super::exploit_types::register(game,owner,args),
        "plugin.vulnerability"=>{
            let key=namespace(owner,string(args,0)?);
            let mut spec=args.get(1).cloned().ok_or(tr!("нужно описание уязвимости", "vulnerability description required"))?;
            if !spec.is_object(){return Err(tr!("нужен словарь уязвимости", "vulnerability dictionary required").into());}
            let access=spec["access"].as_str().ok_or(tr!("нужен access", "access required"))?;
            if !["creds","shell","root","info"].contains(&access) && !access.starts_with("read:/"){return Err(tr!("неизвестный access", "unknown access").into());}
            if spec["local"]==json!(true)&&access!="root"{return Err(tr!("локальное правило privesc должно давать root", "a local privesc rule must grant root").into());}
            if spec["local"]!=json!(true) && spec["service"].as_str().unwrap_or("").is_empty(){return Err(tr!("нужна служба", "service required").into());}
            if let Some(t)=spec.get("device_type").and_then(Value::as_str){if !game.world.plugins.device_types.contains_key(t){return Err(tr!("тип устройства не найден", "device type not found").into());}}
            if let Some(mid)=spec.get("machine").and_then(Value::as_u64){if mid>u32::MAX as u64 || !game.world.has_machine(mid as u32){return Err(tr!("устройство не найдено", "device not found").into());}}
            if let Some(h)=spec.get("handler"){if !h.as_str().map(super::ident).unwrap_or(false){return Err(tr!("неверное имя обработчика уязвимости", "invalid vulnerability handler name").into());}}
            if let Some(exploit) = spec.get("exploit") {
                crate::hack::payload::validate_spec(exploit)?;
                let kind=exploit["kind"].as_str().unwrap();
                if !crate::hack::payload::KINDS.contains(&kind) && !game.world.plugins.exploit_types.contains_key(kind){return Err(tr!("сначала зарегистрируй авторский тип через plugin.exploit_type", "register the custom type with plugin.exploit_type first").into());}
            }
            crate::hack::access::validate_spec(&spec)?;
            spec["name"]=json!(key);
            game.world.plugins.vulnerabilities.insert(key.clone(),spec);refresh(game)?;Ok(json!(key))
        }
        _=>Err(trf!("нет функции {name}", "no function {name}"))
    }
}
