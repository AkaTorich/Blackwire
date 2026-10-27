//! Команды взлома (второй этап): probe, exploit, crack, privesc, clearlogs,
//! proxy, heat, creds. Вся логика — в `crate::hack`; здесь разбор аргументов и
//! вывод с задержками для «кинематографа». Всё — над машинами игровой сети.

use super::util::Args;
use super::{CmdOutput, Ctx};
use crate::hack;
use crate::net::machine::MachineId;
use crate::types::{OutKind, OutLine};
use crate::world::NetError;

fn net_err(cmd: &str, e: &NetError) -> CmdOutput {
    CmdOutput::err(format!("{}: {}", cmd, e))
}

/// При активном рейде наступательные действия заблокированы. Возвращает отказ.
pub fn locked_guard(ctx: &Ctx<'_>, cmd: &str) -> Option<CmdOutput> {
    if hack::is_locked(ctx.world) {
        let left = hack::lock_remaining(ctx.world);
        let mut out = CmdOutput::err(format!("{}: канал под наблюдением после рейда — исходящий доступ закрыт ещё {} с", cmd, left));
        out.status = 1;
        Some(out)
    } else {
        None
    }
}

/// Найти машину-цель по адресу/имени с текущего hop'а.
fn resolve(ctx: &mut Ctx<'_>, cmd: &str, target: &str) -> Result<MachineId, CmdOutput> {
    let from = ctx.mid();
    ctx.world.lookup(from, target).map_err(|e| net_err(cmd, &e))
}

/// Предупредить о следе, если цель под наблюдением.
fn warn_trace(out: &mut CmdOutput, monitored: bool, heat_added: f32, source_ip: &str) {
    if monitored {
        out.lines.push(OutLine::warn(format!("цель под наблюдением — след с {} замечен (подозрение +{:.0})", source_ip, heat_added)).after(300));
    }
}

// ---------------------------------------------------------------- probe

pub fn probe(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    let target = match args.iter().find(|a| !a.starts_with('-')) {
        Some(t) => t.clone(),
        None => return CmdOutput::err("probe: укажите адрес или имя"),
    };
    let id = match resolve(ctx, "probe", &target) {
        Ok(id) => id,
        Err(o) => return o,
    };
    ctx.world.mark_scanned(id);
    let ip = ctx.world.address_for(ctx.mid(), id);
    let (host, services, users, vulns) = {
        let m = ctx.world.machine(id);
        (m.hostname.clone(), m.services.clone(), m.users.names(), hack::payload::available(ctx.world, ctx.session, id))
    };
    let mut out = CmdOutput::ok();
    out.push_kind(format!("Разведка {} ({})", host, ip), OutKind::Accent);
    out.lines.push(OutLine::heading("Службы:").after(150));
    for s in services.iter().filter(|s| s.running) {
        out.lines.push(OutLine::new(format!("  {:<6} {:<10} {}", s.port, s.name, s.version), OutKind::Normal).after(80));
    }
    if vulns.is_empty() {
        out.lines.push(OutLine::dim("Слабых мест не найдено.").after(250));
    } else {
        out.lines.push(OutLine::heading("Слабые места:").after(250));
        for v in &vulns {
            let scope = if v.local { "локально" } else { "по сети" };
            out.lines.push(OutLine::new(format!("  {}  {:<8} {:<9} {} → {} [механика: {}]", hack::payload::id(v), v.service, scope, v.name, hack::access::result(ctx.world,v), hack::payload::kind(ctx.world, v)), OutKind::Warn).after(120));
        }
    }
    out.lines.push(OutLine::dim("Дальше: research адрес id · exploit new файл.adr адрес id · готовые инструменты: market").after(100));
    out.lines.push(OutLine::heading("Пользователи:").after(200));
    out.lines.push(OutLine::new(format!("  {}", users.join(", ")), OutKind::Normal));
    out
}

// ---------------------------------------------------------------- exploit

pub fn research(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    let a = Args::parse(args, &["h"]);
    let Some(target) = a.pos.first() else { return CmdOutput::err("research [-h 1..4] адрес [id]: исследовать уязвимость и получить подсказку"); };
    let id = match resolve(ctx, "research", target) { Ok(id) => id, Err(e) => return e };
    let v = match hack::payload::select(ctx.world, ctx.session, id, a.pos.get(1).map(String::as_str)) {
        Ok(v) => v, Err(e) => return CmdOutput::err(e)
    };
    if let Err(e) = hack::payload::ensure_reachable(ctx.world, ctx.session, id, &v) { return CmdOutput::err(e); }
    let mut p = match hack::payload::profile(ctx.world, id, &v){Ok(p)=>p,Err(e)=>return CmdOutput::err(e)};
    p["conditions"]=hack::access::conditions(ctx.world,id,&v,Some(ctx.session));
    let level = a.opt("h").and_then(|s| s.parse::<usize>().ok()).unwrap_or(1).clamp(1, 4);
    let mut out = CmdOutput::ok();
    out.push_kind(format!("{} · {} · {}", v.name, v.version, p["kind"].as_str().unwrap()), OutKind::Accent);
    out.push_kind(format!("id: {}    fingerprint: {}", p["id"].as_str().unwrap(), p["fingerprint"].as_str().unwrap()), OutKind::Dim);
    out.push_kind(format!("Результат взлома: {}",p["result"]),OutKind::Accent);
    for condition in p["conditions"].as_array().unwrap(){out.push_kind(format!("{} {} · сейчас {} · {}",if condition["ok"]==true{"✓"}else{"✗"},condition["requirement"],condition["observed"],condition["hint"].as_str().unwrap_or("")),OutKind::Warn);}
    out.push_kind(serde_json::to_string_pretty(&p["layout"]).unwrap(), OutKind::Normal);
    for hint in p["hints"].as_array().unwrap().iter().take(level) { out.push_kind(hint.as_str().unwrap(), OutKind::Warn); }
    out.push_kind(format!("Следующая подсказка: research -h {} {} {}", (level+1).min(4), target, p["id"].as_str().unwrap()), OutKind::Dim);
    out.push_kind(format!("Свой код: exploit new ~/scripts/my_exploit.adr {} {}", target, p["id"].as_str().unwrap()), OutKind::Normal);
    if let Some(market)=p["market"].as_str(){out.push_kind(format!("Готовый инструмент: market buy {}. Совместимость: {}.",market,p["kind"].as_str().unwrap()),OutKind::Normal);}
    else{out.push_kind("Готовый эксплоит для этого типа не продаётся; создай свой по подсказкам.",OutKind::Dim);}
    ctx.world.mark_scanned(id);
    out
}

pub fn exploit(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    if args.first().is_some_and(|s| s == "new") {
        if args.len() < 3 { return CmdOutput::err("exploit new файл.adr адрес [id]: создать заготовку и открыть редактор"); }
        let path = ctx.path(&args[1]);
        if !path.ends_with(".adr") { return CmdOutput::err("нужно расширение .adr"); }
        if ctx.fs().exists(&path) { return CmdOutput::err("файл уже существует; открой его командой edit"); }
        let id = match resolve(ctx, "exploit new", &args[2]) { Ok(id) => id, Err(e) => return e };
        let v = match hack::payload::select(ctx.world, ctx.session, id, args.get(3).map(String::as_str)) {
            Ok(v) => v, Err(e) => return CmdOutput::err(e)
        };
        if let Err(e) = hack::payload::ensure_reachable(ctx.world, ctx.session, id, &v) { return CmdOutput::err(e); }
        let code = hack::payload::starter(&args[2], &hack::payload::id(&v));
        let user = ctx.user();
        if let Err(e) = ctx.fs_mut().write_text(&path, &code, &user) { return CmdOutput::err(e.to_string()); }
        ctx.actions.push(crate::types::UiAction::OpenApp { app: "editor".into(), arg: Some(path.clone()) });
        return CmdOutput::text(format!("Создана заготовка {}. Заполни payload по подсказкам research; затем exploit -f {} {}.", path, path, args[2]));
    }
    if let Some(o) = locked_guard(ctx, "exploit") { return o; }
    let a = Args::parse(args, &["s", "f"]);
    let Some(file) = a.opt("f") else {
        return CmdOutput::err("Автоматический взлом отключён. 1) probe адрес; 2) research адрес id; 3) exploit new ~/scripts/my_exploit.adr адрес id; 4) заполни payload в редакторе; 5) exploit -f ~/scripts/my_exploit.adr адрес. Готовые эксплоиты: market.");
    };
    let Some(target) = a.pos.first() else { return CmdOutput::err("exploit -f файл.adr [-s id] адрес"); };
    let mut run = vec![file.to_string(), target.clone()];
    if let Some(selector) = a.opt("s") { run.push(selector.to_string()); }
    super::cmds_misc::adder(ctx, &run)
}

// ---------------------------------------------------------------- crack

pub fn crack(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    if let Some(o) = locked_guard(ctx, "crack") {
        return o;
    }
    let a = Args::parse(args, &["w"]);
    let spec = match a.pos.first() {
        Some(s) => s.clone(),
        None => return CmdOutput::err("crack: использование: crack [-w словарь] пользователь@адрес"),
    };
    // пользователь@адрес, либо адрес и отдельный пользователь вторым словом
    let (user, host) = match spec.split_once('@') {
        Some((u, h)) if !u.is_empty() && !h.is_empty() => (u.to_string(), h.to_string()),
        _ => match a.pos.get(1) {
            Some(u) => (u.clone(), spec.clone()),
            None => return CmdOutput::err("crack: укажите пользователя: crack пользователь@адрес"),
        },
    };
    let id = match resolve(ctx, "crack", &host) {
        Ok(id) => id,
        Err(o) => return o,
    };
    let wl_path = a.opt("w").map(|s| s.to_string()).unwrap_or_else(|| "~/wordlist.txt".to_string());
    let words = match read_wordlist(ctx, &wl_path) {
        Ok(w) if !w.is_empty() => w,
        Ok(_) => return CmdOutput::err(format!("crack: словарь пуст: {}", wl_path)),
        Err(e) => return CmdOutput::err(format!("crack: {}", e)),
    };
    let report = match hack::crack(ctx.world, ctx.session, id, &user, &words) {
        Ok(r) => r,
        Err(e) => return CmdOutput::err(format!("crack: {}", e)),
    };
    let mut out = CmdOutput::ok();
    out.push_kind(format!("Подбор пароля {}@{} ({} слов) …", user, report.target_host, words.len()), OutKind::Accent);
    // прогресс: показываем попытки с задержкой, но не больше 40 строк
    let shown = report.attempts.len().min(40);
    let skip = report.attempts.len() - shown;
    if skip > 0 {
        out.lines.push(OutLine::dim(format!("  … пропущено {} слов", skip)).after(200));
    }
    for w in &report.attempts[skip..] {
        out.lines.push(OutLine::dim(format!("  пробую: {}", w)).after(70));
    }
    match &report.found {
        Some(pw) => {
            out.lines.push(OutLine::success(format!("Пароль найден: {} (после {} попыток)", pw, report.tried)).after(300));
            out.lines.push(OutLine::dim(format!("Теперь: ssh {}@{} {}", user, report.target_host, pw)).after(150));
        }
        None => {
            out.lines.push(OutLine::error(format!("Пароль не подобран ({} попыток).", report.tried)).after(300));
        }
    }
    warn_trace(&mut out, report.monitored, report.heat_added, &report.source_ip);
    out
}

/// Прочитать словарь с текущей машины, раскрыв `~`.
fn read_wordlist(ctx: &mut Ctx<'_>, path: &str) -> Result<Vec<String>, String> {
    let full = if path == "~" {
        ctx.home()
    } else if let Some(rest) = path.strip_prefix("~/") {
        format!("{}/{}", ctx.home(), rest)
    } else {
        ctx.path(path)
    };
    let user = ctx.user();
    let text = ctx.fs().read(&full, &user).map_err(|e| e.to_string())?;
    Ok(text.lines().map(|l| l.trim().to_string()).filter(|l| !l.is_empty()).collect())
}

// ---------------------------------------------------------------- privesc

pub fn privesc(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    let Some(file) = args.first() else { return CmdOutput::err("privesc файл.adr [id]: запусти собственный локальный эксплоит. Сначала research текущий_хост id. Готовый: market buy exp_race."); };
    let mut run = vec![file.clone(), ctx.machine().hostname.clone()];
    run.extend(args[1..].iter().cloned());
    super::cmds_misc::adder(ctx, &run)
}

// ---------------------------------------------------------------- clearlogs

pub fn clearlogs(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    let pattern = args.iter().find(|a| !a.starts_with('-')).map(|s| s.as_str());
    let report = match hack::clear_logs(ctx.world, ctx.session, pattern) {
        Ok(r) => r,
        Err(e) => return CmdOutput::err(format!("clearlogs: {}", e)),
    };
    let mut out = CmdOutput::ok();
    if report.denied {
        out.push_kind(format!("clearlogs: нужны права root на {}", report.host), OutKind::Error);
        out.status = 1;
    } else if report.removed == 0 {
        out.push_kind("Следов не найдено.", OutKind::Dim);
    } else {
        out.lines.push(OutLine::success(format!("Убрано записей: {} на {}", report.removed, report.host)).after(250));
    }
    out
}

// ---------------------------------------------------------------- proxy

pub fn proxy(ctx: &mut Ctx<'_>, _args: &[String]) -> CmdOutput {
    let info = hack::proxy_info(ctx.world, ctx.session);
    let mut out = CmdOutput::ok();
    out.push_kind("Цепочка подключений:", OutKind::Heading);
    for (i, (id, user)) in info.chain.iter().enumerate() {
        let m = ctx.world.machine(*id);
        let mark = if info.monitored_hops.contains(id) { "  ⚠ под наблюдением" } else { "" };
        let arrow = if i == 0 { "вы" } else { "→ " };
        out.push(format!("  {} {}@{} ({}){}", arrow, user, m.hostname, m.ip, mark));
    }
    out.push(format!("Выходной адрес: {} ({})", info.exit_ip, info.exit_host));
    if info.hidden {
        out.push_kind("Реальный адрес скрыт за цепочкой.", OutKind::Success);
    } else {
        out.push_kind("Вы работаете со своей машины — реальный адрес виден.", OutKind::Warn);
    }
    if !info.monitored_hops.is_empty() {
        out.push_kind("Среди hop'ов есть наблюдаемые — риск пеленга.", OutKind::Warn);
    }
    out
}

// ---------------------------------------------------------------- heat

pub fn heat(ctx: &mut Ctx<'_>, _args: &[String]) -> CmdOutput {
    let h = hack::heat(ctx.world);
    let traced = ctx.world.player.traced;
    let evidence = hack::evidence(ctx.world);
    let mut out = CmdOutput::ok();
    let filled = (h / 5.0).round() as usize;
    let bar: String = "█".repeat(filled) + &"░".repeat(20usize.saturating_sub(filled));
    let kind = if h >= 70.0 { OutKind::Error } else if h >= 35.0 { OutKind::Warn } else { OutKind::Success };
    out.lines.push(OutLine::new(format!("Подозрение: [{}] {:.0}/100", bar, h), kind));
    if traced {
        out.push_kind("Вас уже пеленговали — будьте осторожнее.", OutKind::Error);
    }
    if evidence.is_empty() {
        out.push_kind("Улик с вашим адресом нет.", OutKind::Dim);
    } else {
        out.push_kind("Улики (ваш адрес в журналах):", OutKind::Heading);
        for (id, n) in &evidence {
            let m = ctx.world.machine(*id);
            out.push(format!("  {} ({}) — {} записей", m.hostname, m.ip, n));
        }
        out.push_kind("Убрать: ssh на машину и clearlogs.", OutKind::Dim);
    }
    out
}

// ---------------------------------------------------------------- trace

pub fn trace(ctx: &mut Ctx<'_>, _args: &[String]) -> CmdOutput {
    // освежить ведущий орган и улики, не двигая прогресс (dt=0)
    hack::watch::sweep(ctx.world, 0);
    let inv = ctx.world.investigation.clone();
    let mut out = CmdOutput::ok();
    let lead = inv.lead_org.and_then(|id| ctx.world.org(id)).map(|o| o.name.clone());
    out.push_kind("Расследование (слежка):", OutKind::Heading);
    match &lead {
        Some(name) => out.push(format!("  Ведёт: {}", name)),
        None => out.push("  Ведущего органа в этой сети нет."),
    }
    let filled = (inv.progress / 5.0).round() as usize;
    let bar: String = "█".repeat(filled) + &"░".repeat(20usize.saturating_sub(filled));
    let kind = if inv.progress >= 70.0 {
        OutKind::Error
    } else if inv.progress >= 40.0 {
        OutKind::Warn
    } else {
        OutKind::Success
    };
    out.lines.push(OutLine::new(format!("  Стадия: {}  [{}] {:.0}/100", inv.stage.title(), bar, inv.progress), kind));
    if inv.raids > 0 {
        out.push(format!("  Рейдов было: {}", inv.raids));
    }
    if hack::is_locked(ctx.world) {
        out.push_kind(format!("  Канал заблокирован ещё {} с — переждите.", hack::lock_remaining(ctx.world)), OutKind::Error);
    }
    if inv.watched.is_empty() {
        out.push_kind("  Следствие ничего на вас не нашло.", OutKind::Dim);
    } else {
        out.push_kind("  След виден на узлах:", OutKind::Heading);
        for id in &inv.watched {
            if ctx.world.has_machine(*id) {
                let m = ctx.world.machine(*id);
                out.push(format!("    {} ({})", m.hostname, m.ip));
            }
        }
        out.push_kind("  Зайдите на них и clearlogs — дело стихнет.", OutKind::Dim);
    }
    out
}

// ---------------------------------------------------------------- creds

pub fn creds(ctx: &mut Ctx<'_>, _args: &[String]) -> CmdOutput {
    let creds = ctx.world.player.found_creds.clone();
    let mut out = CmdOutput::ok();
    if creds.is_empty() {
        out.push_kind("Найденных логинов пока нет.", OutKind::Dim);
        return out;
    }
    out.push_kind("Найденные логины:", OutKind::Heading);
    for c in &creds {
        let host = if ctx.world.has_machine(c.machine) { ctx.world.machine(c.machine).hostname.clone() } else { c.machine.to_string() };
        out.push(format!("  {:<16} {:<12} {:<16} {}", host, c.user, c.password, c.note));
    }
    out
}

// ---------------------------------------------------------------- файрвол

pub fn firewall(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    let target = match args.iter().find(|a| !a.starts_with('-')) {
        Some(t) => t.clone(),
        None => return CmdOutput::err("firewall: укажите адрес узла"),
    };
    let id = match resolve(ctx, "firewall", &target) {
        Ok(id) => id,
        Err(o) => return o,
    };
    match crate::hack::defense::bypass(ctx.world, ctx.session, id) {
        Ok(true) => {
            let mut out = CmdOutput::ok();
            out.lines.push(OutLine::success(format!("Файрвол на {} обойдён — узел открыт.", ctx.world.machine(id).hostname)).after(300));
            out
        }
        Ok(false) => {
            let mut out = CmdOutput::ok();
            out.lines.push(OutLine::warn("Обойти не удалось — попробуй ещё (и следи за подозрением).").after(300));
            out
        }
        Err(e) => CmdOutput::err(format!("firewall: {}", e)),
    }
}

// ---------------------------------------------------------------- кейлоггер

pub fn keylog(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    let a0 = args.iter().find(|a| !a.starts_with('-')).map(|s| s.as_str());
    if a0 == Some("plant") {
        let addr = match args.iter().filter(|a| !a.starts_with('-')).nth(1) {
            Some(a) => a.clone(),
            None => return CmdOutput::err("keylog: укажите адрес: keylog plant адрес"),
        };
        let id = match resolve(ctx, "keylog", &addr) {
            Ok(id) => id,
            Err(o) => return o,
        };
        return match crate::hack::tools::plant_keylog(ctx.world, id) {
            Ok(()) => {
                let mut out = CmdOutput::ok();
                out.lines.push(OutLine::success("Перехватчик установлен. Через время поймает чужой логин — смотри keylog.").after(300));
                out
            }
            Err(e) => CmdOutput::err(format!("keylog: {}", e)),
        };
    }
    // список
    let logs = ctx.world.keyloggers.clone();
    let mut out = CmdOutput::ok();
    out.push_kind("Перехватчики ввода:", OutKind::Heading);
    if logs.is_empty() {
        out.push_kind("  Нет. Поставить: keylog plant адрес (на взломанной машине).", OutKind::Dim);
        return out;
    }
    for k in &logs {
        let host = if ctx.world.has_machine(k.machine) { ctx.world.machine(k.machine).hostname.clone() } else { k.machine.to_string() };
        match &k.captured {
            Some(c) => out.lines.push(OutLine::new(format!("  {:<16} поймано: {} / {}", host, c.user, c.password), OutKind::Success)),
            None => out.lines.push(OutLine::new(format!("  {:<16} ждёт…", host), OutKind::Dim)),
        }
    }
    out
}

// ---------------------------------------------------------------- ботнет

pub fn botnet(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    let a0 = args.iter().find(|a| !a.starts_with('-')).map(|s| s.as_str());
    if a0 == Some("add") || a0 == Some("remove") {
        let addr = match args.iter().filter(|a| !a.starts_with('-')).nth(1) {
            Some(a) => a.clone(),
            None => return CmdOutput::err("botnet: укажите адрес"),
        };
        let id = match resolve(ctx, "botnet", &addr) {
            Ok(id) => id,
            Err(o) => return o,
        };
        if a0 == Some("add") {
            return match crate::hack::tools::botnet_add(ctx.world, id) {
                Ok(()) => {
                    let mut out = CmdOutput::ok();
                    out.lines.push(OutLine::success(format!("Узел {} в сети. Сеть тихо приносит ₵.", ctx.world.machine(id).hostname)).after(250));
                    out
                }
                Err(e) => CmdOutput::err(format!("botnet: {}", e)),
            };
        } else {
            crate::hack::tools::botnet_remove(ctx.world, id);
            return CmdOutput::ok();
        }
    }
    let size = crate::hack::tools::botnet_size(ctx.world);
    let mut out = CmdOutput::ok();
    out.push_kind(format!("Ботнет: {} живых узлов", size), OutKind::Heading);
    for id in ctx.world.botnet.clone() {
        if ctx.world.has_machine(id) {
            let m = ctx.world.machine(id);
            let state = if m.up { "в сети" } else { "offline" };
            out.push(format!("  {} ({}) — {}", m.hostname, m.ip, state));
        }
    }
    out.push_kind(format!("Намайнено всего: {} ₵", ctx.world.botnet_mined), OutKind::Heading);
    out.push_kind("Добавить: botnet add адрес (на взломанной машине).", OutKind::Dim);
    out
}

// ---------------------------------------------------------------- шифр

pub fn encrypt(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    cipher_cmd(ctx, args, true)
}

pub fn decrypt(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    cipher_cmd(ctx, args, false)
}

fn cipher_cmd(ctx: &mut Ctx<'_>, args: &[String], enc: bool) -> CmdOutput {
    let pos: Vec<&String> = args.iter().filter(|a| !a.starts_with('-')).collect();
    if pos.len() < 2 {
        return CmdOutput::err(if enc { "encrypt: encrypt файл ключ" } else { "decrypt: decrypt файл ключ" });
    }
    let path = ctx.path(pos[0]);
    let key = pos[1].clone();
    let user = ctx.user();
    let text = match ctx.fs().read(&path, &user) {
        Ok(t) => t,
        Err(e) => return CmdOutput::err(format!("{}: {}", if enc { "encrypt" } else { "decrypt" }, e)),
    };
    if enc {
        let out_path = format!("{}.enc", path);
        let data = crate::hack::cipher::encrypt(&text, &key);
        if let Err(e) = ctx.fs_mut().write(&out_path, &data, &user) {
            return CmdOutput::err(format!("encrypt: {}", e));
        }
        let mut out = CmdOutput::ok();
        out.lines.push(OutLine::success(format!("Зашифровано → {}", out_path)).after(200));
        out
    } else {
        match crate::hack::cipher::decrypt(&text, &key) {
            Ok(plain) => {
                let out_path = path.strip_suffix(".enc").map(|s| s.to_string()).unwrap_or_else(|| format!("{}.dec", path));
                if let Err(e) = ctx.fs_mut().write(&out_path, &plain, &user) {
                    return CmdOutput::err(format!("decrypt: {}", e));
                }
                let mut out = CmdOutput::ok();
                out.lines.push(OutLine::success(format!("Расшифровано → {}", out_path)).after(200));
                for l in plain.lines().take(10) {
                    out.push(format!("  {}", l));
                }
                out
            }
            Err(e) => CmdOutput::err(format!("decrypt: {}", e)),
        }
    }
}
