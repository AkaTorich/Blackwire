//! Проверка плагинов на настоящем игровом мире без окон и изменения сохранений игрока.
use blackwire_core::{Game,World,OutKind};
use blackwire_lang::{tr,trf};
fn main(){if let Err(e)=check(){eprintln!("{}",trf!("ОШИБКА: {e}", "ERROR: {e}"));std::process::exit(1);}}
fn check()->Result<(),String>{
 let args:Vec<_>=std::env::args().skip(1).collect();if let Some(code)=args.iter().position(|a|a=="--lang").and_then(|i|args.get(i+1)){blackwire_core::i18n::set(code);}
 let root=args.first().ok_or(tr!("plugin-check папка-плагинов [--command команда] [--ticks N] [--lang ru|en]", "plugin-check plugins-folder [--command command] [--ticks N] [--lang ru|en]"))?;
 let mut game=Game::new(World::new(42,"sdk_test",""));game.load_plugins(std::path::Path::new(root));
 let mut i=1;let mut failed=false;
 while i<args.len(){match args[i].as_str(){
  "--command"=>{i+=1;let cmd=args.get(i).ok_or(tr!("нужна команда", "command required"))?;let sid=game.open_session();let result=game.plugin_command(sid,cmd).ok_or(tr!("команда плагина не найдена", "plugin command not found"))?;for line in result.lines{println!("{}",line.text);failed|=line.kind==OutKind::Error;}game.close_session(sid);},
  "--dump-shell"=>{i+=1;let path=args.get(i).ok_or(tr!("нужен путь справки", "help file path required"))?;std::fs::write(path,blackwire_core::shell::shell_doc()).map_err(|e|e.to_string())?;},
  "--dump-world"=>{i+=1;let path=args.get(i).ok_or(tr!("нужен путь JSON", "JSON path required"))?;std::fs::write(path,game.save_json()).map_err(|e|e.to_string())?;},
  "--ticks"=>{i+=1;let count=args.get(i).ok_or(tr!("нужно число тактов", "tick count required"))?.parse::<u32>().map_err(|e|e.to_string())?;if count>3600{return Err(tr!("не больше 3600 тактов", "at most 3600 ticks").into());}for _ in 0..count{game.tick(1000);}},
  "--lang"=>{i+=1;},
  _=>return Err(trf!("неизвестный параметр {}", "unknown option {}",args[i]))
 }i+=1;}
 println!("{}",serde_json::to_string_pretty(&game.plugin_list()).unwrap());
 if failed || !game.plugins.errors.is_empty() || game.plugins.loaded.values().any(|p|p.error.is_some()){return Err(tr!("проверка не пройдена", "check failed").into());}
 if game.plugins.loaded.is_empty(){return Err(tr!("не найдено ни одного плагина", "no plugins found").into());}
 println!("{}",trf!("Проверка: OK. Устройств в тестовом мире: {}", "Check: OK. Devices in the test world: {}",game.world.machines.len()));Ok(())
}
