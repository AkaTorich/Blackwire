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
        let mut out = CmdOutput::err(trf!("{}: канал под наблюдением после рейда — исходящий доступ закрыт ещё {} с", "{}: channel under surveillance after the raid — outbound access closed for another {} s", cmd, left));
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
        out.lines.push(OutLine::warn(trf!("цель под наблюдением — след с {} замечен (подозрение +{:.0})", "target under surveillance — trace from {} spotted (heat +{:.0})", source_ip, heat_added)).after(300));
    }
}

// ---------------------------------------------------------------- probe

pub fn probe(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    let target = match args.iter().find(|a| !a.starts_with('-')) {
        Some(t) => t.clone(),
        None => return CmdOutput::err(tr!("probe: укажите адрес или имя", "probe: specify an address or name")),
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
    out.push_kind(trf!("Разведка {} ({})", "Recon {} ({})", host, ip), OutKind::Accent);
    out.lines.push(OutLine::heading(tr!("Службы:", "Services:")).after(150));
    for s in services.iter().filter(|s| s.running) {
        out.lines.push(OutLine::new(format!("  {:<6} {:<10} {}", s.port, s.name, s.version), OutKind::Normal).after(80));
    }
    if vulns.is_empty() {
        out.lines.push(OutLine::dim(tr!("Слабых мест не найдено.", "No weak spots found.")).after(250));
    } else {
        out.lines.push(OutLine::heading(tr!("Слабые места:", "Weak spots:")).after(250));
        for v in &vulns {
            let scope = if v.local { tr!("локально", "local") } else { tr!("по сети", "remote") };
            out.lines.push(OutLine::new(trf!("  {}  {:<8} {:<9} {} → {} [механика: {}]", "  {}  {:<8} {:<9} {} → {} [mechanism: {}]", hack::payload::id(v), v.service, scope, v.name, hack::access::result(ctx.world,v), hack::payload::kind(ctx.world, v)), OutKind::Warn).after(120));
        }
    }
    out.lines.push(OutLine::dim(tr!("Дальше: research адрес id · exploit new файл.adr адрес id · готовые инструменты: market", "Next: research address id · exploit new file.adr address id · ready-made tools: market")).after(100));
    out.lines.push(OutLine::heading(tr!("Пользователи:", "Users:")).after(200));
    out.lines.push(OutLine::new(format!("  {}", users.join(", ")), OutKind::Normal));
    out
}

// ---------------------------------------------------------------- exploit

pub fn research(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    let a = Args::parse(args, &["h"]);
    let Some(target) = a.pos.first() else { return CmdOutput::err(tr!("research [-h 1..4] адрес [id]: исследовать уязвимость и получить подсказку", "research [-h 1..4] address [id]: research a vulnerability and get a hint")); };
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
    out.push_kind(trf!("Результат взлома: {}", "Hack result: {}", p["result"]),OutKind::Accent);
    for condition in p["conditions"].as_array().unwrap(){out.push_kind(trf!("{} {} · сейчас {} · {}", "{} {} · now {} · {}", if condition["ok"]==true{"✓"}else{"✗"},condition["requirement"],condition["observed"],condition["hint"].as_str().unwrap_or("")),OutKind::Warn);}
    out.push_kind(serde_json::to_string_pretty(&p["layout"]).unwrap(), OutKind::Normal);
    for hint in p["hints"].as_array().unwrap().iter().take(level) { out.push_kind(hint.as_str().unwrap(), OutKind::Warn); }
    out.push_kind(trf!("Следующая подсказка: research -h {} {} {}", "Next hint: research -h {} {} {}", (level+1).min(4), target, p["id"].as_str().unwrap()), OutKind::Dim);
    out.push_kind(trf!("Свой код: exploit new ~/scripts/my_exploit.adr {} {}", "Your own code: exploit new ~/scripts/my_exploit.adr {} {}", target, p["id"].as_str().unwrap()), OutKind::Normal);
    if let Some(market)=p["market"].as_str(){out.push_kind(trf!("Готовый инструмент: market buy {}. Совместимость: {}.", "Ready-made tool: market buy {}. Compatibility: {}.", market,p["kind"].as_str().unwrap()),OutKind::Normal);}
    else{out.push_kind(tr!("Готовый эксплоит для этого типа не продаётся; создай свой по подсказкам.", "No ready-made exploit is sold for this type; write your own using the hints."),OutKind::Dim);}
    ctx.world.mark_scanned(id);
    out
}

pub fn exploit(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    if args.first().is_some_and(|s| s == "new") {
        if args.len() < 3 { return CmdOutput::err(tr!("exploit new файл.adr адрес [id]: создать заготовку и открыть редактор", "exploit new file.adr address [id]: create a template and open the editor")); }
        let path = ctx.path(&args[1]);
        if !path.ends_with(".adr") { return CmdOutput::err(tr!("нужно расширение .adr", "the .adr extension is required")); }
        if ctx.fs().exists(&path) { return CmdOutput::err(tr!("файл уже существует; открой его командой edit", "file already exists; open it with edit")); }
        let id = match resolve(ctx, "exploit new", &args[2]) { Ok(id) => id, Err(e) => return e };
        let v = match hack::payload::select(ctx.world, ctx.session, id, args.get(3).map(String::as_str)) {
            Ok(v) => v, Err(e) => return CmdOutput::err(e)
        };
        if let Err(e) = hack::payload::ensure_reachable(ctx.world, ctx.session, id, &v) { return CmdOutput::err(e); }
        let code = hack::payload::starter(&args[2], &hack::payload::id(&v));
        let user = ctx.user();
        if let Err(e) = ctx.fs_mut().write_text(&path, &code, &user) { return CmdOutput::err(e.to_string()); }
        ctx.actions.push(crate::types::UiAction::OpenApp { app: "editor".into(), arg: Some(path.clone()) });
        return CmdOutput::text(trf!("Создана заготовка {}. Заполни payload по подсказкам research; затем exploit -f {} {}.", "Template {} created. Fill in the payload using the research hints; then exploit -f {} {}.", path, path, args[2]));
    }
    if let Some(o) = locked_guard(ctx, "exploit") { return o; }
    let a = Args::parse(args, &["s", "f"]);
    let Some(file) = a.opt("f") else {
        return CmdOutput::err(tr!("Автоматический взлом отключён. 1) probe адрес; 2) research адрес id; 3) exploit new ~/scripts/my_exploit.adr адрес id; 4) заполни payload в редакторе; 5) exploit -f ~/scripts/my_exploit.adr адрес. Готовые эксплоиты: market.", "Automatic hacking is disabled. 1) probe address; 2) research address id; 3) exploit new ~/scripts/my_exploit.adr address id; 4) fill in the payload in the editor; 5) exploit -f ~/scripts/my_exploit.adr address. Ready-made exploits: market."));
    };
    let Some(target) = a.pos.first() else { return CmdOutput::err(tr!("exploit -f файл.adr [-s id] адрес", "exploit -f file.adr [-s id] address")); };
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
        None => return CmdOutput::err(tr!("crack: использование: crack [-w словарь] пользователь@адрес", "crack: usage: crack [-w wordlist] user@address")),
    };
    // пользователь@адрес, либо адрес и отдельный пользователь вторым словом
    let (user, host) = match spec.split_once('@') {
        Some((u, h)) if !u.is_empty() && !h.is_empty() => (u.to_string(), h.to_string()),
        _ => match a.pos.get(1) {
            Some(u) => (u.clone(), spec.clone()),
            None => return CmdOutput::err(tr!("crack: укажите пользователя: crack пользователь@адрес", "crack: specify a user: crack user@address")),
        },
    };
    let id = match resolve(ctx, "crack", &host) {
        Ok(id) => id,
        Err(o) => return o,
    };
    let wl_path = a.opt("w").map(|s| s.to_string()).unwrap_or_else(|| "~/wordlist.txt".to_string());
    let words = match read_wordlist(ctx, &wl_path) {
        Ok(w) if !w.is_empty() => w,
        Ok(_) => return CmdOutput::err(trf!("crack: словарь пуст: {}", "crack: wordlist is empty: {}", wl_path)),
        Err(e) => return CmdOutput::err(format!("crack: {}", e)),
    };
    let report = match hack::crack(ctx.world, ctx.session, id, &user, &words) {
        Ok(r) => r,
        Err(e) => return CmdOutput::err(format!("crack: {}", e)),
    };
    let mut out = CmdOutput::ok();
    out.push_kind(trf!("Подбор пароля {}@{} ({} слов) …", "Cracking password {}@{} ({} words) …", user, report.target_host, words.len()), OutKind::Accent);
    // прогресс: показываем попытки с задержкой, но не больше 40 строк
    let shown = report.attempts.len().min(40);
    let skip = report.attempts.len() - shown;
    if skip > 0 {
        out.lines.push(OutLine::dim(trf!("  … пропущено {} слов", "  … {} words skipped", skip)).after(200));
    }
    for w in &report.attempts[skip..] {
        out.lines.push(OutLine::dim(trf!("  пробую: {}", "  trying: {}", w)).after(70));
    }
    match &report.found {
        Some(pw) => {
            out.lines.push(OutLine::success(trf!("Пароль найден: {} (после {} попыток)", "Password found: {} (after {} attempts)", pw, report.tried)).after(300));
            out.lines.push(OutLine::dim(trf!("Теперь: ssh {}@{} {}", "Now: ssh {}@{} {}", user, report.target_host, pw)).after(150));
        }
        None => {
            out.lines.push(OutLine::error(trf!("Пароль не подобран ({} попыток).", "Password not cracked ({} attempts).", report.tried)).after(300));
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
    let Some(file) = args.first() else { return CmdOutput::err(tr!("privesc файл.adr [id]: запусти собственный локальный эксплоит. Сначала research текущий_хост id. Готовый: market buy exp_race.", "privesc file.adr [id]: run your own local exploit. First research current_host id. Ready-made: market buy exp_race.")); };
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
        out.push_kind(trf!("clearlogs: нужны права root на {}", "clearlogs: root access needed on {}", report.host), OutKind::Error);
        out.status = 1;
    } else if report.removed == 0 {
        out.push_kind(tr!("Следов не найдено.", "No traces found."), OutKind::Dim);
    } else {
        out.lines.push(OutLine::success(trf!("Убрано записей: {} на {}", "Entries removed: {} on {}", report.removed, report.host)).after(250));
    }
    out
}

// ---------------------------------------------------------------- proxy

pub fn proxy(ctx: &mut Ctx<'_>, _args: &[String]) -> CmdOutput {
    let info = hack::proxy_info(ctx.world, ctx.session);
    let mut out = CmdOutput::ok();
    out.push_kind(tr!("Цепочка подключений:", "Connection chain:"), OutKind::Heading);
    for (i, (id, user)) in info.chain.iter().enumerate() {
        let m = ctx.world.machine(*id);
        let mark = if info.monitored_hops.contains(id) { tr!("  ⚠ под наблюдением", "  ⚠ under surveillance") } else { "" };
        let arrow = if i == 0 { tr!("вы", "you") } else { tr!("→ ", "→  ") };
        out.push(format!("  {} {}@{} ({}){}", arrow, user, m.hostname, m.ip, mark));
    }
    out.push(trf!("Выходной адрес: {} ({})", "Exit address: {} ({})", info.exit_ip, info.exit_host));
    if info.hidden {
        out.push_kind(tr!("Реальный адрес скрыт за цепочкой.", "Your real address is hidden behind the chain."), OutKind::Success);
    } else {
        out.push_kind(tr!("Вы работаете со своей машины — реальный адрес виден.", "You are working from your own machine — your real address is visible."), OutKind::Warn);
    }
    if !info.monitored_hops.is_empty() {
        out.push_kind(tr!("Среди hop'ов есть наблюдаемые — риск пеленга.", "Some hops are under surveillance — risk of a trace."), OutKind::Warn);
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
    out.lines.push(OutLine::new(trf!("Подозрение: [{}] {:.0}/100", "Heat: [{}] {:.0}/100", bar, h), kind));
    if traced {
        out.push_kind(tr!("Вас уже пеленговали — будьте осторожнее.", "You've already been traced — be more careful."), OutKind::Error);
    }
    if evidence.is_empty() {
        out.push_kind(tr!("Улик с вашим адресом нет.", "No evidence with your address."), OutKind::Dim);
    } else {
        out.push_kind(tr!("Улики (ваш адрес в журналах):", "Evidence (your address in logs):"), OutKind::Heading);
        for (id, n) in &evidence {
            let m = ctx.world.machine(*id);
            out.push(trf!("  {} ({}) — {} записей", "  {} ({}) — {} entries", m.hostname, m.ip, n));
        }
        out.push_kind(tr!("Убрать: ssh на машину и clearlogs.", "To remove: ssh into the machine and clearlogs."), OutKind::Dim);
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
    out.push_kind(tr!("Расследование (слежка):", "Investigation (surveillance):"), OutKind::Heading);
    match &lead {
        Some(name) => out.push(trf!("  Ведёт: {}", "  Led by: {}", name)),
        None => out.push(tr!("  Ведущего органа в этой сети нет.", "  No agency is running cases in this network.")),
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
    out.lines.push(OutLine::new(trf!("  Стадия: {}  [{}] {:.0}/100", "  Stage: {}  [{}] {:.0}/100", inv.stage.title(), bar, inv.progress), kind));
    if inv.raids > 0 {
        out.push(trf!("  Рейдов было: {}", "  Raids so far: {}", inv.raids));
    }
    if hack::is_locked(ctx.world) {
        out.push_kind(trf!("  Канал заблокирован ещё {} с — переждите.", "  Channel blocked for another {} s — wait it out.", hack::lock_remaining(ctx.world)), OutKind::Error);
    }
    if inv.watched.is_empty() {
        out.push_kind(tr!("  Следствие ничего на вас не нашло.", "  The investigation has found nothing on you."), OutKind::Dim);
    } else {
        out.push_kind(tr!("  След виден на узлах:", "  Trail visible on hosts:"), OutKind::Heading);
        for id in &inv.watched {
            if ctx.world.has_machine(*id) {
                let m = ctx.world.machine(*id);
                out.push(format!("    {} ({})", m.hostname, m.ip));
            }
        }
        out.push_kind(tr!("  Зайдите на них и clearlogs — дело стихнет.", "  Log into them and clearlogs — the case will cool down."), OutKind::Dim);
    }
    out
}

// ---------------------------------------------------------------- creds

pub fn creds(ctx: &mut Ctx<'_>, _args: &[String]) -> CmdOutput {
    let creds = ctx.world.player.found_creds.clone();
    let mut out = CmdOutput::ok();
    if creds.is_empty() {
        out.push_kind(tr!("Найденных логинов пока нет.", "No logins found yet."), OutKind::Dim);
        return out;
    }
    out.push_kind(tr!("Найденные логины:", "Found logins:"), OutKind::Heading);
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
        None => return CmdOutput::err(tr!("firewall: укажите адрес узла", "firewall: specify the host address")),
    };
    let id = match resolve(ctx, "firewall", &target) {
        Ok(id) => id,
        Err(o) => return o,
    };
    match crate::hack::defense::bypass(ctx.world, ctx.session, id) {
        Ok(true) => {
            let mut out = CmdOutput::ok();
            out.lines.push(OutLine::success(trf!("Файрвол на {} обойдён — узел открыт.", "Firewall on {} bypassed — host is open.", ctx.world.machine(id).hostname)).after(300));
            out
        }
        Ok(false) => {
            let mut out = CmdOutput::ok();
            out.lines.push(OutLine::warn(tr!("Обойти не удалось — попробуй ещё (и следи за подозрением).", "Bypass failed — try again (and watch your heat).")).after(300));
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
            None => return CmdOutput::err(tr!("keylog: укажите адрес: keylog plant адрес", "keylog: specify an address: keylog plant address")),
        };
        let id = match resolve(ctx, "keylog", &addr) {
            Ok(id) => id,
            Err(o) => return o,
        };
        return match crate::hack::tools::plant_keylog(ctx.world, id) {
            Ok(()) => {
                let mut out = CmdOutput::ok();
                out.lines.push(OutLine::success(tr!("Перехватчик установлен. Через время поймает чужой логин — смотри keylog.", "Keylogger planted. In time it will catch someone's login — check keylog.")).after(300));
                out
            }
            Err(e) => CmdOutput::err(format!("keylog: {}", e)),
        };
    }
    // список
    let logs = ctx.world.keyloggers.clone();
    let mut out = CmdOutput::ok();
    out.push_kind(tr!("Перехватчики ввода:", "Keyloggers:"), OutKind::Heading);
    if logs.is_empty() {
        out.push_kind(tr!("  Нет. Поставить: keylog plant адрес (на взломанной машине).", "  None. Plant one: keylog plant address (on a hacked machine)."), OutKind::Dim);
        return out;
    }
    for k in &logs {
        let host = if ctx.world.has_machine(k.machine) { ctx.world.machine(k.machine).hostname.clone() } else { k.machine.to_string() };
        match &k.captured {
            Some(c) => out.lines.push(OutLine::new(trf!("  {:<16} поймано: {} / {}", "  {:<16} caught: {} / {}", host, c.user, c.password), OutKind::Success)),
            None => out.lines.push(OutLine::new(trf!("  {:<16} ждёт…", "  {:<16} waiting…", host), OutKind::Dim)),
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
            None => return CmdOutput::err(tr!("botnet: укажите адрес", "botnet: specify an address")),
        };
        let id = match resolve(ctx, "botnet", &addr) {
            Ok(id) => id,
            Err(o) => return o,
        };
        if a0 == Some("add") {
            return match crate::hack::tools::botnet_add(ctx.world, id) {
                Ok(()) => {
                    let mut out = CmdOutput::ok();
                    out.lines.push(OutLine::success(trf!("Узел {} в сети. Сеть тихо приносит ₵.", "Host {} joined. The network quietly brings in ₵.", ctx.world.machine(id).hostname)).after(250));
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
    out.push_kind(trf!("Ботнет: {} живых узлов", "Botnet: {} live hosts", size), OutKind::Heading);
    for id in ctx.world.botnet.clone() {
        if ctx.world.has_machine(id) {
            let m = ctx.world.machine(id);
            let state = if m.up { tr!("в сети", "online") } else { "offline" };
            out.push(format!("  {} ({}) — {}", m.hostname, m.ip, state));
        }
    }
    out.push_kind(trf!("Намайнено всего: {} ₵", "Mined in total: {} ₵", ctx.world.botnet_mined), OutKind::Heading);
    out.push_kind(tr!("Добавить: botnet add адрес (на взломанной машине).", "Add: botnet add address (on a hacked machine)."), OutKind::Dim);
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
        return CmdOutput::err(if enc { tr!("encrypt: encrypt файл ключ", "encrypt: encrypt file key") } else { tr!("decrypt: decrypt файл ключ", "decrypt: decrypt file key") });
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
        out.lines.push(OutLine::success(trf!("Зашифровано → {}", "Encrypted → {}", out_path)).after(200));
        out
    } else {
        match crate::hack::cipher::decrypt(&text, &key) {
            Ok(plain) => {
                let out_path = path.strip_suffix(".enc").map(|s| s.to_string()).unwrap_or_else(|| format!("{}.dec", path));
                if let Err(e) = ctx.fs_mut().write(&out_path, &plain, &user) {
                    return CmdOutput::err(format!("decrypt: {}", e));
                }
                let mut out = CmdOutput::ok();
                out.lines.push(OutLine::success(trf!("Расшифровано → {}", "Decrypted → {}", out_path)).after(200));
                for l in plain.lines().take(10) {
                    out.push(format!("  {}", l));
                }
                out
            }
            Err(e) => CmdOutput::err(format!("decrypt: {}", e)),
        }
    }
}
