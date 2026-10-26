use super::*;
#[derive(Clone,Debug,Serialize,Deserialize)]
pub struct Quest {pub title:String,pub objective:String,pub reward:i64,#[serde(default)]pub done:bool}
#[derive(Clone,Debug,Serialize,Deserialize)]
pub struct Timer {pub due:u64,pub event:String,pub payload:Value,pub repeat:u64}
fn text<'a>(args:&'a [Value],i:usize)->Result<&'a str,String>{args.get(i).and_then(Value::as_str).ok_or_else(||format!("нужна строка в аргументе {}",i+1))}
fn mid(game:&Game,args:&[Value])->Result<u32,String>{let id=args.first().and_then(Value::as_u64).filter(|i|*i<=u32::MAX as u64).ok_or("нужен id устройства")? as u32;if !game.world.has_machine(id){return Err("устройство не найдено".into());}Ok(id)}
pub fn call(game:&mut Game,owner:&str,name:&str,args:&[Value])->Option<Result<Value,String>>{
 let result=(||->Result<Value,String>{match name{
 "plugin.machine"=>Ok(json!(game.world.machine(mid(game,args)?))),
 "plugin.update_machine"=>{let id=mid(game,args)?;let mut machines=json!(game.world.machines);let m=machines.as_array_mut().unwrap().iter_mut().find(|m|m["id"]==json!(id)).unwrap();let patch=args.get(1).and_then(Value::as_object).ok_or("нужны свойства устройства")?;for(k,v)in patch{if k=="id"{return Err("id устройства неизменяем".into());}m[k]=v.clone();}super::content::apply(game,&json!({"machines":machines}))},
 "plugin.service"=>{let id=mid(game,args)?;let service:crate::net::services::Service=serde_json::from_value(args.get(1).cloned().ok_or("нужна служба")?).map_err(|e|e.to_string())?;if service.port==0 || service.name.is_empty(){return Err("нужны имя и порт 1..65535".into());}let m=game.world.machine_mut(id);m.services.retain(|s|s.port!=service.port);m.services.push(service);super::content::refresh(game)?;Ok(Value::Null)},
 "plugin.page"=>{let id=mid(game,args)?;let port=args.get(1).and_then(Value::as_u64).filter(|n|*n>0&&*n<=65535).ok_or("нужен порт")? as u16;let page:crate::net::services::WebPage=serde_json::from_value(args.get(2).cloned().ok_or("нужна страница")?).map_err(|e|e.to_string())?;if !page.path.starts_with('/'){return Err("путь страницы должен начинаться с /".into());}let service=game.world.machine_mut(id).services.iter_mut().find(|s|s.port==port).ok_or("служба не найдена")?;service.pages.retain(|p|p.path!=page.path);service.pages.push(page);Ok(Value::Null)},
 "plugin.mail"=>{let id=mid(game,args)?;let spec=args.get(1).ok_or("нужно письмо")?;let from=spec["from"].as_str().ok_or("нужен from")?;let subject=spec["subject"].as_str().ok_or("нужен subject")?;let body=spec["body"].as_str().ok_or("нужен body")?;let to=spec["to"].as_str().map(String::from).unwrap_or_else(||game.mail_address());let time=game.world.now();game.world.machine_mut(id).push_mail(crate::net::machine::MailMessage{id:0,from:from.into(),to,subject:subject.into(),body:body.into(),time,read:false,folder:crate::net::machine::MailFolder::Inbox});Ok(Value::Null)},
 "plugin.news"=>{let time=game.world.now();game.world.life.feed.push(crate::life::FeedItem{time,text:text(args,0)?.into()});let len=game.world.life.feed.len();if len>100{game.world.life.feed.drain(..len-100);}Ok(Value::Null)},
 "plugin.log"=>{let id=mid(game,args)?;let kind=serde_json::from_value(args.get(1).cloned().ok_or("нужен вид журнала")?).map_err(|e|e.to_string())?;game.world.log(id,kind,text(args,2)?);Ok(Value::Null)},
 "plugin.org"=>{let mut spec=args.first().cloned().ok_or("нужна организация")?;let id=game.world.orgs.iter().map(|o|o.id).max().unwrap_or(0).checked_add(1).ok_or("нет свободных id")?;spec["id"]=json!(id);let org:crate::net::orgs::Org=serde_json::from_value(spec.clone()).map_err(|e|e.to_string())?;let mut orgs=json!(game.world.orgs);orgs.as_array_mut().unwrap().push(spec);let mut machines=json!(game.world.machines);for m in machines.as_array_mut().unwrap(){if org.members.contains(&(m["id"].as_u64().unwrap() as u32)){m["org"]=json!(id);}}super::content::apply(game,&json!({"orgs":orgs,"machines":machines}))?;Ok(json!(id))},
 "plugin.gig"=>{let mut spec=args.first().cloned().ok_or("нужно задание")?;let id=game.world.gigs.next_id.checked_add(1).ok_or("переполнение id задания")?;spec["id"]=json!(id);let gig:crate::gigs::Gig=serde_json::from_value(spec).map_err(|e|e.to_string())?;if gig.reward<0||gig.reward>1_000_000_000{return Err("неверная награда".into());}let mut state=game.world.gigs.clone();state.next_id=id;state.gigs.push(gig);super::content::apply(game,&json!({"gigs":state}))?;Ok(json!(id))},
 "plugin.scenario"=>{let scenario:crate::scenarios::Scenario=serde_json::from_value(args.first().cloned().ok_or("нужен сценарий")?).map_err(|e|e.to_string())?;if scenario.reward<0||scenario.reward>1_000_000_000{return Err("неверная награда".into());}crate::scenarios::start(&mut game.world,scenario);Ok(Value::Null)},
 "plugin.quest"=>{let key=text(args,0)?;if !super::ident(key){return Err("неверный id задания".into());}let id=format!("{owner}:{key}");let mut quest:Quest=serde_json::from_value(args.get(1).cloned().ok_or("нужно описание задания")?).map_err(|e|e.to_string())?;if quest.title.is_empty()||quest.reward<0||quest.reward>1_000_000_000{return Err("неверное задание".into());}quest.done=game.world.plugins.quests.get(&id).map(|q|q.done).unwrap_or(false);game.world.plugins.quests.insert(id.clone(),quest);Ok(json!(id))},
 "plugin.quest_complete"=>{let id=text(args,0)?;if !id.starts_with(&format!("{owner}:")){return Err("это задание другого плагина".into());}let quest=game.world.plugins.quests.get_mut(id).ok_or("задание не найдено")?;if quest.done{return Ok(json!(false));}let money=game.world.player.money.checked_add(quest.reward).ok_or("переполнение счёта")?;quest.done=true;let title=quest.title.clone();game.world.player.money=money;game.notify("Задание выполнено",&title);Ok(json!(true))},
 "plugin.timer"=>{let key=text(args,0)?;if !super::ident(key){return Err("неверное имя таймера".into());}let delay=args.get(1).and_then(Value::as_u64).filter(|n|*n>0&&*n<=31_536_000).ok_or("нужна задержка 1..31536000 секунд")?;let event=text(args,2)?;if !event.starts_with("custom."){return Err("событие таймера должно начинаться с custom.".into());}let repeat=args.get(4).and_then(Value::as_u64).unwrap_or(0);if repeat>31_536_000{return Err("слишком большой период".into());}let due=game.world.now()+delay;game.world.plugins.timers.entry(owner.into()).or_default().insert(key.into(),Timer{due,event:event.into(),payload:args.get(3).cloned().unwrap_or(Value::Null),repeat});Ok(Value::Null)},
 "plugin.cancel_timer"=>{if let Some(t)=game.world.plugins.timers.get_mut(owner){t.remove(text(args,0)?);}Ok(Value::Null)},
 "plugin.window"=>{let key=text(args,0)?;if !super::ident(key){return Err("неверное имя окна".into());}let spec=args.get(1).cloned().ok_or("нужно описание окна")?;if !spec.is_object()||spec["title"].as_str().unwrap_or("").is_empty(){return Err("нужен title окна".into());}if serde_json::to_vec(&spec).unwrap().len()>65536{return Err("окно больше 64 КБ".into());}let id=format!("{owner}:{key}");game.world.plugins.windows.insert(id.clone(),spec);Ok(json!(id))},
 "plugin.open"=>{let id=text(args,0)?;if !id.starts_with(&format!("{owner}:"))||!game.world.plugins.windows.contains_key(id){return Err("окно плагина не найдено".into());}game.pending_ui.push(crate::UiAction::OpenApp{app:"plugin_panel".into(),arg:Some(id.into())});Ok(Value::Null)},
 _=>return Err("__unknown__".into())
 }})();
 if result.as_ref().err().map(String::as_str)==Some("__unknown__"){None}else{Some(result)}
}
impl Game {
 pub fn plugin_timers(&mut self){
    let now=self.world.now();let mut due=Vec::new();
    for(owner,timers)in &self.world.plugins.timers{if self.world.plugins.disabled.contains(owner){continue;}for(key,timer)in timers{if timer.due<=now{due.push((owner.clone(),key.clone(),timer.clone()));if due.len()>=64{break;}}}if due.len()>=64{break;}}
    for(owner,key,timer)in due{
        if let Some(timers)=self.world.plugins.timers.get_mut(&owner){if timer.repeat==0{timers.remove(&key);}else if let Some(t)=timers.get_mut(&key){t.due=now+timer.repeat;}}
        if let Some(p)=self.plugins.loaded.get(&owner).filter(|p|p.error.is_none()).cloned(){if let Some(handler)=p.manifest.events.get(&timer.event){self.run_plugin(&p,handler,json!({"timer":key,"data":timer.payload}),None);}}
    }
 }
 pub fn plugin_ui(&mut self,id:&str,action:&str,values:Value)->Result<Vec<OutLine>,String>{
    let(owner,_)=id.split_once(':').ok_or("неверный id окна")?;
    if self.world.plugins.disabled.contains(owner){return Err("плагин выключен".into());}
    let p=self.plugins.loaded.get(owner).filter(|p|p.error.is_none()).cloned().ok_or("плагин не работает")?;
    let spec=self.world.plugins.windows.get(id).ok_or("окно не найдено")?;
    if !spec["buttons"].as_array().map(|b|b.iter().any(|b|b["id"]==json!(action))).unwrap_or(false){return Err("кнопка не найдена".into());}
    let handler=p.manifest.events.get("on_ui").ok_or("нет обработчика on_ui")?;
    Ok(self.run_plugin(&p,handler,json!({"window":id,"action":action,"values":values}),None))
 }
}
