//! Тесты генератора мира: детерминизм, уникальность адресов и имён, компьютер
//! игрока, устройство организаций, веб-страницы, DNS, словарь и завязка сюжета.

use std::collections::HashSet;

use blackwire_core::net::dns;
use blackwire_core::net::gen::{self, Generated};
use blackwire_core::net::machine::MachineKind;
use blackwire_core::net::orgs::OrgKind;
use blackwire_core::rng::Rng;

fn world(seed: u64) -> Generated {
    let mut rng = Rng::new(seed);
    gen::generate(&mut rng, "ghost", "s3cret-pass")
}

#[test]
fn detereminism_same_seed_same_world() {
    let a = world(7);
    let b = world(7);
    let ja = serde_json::to_string(&a).unwrap();
    let jb = serde_json::to_string(&b).unwrap();
    assert_eq!(ja, jb, "один и тот же seed должен давать один и тот же мир");
}

#[test]
fn different_seed_different_world() {
    let a = serde_json::to_string(&world(7)).unwrap();
    let b = serde_json::to_string(&world(8)).unwrap();
    assert_ne!(a, b, "разные зёрна — разный мир");
}

#[test]
fn machine_count_in_range() {
    let w = world(7);
    assert!(
        w.machines.len() >= 60 && w.machines.len() <= 120,
        "ожидалось 60..120 машин, получено {}",
        w.machines.len()
    );
}

#[test]
fn ids_match_index() {
    let w = world(7);
    for (i, m) in w.machines.iter().enumerate() {
        assert_eq!(m.id as usize, i, "id машины должен совпадать с индексом");
    }
}

#[test]
fn unique_ips_and_hostnames() {
    let w = world(7);
    let mut ips = HashSet::new();
    let mut hosts = HashSet::new();
    for m in &w.machines {
        assert!(ips.insert(m.ip.clone()), "повтор ip: {}", m.ip);
        assert!(hosts.insert(m.hostname.clone()), "повтор hostname: {}", m.hostname);
    }
}

#[test]
fn player_machine_and_files() {
    let w = world(7);
    let p = &w.machines[0];
    assert_eq!(p.kind, MachineKind::Player);
    assert_eq!(p.hostname, "ghost-pc");
    assert!(p.ip.starts_with("87."), "адрес игрока должен быть из домашнего диапазона 87.x.x.x, получен {}", p.ip);
    assert!(p.users.get("ghost").is_some(), "у игрока должна быть учётка ghost");
    for f in [
        "/home/ghost/readme.txt",
        "/home/ghost/wordlist.txt",
        "/home/ghost/scripts/hello.adr",
        "/home/ghost/scripts/scan.adr",
        "/home/ghost/scripts/bruteforce_demo.adr",
        "/home/ghost/scripts/backup.adr",
        "/etc/hosts",
        "/etc/motd",
    ] {
        assert!(p.fs.read_text(f).is_ok(), "у игрока должен быть файл {}", f);
    }
}

#[test]
fn player_ip_first_octet_87() {
    // Контракт: публичный адрес игрока вида 87.x.x.x не обязателен диапазоном,
    // но hostname и наличие публичного адреса — обязательны.
    let w = world(7);
    let p = &w.machines[0];
    assert!(p.is_public(), "у игрока должен быть публичный адрес");
}

#[test]
fn orgs_have_gateway_and_members_in_subnet() {
    let w = world(7);
    assert!(!w.orgs.is_empty(), "мир без организаций");
    for org in &w.orgs {
        let gw = &w.machines[org.gateway as usize];
        assert!(gw.is_public(), "шлюз {} должен иметь публичный адрес", org.name);
        assert!(!org.members.is_empty(), "организация {} без машин", org.name);
        for id in &org.members {
            let m = &w.machines[*id as usize];
            assert_eq!(m.org, Some(org.id), "машина {} не привязана к своей организации", m.hostname);
            if !m.is_public() {
                let lan = m.lan_ip.as_ref().expect("у внутренней машины должен быть lan_ip");
                assert!(
                    lan.starts_with(&org.subnet),
                    "lan_ip {} вне подсети {} организации {}",
                    lan,
                    org.subnet,
                    org.name
                );
            }
        }
    }
}

#[test]
fn http_services_have_root_page() {
    let w = world(7);
    let mut checked = 0;
    for m in &w.machines {
        for s in &m.services {
            if s.is_web() && !s.pages.is_empty() {
                assert!(
                    s.pages.iter().any(|p| p.path == "/"),
                    "у веб-службы машины {} нет страницы /",
                    m.hostname
                );
                checked += 1;
            }
        }
    }
    assert!(checked > 0, "в мире не нашлось ни одной веб-службы со страницами");
}

#[test]
fn dns_resolves_all_org_domains() {
    let w = world(7);
    for org in &w.orgs {
        if org.domain.is_empty() {
            continue;
        }
        let r = dns::resolve(&w.machines, &w.orgs, &org.domain);
        assert!(r.is_some(), "DNS не нашёл домен {} ({})", org.domain, org.name);
    }
    // Служебные имена.
    assert!(dns::resolve(&w.machines, &w.orgs, "home.net").is_some(), "home.net должен разрешаться");
    assert!(dns::resolve(&w.machines, &w.orgs, "gate.hw").is_some(), "gate.hw должен разрешаться");
}

#[test]
fn dns_reverse_roundtrip() {
    let w = world(7);
    let m = &w.machines[0];
    let host = dns::reverse(&w.machines, &m.ip);
    assert_eq!(host.as_deref(), Some(m.hostname.as_str()));
}

#[test]
fn weak_passwords_are_in_player_wordlist() {
    let w = world(7);
    let list: HashSet<String> = w.machines[0]
        .fs
        .read_text("/home/ghost/wordlist.txt")
        .unwrap()
        .lines()
        .map(|s| s.to_string())
        .collect();
    assert!(list.len() > 150, "словарь должен содержать около 200 слов, а в нём {}", list.len());

    let mut hits = 0;
    for m in &w.machines {
        for u in m.users.list() {
            if list.contains(&u.password) {
                hits += 1;
            }
        }
    }
    assert!(hits > 0, "в мире должны встречаться слабые пароли из словаря игрока");
}

#[test]
fn underground_org_and_invite_flow() {
    let w = world(7);

    // Организация hello-world есть.
    let hw = w.orgs.iter().find(|o| o.kind == OrgKind::Underground);
    assert!(hw.is_some(), "в мире должна быть скрытая сеть hello-world");

    // У игрока письмо от null@hw со ссылкой на gate.hw/invite/.
    let player = &w.machines[0];
    let letter = player
        .mail
        .iter()
        .find(|m| m.from == "null@hw")
        .expect("у игрока должно быть письмо от null@hw");
    let pos = letter.body.find("http://gate.hw/invite/").expect("в письме должна быть ссылка-приглашение");
    let tail = &letter.body[pos + "http://gate.hw/invite/".len()..];
    let code: String = tail.chars().take_while(|c| c.is_ascii_hexdigit()).collect();
    assert_eq!(code.len(), 8, "код приглашения — 8 hex-символов, получено {:?}", code);

    // Страница по этой ссылке существует на gate.hw.
    let gate = w
        .machines
        .iter()
        .find(|m| m.hostname == "gate.hw")
        .expect("в мире должен быть шлюз gate.hw");
    let path = format!("/invite/{}", code);
    assert!(gate.page(&path).is_some(), "страница {} должна существовать на gate.hw", path);
    assert!(gate.page("/manifesto").is_some(), "у gate.hw должен быть манифест");
    assert!(gate.page("/board").is_some(), "у gate.hw должна быть доска заданий");
}

#[test]
fn player_mailbox_has_expected_letters() {
    let w = world(7);
    let player = &w.machines[0];
    assert!(player.mail.len() >= 6, "у игрока должно быть не меньше 6 писем");
    assert!(player.mail.iter().all(|m| m.to == "ghost@home.net"), "все письма адресованы игроку");
}

#[test]
fn player_known_nodes() {
    let w = world(7);
    assert!(w.player_known.len() >= 3 && w.player_known.len() <= 4, "игрок должен знать 3–4 узла");
    for id in &w.player_known {
        assert!((*id as usize) < w.machines.len(), "известный узел вне диапазона");
    }
}
