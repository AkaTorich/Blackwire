//! Генератор мира: раскладка организаций по подсетям, адреса, карта, компьютер
//! игрока и его почта. Один и тот же генератор от одного зерна даёт один мир.
//! Данные (шаблоны организаций, службы, имена, тексты) — в соседних модулях.

use std::collections::HashSet;

use super::gen_machine::{self, MachineSpec};
use super::machine::{Machine, MachineId, MachineKind, WORLD_EPOCH};
use super::names::{self, Person};
use super::orgs::{Org, OrgKind, OrgTemplate, Role};
use super::player::{self, Known};
use super::web::{self, Contact};
use crate::rng::Rng;

/// Готовый мир: машины (machines[0] — игрок), организации, узлы, известные игроку.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct Generated {
    pub machines: Vec<Machine>,
    pub orgs: Vec<Org>,
    pub player_known: Vec<MachineId>,
}

/// Запланированная машина организации: адреса и люди известны до сборки,
/// чтобы соседи попали в журналы и зацепки друг друга.
struct Planned {
    id: MachineId,
    hostname: String,
    ip: String,
    lan_ip: Option<String>,
    kind: MachineKind,
    people: Vec<&'static Person>,
}

/// Диапазоны публичных адресов (первый октет).
const PUBLIC_FIRST: &[u16] = &[45, 77, 85, 91, 103, 178, 185, 193, 203, 212];

/// Выдать неиспользованный публичный адрес.
fn alloc_public(rng: &mut Rng, used: &mut HashSet<String>) -> String {
    loop {
        let a = *rng.pick(PUBLIC_FIRST);
        let ip = format!("{}.{}.{}.{}", a, rng.range(0, 256), rng.range(0, 256), rng.range(1, 255));
        if used.insert(ip.clone()) {
            return ip;
        }
    }
}

/// Выдать неиспользованный публичный адрес домашнего диапазона `87.x.x.x`.
fn alloc_home_public(rng: &mut Rng, used: &mut HashSet<String>) -> String {
    loop {
        let ip = format!("87.{}.{}.{}", rng.range(0, 256), rng.range(0, 256), rng.range(1, 255));
        if used.insert(ip.clone()) {
            return ip;
        }
    }
}

/// Выдать неиспользованную подсеть вида `10.N.M.` или `192.168.N.`.
fn alloc_subnet(rng: &mut Rng, used: &mut HashSet<String>) -> String {
    loop {
        let s = if rng.chance(0.6) {
            format!("10.{}.{}.", rng.range(1, 240), rng.range(0, 40))
        } else {
            format!("192.168.{}.", rng.range(1, 240))
        };
        if used.insert(s.clone()) {
            return s;
        }
    }
}

/// Имя машины по роли, тегу организации, человеку и номеру.
fn host_name(role: &Role, tag: &str, person: Option<&Person>, idx: usize, count: usize) -> String {
    if role.host.contains('.') {
        return role.host.to_string();
    }
    if role.host.contains("{name}") {
        let login = person.map(|p| p.login).unwrap_or("user");
        return format!("{}-{}", tag, role.host.replace("{name}", login));
    }
    if count > 1 {
        format!("{}-{}{}", tag, role.host, idx + 1)
    } else {
        format!("{}-{}", tag, role.host)
    }
}

/// Нужен ли машине вида `kind` сотрудник-владелец.
fn wants_owner(kind: MachineKind) -> bool {
    matches!(
        kind,
        MachineKind::Workstation | MachineKind::Server | MachineKind::Database | MachineKind::FileServer | MachineKind::Mail | MachineKind::Web
    )
}

/// Главная точка входа: построить мир.
pub fn generate(rng: &mut Rng, handle: &str, player_password: &str) -> Generated {
    let epoch = WORLD_EPOCH;
    let mut used_ips: HashSet<String> = HashSet::new();
    let mut used_subnets: HashSet<String> = HashSet::new();

    // Код приглашения hello-world — один на письмо игрока и на страницу шлюза.
    let invite = names::invite_code(rng);

    // Резерв адресов игрока и его маршрутизатора — домашний диапазон 87.x.x.x.
    let player_ip = alloc_home_public(rng, &mut used_ips);
    let router_ip = alloc_home_public(rng, &mut used_ips);

    // Машины организаций нумеруются с 2 (0 — игрок, 1 — маршрутизатор).
    let mut next_id: MachineId = 2;
    let mut org_machines: Vec<Machine> = Vec::new();
    let mut orgs: Vec<Org> = Vec::new();

    // Поле карты: организации кучками. Провайдер в центре.
    let templates: &[OrgTemplate] = super::orgs::templates();
    for (ti, tpl) in templates.iter().enumerate() {
        let count = rng.range(tpl.count.0 as i64, tpl.count.1 as i64 + 1).max(1) as usize;
        // Разные варианты имени без повторов.
        let mut variant_order: Vec<usize> = (0..tpl.variants.len()).collect();
        rng.shuffle(&mut variant_order);

        for inst in 0..count {
            let variant = &tpl.variants[variant_order[inst % tpl.variants.len()]];
            let subnet = alloc_subnet(rng, &mut used_subnets);
            let org_id = orgs.len() as u32;

            // Центр кучки на карте.
            let (cx, cy) = cluster_center(tpl.kind, ti, inst, rng);

            // Люди организации: перемешанный пул.
            let mut pool: Vec<&'static Person> = names::PEOPLE.iter().collect();
            rng.shuffle(&mut pool);
            let mut pool_iter = pool.into_iter();

            let mut planned: Vec<Planned> = Vec::new();
            let mut host_counter: u32 = 1;

            // Шлюз (кроме домашней сети, где машина сама себе шлюз).
            let gateway_id;
            if tpl.kind == OrgKind::Home {
                gateway_id = next_id; // назначим на единственной машине ниже
            } else {
                let gw_host = match tpl.gateway_host {
                    Some(h) if h.contains('.') => h.to_string(),
                    Some(h) => format!("{}-{}", variant.tag, h),
                    None => format!("{}-gw", variant.tag),
                };
                let gw_ip = alloc_public(rng, &mut used_ips);
                let gw_lan = format!("{}{}", subnet, host_counter);
                host_counter += 1;
                gateway_id = next_id;
                planned.push(Planned {
                    id: next_id,
                    hostname: gw_host,
                    ip: gw_ip,
                    lan_ip: Some(gw_lan),
                    kind: MachineKind::Gateway,
                    people: Vec::new(),
                });
                next_id += 1;
            }

            // Машины по ролям.
            for role in tpl.roles {
                let rc = rng.range(role.min as i64, role.max as i64 + 1).max(0) as usize;
                for i in 0..rc {
                    let person = if role.host.contains("{name}") || wants_owner(role.kind) {
                        pool_iter.next()
                    } else {
                        None
                    };
                    let person = if role.host.contains("{name}") {
                        // именованная рабочая станция обязательно с человеком
                        person.or_else(|| names::PEOPLE.first())
                    } else if wants_owner(role.kind) && rng.chance(0.5) {
                        person
                    } else {
                        None
                    };

                    let hostname = host_name(role, variant.tag, person, i, rc);
                    let lan = format!("{}{}", subnet, host_counter + 1);
                    host_counter += 1;
                    let ip = if role.exposed { alloc_public(rng, &mut used_ips) } else { lan.clone() };
                    let people: Vec<&'static Person> = person.into_iter().collect();
                    planned.push(Planned {
                        id: next_id,
                        hostname,
                        ip,
                        lan_ip: Some(lan),
                        kind: role.kind,
                        people,
                    });
                    next_id += 1;
                }
            }

            // Адреса соседей — для журналов и зацепок.
            let peer_ips: Vec<String> = planned.iter().map(|p| p.lan_ip.clone().unwrap_or_else(|| p.ip.clone())).collect();

            // Сборка машин.
            let mut member_ids: Vec<MachineId> = Vec::new();
            let positions = scatter(cx, cy, planned.len(), rng);
            for (pi, pl) in planned.iter().enumerate() {
                let (x, y) = positions[pi];
                let spec = MachineSpec {
                    id: pl.id,
                    hostname: pl.hostname.clone(),
                    ip: pl.ip.clone(),
                    lan_ip: pl.lan_ip.clone(),
                    domain: Some(variant.domain.to_string()),
                    kind: pl.kind,
                    org: Some(org_id),
                    router: Some(gateway_id),
                    org_name: variant.name.to_string(),
                    care: tpl.care,
                    x,
                    y,
                    people: pl.people.clone(),
                    peer_ips: peer_ips.iter().filter(|ip| *ip != &pl.lan_ip.clone().unwrap_or_default()).cloned().collect(),
                    epoch,
                };
                let m = gen_machine::build(spec, rng);
                member_ids.push(m.id);
                org_machines.push(m);
            }

            let org = Org {
                id: org_id,
                name: variant.name.to_string(),
                kind: tpl.kind,
                domain: variant.domain.to_string(),
                gateway: gateway_id,
                subnet: subnet.clone(),
                members: member_ids.clone(),
                description: variant.description.to_string(),
            };

            attach_pages(tpl, variant, &org, &mut org_machines, &invite, rng);
            orgs.push(org);
        }
    }

    // Известные игроку узлы: провайдерский DNS, один сайт, один домашний ПК.
    let dns_id = org_machines.iter().find(|m| m.kind == MachineKind::Dns).map(|m| m.id);
    let site_id = org_machines
        .iter()
        .find(|m| m.kind == MachineKind::Web && m.is_public())
        .map(|m| m.id);
    let home_pc = org_machines.iter().find(|m| m.kind == MachineKind::HomePc).map(|m| m.id);

    let mut known: Vec<Known> = Vec::new();
    known.push(Known { host: "home-router".to_string(), ip: router_ip.clone() });
    for id in [dns_id, site_id, home_pc].into_iter().flatten() {
        if let Some(m) = org_machines.get((id - 2) as usize) {
            known.push(Known { host: m.hostname.clone(), ip: m.ip.clone() });
        }
    }

    // Компьютер игрока и маршрутизатор.
    let player_lan = "192.168.0.2".to_string();
    let router_lan = "192.168.0.1".to_string();
    let player = player::build_player(
        0,
        handle,
        player_password,
        player_ip.clone(),
        player_lan,
        Some(1),
        &known,
        0.08,
        0.9,
        epoch,
        rng,
    );
    let router = player::build_router(1, router_ip, router_lan, true, 0.12, 0.82, epoch, rng);

    // Почта игрока.
    let bank = orgs.iter().find(|o| o.kind == OrgKind::Bank);
    let corp = orgs.iter().find(|o| o.kind == OrgKind::Corporation);
    let isp = orgs.iter().find(|o| o.kind == OrgKind::Isp);
    let challenge_ip = org_machines
        .iter()
        .find(|m| m.kind == MachineKind::HomePc)
        .map(|m| m.ip.clone())
        .unwrap_or_else(|| player_ip.clone());
    let inbox = super::mail::player_inbox(
        handle,
        &player_ip,
        bank.map(|o| o.name.as_str()).unwrap_or("Банк"),
        bank.map(|o| o.domain.as_str()).unwrap_or("bank.net"),
        corp.map(|o| o.name.as_str()).unwrap_or("Корпорация"),
        corp.map(|o| o.domain.as_str()).unwrap_or("corp.net"),
        isp.map(|o| o.domain.as_str()).unwrap_or("meridian.net"),
        &invite,
        epoch,
    );

    // Обновить текст первого испытания адресом настоящего домашнего ПК.
    let _ = challenge_ip; // адрес уже подставлен в attach_pages

    let mut player = player;
    for msg in inbox {
        player.push_mail(msg);
    }

    // Итоговый вектор: id == индекс.
    let mut machines: Vec<Machine> = Vec::with_capacity(org_machines.len() + 2);
    machines.push(player);
    machines.push(router);
    machines.extend(org_machines);

    // Известные игроку: маршрутизатор и найденные узлы.
    let mut player_known: Vec<MachineId> = vec![1];
    for id in [dns_id, site_id, home_pc].into_iter().flatten() {
        player_known.push(id);
    }

    Generated { machines, orgs, player_known }
}

/// Центр кучки организации на карте (0..1). Провайдер посередине.
fn cluster_center(kind: OrgKind, ti: usize, inst: usize, rng: &mut Rng) -> (f32, f32) {
    if kind == OrgKind::Isp {
        return (0.5, 0.5);
    }
    // Разложить организации по кольцу вокруг центра.
    let k = (ti * 3 + inst) as f32;
    let angle = k * 0.7;
    let radius = 0.22 + (k % 4.0) * 0.06;
    let cx = 0.5 + radius * angle.cos();
    let cy = 0.5 + radius * angle.sin();
    (rng.jitter(cx.clamp(0.08, 0.92), 0.03), rng.jitter(cy.clamp(0.08, 0.92), 0.03))
}

/// Разбросать членов организации вокруг центра кучки.
fn scatter(cx: f32, cy: f32, n: usize, rng: &mut Rng) -> Vec<(f32, f32)> {
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        if i == 0 {
            out.push((cx.clamp(0.02, 0.98), cy.clamp(0.02, 0.98)));
        } else {
            let a = rng.range_f(0.0, std::f64::consts::TAU) as f32;
            let r = rng.range_f(0.02, 0.08) as f32;
            out.push(((cx + r * a.cos()).clamp(0.02, 0.98), (cy + r * a.sin()).clamp(0.02, 0.98)));
        }
    }
    out
}

/// Навесить веб-страницы на машины организации.
fn attach_pages(
    tpl: &OrgTemplate,
    variant: &super::orgs::NameVariant,
    org: &Org,
    machines: &mut [Machine],
    invite: &str,
    rng: &mut Rng,
) {
    match tpl.kind {
        OrgKind::Underground => {
            // challenge_ip — адрес домашнего ПК, если он уже построен.
            let challenge_ip = machines
                .iter()
                .find(|m| m.kind == MachineKind::HomePc)
                .map(|m| m.ip.clone())
                .unwrap_or_else(|| "10.10.0.2".to_string());
            for id in &org.members {
                if let Some(m) = find_mut(machines, *id) {
                    let pages = if m.hostname == "gate.hw" || m.kind == MachineKind::Gateway {
                        web::underground_gate(invite, &challenge_ip)
                    } else {
                        web::underground_node(&m.hostname)
                    };
                    set_web_pages(m, pages);
                }
            }
        }
        OrgKind::Isp => {
            // Страница статуса на веб-узле.
            for id in &org.members {
                if let Some(m) = find_mut(machines, *id) {
                    if m.kind == MachineKind::Web {
                        set_web_pages(m, web::isp_status(&variant.name, variant.domain));
                    }
                }
            }
        }
        OrgKind::Hosting => {
            let sites = ["Блог путешественника", "Кулинарная страница", "Мастерская по ремонту", "Городская афиша", "Клуб любителей книг", "Фотоальбом", "Магазинчик у дома"];
            for id in &org.members {
                if let Some(m) = find_mut(machines, *id) {
                    if m.kind == MachineKind::Web {
                        let name = *rng.pick(&sites);
                        set_web_pages(m, web::hosting_site(name, "Личный проект на общем хостинге."));
                    }
                }
            }
        }
        _ => {
            // Обычный сайт организации на выставленном веб-узле (или шлюзе).
            let contacts = contacts_of(org, machines);
            let target = org
                .members
                .iter()
                .copied()
                .find(|id| find_ref(machines, *id).map(|m| m.kind == MachineKind::Web && m.is_public()).unwrap_or(false))
                .or(Some(org.gateway));
            if let Some(id) = target {
                if let Some(m) = find_mut(machines, id) {
                    set_web_pages(m, web::org_site(tpl.kind, &variant.name, variant.domain, variant.description, &contacts));
                }
            }
        }
    }
}

/// Контакты организации: сотрудники с адресами почты.
fn contacts_of(org: &Org, machines: &[Machine]) -> Vec<Contact> {
    let roles = ["специалист", "менеджер", "инженер", "администратор", "бухгалтер"];
    let mut out = Vec::new();
    let mut ri = 0usize;
    for id in &org.members {
        if let Some(m) = find_ref(machines, *id) {
            for u in m.users.list() {
                if u.name == "root" {
                    continue;
                }
                if let Some(p) = names::PEOPLE.iter().find(|p| p.login == u.name) {
                    out.push(Contact {
                        name: format!("{} {}", p.first, p.last),
                        email: format!("{}@{}", p.login, org.domain),
                        role: roles[ri % roles.len()].to_string(),
                    });
                    ri += 1;
                }
            }
        }
        if out.len() >= 6 {
            break;
        }
    }
    out
}

fn find_mut(machines: &mut [Machine], id: MachineId) -> Option<&mut Machine> {
    machines.iter_mut().find(|m| m.id == id)
}
fn find_ref(machines: &[Machine], id: MachineId) -> Option<&Machine> {
    machines.iter().find(|m| m.id == id)
}

/// Назначить страницы всем веб-службам машины (у http обязательно есть `/`).
fn set_web_pages(m: &mut Machine, pages: Vec<super::services::WebPage>) {
    for s in &mut m.services {
        if s.is_web() {
            s.pages = pages.clone();
        }
    }
    // Если веб-службы нет — добавить http, чтобы страница была доступна.
    if !m.services.iter().any(|s| s.is_web()) {
        m.services.push(super::services::Service {
            port: 80,
            name: "http".to_string(),
            version: "nginx/1.24.0".to_string(),
            running: true,
            banner: "HTTP/1.1 200 OK\r\nServer: nginx/1.24.0".to_string(),
            pages,
        });
        m.services.sort_by_key(|s| s.port);
    }
}
