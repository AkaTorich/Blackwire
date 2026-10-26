use blackwire_core::{Game, World, OutKind};
use blackwire_lang::{Host, LangError, Value as Adder};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

fn root() -> PathBuf { Path::new(env!("CARGO_MANIFEST_DIR")).join("../plugins/community") }
fn catalog() -> Vec<Value> {
    serde_json::from_str(include_str!("../../plugins/catalog.json")).unwrap()
}
fn clean(game: &Game, context: &str) {
    assert!(game.plugins.errors.is_empty(), "{context}: {:?}", game.plugins.errors);
    for (id, plugin) in &game.plugins.loaded {
        assert!(plugin.error.is_none(), "{context} / {id}: {:?}", plugin.error);
    }
}
fn command(game: &mut Game, sid: u32, line: &str) {
    let result = game.exec(sid, line);
    assert!(!result.lines.iter().any(|l| l.kind == OutKind::Error), "{line}: {result:?}");
    clean(game, line);
}
fn content(item: &Value) -> bool {
    matches!(item["family"].as_str().unwrap(), "device"|"network"|"case"|"event")
}
struct RealHost<'a> { game: &'a mut Game, sid: u32, output: Vec<String> }
impl Host for RealHost<'_> {
    fn print(&mut self, text: &str) { self.output.push(text.into()); }
    fn input(&mut self, _: &str) -> Result<String, LangError> { Err(LangError::runtime(0, "unexpected input")) }
    fn call(&mut self, name: &str, args: Vec<Adder>) -> Result<Adder, LangError> {
        blackwire_core::script_host::call(self.game, self.sid, 0, name, args.iter().map(Adder::to_json).collect())
            .map(|v| Adder::from_json(&v)).map_err(|e| LangError::runtime(0,e))
    }
    fn should_stop(&self) -> bool { false }
    fn sleep(&mut self, _: u64) {}
}

#[test]
fn every_plugin_works_independently_and_survives_save_reload() {
    let entries = catalog();
    assert_eq!(entries.len(), 100);
    for item in &entries {
        let id = item["id"].as_str().unwrap();
        eprintln!("Checking {id}");
        let folder = root().join(id);
        assert!(folder.join("START_HERE.md").is_file(), "{id}: missing guide");
        let mut game = Game::new(World::new(42, "pack_test", ""));
        let before = game.world.machines.len();
        game.load_plugins(&folder);
        assert_eq!(game.plugins.loaded.len(), 1, "{id}");
        clean(&game, id);
        let sid = game.open_session();
        command(&mut game, sid, &format!("{id}_open"));
        if content(item) {
            let lines = game.plugin_ui(&format!("{id}:guide"), "start", json!({})).unwrap();
            assert!(!lines.iter().any(|l| l.kind == OutKind::Error), "{id}: {lines:?}");
            clean(&game, id);
            let nodes = item["nodes"].as_array().unwrap();
            assert_eq!(game.world.machines.len(), before + nodes.len(), "{id}");
            command(&mut game, sid, &format!("{id}_start"));
            assert_eq!(game.world.machines.len(), before + nodes.len(), "{id}: duplicate spawn");
            let money = game.world.player.money;
            for node in nodes {
                let host = node["hostname"].as_str().unwrap();
                let mid = game.world.find_by_hostname(host).unwrap();
                let machine = game.world.machine(mid);
                assert!(machine.fs.read_text(node["file"].as_str().unwrap()).is_ok(), "{id}: missing evidence");
                assert_eq!(blackwire_core::hack::remote_vulns(machine).iter()
                    .filter(|v| v.name.starts_with(&format!("{id}:"))).count(), 2, "{id}: custom remote vulnerabilities");
                assert!(!blackwire_core::hack::local_vulns(machine).is_empty(), "{id}: local vulnerability");
                if blackwire_core::hack::defense::firewall_up(&game.world, mid) {
                    for _ in 0..20 {
                        command(&mut game, sid, &format!("firewall {}", node["ip"].as_str().unwrap()));
                        if !blackwire_core::hack::defense::firewall_up(&game.world, mid) { break; }
                    }
                    assert!(!blackwire_core::hack::defense::firewall_up(&game.world, mid), "{id}: firewall still up");
                }
                let service = format!("{id}:{}_entry", node["key"].as_str().unwrap());
                command(&mut game, sid, &format!("exploit -s {service} {}", node["ip"].as_str().unwrap()));
            }
            let quest = format!("{id}:mission");
            assert!(game.world.plugins.quests[&quest].done, "{id}: quest not complete");
            assert_eq!(game.world.player.money, money + item["reward"].as_i64().unwrap(), "{id}: wrong reward");
            let paid = game.world.player.money;
            game.close_session(sid);
            let repeat = game.open_session();
            for node in nodes {
                command(&mut game, repeat, &format!("exploit -s {id}:{}_entry {}",
                    node["key"].as_str().unwrap(), node["ip"].as_str().unwrap()));
            }
            assert_eq!(game.world.player.money, paid, "{id}: repeated reward");
            game.close_session(repeat);
            for _ in 0..65 { game.tick(1000); }
            clean(&game, id);
            if item["family"] == "event" {
                assert_eq!(game.world.plugins.data[id]["phase"], json!(3), "{id}: incident phases");
                assert!(game.world.plugins.timers.get(id).is_none_or(|t| t.is_empty()), "{id}: timer not stopped");
            }
        } else if item["family"] == "utility" {
            command(&mut game, sid, &format!("{id}_report"));
            let lines = game.plugin_ui(&format!("{id}:guide"), "export", json!({})).unwrap();
            assert!(!lines.iter().any(|l| l.kind == OutKind::Error), "{id}: {lines:?}");
            clean(&game, id);
            let path = format!("/home/pack_test/reports/{id}.txt");
            assert!(game.world.machines[0].fs.read_text(&path).unwrap().contains(item["name"].as_str().unwrap()));
        } else {
            game.world.player.money = 2000;
            if item["mode"] == "hardware" {
                let field = item["field"].as_str().unwrap();
                let initial = json!(game.world.machines[0])[field].as_u64().unwrap();
                command(&mut game, sid, &format!("buy {id}:item"));
                assert_eq!(json!(game.world.machines[0])[field].as_u64().unwrap(),
                    initial + item["amount"].as_u64().unwrap(), "{id}: upgrade");
                game.plugin_event("on_purchase", json!({"catalog":"shop","id":format!("{id}:item")}));
                assert_eq!(json!(game.world.machines[0])[field].as_u64().unwrap(),
                    initial + item["amount"].as_u64().unwrap(), "{id}: duplicate upgrade");
            } else {
                command(&mut game, sid, &format!("market buy {id}:item"));
                let file = item["file"].as_str().unwrap();
                let code = game.world.machines[0].fs.read_text(&format!("/home/pack_test/scripts/{file}")).unwrap();
                let mut host = RealHost {game: &mut game, sid, output: Vec::new()};
                blackwire_lang::run(&code, &mut host, vec![file.into()]).unwrap_or_else(|e| panic!("{id}: {e}"));
                assert!(!host.output.is_empty(), "{id}: script produced no output");
            }
            assert_eq!(game.world.player.money, 2000-item["price"].as_i64().unwrap(), "{id}: purchase price");
        }
        let machine_count = game.world.machines.len();
        let money = game.world.player.money;
        let mut restored = Game::from_json(&game.save_json()).unwrap();
        restored.load_plugins(&folder);
        clean(&restored, id);
        assert_eq!(restored.world.machines.len(), machine_count, "{id}: reload duplicated content");
        assert_eq!(restored.world.player.money, money, "{id}: reload changed money");
        if content(item) {
            assert!(restored.world.plugins.quests[&format!("{id}:mission")].done, "{id}: lost completion");
            let sid = restored.open_session();
            command(&mut restored, sid, &format!("{id}_start"));
            assert_eq!(restored.world.machines.len(), machine_count, "{id}: saved start duplicated content");
        }
        restored.plugin_enable(id, false).unwrap();
        restored.plugin_enable(id, true).unwrap();
        clean(&restored, id);
        assert_eq!(restored.world.machines.len(), machine_count, "{id}: re-enable duplicated content");
    }
}

#[test]
#[ignore = "Combined load test runs on GitHub runners only"]
fn all_100_plugins_coexist_and_keep_their_content_after_reload() {
    let entries = catalog();
    let mut game = Game::new(World::new(42, "pack_test", ""));
    let initial = game.world.machines.len();
    game.load_plugins(&root());
    assert_eq!(game.plugins.loaded.len(), 100);
    clean(&game, "load all");
    let sid = game.open_session();
    let mut created = 0;
    for item in &entries {
        let id = item["id"].as_str().unwrap();
        eprintln!("Checking {id}");
        if content(item) {
            command(&mut game, sid, &format!("{id}_start"));
            created += item["nodes"].as_array().unwrap().len();
        }
    }
    assert_eq!(game.world.machines.len(), initial + created);
    assert_eq!(game.world.plugins.quests.len(), 70);
    for item in &entries {
        let id = item["id"].as_str().unwrap();
        eprintln!("Checking {id}");
        if item["family"] == "utility" {
            command(&mut game, sid, &format!("{id}_report"));
            command(&mut game, sid, &format!("{id}_export"));
        }
    }
    for _ in 0..65 { game.tick(1000); }
    clean(&game, "all plugins after 65 seconds");
    for item in entries.iter().filter(|i| i["family"] == "event") {
        assert_eq!(game.world.plugins.data[item["id"].as_str().unwrap()]["phase"], json!(3));
    }
    let mut restored = Game::from_json(&game.save_json()).unwrap();
    restored.load_plugins(&root());
    clean(&restored, "reload all");
    assert_eq!(restored.plugins.loaded.len(), 100);
    assert_eq!(restored.world.machines.len(), initial+created);
    let sid = restored.open_session();
    for item in entries.iter().filter(|i| content(i)) {
        command(&mut restored, sid, &format!("{}_start", item["id"].as_str().unwrap()));
    }
    assert_eq!(restored.world.machines.len(), initial+created);
    let ips: std::collections::BTreeSet<_> = restored.world.machines.iter().map(|m| &m.ip).collect();
    assert_eq!(ips.len(), restored.world.machines.len());
}
