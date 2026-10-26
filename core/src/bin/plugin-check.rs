//! Проверка плагинов на настоящем игровом мире без окон и изменения сохранений игрока.
use blackwire_core::{Game,World,OutKind};
fn main(){if let Err(e)=check(){eprintln!("ОШИБКА: {e}");std::process::exit(1);}}
fn check()->Result<(),String>{
 let args:Vec<_>=std::env::args().skip(1).collect();let root=args.first().ok_or("plugin-check папка-плагинов [--command команда] [--ticks N]")?;
 let mut game=Game::new(World::new(42,"sdk_test",""));game.load_plugins(std::path::Path::new(root));
 let mut i=1;let mut failed=false;
 while i<args.len(){match args[i].as_str(){
  "--command"=>{i+=1;let cmd=args.get(i).ok_or("нужна команда")?;let sid=game.open_session();let result=game.plugin_command(sid,cmd).ok_or("команда плагина не найдена")?;for line in result.lines{println!("{}",line.text);failed|=line.kind==OutKind::Error;}game.close_session(sid);},
  "--dump-shell"=>{i+=1;let path=args.get(i).ok_or("нужен путь справки")?;std::fs::write(path,blackwire_core::shell::shell_doc()).map_err(|e|e.to_string())?;},
  "--dump-world"=>{i+=1;let path=args.get(i).ok_or("нужен путь JSON")?;std::fs::write(path,game.save_json()).map_err(|e|e.to_string())?;},
  "--ticks"=>{i+=1;let count=args.get(i).ok_or("нужно число тактов")?.parse::<u32>().map_err(|e|e.to_string())?;if count>3600{return Err("не больше 3600 тактов".into());}for _ in 0..count{game.tick(1000);}},
  _=>return Err(format!("неизвестный параметр {}",args[i]))
 }i+=1;}
 println!("{}",serde_json::to_string_pretty(&game.plugin_list()).unwrap());
 if failed || !game.plugins.errors.is_empty() || game.plugins.loaded.values().any(|p|p.error.is_some()){return Err("проверка не пройдена".into());}
 if game.plugins.loaded.is_empty(){return Err("не найдено ни одного плагина".into());}
 println!("Проверка: OK. Устройств в тестовом мире: {}",game.world.machines.len());Ok(())
}
