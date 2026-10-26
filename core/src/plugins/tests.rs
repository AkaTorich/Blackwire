use super::*;
fn game()->Game{Game::new(crate::World::new(42,"sdk_test",""))}
fn install(g:&mut Game,id:&str,source:&str,events:&[(&str,&str)],commands:&[(&str,&str)],permissions:&[&str]){
    let manifest=Manifest{id:id.into(),name:id.into(),version:"1.0".into(),api:1,entry:"main.adr".into(),libraries:Vec::new(),assets:Vec::new(),requires:Vec::new(),permissions:permissions.iter().map(|s|s.to_string()).collect(),events:events.iter().map(|(a,b)|(a.to_string(),b.to_string())).collect(),commands:commands.iter().map(|(a,b)|(a.to_string(),b.to_string())).collect()};
    manifest.validate().unwrap();g.plugins.loaded.insert(id.into(),Plugin{manifest,source:source.into(),assets:Default::default(),error:None});
}
#[test]
fn device_types_and_per_device_vulnerabilities_work_in_real_game(){
 let mut g=game();let manifest:Manifest=serde_json::from_str(include_str!("../../../sdk/examples/device_lab/plugin.json")).unwrap();
 g.plugins.loaded.insert(manifest.id.clone(),Plugin{manifest,source:include_str!("../../../sdk/examples/device_lab/main.adr").into(),assets:Default::default(),error:None});
 g.plugin_event("on_load",json!({}));assert!(g.plugins.loaded["device_lab"].error.is_none(),"{:?}",g.plugin_list());
 let s=g.open_session();let r=g.exec(s,"lab_create");assert!(!r.lines.iter().any(|l|l.kind==crate::OutKind::Error),"{:?}",r);
 let m=g.world.machines.last().unwrap();let id=m.id;let ip=m.ip.clone();
 assert_eq!(m.device_type.as_deref(),Some("device_lab:smart_camera"));
 assert_eq!(crate::hack::remote_vulns(m).len(),2);assert!(!crate::hack::local_vulns(m).is_empty());
 assert!(!crate::hack::remote_vulns(&g.world.machines[0]).iter().any(|v|v.name.starts_with("device_lab:")));
 let probe=g.exec(s,&format!("probe {ip}"));assert!(probe.lines.iter().any(|l|l.text.contains("device_lab:camera_access")),"{:?}",probe);
 let result=crate::hack::exploit(&mut g.world,g.sessions.get_mut(&s).unwrap(),id,Some("lab_control")).unwrap();
 assert!(matches!(result.outcome,crate::hack::ExploitOutcome::Shell{..}));
 let elevated=crate::hack::privesc(&mut g.world,g.sessions.get_mut(&s).unwrap()).unwrap();assert!(elevated.ok);
 let saved=g.save_json();let restored=Game::from_json(&saved).unwrap();assert_eq!(restored.world.machines.last().unwrap().device_type.as_deref(),Some("device_lab:smart_camera"));
 assert_eq!(restored.world.plugins.data["device_lab"]["created"],json!(1));
}
#[test]
fn runtime_errors_rollback_and_other_plugins_continue(){
 let mut g=game();install(&mut g,"bad","def go(e):\n    plugin.set(\"bad\", 1)\n    raise \"oops\"\n",&[("on_tick","go")],&[],&[]);
 install(&mut g,"good","def go(e):\n    plugin.set(\"count\", plugin.get(\"count\", 0)+1)\n",&[("on_tick","go")],&[],&[]);
 g.tick(1000);assert!(g.plugins.loaded["bad"].error.is_some());assert!(!g.world.plugins.data.contains_key("bad"));assert_eq!(g.world.plugins.data["good"]["count"],json!(1));
 g.tick(1000);assert_eq!(g.world.plugins.data["good"]["count"],json!(2));assert_eq!(g.plugins.sessions.len(),1);
}
#[test]
fn loop_budget_cannot_be_caught(){
 let mut g=game();install(&mut g,"looping","def go(e):\n    while true:\n        try:\n            while true:\n                pass\n        except:\n            pass\n",&[("on_load","go")],&[],&[]);
 let start=std::time::Instant::now();g.plugin_event("on_load",json!({}));assert!(g.plugins.loaded["looping"].error.is_some());assert!(start.elapsed().as_secs()<3);
}
#[test]
fn permission_denial_and_state_isolation(){
 let mut g=game();install(&mut g,"one","def go(e):\n    plugin.world()\n",&[("on_load","go")],&[],&[]);
 g.plugin_event("on_load",json!({}));assert!(g.plugins.loaded["one"].error.as_ref().unwrap().contains("content"));
 install(&mut g,"two","def go(e):\n    plugin.set(\"value\", plugin.get(\"value\", 7))\n",&[("on_tick","go")],&[],&[]);g.tick(1000);assert_eq!(g.world.plugins.data["two"]["value"],json!(7));
}
#[test]
fn disabled_plugins_preserve_save_data(){
 let mut g=game();install(&mut g,"counter","def go(e):\n    plugin.set(\"count\", plugin.get(\"count\",0)+1)\n",&[("on_tick","go")],&[],&[]);
 g.tick(1000);g.plugin_enable("counter",false).unwrap();g.tick(1000);assert_eq!(g.world.plugins.data["counter"]["count"],json!(1));
 let restored=Game::from_json(&g.save_json()).unwrap();assert!(restored.world.plugins.disabled.contains("counter"));
}
#[test]
fn invalid_world_change_is_atomic(){
 let mut g=game();let before=g.world.machines.len();
 assert!(content::call(&mut g,"test","plugin.apply",&[json!({"machines":[]})]).is_err());assert_eq!(g.world.machines.len(),before);
 let mut machines=serde_json::to_value(&g.world.machines).unwrap();machines[1]["ip"]=machines[0]["ip"].clone();
 assert!(content::call(&mut g,"test","plugin.apply",&[json!({"machines":machines})]).is_err());assert_ne!(g.world.machines[1].ip,g.world.machines[0].ip);
}
#[test]
fn custom_market_shop_and_achievements_are_consumed_by_game(){
 let mut g=game();g.world.player.money=1000;
 content::call(&mut g,"test","plugin.catalog",&[json!("market"),json!("tool"),json!({"name":"Tool","desc":"demo","price":10,"file":"test_plugin.adr","code":"print(42)\n"})]).unwrap();
 crate::market::buy(&mut g.world,"test:tool").unwrap();assert_eq!(g.world.player.money,990);assert!(g.world.machines[0].fs.read_text("/home/sdk_test/scripts/test_plugin.adr").unwrap().contains("42"));
 assert!(crate::market::market_html(&g.world).contains("test:tool"));
 content::call(&mut g,"test","plugin.catalog",&[json!("shop"),json!("upgrade"),json!({"name":"Upgrade","desc":"demo","price":20})]).unwrap();crate::money::buy(&mut g.world,"test:upgrade").unwrap();assert!(g.world.player.upgrades.contains("test:upgrade"));assert_eq!(g.world.player.money,970);
 content::call(&mut g,"test","plugin.catalog",&[json!("achievements"),json!("win"),json!({"name":"Winner","desc":"demo"})]).unwrap();content::call(&mut g,"test","plugin.unlock",&[json!("test:win")]).unwrap();assert!(g.world.achievements.unlocked.contains("test:win"));
}
#[test]
fn loader_checks_api_duplicates_and_entry_paths(){
 let dir=std::env::temp_dir().join(format!("blackwire-sdk-{}",std::process::id()));std::fs::create_dir_all(dir.join("bad")).unwrap();
 std::fs::write(dir.join("outside.adr"),"pass\n").unwrap();
 std::fs::write(dir.join("bad/plugin.json"),r#"{"id":"bad","name":"bad","version":"1","api":1,"entry":"../outside.adr"}"#).unwrap();
 let mut g=game();g.load_plugins(&dir);assert!(g.plugins.loaded.is_empty());assert!(g.plugins.errors[0].contains("entry"));
 // Only test-owned directory, no recursive removal of external data.
 std::fs::remove_file(dir.join("bad/plugin.json")).unwrap();std::fs::remove_file(dir.join("outside.adr")).unwrap();std::fs::remove_dir(dir.join("bad")).unwrap();std::fs::remove_dir(dir).unwrap();
}
fn install_district(g:&mut Game){let manifest:Manifest=serde_json::from_str(include_str!("../../../sdk/examples/district/plugin.json")).unwrap();g.plugins.loaded.insert(manifest.id.clone(),Plugin{manifest,source:include_str!("../../../sdk/examples/district/main.adr").into(),assets:Default::default(),error:None});g.plugin_event("on_load",json!({}));}
#[test]
fn full_district_integrates_topology_web_mail_missions_economy_and_vuln_callback(){
 let mut g=game();install_district(&mut g);let sid=g.open_session();let n=g.world.machines.len();
 let out=g.exec(sid,"district_create");assert!(!out.lines.iter().any(|l|l.kind==crate::OutKind::Error),"{:?}",out);
 assert_eq!(g.world.machines.len(),n+2);assert!(g.world.orgs.iter().any(|o|o.domain=="line1.city"));
 let page=g.http("http://198.19.1.1/");assert_eq!(page.status,200);assert!(page.body.contains("Линия-7"));assert!(g.world.machines[0].mail.iter().any(|m|m.subject=="Район 1: задание"));
 g.exec(sid,"exploit -s line_gate 198.19.1.1");g.exec(sid,"exploit -s line_archive 10.250.1.2");
 assert_eq!(g.sessions[&sid].user(),"root");let server=g.world.find_by_hostname("line-archive-1").unwrap();assert!(g.world.machine(server).fs.read_text("/srv/line/access.txt").unwrap().contains("зарегистрирован"));
 let before=g.world.player.money;g.tick(1000);assert_eq!(g.world.player.money,before+150);assert!(g.world.plugins.quests["district:archive_1"].done);assert!(g.world.achievements.unlocked.contains("district:archivist"));
 g.tick(1000);assert_eq!(g.world.player.money,before+150); // reward cannot be farmed
 let ram=g.world.machines[0].ram_mb;let result=g.exec(sid,"buy district:memory");assert!(!result.lines.iter().any(|l|l.kind==crate::OutKind::Error));assert_eq!(g.world.machines[0].ram_mb,ram+1024);
 assert_eq!(g.world.player.money,before+100);
 let saved=g.save_json();let mut restored=Game::from_json(&saved).unwrap();install_district(&mut restored);assert!(restored.world.plugins.quests["district:archive_1"].done);assert_eq!(restored.world.machines.len(),n+2);assert_eq!(restored.world.machines[0].ram_mb,ram+1024);
 for _ in 0..60{restored.tick(1000);}assert!(restored.world.life.feed.iter().any(|f|f.text.contains("продолжают принимать")));
}
#[test]
fn declarative_window_buttons_invoke_adder_and_validate_action(){
 let mut g=game();install_district(&mut g);let n=g.world.machines.len();let result=g.plugin_ui("district:control","create",json!({})).unwrap();assert!(!result.iter().any(|l|l.kind==crate::OutKind::Error));assert_eq!(g.world.machines.len(),n+2);
 assert!(g.plugin_ui("district:control","not_a_button",json!({})).is_err());g.plugin_enable("district",false).unwrap();assert!(g.plugin_ui("district:control","create",json!({})).is_err());
}
#[test]
fn invalid_cross_references_and_router_cycles_are_rejected(){
 let mut g=game();let mut p=json!(g.world.player);p["known_machines"]=json!([4294967295u32]);assert!(content::call(&mut g,"test","plugin.apply",&[json!({"player":p})]).is_err());
 let mut m=json!(g.world.machines);let a=m[1]["id"].clone();let b=m[2]["id"].clone();m[1]["router"]=b;m[2]["router"]=a;assert!(content::call(&mut g,"test","plugin.apply",&[json!({"machines":m})]).is_err());
}
#[test]
fn network_packets_are_scoped_to_installed_opted_in_plugin(){
 let mut g=game();install(&mut g,"wire","def got(e):\n    plugin.set(\"received\", e[\"data\"])\n",&[("on_network","got")],&[],&["network"]);
 g.plugin_network_receive(json!({"plugin":"wire","channel":"test","from":"local","data":{"answer":42}}));assert_eq!(g.world.plugins.data["wire"]["received"]["answer"],json!(42));
 g.plugin_enable("wire",false).unwrap();g.plugin_network_receive(json!({"plugin":"wire","data":0}));assert_eq!(g.world.plugins.data["wire"]["received"]["answer"],json!(42));
}
#[test]
fn runtime_exit_nonzero_rolls_back_pending_world_changes(){let mut g=game();install(&mut g,"exitbad","def go(e):\n    plugin.set(\"key\", 1)\n    exit(2)\n",&[("on_load","go")],&[],&[]);g.plugin_event("on_load",json!({}));assert!(!g.world.plugins.data.contains_key("exitbad"));assert!(g.plugins.loaded["exitbad"].error.is_some());}
#[test]
fn timer_repeats_and_cancellation_survive_roundtrip(){
 let mut g=game();install(&mut g,"clock","def load(e):\n    plugin.timer(\"test\", 2, \"custom.clock\", {\"v\":3}, 2)\ndef count(e):\n    plugin.set(\"n\",plugin.get(\"n\",0)+e[\"data\"][\"v\"])\n",&[("on_load","load"),("custom.clock","count")],&[],&["content"]);
 g.plugin_event("on_load",json!({}));g.tick(1000);assert!(!g.world.plugins.data.contains_key("clock"));g.tick(1000);assert_eq!(g.world.plugins.data["clock"]["n"],json!(3));g.tick(2000);assert_eq!(g.world.plugins.data["clock"]["n"],json!(6));content::call(&mut g,"clock","plugin.cancel_timer",&[json!("test")]).unwrap();g.tick(5000);assert_eq!(g.world.plugins.data["clock"]["n"],json!(6));
}
#[test]
fn package_assets_libraries_and_dependency_order_are_real(){
 let root=std::env::temp_dir().join(format!("bw-package-{}",std::process::id()));
 for name in ["base","consumer"]{std::fs::create_dir_all(root.join(name)).unwrap();}
 std::fs::write(root.join("base/plugin.json"),r#"{"id":"base","name":"base","version":"1","api":1,"entry":"main.adr","permissions":["content"],"events":{"on_load":"load"}}"#).unwrap();
 std::fs::write(root.join("base/main.adr"),"def load(e):\n    plugin.device_type(\"sample\", plugin.template(\"camera\"))\n").unwrap();
 std::fs::write(root.join("consumer/plugin.json"),r#"{"id":"consumer","name":"consumer","version":"1","api":1,"entry":"main.adr","libraries":["lib.adr"],"assets":["data.json"],"requires":["base"],"events":{"on_load":"load"}}"#).unwrap();
 std::fs::write(root.join("consumer/lib.adr"),"def twice(x):\n    return x*2\n").unwrap();
 std::fs::write(root.join("consumer/data.json"),"{\"n\":21}").unwrap();
 std::fs::write(root.join("consumer/main.adr"),"def load(e):\n    plugin.set(\"answer\", twice(json.loads(plugin.asset(\"data.json\"))[\"n\"]))\n").unwrap();
 let mut g=game();g.load_plugins(&root);assert!(g.plugins.errors.is_empty(),"{:?}",g.plugins.errors);assert_eq!(g.plugins.order,vec!["base","consumer"]);assert_eq!(g.world.plugins.data["consumer"]["answer"],json!(42));
 g.plugin_enable("base",false).unwrap();assert!(g.plugin_enable("consumer",true).is_err());
 for file in ["base/plugin.json","base/main.adr","consumer/plugin.json","consumer/lib.adr","consumer/data.json","consumer/main.adr"]{std::fs::remove_file(root.join(file)).unwrap();}std::fs::remove_dir(root.join("base")).unwrap();std::fs::remove_dir(root.join("consumer")).unwrap();std::fs::remove_dir(root).unwrap();
}
#[test]
fn shell_scripts_run_in_persistent_plugin_session_and_stop_on_disable(){
 let mut g=game();install(&mut g,"daemon","def load(e):\n    fs.write(\"/home/sdk_test/background.adr\", \"print(42)\\n\")\n    shell.exec(\"adder /home/sdk_test/background.adr\")\n",&[("on_load","load")],&[],&["shell","fs"]);
 g.plugin_event("on_load",json!({}));assert!(g.plugins.loaded["daemon"].error.is_none());assert_eq!(g.pending_scripts.len(),1);let sid=g.pending_scripts[0].session;assert!(g.sessions.contains_key(&sid));g.plugin_enable("daemon",false).unwrap();assert!(!g.sessions.contains_key(&sid));
}

#[test]
fn lightweight_world_overview_preserves_metadata_without_file_systems() {
 let mut g=game();
 let view=content::call(&mut g,"test","plugin.world",&[json!("overview")]).unwrap();
 assert_eq!(view["machines"].as_array().unwrap().len(),g.world.machines.len());
 assert_eq!(view["player"]["handle"],json!(g.world.player.handle));
 assert_eq!(view["machines"][0]["hostname"],json!(g.world.machines[0].hostname));
 assert!(view["machines"][0].get("fs").is_none());
 assert!(view["machines"][0].get("users").is_none());
 let full=content::call(&mut g,"test","plugin.world",&[]).unwrap();
 assert!(full["machines"][0].get("fs").is_some());
 assert!(content::call(&mut g,"test","plugin.world",&[json!("typo")]).is_err());
}
