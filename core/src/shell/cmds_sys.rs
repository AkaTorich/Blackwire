//! Системные команды: пользователи, процессы, окружение, сведения о машине,
//! журнал, выключение, выход.

use super::util::{human_size, table, Args};
use super::{help, table as cmdtable, CmdOutput, Ctx};
use crate::net::machine::LogKind;
use crate::procs::ProcKind;
use crate::types::{OutKind, UiAction};
use crate::world::{format_log, GameClock};

/// Переписать /etc/passwd по базе пользователей.
fn sync_passwd(ctx: &mut Ctx<'_>) {
    let text = ctx.machine().users.passwd_text();
    ctx.fs_mut().write_text("/etc/passwd", &text, "root").ok();
}

pub fn whoami(ctx: &mut Ctx<'_>, _args: &[String]) -> CmdOutput {
    CmdOutput::text(ctx.user())
}

pub fn id(ctx: &mut Ctx<'_>, _args: &[String]) -> CmdOutput {
    let user = ctx.user();
    match ctx.machine().users.get(&user) {
        Some(u) => CmdOutput::text(trf!(
            "uid={}({}) gid={}({}) группы={}{}",
            "uid={}({}) gid={}({}) groups={}{}",
            u.uid,
            u.name,
            u.uid,
            u.name,
            u.name,
            if u.is_admin { ",sudo" } else { "" }
        )),
        None => CmdOutput::text(format!("uid=?({})", user)),
    }
}

pub fn hostname(ctx: &mut Ctx<'_>, _args: &[String]) -> CmdOutput {
    CmdOutput::text(ctx.machine().hostname.clone())
}

pub fn uname(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    let a = Args::parse(args, &[]);
    let m = ctx.machine();
    if a.has("a") {
        CmdOutput::text(format!("Blackwire {} 6.1.0-bw #1 SMP x86_64 {}", m.hostname, m.os))
    } else if a.has("r") {
        CmdOutput::text("6.1.0-bw")
    } else if a.has("n") {
        CmdOutput::text(m.hostname.clone())
    } else {
        CmdOutput::text("Blackwire")
    }
}

pub fn date(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    let now = ctx.now();
    if args.first().map(|s| s == "+%s").unwrap_or(false) {
        return CmdOutput::text(now.to_string());
    }
    let (y, m, d, _, _, _) = GameClock::civil(now);
    let wd = GameClock::weekday(y, m, d);
    CmdOutput::text(trf!(
        "{}, {} {} {} г., {}",
        "{}, {} {} {}, {}",
        weekday_name(wd),
        d,
        genitive_month(m),
        y,
        GameClock::format_hms(now)
    ))
}

fn weekday_name(wd: u32) -> &'static str {
    const W_EN: [&str; 7] = ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday"];
    if crate::i18n::is_en() { W_EN[(wd % 7) as usize] } else { GameClock::weekday_name_ru(wd) }
}

fn genitive_month(m: u32) -> &'static str {
    const M: [&str; 12] = [
        "января", "февраля", "марта", "апреля", "мая", "июня", "июля", "августа", "сентября", "октября", "ноября",
        "декабря",
    ];
    const M_EN: [&str; 12] = [
        "January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November",
        "December",
    ];
    let i = ((m.max(1) - 1) % 12) as usize;
    if crate::i18n::is_en() { M_EN[i] } else { M[i] }
}

pub fn uptime(ctx: &mut Ctx<'_>, _args: &[String]) -> CmdOutput {
    let st = crate::sysinfo::status(ctx.world, ctx.mid());
    let load = st.cpu / 100.0 * 4.0;
    CmdOutput::text(trf!(
        " {} работает {}, пользователей: 1, нагрузка: {:.2}, {:.2}, {:.2}",
        " {} up {}, 1 user, load average: {:.2}, {:.2}, {:.2}",
        GameClock::format_hms(ctx.now()),
        GameClock::format_duration(st.uptime),
        load,
        load * 0.9,
        load * 0.8
    ))
}

fn proc_rows(ctx: &Ctx<'_>, all: bool, sort_cpu: bool, limit: usize) -> Vec<Vec<String>> {
    let user = ctx.user();
    let mut procs: Vec<&crate::procs::Proc> =
        ctx.machine().procs.list().iter().filter(|p| all || user == "root" || p.user == user).collect();
    if sort_cpu {
        procs.sort_by(|a, b| b.cpu.partial_cmp(&a.cpu).unwrap_or(std::cmp::Ordering::Equal));
    }
    let mut rows = vec![vec!["PID".to_string(), tr!("ПОЛЬЗ.", "USER").into(), "%CPU".into(), tr!("ПАМ.МБ", "MEM.MB").into(), tr!("ЗАПУЩЕН", "STARTED").into(), tr!("КОМАНДА", "COMMAND").into()]];
    for p in procs.into_iter().take(limit) {
        let name = match p.kind {
            ProcKind::Script { .. } if p.killed => trf!("{} (останавливается)", "{} (stopping)", p.name),
            _ => p.name.clone(),
        };
        rows.push(vec![
            p.pid.to_string(),
            p.user.clone(),
            format!("{:.1}", p.cpu),
            p.mem_mb.to_string(),
            GameClock::format_hms(p.started),
            name,
        ]);
    }
    rows
}

pub fn ps(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    let a = Args::parse(args, &[]);
    let all = a.has("a") || a.has("e") || a.has("A") || a.pos.iter().any(|p| p == "aux" || p == "ax");
    let rows = proc_rows(ctx, all, false, 500);
    let mut out = CmdOutput::ok();
    let lines = table(&rows, &[true, false, true, true, false, false]);
    for (i, l) in lines.into_iter().enumerate() {
        if i == 0 {
            out.push_kind(l, OutKind::Heading);
        } else {
            out.push(l);
        }
    }
    out
}

pub fn top(ctx: &mut Ctx<'_>, _args: &[String]) -> CmdOutput {
    let st = crate::sysinfo::status(ctx.world, ctx.mid());
    let mut out = CmdOutput::ok();
    out.push_kind(
        trf!(
            "top - {}  работает {}  задач: {}  нагрузка: {:.2}",
            "top - {}  up {}  tasks: {}  load average: {:.2}",
            GameClock::format_hms(ctx.now()),
            GameClock::format_duration(st.uptime),
            st.procs.len(),
            st.cpu / 100.0 * 4.0
        ),
        OutKind::Accent,
    );
    out.push(trf!("%Cpu: {:5.1} польз., {:5.1} сист., {:5.1} свободно", "%Cpu: {:5.1} us, {:5.1} sy, {:5.1} id", st.cpu * 0.7, st.cpu * 0.3, 100.0 - st.cpu));
    out.push(trf!("Память МБ: {:6} всего, {:6} занято, {:6} свободно", "MiB Mem: {:6} total, {:6} used, {:6} free", st.mem_total, st.mem_used, st.mem_total - st.mem_used));
    out.push("");
    let rows = proc_rows(ctx, true, true, 15);
    for (i, l) in table(&rows, &[true, false, true, true, false, false]).into_iter().enumerate() {
        if i == 0 {
            out.push_kind(l, OutKind::Heading);
        } else {
            out.push(l);
        }
    }
    out
}

pub fn kill(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    let a = Args::parse(args, &[]);
    if a.pos.is_empty() {
        return CmdOutput::err(tr!("kill: укажите pid (список — ps)", "kill: specify a pid (list — ps)"));
    }
    let user = ctx.user();
    let mut out = CmdOutput::ok();
    for p in &a.pos {
        let pid: u32 = match p.parse() {
            Ok(n) => n,
            Err(_) => {
                out.push_err(trf!("kill: не число: {}", "kill: not a number: {}", p));
                continue;
            }
        };
        let (is_script, name) = match ctx.machine().procs.get(pid) {
            Some(pr) => (matches!(pr.kind, ProcKind::Script { .. }), pr.name.clone()),
            None => {
                out.push_err(trf!("kill: нет процесса с pid {}", "kill: no process with pid {}", pid));
                continue;
            }
        };
        if is_script {
            // проверка прав как при снятии, но процесс остаётся до остановки потока
            if let Err(e) = ctx.machine().procs.list().iter().find(|x| x.pid == pid).map(|x| {
                if user != "root" && x.user != user {
                    Err(trf!("отказано в доступе: процесс {} принадлежит {}", "permission denied: process {} belongs to {}", pid, x.user))
                } else {
                    Ok(())
                }
            }).unwrap_or(Ok(())) {
                out.push_err(format!("kill: {}", e));
                continue;
            }
            ctx.machine_mut().procs.mark_killed(pid);
            out.push_kind(trf!("скрипту {} (pid {}) велено остановиться", "script {} (pid {}) told to stop", name, pid), OutKind::Dim);
        } else {
            match ctx.machine_mut().procs.kill_as(pid, &user) {
                Ok(()) => {
                    ctx.log(LogKind::Exec, &trf!("{} снял процесс {} ({})", "{} killed process {} ({})", user, pid, name));
                }
                Err(e) => out.push_err(format!("kill: {}", e)),
            }
        }
    }
    out
}

pub fn free(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    let a = Args::parse(args, &[]);
    let human = a.has("h");
    let st = crate::sysinfo::status(ctx.world, ctx.mid());
    let f = |mb: u32| if human { human_size(mb as u64 * 1024 * 1024) } else { (mb as u64 * 1024).to_string() };
    let rows = vec![
        vec!["".to_string(), tr!("всего", "total").into(), tr!("занято", "used").into(), tr!("свободно", "free").into(), tr!("кэш", "cache").into()],
        vec![tr!("Память:", "Mem:").to_string(), f(st.mem_total), f(st.mem_used), f(st.mem_total - st.mem_used), f(st.mem_used / 4)],
        vec![tr!("Подкачка:", "Swap:").to_string(), f(st.mem_total / 2), f(0), f(st.mem_total / 2), f(0)],
    ];
    let mut out = CmdOutput::ok();
    out.push_lines(table(&rows, &[false, true, true, true, true]));
    out
}

pub fn env(ctx: &mut Ctx<'_>, _args: &[String]) -> CmdOutput {
    let mut out = CmdOutput::ok();
    for (k, v) in &ctx.session.env {
        out.push(format!("{}={}", k, v));
    }
    out
}

pub fn export(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    if args.is_empty() {
        return env(ctx, args);
    }
    let mut out = CmdOutput::ok();
    for a in args {
        match a.split_once('=') {
            Some((k, v)) if !k.is_empty() => {
                ctx.session.env.insert(k.to_string(), v.to_string());
            }
            _ => {
                if !ctx.session.env.contains_key(a) {
                    ctx.session.env.insert(a.clone(), String::new());
                }
                out.push_kind(trf!("{}: значение не задано (export ИМЯ=значение)", "{}: no value set (export NAME=value)", a), OutKind::Dim);
            }
        }
    }
    out
}

pub fn unset(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    for a in args {
        ctx.session.env.remove(a);
    }
    CmdOutput::ok()
}

pub fn alias(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    let mut out = CmdOutput::ok();
    if args.is_empty() {
        for (k, v) in &ctx.session.aliases {
            out.push(format!("alias {}='{}'", k, v));
        }
        return out;
    }
    let joined = args.join(" ");
    match joined.split_once('=') {
        Some((k, v)) if !k.trim().is_empty() => {
            let v = v.trim().trim_matches(|c| c == '\'' || c == '"').to_string();
            ctx.session.aliases.insert(k.trim().to_string(), v);
        }
        _ => match ctx.session.aliases.get(&joined) {
            Some(v) => out.push(format!("alias {}='{}'", joined, v)),
            None => out.push_err(trf!("alias: {}: не найден", "alias: {}: not found", joined)),
        },
    }
    out
}

pub fn unalias(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    let mut out = CmdOutput::ok();
    for a in args {
        if ctx.session.aliases.remove(a).is_none() {
            out.push_err(trf!("unalias: {}: не найден", "unalias: {}: not found", a));
        }
    }
    out
}

pub fn history(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    let a = Args::parse(args, &[]);
    if a.has("c") {
        ctx.session.history.clear();
        return CmdOutput::ok();
    }
    let mut out = CmdOutput::ok();
    for (i, h) in ctx.session.history.iter().enumerate() {
        out.push(format!("{:5}  {}", i + 1, h));
    }
    out
}

pub fn clear(ctx: &mut Ctx<'_>, _args: &[String]) -> CmdOutput {
    ctx.actions.push(UiAction::Clear);
    CmdOutput::ok()
}

pub fn help(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    let a = Args::parse(args, &[]);
    if a.has("w") || a.has("window") {
        ctx.actions.push(UiAction::OpenApp { app: "help".to_string(), arg: None });
        return CmdOutput::text(tr!("открываю окно справки", "opening the help window"));
    }
    if let Some(name) = a.pos.first() {
        return match cmdtable::lookup(name) {
            Some(info) => help::command_help(info),
            None => CmdOutput::err(trf!("help: нет команды «{}»", "help: no command '{}'", name)),
        };
    }
    let mut out = CmdOutput::ok();
    for l in help::help_table() {
        if l.starts_with("== ") {
            out.push_kind(l.trim_start_matches("== ").to_string(), OutKind::Heading);
        } else {
            out.push(l);
        }
    }
    out
}

pub fn man(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    match args.first() {
        None => CmdOutput::err(tr!("man: какую страницу показать? Например: man ls", "man: which page? For example: man ls")),
        Some(name) => match cmdtable::lookup(name) {
            Some(info) => help::command_help(info),
            None => {
                let _ = ctx;
                CmdOutput::err(trf!("man: нет справки для «{}»", "man: no manual entry for '{}'", name))
            }
        },
    }
}

pub fn which(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    let mut out = CmdOutput::ok();
    for name in args {
        if ctx.session.aliases.contains_key(name) {
            out.push(trf!("{}: псевдоним → {}", "{}: alias → {}", name, ctx.session.aliases[name]));
        } else if cmdtable::lookup(name).is_some() {
            let path = if ctx.fs().exists(&format!("/bin/{}", name)) {
                format!("/bin/{}", name)
            } else if ctx.fs().exists(&format!("/usr/bin/{}", name)) {
                format!("/usr/bin/{}", name)
            } else {
                trf!("/bin/{} (встроенная)", "/bin/{} (built-in)", name)
            };
            out.push(path);
        } else {
            out.push_err(trf!("{}: не найдено", "{}: not found", name));
        }
    }
    out
}

pub fn passwd(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    let me = ctx.user();
    let (target, newpass) = match args.len() {
        0 => return CmdOutput::err(tr!("passwd: укажите новый пароль: passwd новый_пароль (или passwd пользователь новый_пароль)", "passwd: specify a new password: passwd new_password (or passwd user new_password)")),
        1 => (me.clone(), args[0].clone()),
        _ => (args[0].clone(), args[1].clone()),
    };
    if target != me && me != "root" {
        return CmdOutput::err(trf!("passwd: сменить пароль {} может только root", "passwd: only root can change the password of {}", target));
    }
    if !ctx.machine().users.exists(&target) {
        return CmdOutput::err(trf!("passwd: нет пользователя «{}»", "passwd: no user '{}'", target));
    }
    ctx.machine_mut().users.set_password(&target, &newpass);
    ctx.log(LogKind::Warn, &trf!("{} сменил пароль пользователя {}", "{} changed the password of user {}", me, target));
    let mut out = CmdOutput::ok();
    out.push_kind(trf!("passwd: пароль пользователя {} обновлён", "passwd: password for user {} updated", target), OutKind::Success);
    out
}

pub fn useradd(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    let a = Args::parse(args, &[]);
    if !ctx.is_admin() {
        return CmdOutput::err(tr!("useradd: нужны права администратора (sudo useradd …)", "useradd: administrator rights required (sudo useradd …)"));
    }
    let name = match a.pos.first() {
        Some(n) => n.clone(),
        None => return CmdOutput::err(tr!("useradd: укажите имя: useradd имя [пароль]", "useradd: specify a name: useradd name [password]")),
    };
    if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') || name.is_empty() {
        return CmdOutput::err(tr!("useradd: имя — латинские буквы, цифры, «_» и «-»", "useradd: name may contain Latin letters, digits, '_' and '-'"));
    }
    if ctx.machine().users.exists(&name) {
        return CmdOutput::err(trf!("useradd: пользователь «{}» уже есть", "useradd: user '{}' already exists", name));
    }
    let password = a.pos.get(1).cloned().unwrap_or_default();
    let admin = a.has("a") || a.has("admin");
    ctx.machine_mut().users.add(&name, &password, admin);
    ctx.fs_mut().add_user_home(&name);
    sync_passwd(ctx);
    let me = ctx.user();
    ctx.log(LogKind::Warn, &trf!("{} добавил пользователя {}", "{} added user {}", me, name));
    let mut out = CmdOutput::ok();
    out.push_kind(trf!("пользователь {} создан, дом /home/{}", "user {} created, home /home/{}", name, name), OutKind::Success);
    if password.is_empty() {
        out.push_kind(tr!("пароль пустой — задайте его: passwd имя пароль", "password is empty — set it: passwd name password"), OutKind::Dim);
    }
    out
}

pub fn userdel(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    if !ctx.is_admin() {
        return CmdOutput::err(tr!("userdel: нужны права администратора", "userdel: administrator rights required"));
    }
    let name = match args.first() {
        Some(n) => n.clone(),
        None => return CmdOutput::err(tr!("userdel: укажите имя", "userdel: specify a name")),
    };
    if name == "root" {
        return CmdOutput::err(tr!("userdel: root удалить нельзя", "userdel: root cannot be deleted"));
    }
    if name == ctx.session.user() {
        return CmdOutput::err(tr!("userdel: нельзя удалить себя", "userdel: you cannot delete yourself"));
    }
    if !ctx.machine_mut().users.remove(&name) {
        return CmdOutput::err(trf!("userdel: нет пользователя «{}»", "userdel: no user '{}'", name));
    }
    sync_passwd(ctx);
    let me = ctx.user();
    ctx.log(LogKind::Warn, &trf!("{} удалил пользователя {}", "{} deleted user {}", me, name));
    CmdOutput::ok()
}

pub fn users(ctx: &mut Ctx<'_>, _args: &[String]) -> CmdOutput {
    let rows: Vec<Vec<String>> = ctx
        .machine()
        .users
        .list()
        .iter()
        .map(|u| vec![u.name.clone(), u.uid.to_string(), u.home.clone(), if u.is_admin { tr!("администратор", "administrator").into() } else { String::new() }])
        .collect();
    let mut out = CmdOutput::ok();
    out.push_lines(table(&rows, &[false, true, false, false]));
    out
}

pub fn su(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    let (target, password) = match args.len() {
        0 => return CmdOutput::err(tr!("su: укажите пароль: su [пользователь] пароль", "su: specify a password: su [user] password")),
        1 => {
            if ctx.machine().users.exists(&args[0]) {
                return CmdOutput::err(trf!("su: укажите пароль третьим словом: su {} пароль", "su: give the password as the third word: su {} password", args[0]));
            }
            ("root".to_string(), args[0].clone())
        }
        _ => (args[0].clone(), args[1].clone()),
    };
    let me = ctx.user();
    if !ctx.machine().users.exists(&target) {
        return CmdOutput::err(trf!("su: нет пользователя «{}»", "su: no user '{}'", target));
    }
    if !ctx.machine().users.check_password(&target, &password) && me != "root" {
        ctx.log(LogKind::LoginFailed, &trf!("su: {} → {}: неверный пароль", "su: {} → {}: wrong password", me, target));
        return CmdOutput::err(tr!("su: неверный пароль", "su: wrong password"));
    }
    ctx.log(LogKind::Login, &format!("su: {} → {}", me, target));
    let home = ctx.machine().users.home_of(&target);
    ctx.session.current_mut().user = target.clone();
    ctx.session.set_cwd(&home);
    ctx.session.env.insert("USER".to_string(), target);
    ctx.session.env.insert("HOME".to_string(), home);
    CmdOutput::ok()
}

pub fn sudo(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    if args.is_empty() {
        return CmdOutput::err(tr!("sudo: укажите команду", "sudo: specify a command"));
    }
    let me = ctx.user();
    if !ctx.is_admin() {
        ctx.log(LogKind::Warn, &trf!("sudo: {} не в списке sudoers: {}", "sudo: {} is not in the sudoers file: {}", me, args.join(" ")));
        return CmdOutput::err(trf!("sudo: пользователь {} не в списке sudoers. Об этом будет сообщено.", "sudo: user {} is not in the sudoers file. This incident will be reported.", me));
    }
    ctx.log(LogKind::Exec, &format!("sudo: {} → {}", me, args.join(" ")));
    ctx.sudo = true;
    let out = super::run_simple(ctx, args);
    ctx.sudo = false;
    out
}

fn power(ctx: &mut Ctx<'_>, reboot: bool) -> CmdOutput {
    let mut out = CmdOutput::ok();
    let me = ctx.user();
    if !ctx.is_admin() {
        return CmdOutput::err(trf!("{}: нужны права администратора", "{}: administrator rights required", if reboot { "reboot" } else { "shutdown" }));
    }
    if ctx.mid() == 0 {
        ctx.log(LogKind::Service, &format!("{}: {}", if reboot { tr!("перезагрузка", "reboot") } else { tr!("выключение", "shutdown") }, me));
        if !reboot {
            out.push_kind(tr!("Сохранение прогресса и выход из игры…", "Saving progress and quitting the game…"), OutKind::Warn);
            ctx.actions.push(UiAction::QuitGame);
            return out;
        }
        out.push_kind(tr!("Система уходит на перезагрузку…", "The system is going down for reboot…"), OutKind::Warn);
        out.lines.push(crate::types::OutLine::dim(tr!("[ ок ] остановка служб", "[ ok ] stopping services")).after(300));
        out.lines.push(crate::types::OutLine::dim(tr!("[ ок ] отключение сети", "[ ok ] bringing down network")).after(300));
        out.lines.push(crate::types::OutLine::dim(tr!("[ ок ] синхронизация дисков", "[ ok ] syncing disks")).after(300));
        out.lines.push(crate::types::OutLine::success(tr!("Blackwire OS 1.0 — система запущена", "Blackwire OS 1.0 — system started")).after(900));
        ctx.world.reboot_player();
        ctx.actions.push(UiAction::Clear);
        return out;
    }
    let secs = if reboot { 90 } else { 600 };
    let host = ctx.machine().hostname.clone();
    ctx.log(LogKind::Service, &trf!("{} выполнил {}", "{} ran {}", me, if reboot { "reboot" } else { "shutdown" }));
    let id = ctx.mid();
    ctx.world.shutdown_machine(id, secs);
    ctx.session.pop_hop(ctx.world);
    out.push_kind(trf!("{}: система останавливается…", "{}: the system is going down…", host), OutKind::Warn);
    out.lines.push(crate::types::OutLine::dim(trf!("Соединение с {} закрыто удалённой стороной.", "Connection to {} closed by remote host.", host)).after(600));
    if reboot {
        out.push_kind(tr!("машина поднимется через полторы минуты игрового времени", "the machine will be back up in a minute and a half of game time"), OutKind::Dim);
    } else {
        out.push_kind(tr!("машина выключена; её включат через десять минут игрового времени", "the machine is off; it will be powered on in ten minutes of game time"), OutKind::Dim);
    }
    out
}

pub fn shutdown(ctx: &mut Ctx<'_>, _args: &[String]) -> CmdOutput {
    power(ctx, false)
}

pub fn reboot(ctx: &mut Ctx<'_>, _args: &[String]) -> CmdOutput {
    power(ctx, true)
}

pub fn logs(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    let a = Args::parse(args, &["n", "f"]);
    if !ctx.is_admin() {
        return CmdOutput::err(tr!("logs: журнал читает только администратор (sudo logs)", "logs: only an administrator can read the log (sudo logs)"));
    }
    let n = a.opt_usize("n", 25);
    let mut out = CmdOutput::ok();
    if let Some(f) = a.opt("f") {
        let path = match f {
            "auth" => "/var/log/auth.log",
            "system" | "sys" => "/var/log/system.log",
            other => return CmdOutput::err(trf!("logs: нет журнала «{}» (auth или system)", "logs: no log '{}' (auth or system)", other)),
        };
        let text = ctx.fs().read_text(path).unwrap_or_default();
        let lines: Vec<&str> = text.lines().collect();
        let skip = lines.len().saturating_sub(n);
        out.push_lines(lines.into_iter().skip(skip).map(|s| s.to_string()));
        return out;
    }
    let logs = &ctx.machine().logs;
    if logs.is_empty() {
        out.push_kind(tr!("журнал пуст", "log is empty"), OutKind::Dim);
        return out;
    }
    let skip = logs.len().saturating_sub(n);
    for e in logs.iter().skip(skip) {
        let kind = match e.kind {
            LogKind::LoginFailed | LogKind::Warn => OutKind::Warn,
            LogKind::Login | LogKind::Logout => OutKind::Accent,
            _ => OutKind::Normal,
        };
        out.push_kind(format_log(e), kind);
    }
    out
}

pub fn exit(ctx: &mut Ctx<'_>, _args: &[String]) -> CmdOutput {
    if ctx.session.at_home() {
        ctx.actions.push(UiAction::Exit);
        return CmdOutput::text(tr!("выход", "logout"));
    }
    let me = ctx.session.user().to_string();
    let host = ctx.machine().hostname.clone();
    ctx.log(LogKind::Logout, &trf!("{} отключился", "{} disconnected", me));
    ctx.session.pop_hop(ctx.world);
    let mut out = CmdOutput::ok();
    out.push_kind(trf!("Соединение с {} закрыто.", "Connection to {} closed.", host), OutKind::Dim);
    out
}

pub fn cmd_true(_ctx: &mut Ctx<'_>, _args: &[String]) -> CmdOutput {
    CmdOutput::ok()
}

pub fn cmd_false(_ctx: &mut Ctx<'_>, _args: &[String]) -> CmdOutput {
    CmdOutput { lines: Vec::new(), status: 1 }
}
