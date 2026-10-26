use blackwire_core::net::machine::MachineKind;
use blackwire_core::net::services::WebPage;
use blackwire_core::world::World;

fn router_id(world: &World) -> u32 {
    world.machines.iter().find(|m| m.hostname == "home-router").unwrap().id
}

#[test]
fn router_opens_from_home_page_in_new_worlds() {
    for seed in [7, 42, 2026] {
        let world = World::new(seed, "ghost", "pass");
        let router = world.machine(router_id(&world));
        let url = format!("http://{}/", router.ip);
        assert!(world.http(0, "home://").body.contains(&url));
        let page = world.http(0, &url);
        assert_eq!(page.status, 200);
        for expected in [&router.hostname, &router.ip, &router.os, router.lan_ip.as_ref().unwrap()] {
            assert!(page.body.contains(expected));
        }
        assert!(page.body.contains("Состояние маршрутизатора"));
        assert_eq!(world.http(0, "http://home-router/").status, 200);
    }
}

#[test]
fn router_opens_in_existing_save_without_changing_it() {
    let mut world = World::new(7, "ghost", "pass");
    let id = router_id(&world);
    for service in &mut world.machine_mut(id).services {
        service.pages.clear();
    }
    world.player.money = 12345;
    let saved = world.to_json();
    let restored = World::from_json(&saved).unwrap();
    assert_eq!(restored.http(0, "http://home-router/").status, 200);
    assert_eq!(restored.to_json(), saved);
}

#[test]
fn router_panel_uses_live_values_and_escapes_html() {
    let mut world = World::new(7, "ghost", "pass");
    let id = router_id(&world);
    let router = world.machine_mut(id);
    router.os = "RouterOS <updated> & patched".into();
    router.lan_ip = Some("192.168.9.1".into());
    let ssh = router.services.iter_mut().find(|s| s.name == "ssh").unwrap();
    ssh.running = false;
    ssh.version = "<script>".into();
    let page = world.http(0, "http://home-router/");
    assert_eq!(page.status, 200);
    assert!(page.body.contains("RouterOS &lt;updated&gt; &amp; patched"));
    assert!(page.body.contains("192.168.9.1"));
    assert!(page.body.contains("&lt;script&gt;"));
    assert!(!page.body.contains("<script>"));
    assert!(page.body.contains("Служба SSH остановлена"));
}

#[test]
fn custom_router_pages_take_precedence_and_unknown_paths_stay_404() {
    let mut world = World::new(7, "ghost", "pass");
    assert_eq!(world.http(0, "http://home-router/missing").status, 404);
    let id = router_id(&world);
    let service = world.machine_mut(id).services.iter_mut().find(|s| s.name == "http").unwrap();
    service.pages.push(WebPage { path: "/".into(), title: "Custom panel".into(), html: "Custom content".into() });
    let page = world.http(0, "http://home-router/");
    assert_eq!(page.status, 200);
    assert_eq!(page.body, "Custom content");
    assert_eq!(world.http(0, "http://home-router/missing").status, 404);
}

#[test]
fn router_panel_respects_closed_ports_and_offline_nodes() {
    let mut world = World::new(7, "ghost", "pass");
    let id = router_id(&world);
    world.machine_mut(id).services.iter_mut().find(|s| s.name == "http").unwrap().running = false;
    assert_eq!(world.http(0, "http://home-router/").status, 502);
    world.machine_mut(id).up = false;
    assert_eq!(world.http(0, "http://home-router/").status, 504);
}

#[test]
fn empty_web_service_on_other_device_does_not_become_router_panel() {
    let mut world = World::new(7, "ghost", "pass");
    let id = router_id(&world);
    world.machine_mut(id).kind = MachineKind::Server;
    assert_eq!(world.http(0, "http://home-router/").status, 404);
}
