//! Сетевые команды: адреса, ping, nslookup, scan, ssh, scp, netstat, whois,
//! traceroute, wget/curl, arp, route. Достижимость считает `World::lookup`.

use super::util::{table, Args};
use super::{CmdOutput, Ctx};
use crate::net::machine::{LogKind, MachineId, MachineKind};
use crate::types::{OutKind, OutLine};
use crate::world::NetError;

/// Условный MAC-адрес по номеру машины.
fn mac(id: MachineId) -> String {
    let h = (id as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15) >> 16;
    format!("02:bw:{:02x}:{:02x}:{:02x}:{:02x}", (h >> 24) & 0xff, (h >> 16) & 0xff, (h >> 8) & 0xff, h & 0xff)
}

/// Условная задержка до машины, мс.
fn latency(ctx: &mut Ctx<'_>, id: MachineId) -> f32 {
    let base = 3.0 + ((id as u64 * 37) % 48) as f32;
    base + ctx.world.rng.range_f(0.0, 4.0) as f32
}

fn net_err(cmd: &str, e: &NetError) -> CmdOutput {
    CmdOutput::err(format!("{}: {}", cmd, e))
}

pub fn ifconfig(ctx: &mut Ctx<'_>, _args: &[String]) -> CmdOutput {
    let m = ctx.machine();
    let mut out = CmdOutput::ok();
    let iface = |name: &str, ip: &str, out: &mut CmdOutput| {
        out.push_kind(format!("{}: flags=4163<UP,BROADCAST,RUNNING,MULTICAST>  mtu 1500", name), OutKind::Accent);
        out.push(format!("        inet {}  netmask 255.255.255.0", ip));
        out.push(format!("        ether {}", mac(m.id)));
        out.push(format!("        RX packets {}  TX packets {}", (ctx.world.net_in * 1000.0) as u64, (ctx.world.net_out * 800.0) as u64));
    };
    iface("eth0", &m.ip, &mut out);
    if let Some(lan) = &m.lan_ip {
        if lan != &m.ip {
            iface("eth1", lan, &mut out);
        }
    }
    out.push_kind("lo: flags=73<UP,LOOPBACK,RUNNING>  mtu 65536", OutKind::Accent);
    out.push("        inet 127.0.0.1  netmask 255.0.0.0");
    out
}

pub fn ip(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    match args.first().map(|s| s.as_str()) {
        Some("route") | Some("r") => route(ctx, &[]),
        _ => {
            let m = ctx.machine();
            let mut out = CmdOutput::ok();
            out.push(format!("1: lo    inet 127.0.0.1/8 scope host"));
            out.push(format!("2: eth0  inet {}/24 scope global", m.ip));
            if let Some(lan) = &m.lan_ip {
                if lan != &m.ip {
                    out.push(format!("3: eth1  inet {}/24 scope global", lan));
                }
            }
            out
        }
    }
}

pub fn ping(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    let a = Args::parse(args, &["c"]);
    let target = match a.pos.first() {
        Some(t) => t.clone(),
        None => return CmdOutput::err("ping: укажите адрес или имя"),
    };
    let count = a.opt_usize("c", 4).clamp(1, 10);
    let from = ctx.mid();
    let mut out = CmdOutput::ok();
    match ctx.world.lookup(from, &target) {
        Err(NetError::Unknown(n)) => return CmdOutput::err(format!("ping: неизвестный узел {}", n)),
        Err(e) => {
            let ip = ctx.world.resolve(&target).unwrap_or(target.clone());
            out.push(format!("PING {} ({}) 56(84) байт данных.", target, ip));
            for i in 1..=count {
                out.lines.push(OutLine::warn(format!("icmp_seq={}: превышен интервал ожидания", i)).after(350));
            }
            out.push("");
            out.push(format!("--- {} статистика ---", target));
            out.push(format!("{} передано, 0 получено, 100% потерь", count));
            out.push_kind(e.to_string(), OutKind::Dim);
            out.status = 1;
            out
        }
        Ok(id) => {
            let ip = ctx.world.address_for(from, id);
            let host = ctx.world.machine(id).hostname.clone();
            out.push(format!("PING {} ({}) 56(84) байт данных.", host, ip));
            let mut total = 0.0;
            for i in 1..=count {
                let ms = latency(ctx, id);
                total += ms;
                out.lines.push(OutLine::normal(format!("64 байта от {} ({}): icmp_seq={} ttl=56 время={:.1} мс", host, ip, i, ms)).after(350));
            }
            out.push("");
            out.push(format!("--- {} статистика ---", host));
            out.push(format!("{} передано, {} получено, 0% потерь, среднее {:.1} мс", count, count, total / count as f32));
            ctx.world.mark_known(id);
            ctx.world.log(id, LogKind::Net, &format!("icmp echo от {}", ctx.world.machine(from).ip.clone()));
            out
        }
    }
}

pub fn nslookup(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    let name = match args.first() {
        Some(n) => n.clone(),
        None => return CmdOutput::err("nslookup: укажите имя"),
    };
    let mut out = CmdOutput::ok();
    out.push_kind("Сервер:  10.0.0.1#53", OutKind::Dim);
    out.push("");
    match ctx.world.resolve(&name) {
        Some(ip) => {
            // обратный поиск: имя машины по адресу
            let host = ctx.world.find_by_ip(&ip).map(|id| ctx.world.machine(id).hostname.clone());
            out.push(format!("Имя:    {}", name));
            out.push(format!("Адрес:  {}", ip));
            if let Some(h) = host {
                if h != name {
                    out.push_kind(format!("Узел:   {}", h), OutKind::Dim);
                }
            }
        }
        None => out.push_err(format!("** сервер не может найти {}: NXDOMAIN", name)),
    }
    out
}

pub fn scan(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    let target = match args.iter().find(|a| !a.starts_with('-')) {
        Some(t) => t.clone(),
        None => return CmdOutput::err("scan: укажите адрес или имя"),
    };
    let from = ctx.mid();
    let id = match ctx.world.lookup(from, &target) {
        Ok(id) => id,
        Err(e) => return net_err("scan", &e),
    };
    let ip = ctx.world.address_for(from, id);
    let (host, kind, services) = {
        let m = ctx.world.machine(id);
        (m.hostname.clone(), m.kind, m.services.clone())
    };
    let mut out = CmdOutput::ok();
    out.push_kind(format!("Сканирование {} ({}) …", host, ip), OutKind::Accent);
    out.lines.push(OutLine::dim(format!("Узел активен ({:.1} мс задержка), вид: {}", latency(ctx, id), kind.title())).after(200));
    if services.is_empty() {
        out.lines.push(OutLine::warn("Открытых портов не найдено.").after(400));
    } else {
        out.lines.push(OutLine::heading(format!("{:<9} {:<9} {:<10} ВЕРСИЯ", "ПОРТ", "СОСТОЯНИЕ", "СЛУЖБА")).after(250));
        for s in &services {
            let delay = ctx.world.rng.range(120, 251) as u32;
            let state = if s.running { "open" } else { "closed" };
            let line = format!("{:<9} {:<9} {:<10} {}", format!("{}/tcp", s.port), state, s.name, s.version);
            out.lines.push(OutLine::new(line, if s.running { OutKind::Normal } else { OutKind::Dim }).after(delay));
            if s.running && !s.banner.is_empty() {
                out.lines.push(OutLine::dim(format!("          └ {}", s.banner)));
            }
        }
    }
    out.lines.push(OutLine::dim(format!("Готово: {} служб.", services.iter().filter(|s| s.running).count())).after(200));
    ctx.world.mark_scanned(id);
    let from_ip = ctx.world.machine(from).ip.clone();
    ctx.world.log(id, LogKind::Net, &format!("сканирование портов с {}", from_ip));
    out
}

/// Разобрать `user@host`.
fn split_user_host(s: &str) -> Option<(String, String)> {
    let (u, h) = s.split_once('@')?;
    if u.is_empty() || h.is_empty() {
        return None;
    }
    Some((u.to_string(), h.to_string()))
}

/// Проверить пользователя и пароль на машине; пишет в её журнал.
fn authenticate(ctx: &mut Ctx<'_>, id: MachineId, user: &str, password: &str, what: &str) -> Result<(), String> {
    let from_ip = ctx.world.machine(ctx.mid()).ip.clone();
    let m = ctx.world.machine(id);
    if m.service_named("ssh").is_none() {
        return Err(format!("{}: соединение отклонено — порт 22 закрыт", what));
    }
    if !m.users.exists(user) {
        ctx.world.log(id, LogKind::LoginFailed, &format!("{}: неизвестный пользователь {} с {}", what, user, from_ip));
        return Err(format!("{}: доступ запрещён (неверное имя или пароль)", what));
    }
    if !m.users.check_password(user, password) {
        ctx.world.log(id, LogKind::LoginFailed, &format!("{}: неверный пароль для {} с {}", what, user, from_ip));
        return Err(format!("{}: доступ запрещён (неверное имя или пароль)", what));
    }
    ctx.world.log(id, LogKind::Login, &format!("{}: вход {} с {}", what, user, from_ip));
    Ok(())
}

pub fn ssh(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    if let Some(o) = super::cmds_hack::locked_guard(ctx, "ssh") {
        return o;
    }
    let a = Args::parse(args, &["p"]);
    let spec = match a.pos.first() {
        Some(s) => s.clone(),
        None => return CmdOutput::err("ssh: использование: ssh пользователь@адрес пароль"),
    };
    let (user, host) = match split_user_host(&spec) {
        Some(x) => x,
        None => return CmdOutput::err("ssh: укажите пользователя: ssh пользователь@адрес пароль"),
    };
    let password = match a.pos.get(1) {
        Some(p) => p.clone(),
        None => return CmdOutput::err(format!("ssh: укажите пароль третьим словом: ssh {}@{} пароль", user, host)),
    };
    if ctx.session.hops.len() >= 8 {
        return CmdOutput::err("ssh: слишком длинная цепочка подключений");
    }
    let from = ctx.mid();
    let id = match ctx.world.lookup(from, &host) {
        Ok(id) => id,
        Err(e) => return net_err("ssh", &e),
    };
    if id == from {
        return CmdOutput::err("ssh: вы уже на этой машине");
    }
    if crate::hack::defense::firewall_up(ctx.world, id) {
        return CmdOutput::err("ssh: узел закрыт файрволом — сначала обойди его: firewall <адрес>");
    }
    let ip = ctx.world.address_for(from, id);
    let mut out = CmdOutput::ok();
    out.lines.push(OutLine::dim(format!("Подключение к {} …", ip)));
    ctx.world.mark_known(id);
    if let Err(e) = authenticate(ctx, id, &user, &password, "ssh") {
        out.lines.push(OutLine::error(e).after(500));
        out.status = 1;
        return out;
    }
    let (hostname, motd, home) = {
        let m = ctx.world.machine(id);
        (m.hostname.clone(), m.fs.read_text("/etc/motd").unwrap_or_default(), m.users.home_of(&user))
    };
    ctx.session.push_hop(id, &user, &home);
    out.lines.push(OutLine::success(format!("Вход выполнен: {}@{}", user, hostname)).after(400));
    for l in motd.lines() {
        out.lines.push(OutLine::accent(l));
    }
    out
}

/// Разобрать аргумент scp: (user@host, путь) или (None, путь).
fn scp_spec(s: &str) -> (Option<(String, String)>, String) {
    if let Some((left, path)) = s.split_once(':') {
        if let Some(uh) = split_user_host(left) {
            return (Some(uh), if path.is_empty() { ".".to_string() } else { path.to_string() });
        }
    }
    (None, s.to_string())
}

pub fn scp(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    if args.len() < 2 {
        return CmdOutput::err("scp: использование: scp [user@ip:]откуда [user@ip:]куда пароль");
    }
    let (src_remote, src_path) = scp_spec(&args[0]);
    let (dst_remote, dst_path) = scp_spec(&args[1]);
    if src_remote.is_none() && dst_remote.is_none() {
        return CmdOutput::err("scp: обе стороны локальные — используйте cp");
    }
    if src_remote.is_some() && dst_remote.is_some() {
        return CmdOutput::err("scp: копирование между двумя удалёнными машинами не поддерживается");
    }
    let password = match args.get(2) {
        Some(p) => p.clone(),
        None => return CmdOutput::err("scp: укажите пароль последним словом"),
    };
    let from = ctx.mid();
    let (remote_user, remote_host) = src_remote.clone().or(dst_remote.clone()).expect("одна сторона удалённая");
    let id = match ctx.world.lookup(from, &remote_host) {
        Ok(id) => id,
        Err(e) => return net_err("scp", &e),
    };
    ctx.world.mark_known(id);
    if let Err(e) = authenticate(ctx, id, &remote_user, &password, "scp") {
        return CmdOutput::err(e);
    }
    let me = ctx.user();
    let remote_home = ctx.world.machine(id).users.home_of(&remote_user);
    let remote_abs = |p: &str| crate::vfs::resolve(&remote_home, &p.replace("~", &remote_home));
    let mut out = CmdOutput::ok();
    if src_remote.is_some() {
        // скачать
        let rp = remote_abs(&src_path);
        let data = match ctx.world.machine(id).fs.read_bytes(&rp, &remote_user) {
            Ok(d) => d,
            Err(e) => return CmdOutput::err(format!("scp: {}", e)),
        };
        let mut local = ctx.path(&dst_path);
        if ctx.fs().is_dir(&local) {
            local = crate::vfs::join(&local, &crate::vfs::basename(&rp));
        }
        let r = match String::from_utf8(data.clone()) {
            Ok(text) => ctx.fs_mut().write(&local, &text, &me),
            Err(_) => ctx.fs_mut().write_bytes(&local, data.clone(), &me),
        };
        if let Err(e) = r {
            return CmdOutput::err(format!("scp: {}", e));
        }
        out.lines.push(OutLine::success(format!("{} → {} ({} байт)", rp, local, data.len())).after(400));
    } else {
        // загрузить
        let lp = ctx.path(&src_path);
        let data = match ctx.fs().read_bytes(&lp, &me) {
            Ok(d) => d,
            Err(e) => return CmdOutput::err(format!("scp: {}", e)),
        };
        let mut rp = remote_abs(&dst_path);
        if ctx.world.machine(id).fs.is_dir(&rp) {
            rp = crate::vfs::join(&rp, &crate::vfs::basename(&lp));
        }
        let r = match String::from_utf8(data.clone()) {
            Ok(text) => ctx.world.machine_mut(id).fs.write(&rp, &text, &remote_user),
            Err(_) => ctx.world.machine_mut(id).fs.write_bytes(&rp, data.clone(), &remote_user),
        };
        if let Err(e) = r {
            return CmdOutput::err(format!("scp: {}", e));
        }
        ctx.world.log(id, LogKind::FileChange, &format!("scp: {} записал {}", remote_user, rp));
        out.lines.push(OutLine::success(format!("{} → {} ({} байт)", lp, rp, data.len())).after(400));
    }
    out
}

pub fn netstat(ctx: &mut Ctx<'_>, _args: &[String]) -> CmdOutput {
    let mut out = CmdOutput::ok();
    out.push_kind("Активные соединения", OutKind::Accent);
    let mut rows = vec![vec!["Прото".to_string(), "Локальный адрес".into(), "Внешний адрес".into(), "Состояние".into()]];
    let hops = ctx.session.hops.clone();
    let me = ctx.mid();
    for w in hops.windows(2) {
        let (a, b) = (w[0].machine, w[1].machine);
        if b == me {
            rows.push(vec!["tcp".into(), format!("{}:22", ctx.world.machine(b).ip), format!("{}:{}", ctx.world.machine(a).ip, 40000 + a * 7), "ESTABLISHED".into()]);
        }
        if a == me {
            rows.push(vec!["tcp".into(), format!("{}:{}", ctx.world.machine(a).ip, 40000 + b * 7), format!("{}:22", ctx.world.machine(b).ip), "ESTABLISHED".into()]);
        }
    }
    out.push_lines(table(&rows, &[false, false, false, false]));
    out.push("");
    out.push_kind("Прослушиваемые порты", OutKind::Accent);
    let mut rows = vec![vec!["Прото".to_string(), "Адрес".into(), "Служба".into(), "Процесс".into()]];
    for s in &ctx.machine().services {
        if s.running {
            rows.push(vec!["tcp".into(), format!("0.0.0.0:{}", s.port), s.name.clone(), s.proc_name().to_string()]);
        }
    }
    out.push_lines(table(&rows, &[false, false, false, false]));
    out
}

pub fn whois(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    let target = match args.first() {
        Some(t) => t.clone(),
        None => return CmdOutput::err("whois: укажите адрес или имя"),
    };
    let ip = match ctx.world.resolve(&target) {
        Some(ip) => ip,
        None => return CmdOutput::err(format!("whois: неизвестный узел {}", target)),
    };
    let id = match ctx.world.find_by_ip(&ip) {
        Some(id) => id,
        None => return CmdOutput::err(format!("whois: адрес {} не зарегистрирован", ip)),
    };
    let m = ctx.world.machine(id);
    let mut rows: Vec<Vec<String>> = vec![vec!["Адрес:".into(), ip.clone()], vec!["Узел:".into(), m.hostname.clone()]];
    if let Some(d) = &m.domain {
        rows.push(vec!["Домен:".into(), d.clone()]);
    }
    match ctx.world.org_of(id) {
        Some(org) => {
            rows.push(vec!["Организация:".into(), org.name.clone()]);
            rows.push(vec!["Вид:".into(), format!("{:?}", org.kind)]);
            rows.push(vec!["Домен орг.:".into(), org.domain.clone()]);
            rows.push(vec!["Подсеть:".into(), format!("{}0/24", org.subnet)]);
            rows.push(vec!["Описание:".into(), org.description.clone()]);
        }
        None => {
            let who = match m.kind {
                MachineKind::Isp | MachineKind::Dns => "провайдер NorthLink ISP",
                MachineKind::Player | MachineKind::HomePc | MachineKind::Router => "частное лицо (домашнее подключение)",
                _ => "не указано",
            };
            rows.push(vec!["Владелец:".into(), who.to_string()]);
        }
    }
    rows.push(vec!["Состояние:".into(), if m.up { "в сети".into() } else { "не отвечает".into() }]);
    let mut out = CmdOutput::ok();
    out.push_lines(table(&rows, &[false, false]));
    out
}

pub fn traceroute(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    let target = match args.first() {
        Some(t) => t.clone(),
        None => return CmdOutput::err("traceroute: укажите адрес или имя"),
    };
    let from = ctx.mid();
    let id = match ctx.world.lookup(from, &target) {
        Ok(id) => id,
        Err(e) => return net_err("traceroute", &e),
    };
    let mut path: Vec<MachineId> = Vec::new();
    let f = ctx.world.machine(from);
    if let Some(r) = f.router {
        path.push(r);
    }
    let same_org = f.org.is_some() && f.org == ctx.world.machine(id).org;
    if !same_org && f.router != Some(id) {
        if let Some(isp) = ctx.world.machines.iter().find(|m| m.kind == MachineKind::Isp).map(|m| m.id) {
            if isp != id && isp != from {
                path.push(isp);
            }
        }
        if let Some(org) = ctx.world.org_of(id) {
            if org.gateway != id {
                path.push(org.gateway);
            }
        }
    }
    path.push(id);
    path.retain(|m| *m != from);
    path.dedup();
    let mut out = CmdOutput::ok();
    out.push(format!("traceroute до {} ({}), не больше 30 переходов", target, ctx.world.address_for(from, id)));
    let mut acc = 0.0;
    for (i, hop) in path.iter().enumerate() {
        let m = ctx.world.machine(*hop);
        let (host, ip) = (m.hostname.clone(), ctx.world.address_for(from, *hop));
        acc += latency(ctx, *hop) / 3.0;
        out.lines.push(OutLine::normal(format!("{:2}  {} ({})  {:.1} мс", i + 1, host, ip, acc)).after(220));
    }
    ctx.world.mark_known(id);
    out
}

/// Имя файла для wget по адресу страницы.
fn file_from_url(url: &str) -> String {
    let path = url.split("://").nth(1).unwrap_or(url);
    let name = path.rsplit('/').next().unwrap_or("");
    if name.is_empty() || !path.contains('/') {
        "index.html".to_string()
    } else {
        name.to_string()
    }
}

pub fn wget(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    let a = Args::parse(args, &["O"]);
    let url = match a.pos.first() {
        Some(u) => u.clone(),
        None => return CmdOutput::err("wget: укажите url"),
    };
    let page = ctx.world.http(ctx.mid(), &url);
    let mut out = CmdOutput::ok();
    out.lines.push(OutLine::dim(format!("--  {}", url)));
    if page.status != 200 {
        out.lines.push(OutLine::error(format!("wget: ошибка {}: {}", page.status, page.title)).after(300));
        out.status = 1;
        return out;
    }
    let name = a.opt("O").map(|s| s.to_string()).unwrap_or_else(|| file_from_url(&url));
    let path = ctx.path(&name);
    let me = ctx.user();
    let body = format!("<!doctype html>\n<title>{}</title>\n{}\n", page.title, page.body);
    if let Err(e) = ctx.fs_mut().write(&path, &body, &me) {
        return CmdOutput::err(format!("wget: {}", e));
    }
    out.lines.push(OutLine::dim(format!("Ответ HTTP 200 OK, длина {}", body.len())).after(250));
    out.lines.push(OutLine::success(format!("Сохранено: {} ({} байт)", path, body.len())).after(250));
    out
}

pub fn curl(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    let a = Args::parse(args, &[]);
    let url = match a.pos.first() {
        Some(u) => u.clone(),
        None => return CmdOutput::err("curl: укажите url"),
    };
    let page = ctx.world.http(ctx.mid(), &url);
    let mut out = CmdOutput::ok();
    if a.has("I") || a.has("i") {
        out.push_kind(format!("HTTP/1.1 {} {}", page.status, page.title), OutKind::Accent);
        out.push("Content-Type: text/html; charset=utf-8");
        out.push(format!("Content-Length: {}", page.body.len()));
        if a.has("I") {
            return out;
        }
        out.push("");
    }
    if page.status != 200 {
        out.push_kind(format!("curl: ({}) {}", page.status, page.title), OutKind::Warn);
        out.status = 1;
    }
    out.push(page.body);
    out
}

pub fn arp(ctx: &mut Ctx<'_>, _args: &[String]) -> CmdOutput {
    let me = ctx.mid();
    let f = ctx.world.machine(me);
    let mut rows = vec![vec!["Адрес".to_string(), "Тип".into(), "MAC".into(), "Интерфейс".into()]];
    for m in &ctx.world.machines {
        if m.id == me {
            continue;
        }
        let neighbour = (f.org.is_some() && f.org == m.org) || f.router == Some(m.id) || m.router == Some(me) || (f.router.is_some() && f.router == m.router);
        if neighbour {
            let ip = m.lan_ip.clone().unwrap_or_else(|| m.ip.clone());
            rows.push(vec![ip, "ether".into(), mac(m.id), "eth0".into()]);
        }
    }
    let mut out = CmdOutput::ok();
    if rows.len() == 1 {
        out.push_kind("соседей в локальной сети нет", OutKind::Dim);
        return out;
    }
    out.push_lines(table(&rows, &[false, false, false, false]));
    out
}

pub fn route(ctx: &mut Ctx<'_>, _args: &[String]) -> CmdOutput {
    let m = ctx.machine();
    let gateway = m
        .router
        .map(|r| {
            let rm = ctx.world.machine(r);
            rm.lan_ip.clone().unwrap_or_else(|| rm.ip.clone())
        })
        .or_else(|| ctx.world.org_of(m.id).map(|o| format!("{}1", o.subnet)))
        .unwrap_or_else(|| "10.0.0.1".to_string());
    let lan = m.lan_ip.clone().unwrap_or_else(|| m.ip.clone());
    let net = lan.rsplit_once('.').map(|(a, _)| format!("{}.0", a)).unwrap_or_default();
    let rows = vec![
        vec!["Назначение".to_string(), "Шлюз".into(), "Маска".into(), "Интерфейс".into()],
        vec!["0.0.0.0".into(), gateway, "0.0.0.0".into(), "eth0".into()],
        vec![net, "0.0.0.0".into(), "255.255.255.0".into(), "eth0".into()],
    ];
    let mut out = CmdOutput::ok();
    out.push_lines(table(&rows, &[false, false, false, false]));
    out
}
