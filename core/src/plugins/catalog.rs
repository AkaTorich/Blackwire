//! Расширяемые каталоги; встроенные записи остаются в прежних таблицах.
use super::*;
use crate::World;
#[derive(Clone,Debug,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CatalogItem {
    pub name:String, pub desc:String,
    #[serde(default)] pub price:i64,
    #[serde(default)] pub file:String,
    #[serde(default)] pub code:String,
}
pub fn entries(world:&World,kind:&str)->Vec<(String,CatalogItem)>{world.plugins.catalogs.get(kind).map(|m|m.iter().map(|(id,v)|(id.clone(),v.clone())).collect()).unwrap_or_default()}
pub fn buy(world:&mut World,kind:&str,id:&str)->Option<Result<String,String>>{
    let item=world.plugins.catalogs.get(kind)?.get(id)?.clone();
    Some((||{
        let owned=if kind=="market"{world.market.bought.contains(id)}else{world.player.upgrades.contains(id)};
        if owned{return Err("уже куплено".into());}
        if world.player.money<item.price{return Err("недостаточно денег".into());}
        if kind=="market"{
            let home=world.machines[0].users.home_of(&world.player.handle);
            let dir=format!("{home}/scripts");let user=world.player.handle.clone();
            world.machines[0].fs.mkdir_all(&dir,&user).map_err(|e|e.to_string())?;
            world.machines[0].fs.write_text(&format!("{dir}/{}",item.file),&item.code,&user).map_err(|e|e.to_string())?;
            world.market.bought.insert(id.into());
        }else{world.player.upgrades.insert(id.into());}
        world.player.money-=item.price;Ok(format!("Куплено: {} ({} ₵)",item.name,item.price))
    })())
}
pub fn html(world:&World,kind:&str)->String{
    fn escape(s:&str)->String{s.replace('&',"&amp;").replace('<',"&lt;").replace('>',"&gt;").replace('"',"&quot;")}
    entries(world,kind).iter().map(|(id,i)|format!("<li><b>{}</b> ({}) — {} ₵ · {}</li>",escape(&i.name),escape(id),i.price,escape(&i.desc))).collect::<Vec<_>>().join("")
}
pub fn register(game:&mut Game,owner:&str,args:&[Value])->Result<Value,String>{
    let kind=args.first().and_then(Value::as_str).ok_or("нужен каталог")?;
    if !["market","shop","achievements"].contains(&kind){return Err("каталоги: market, shop, achievements".into());}
    let key=args.get(1).and_then(Value::as_str).ok_or("нужен id")?;
    if !super::ident(key){return Err("неверный id записи".into());}
    let item:CatalogItem=serde_json::from_value(args.get(2).cloned().ok_or("нужна запись")?).map_err(|e|e.to_string())?;
    if item.name.is_empty()||item.price<0||item.price>1_000_000_000{return Err("неверное имя или цена".into());}
    if kind=="market"{
        if item.file.contains(['/', '\\', ':'])||!item.file.ends_with(".adr"){return Err("file: нужно имя .adr без пути".into());}
        blackwire_lang::parse(&item.code).map_err(|e|e.to_string())?;
    }
    let id=format!("{owner}:{key}");game.world.plugins.catalogs.entry(kind.into()).or_default().insert(id.clone(),item);Ok(json!(id))
}
