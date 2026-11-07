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
        None => return CmdOutput::err(tr!("ping: укажите адрес или имя", "ping: specify an address or name")),
    };
    let count = a.opt_usize("c", 4).clamp(1, 10);
    let from = ctx.mid();
    let mut out = CmdOutput::ok();
    match ctx.world.lookup(from, &target) {
        Err(NetError::Unknown(n)) => return CmdOutput::err(trf!("ping: неизвестный узел {}", "ping: unknown host {}", n)),
        Err(e) => {
            let ip = ctx.world.resolve(&target).unwrap_or(target.clone());
            out.push(trf!("PING {} ({}) 56(84) байт данных.", "PING {} ({}) 56(84) bytes of data.", target, ip));
            for i in 1..=count {
                out.lines.push(OutLine::warn(trf!("icmp_seq={}: превышен интервал ожидания", "icmp_seq={}: request timed out", i)).after(350));
            }
            out.push("");
            out.push(trf!("--- {} статистика ---", "--- {} ping statistics ---", target));
            out.push(trf!("{} передано, 0 получено, 100% потерь", "{} transmitted, 0 received, 100% packet loss", count));
            out.push_kind(e.to_string(), OutKind::Dim);
            out.status = 1;
            out
        }
        Ok(id) => {
            let ip = ctx.world.address_for(from, id);
            let host = ctx.world.machine(id).hostname.clone();
            out.push(trf!("PING {} ({}) 56(84) байт данных.", "PING {} ({}) 56(84) bytes of data.", host, ip));
            let mut total = 0.0;
            for i in 1..=count {
                let ms = latency(ctx, id);
                total += ms;
                out.lines.push(OutLine::normal(trf!("64 байта от {} ({}): icmp_seq={} ttl=56 время={:.1} мс", "64 bytes from {} ({}): icmp_seq={} ttl=56 time={:.1} ms", host, ip, i, ms)).after(350));
            }
            out.push("");
            out.push(trf!("--- {} статистика ---", "--- {} ping statistics ---", host));
            out.push(trf!("{} передано, {} получено, 0% потерь, среднее {:.1} мс", "{} transmitted, {} received, 0% packet loss, avg {:.1} ms", count, count, total / count as f32));
            ctx.world.mark_known(id);
            ctx.world.log(id, LogKind::Net, &trf!("icmp echo от {}", "icmp echo from {}", ctx.world.machine(from).ip.clone()));
            out
        }
    }
}

pub fn nslookup(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    let name = match args.first() {
        Some(n) => n.clone(),
        None => return CmdOutput::err(tr!("nslookup: укажите имя", "nslookup: specify a name")),
    };
    let mut out = CmdOutput::ok();
    out.push_kind(tr!("Сервер:  10.0.0.1#53", "Server:  10.0.0.1#53"), OutKind::Dim);
    out.push("");
    match ctx.world.resolve(&name) {
        Some(ip) => {
            // обратный поиск: имя машины по адресу
            let host = ctx.world.find_by_ip(&ip).map(|id| ctx.world.machine(id).hostname.clone());
            out.push(trf!("Имя:    {}", "Name:    {}", name));
            out.push(trf!("Адрес:  {}", "Address: {}", ip));
            if let Some(h) = host {
                if h != name {
                    out.push_kind(trf!("Узел:   {}", "Host:    {}", h), OutKind::Dim);
                }
            }
        }
        None => out.push_err(trf!("** сервер не может найти {}: NXDOMAIN", "** server can't find {}: NXDOMAIN", name)),
    }
    out
}

pub fn scan(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    let target = match args.iter().find(|a| !a.starts_with('-')) {
        Some(t) => t.clone(),
        None => return CmdOutput::err(tr!("scan: укажите адрес или имя", "scan: specify an address or name")),
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
    out.push_kind(trf!("Сканирование {} ({}) …", "Scanning {} ({}) …", host, ip), OutKind::Accent);
    out.lines.push(OutLine::dim(trf!("Узел активен ({:.1} мс задержка), вид: {}", "Host is up ({:.1} ms latency), type: {}", latency(ctx, id), kind.title())).after(200));
    if services.is_empty() {
        out.lines.push(OutLine::warn(tr!("Открытых портов не найдено.", "No open ports found.")).after(400));
    } else {
        out.lines.push(OutLine::heading(trf!("{:<9} {:<9} {:<10} ВЕРСИЯ", "{:<9} {:<9} {:<10} VERSION", tr!("ПОРТ", "PORT"), tr!("СОСТОЯНИЕ", "STATE"), tr!("СЛУЖБА", "SERVICE"))).after(250));
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
    out.lines.push(OutLine::dim(trf!("Готово: {} служб.", "Done: {} services.", services.iter().filter(|s| s.running).count())).after(200));
    ctx.world.mark_scanned(id);
    let from_ip = ctx.world.machine(from).ip.clone();
    ctx.world.log(id, LogKind::Net, &trf!("сканирование портов с {}", "port scan from {}", from_ip));
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
        return Err(trf!("{}: соединение отклонено — порт 22 закрыт", "{}: connection refused — port 22 closed", what));
    }
    if !m.users.exists(user) {
        ctx.world.log(id, LogKind::LoginFailed, &trf!("{}: неизвестный пользователь {} с {}", "{}: unknown user {} from {}", what, user, from_ip));
        return Err(trf!("{}: доступ запрещён (неверное имя или пароль)", "{}: access denied (wrong login or password)", what));
    }
    if !m.users.check_password(user, password) {
        ctx.world.log(id, LogKind::LoginFailed, &trf!("{}: неверный пароль для {} с {}", "{}: wrong password for {} from {}", what, user, from_ip));
        return Err(trf!("{}: доступ запрещён (неверное имя или пароль)", "{}: access denied (wrong login or password)", what));
    }
    ctx.world.log(id, LogKind::Login, &trf!("{}: вход {} с {}", "{}: login {} from {}", what, user, from_ip));
    Ok(())
}

pub fn ssh(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    if let Some(o) = super::cmds_hack::locked_guard(ctx, "ssh") {
        return o;
    }
    let a = Args::parse(args, &["p"]);
    let spec = match a.pos.first() {
        Some(s) => s.clone(),
        None => return CmdOutput::err(tr!("ssh: использование: ssh пользователь@адрес пароль", "ssh: usage: ssh user@address password")),
    };
    let (user, host) = match split_user_host(&spec) {
        Some(x) => x,
        None => return CmdOutput::err(tr!("ssh: укажите пользователя: ssh пользователь@адрес пароль", "ssh: specify a user: ssh user@address password")),
    };
    let password = match a.pos.get(1) {
        Some(p) => p.clone(),
        None => return CmdOutput::err(trf!("ssh: укажите пароль третьим словом: ssh {}@{} пароль", "ssh: give the password as the third word: ssh {}@{} password", user, host)),
    };
    if ctx.session.hops.len() >= 8 {
        return CmdOutput::err(tr!("ssh: слишком длинная цепочка подключений", "ssh: connection chain too long"));
    }
    let from = ctx.mid();
    let id = match ctx.world.lookup(from, &host) {
        Ok(id) => id,
        Err(e) => return net_err("ssh", &e),
    };
    if id == from {
        return CmdOutput::err(tr!("ssh: вы уже на этой машине", "ssh: you are already on this machine"));
    }
    if crate::hack::defense::firewall_up(ctx.world, id) {
        return CmdOutput::err(tr!("ssh: узел закрыт файрволом — сначала обойди его: firewall <адрес>", "ssh: host is behind a firewall — bypass it first: firewall <address>"));
    }
    let ip = ctx.world.address_for(from, id);
    let mut out = CmdOutput::ok();
    out.lines.push(OutLine::dim(trf!("Подключение к {} …", "Connecting to {} …", ip)));
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
    out.lines.push(OutLine::success(trf!("Вход выполнен: {}@{}", "Logged in: {}@{}", user, hostname)).after(400));
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
        return CmdOutput::err(tr!("scp: использование: scp [user@ip:]откуда [user@ip:]куда пароль", "scp: usage: scp [user@ip:]source [user@ip:]dest password"));
    }
    let (src_remote, src_path) = scp_spec(&args[0]);
    let (dst_remote, dst_path) = scp_spec(&args[1]);
    if src_remote.is_none() && dst_remote.is_none() {
        return CmdOutput::err(tr!("scp: обе стороны локальные — используйте cp", "scp: both sides are local — use cp"));
    }
    if src_remote.is_some() && dst_remote.is_some() {
        return CmdOutput::err(tr!("scp: копирование между двумя удалёнными машинами не поддерживается", "scp: copying between two remote machines is not supported"));
    }
    let password = match args.get(2) {
        Some(p) => p.clone(),
        None => return CmdOutput::err(tr!("scp: укажите пароль последним словом", "scp: give the password as the last word")),
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
        out.lines.push(OutLine::success(trf!("{} → {} ({} байт)", "{} → {} ({} bytes)", rp, local, data.len())).after(400));
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
        ctx.world.log(id, LogKind::FileChange, &trf!("scp: {} записал {}", "scp: {} wrote {}", remote_user, rp));
        out.lines.push(OutLine::success(trf!("{} → {} ({} байт)", "{} → {} ({} bytes)", lp, rp, data.len())).after(400));
    }
    out
}

pub fn netstat(ctx: &mut Ctx<'_>, _args: &[String]) -> CmdOutput {
    let mut out = CmdOutput::ok();
    out.push_kind(tr!("Активные соединения", "Active connections"), OutKind::Accent);
    let mut rows = vec![vec![tr!("Прото", "Proto").to_string(), tr!("Локальный адрес", "Local Address").into(), tr!("Внешний адрес", "Foreign Address").into(), tr!("Состояние", "State").into()]];
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
    out.push_kind(tr!("Прослушиваемые порты", "Listening ports"), OutKind::Accent);
    let mut rows = vec![vec![tr!("Прото", "Proto").to_string(), tr!("Адрес", "Address").into(), tr!("Служба", "Service").into(), tr!("Процесс", "Process").into()]];
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
        None => return CmdOutput::err(tr!("whois: укажите адрес или имя", "whois: specify an address or name")),
    };
    let ip = match ctx.world.resolve(&target) {
        Some(ip) => ip,
        None => return CmdOutput::err(trf!("whois: неизвестный узел {}", "whois: unknown host {}", target)),
    };
    let id = match ctx.world.find_by_ip(&ip) {
        Some(id) => id,
        None => return CmdOutput::err(trf!("whois: адрес {} не зарегистрирован", "whois: address {} is not registered", ip)),
    };
    let m = ctx.world.machine(id);
    let mut rows: Vec<Vec<String>> = vec![vec![tr!("Адрес:", "Address:").into(), ip.clone()], vec![tr!("Узел:", "Host:").into(), m.hostname.clone()]];
    if let Some(d) = &m.domain {
        rows.push(vec![tr!("Домен:", "Domain:").into(), d.clone()]);
    }
    match ctx.world.org_of(id) {
        Some(org) => {
            rows.push(vec![tr!("Организация:", "Organization:").into(), org.name.clone()]);
            rows.push(vec![tr!("Вид:", "Type:").into(), format!("{:?}", org.kind)]);
            rows.push(vec![tr!("Домен орг.:", "Org domain:").into(), org.domain.clone()]);
            rows.push(vec![tr!("Подсеть:", "Subnet:").into(), format!("{}0/24", org.subnet)]);
            rows.push(vec![tr!("Описание:", "Description:").into(), org.description.clone()]);
        }
        None => {
            let who = match m.kind {
                MachineKind::Isp | MachineKind::Dns => tr!("провайдер NorthLink ISP", "NorthLink ISP provider"),
                MachineKind::Player | MachineKind::HomePc | MachineKind::Router => tr!("частное лицо (домашнее подключение)", "private individual (home connection)"),
                _ => tr!("не указано", "not specified"),
            };
            rows.push(vec![tr!("Владелец:", "Owner:").into(), who.to_string()]);
        }
    }
    rows.push(vec![tr!("Состояние:", "Status:").into(), if m.up { tr!("в сети", "online").into() } else { tr!("не отвечает", "not responding").into() }]);
    let mut out = CmdOutput::ok();
    out.push_lines(table(&rows, &[false, false]));
    out
}

pub fn traceroute(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    let target = match args.first() {
        Some(t) => t.clone(),
        None => return CmdOutput::err(tr!("traceroute: укажите адрес или имя", "traceroute: specify an address or name")),
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
    out.push(trf!("traceroute до {} ({}), не больше 30 переходов", "traceroute to {} ({}), 30 hops max", target, ctx.world.address_for(from, id)));
    let mut acc = 0.0;
    for (i, hop) in path.iter().enumerate() {
        let m = ctx.world.machine(*hop);
        let (host, ip) = (m.hostname.clone(), ctx.world.address_for(from, *hop));
        acc += latency(ctx, *hop) / 3.0;
        out.lines.push(OutLine::normal(trf!("{:2}  {} ({})  {:.1} мс", "{:2}  {} ({})  {:.1} ms", i + 1, host, ip, acc)).after(220));
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
        None => return CmdOutput::err(tr!("wget: укажите url", "wget: specify a url")),
    };
    let page = ctx.world.http(ctx.mid(), &url);
    let mut out = CmdOutput::ok();
    out.lines.push(OutLine::dim(format!("--  {}", url)));
    if page.status != 200 {
        out.lines.push(OutLine::error(trf!("wget: ошибка {}: {}", "wget: error {}: {}", page.status, page.title)).after(300));
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
    out.lines.push(OutLine::dim(trf!("Ответ HTTP 200 OK, длина {}", "HTTP response 200 OK, length {}", body.len())).after(250));
    out.lines.push(OutLine::success(trf!("Сохранено: {} ({} байт)", "Saved: {} ({} bytes)", path, body.len())).after(250));
    out
}

pub fn curl(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    let a = Args::parse(args, &[]);
    let url = match a.pos.first() {
        Some(u) => u.clone(),
        None => return CmdOutput::err(tr!("curl: укажите url", "curl: specify a url")),
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
    let mut rows = vec![vec![tr!("Адрес", "Address").to_string(), tr!("Тип", "Type").into(), "MAC".into(), tr!("Интерфейс", "Interface").into()]];
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
        out.push_kind(tr!("соседей в локальной сети нет", "no neighbours on the local network"), OutKind::Dim);
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
        vec![tr!("Назначение", "Destination").to_string(), tr!("Шлюз", "Gateway").into(), tr!("Маска", "Mask").into(), tr!("Интерфейс", "Interface").into()],
        vec!["0.0.0.0".into(), gateway, "0.0.0.0".into(), "eth0".into()],
        vec![net, "0.0.0.0".into(), "255.255.255.0".into(), "eth0".into()],
    ];
    let mut out = CmdOutput::ok();
    out.push_lines(table(&rows, &[false, false, false, false]));
    out
}
