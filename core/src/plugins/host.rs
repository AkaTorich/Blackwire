use super::*;
use blackwire_lang::{Host, LangError, Value as A};
use std::time::{Duration, Instant};
pub struct PluginHost<'a> {
    game: &'a mut Game, plugin: &'a Plugin, session: u32, event: Value,
    deadline: Instant, pub lines: Vec<OutLine>,
}
impl<'a> PluginHost<'a> {
    pub fn new(game: &'a mut Game, plugin: &'a Plugin, session: u32, event: Value) -> Self {
        Self {game,plugin,session,event,deadline:Instant::now()+Duration::from_millis(500),lines:Vec::new()}
    }
    fn invoke(&mut self, name: &str, args: Vec<Value>) -> Result<Value,String> {
        let id=&self.plugin.manifest.id;
        if name.starts_with("plugin.") {
            match name {
                "plugin.network" => return Ok(self.game.plugins.network.clone()),
                "plugin.net_send" => {
                    if !self.plugin.manifest.permissions.contains("network"){return Err(tr!("нужно разрешение network", "network permission required").into());}
                    if self.game.plugins.network.is_null() || self.game.plugins.network["role"]==json!("None"){return Err(tr!("сетевая партия не подключена", "multiplayer session not connected").into());}
                    let channel=args.first().and_then(Value::as_str).ok_or(tr!("нужен канал", "channel required"))?;if channel.len()>64{return Err(tr!("канал длиннее 64 символов", "channel longer than 64 characters").into());}
                    let packet=json!({"t":"plugin","plugin":id,"channel":channel,"data":args.get(1).cloned().unwrap_or(Value::Null)});
                    if packet.to_string().len()>16384 || self.game.plugins.outbox.len()>=32{return Err(tr!("очередь сети полна или сообщение больше 16 КБ", "network queue full or message larger than 16 KB").into());}
                    self.game.plugins.outbox.push(packet);return Ok(Value::Null);
                },
                "plugin.event" => return Ok(self.event.clone()),
                "plugin.asset" => {let path=args.first().and_then(Value::as_str).ok_or(tr!("нужно имя ресурса", "resource name required"))?;return self.plugin.assets.get(path).cloned().map(|s|json!(s)).ok_or_else(||tr!("ресурс не объявлен в assets", "resource not declared in assets").into());},
                "plugin.id" => return Ok(json!(id)),
                "plugin.lang" => return Ok(json!(crate::i18n::code())),
                "plugin.get" => return Ok(self.game.world.plugins.data.get(id).and_then(|d|d.get(args.first()?.as_str()?)).cloned().unwrap_or_else(||args.get(1).cloned().unwrap_or(Value::Null))),
                "plugin.set" => {
                    let key=args.first().and_then(Value::as_str).ok_or(tr!("нужен ключ", "key required"))?;
                    if key.len()>128 {return Err(tr!("ключ длиннее 128 символов", "key longer than 128 characters").into());}
                    let mut data=self.game.world.plugins.data.get(id).cloned().unwrap_or_default();
                    data.insert(key.into(),args.get(1).cloned().unwrap_or(Value::Null));
                    if serde_json::to_vec(&data).map_err(|e|e.to_string())?.len()>65536 {return Err(tr!("данные плагина больше 64 КБ", "plugin data larger than 64 KB").into());}
                    self.game.world.plugins.data.insert(id.clone(),data);return Ok(Value::Null);
                }
                _=>{}
            }
            if !self.plugin.manifest.permissions.contains("content") {return Err(tr!("нужно разрешение content", "content permission required").into());}
            return content::call(self.game,id,name,&args);
        }
        let module=name.split('.').next().unwrap_or("");
        if !self.plugin.manifest.permissions.contains(module) {return Err(trf!("нет разрешения {module}", "no {module} permission"));}
        // Нельзя блокировать общий игровой поток, запускать вложенные скрипты и процессы.
        if ["sys.kill"].contains(&name) {return Err(tr!("эта функция недоступна обработчику плагина", "this function is not available to a plugin handler").into());}
        crate::script_host::call(self.game,self.session,0,name,args)
    }
}
impl Host for PluginHost<'_> {
    fn print(&mut self,text:&str){if self.lines.len()<100 {self.lines.push(OutLine::normal(text.chars().take(4096).collect::<String>()));}}
    fn input(&mut self,_:&str)->Result<String,LangError>{Err(LangError::runtime(0,tr!("input недоступен в обработчике плагина", "input is not available in a plugin handler")))}
    fn call(&mut self,name:&str,args:Vec<A>)->Result<A,LangError>{
        self.invoke(name,args.iter().map(A::to_json).collect()).map(|v|A::from_json(&v)).map_err(|e|LangError::runtime(0,e))
    }
    fn should_stop(&self)->bool{Instant::now()>=self.deadline}
    fn sleep(&mut self,_:u64){self.deadline=Instant::now();}
}
