//! Связь скрипта Adder с миром: модули `sys`, `fs`, `net`, `shell` (см. docs/ADDER.md).
//! Язык живёт в отдельном крейте и ничего о мире не знает; оболочка Tauri
//! получает от него вызов `Host::call("fs.read", args)` и приносит сюда
//! аргументы в JSON. Ответ — тоже JSON. Ошибка — строка по-русски, в скрипте
//! она становится исключением для `try/except`.

use serde_json::{json, Value};

use crate::game::Game;
use crate::net::machine::{LogKind, MachineId};
use crate::types::{OutKind, UiAction};

type R = Result<Value, String>;

fn arg_str(args: &[Value], i: usize, what: &str) -> Result<String, String> {
    match args.get(i) {
        Some(Value::String(s)) => Ok(s.clone()),
        Some(Value::Null) | None => Err(trf!("нужен аргумент: {}", "argument required: {}", what)),
        Some(v) => Ok(v.to_string().trim_matches('"').to_string()),
    }
}

fn arg_str_or(args: &[Value], i: usize, default: &str) -> String {
    match args.get(i) {
        Some(Value::String(s)) => s.clone(),
        Some(Value::Null) | None => default.to_string(),
        Some(v) => v.to_string().trim_matches('"').to_string(),
    }
}

fn arg_int(args: &[Value], i: usize, what: &str) -> Result<i64, String> {
    match args.get(i) {
        Some(Value::Number(n)) => n.as_i64().or_else(|| n.as_f64().map(|f| f as i64)).ok_or_else(|| trf!("нужно число: {}", "number required: {}", what)),
        Some(Value::String(s)) => s.trim().parse::<i64>().map_err(|_| trf!("нужно число: {}", "number required: {}", what)),
        _ => Err(trf!("нужно число: {}", "number required: {}", what)),
    }
}

/// Машина и пользователь, с которыми работает скрипт (текущий hop его сеанса).
fn whoami(game: &Game, session: u32) -> (MachineId, String) {
    game.target(Some(session))
}

/// Вызов игровой функции по полному имени.
pub fn call(game: &mut Game, session: u32, pid: u32, name: &str, args: Vec<Value>) -> R {
    game.sync_exploit_presence();
    let (module, func) = name.split_once('.').ok_or_else(|| trf!("неизвестная функция: {}", "unknown function: {}", name))?;
    let stamp=crate::plugins::events::Stamp::new(&game.world);
    let result = match module {
        "sys" => call_sys(game, session, pid, func, &args),
        "fs" => call_fs(game, session, func, &args),
        "net" => call_net(game, session, func, &args),
        "shell" => call_shell(game, session, func, &args),
        "time" => match func {
            "now" => Ok(json!(game.world.now())),
            _ => Err(trf!("нет функции time.{}", "no function time.{}", func)),
        },
        _ => Err(trf!("неизвестный модуль: {}", "unknown module: {}", module)),
    };
    game.plugin_changes(stamp);
    if module=="fs" && !["read","list","exists","is_dir"].contains(&func) && result.is_ok(){game.plugin_event("on_file",json!({"operation":func,"args":args,"session":session}));}
    result
}

// ---------------------------------------------------------------- sys

fn call_sys(game: &mut Game, session: u32, pid: u32, func: &str, args: &[Value]) -> R {
    let (mid, user) = whoami(game, session);
    match func {
        "hostname" => Ok(json!(game.world.machine(mid).hostname)),
        "whoami" => Ok(json!(user)),
        "is_root" => Ok(json!(user == "root" || game.world.machine(mid).users.is_admin(&user))),
        "cwd" => Ok(json!(game.sessions.get(&session).map(|s| s.cwd.clone()).unwrap_or_else(|| "/".into()))),
        "env" => {
            let key = arg_str(args, 0, tr!("имя переменной", "variable name"))?;
            Ok(game.sessions.get(&session).and_then(|s| s.env.get(&key)).map(|v| json!(v)).unwrap_or(Value::Null))
        }
        "set_env" => {
            let key = arg_str(args, 0, tr!("имя переменной", "variable name"))?;
            let val = arg_str_or(args, 1, "");
            if let Some(s) = game.sessions.get_mut(&session) {
                s.env.insert(key, val);
            }
            Ok(Value::Null)
        }
        "users" => Ok(json!(game.world.machine(mid).users.names())),
        "procs" => {
            let list: Vec<Value> = game
                .world
                .machine(mid)
                .procs
                .list()
                .iter()
                .map(|p| json!({ "pid": p.pid, "name": p.name, "user": p.user, "cpu": p.cpu, "mem": p.mem_mb }))
                .collect();
            Ok(Value::Array(list))
        }
        "kill" => {
            let target = arg_int(args, 0, "pid")? as u32;
            if target == pid {
                return Err(tr!("скрипт не может снять сам себя — используйте exit()", "a script cannot kill itself — use exit()").into());
            }
            game.world.machine_mut(mid).procs.kill_as(target, &user)?;
            Ok(json!(true))
        }
        "uptime" => Ok(json!(game.world.uptime())),
        "notify" => {
            let title = arg_str(args, 0, tr!("заголовок", "title"))?;
            let text = arg_str_or(args, 1, "");
            game.notify(&title, &text);
            Ok(Value::Null)
        }
        "open" => {
            let app = arg_str(args, 0, tr!("имя приложения", "app name"))?;
            let arg = args.get(1).and_then(|v| v.as_str()).map(|s| s.to_string());
            game.pending_ui.push(UiAction::OpenApp { app, arg });
            Ok(Value::Null)
        }
        "pid" => Ok(json!(pid)),
        "lang" => Ok(json!(crate::i18n::code())),
        "money" => Ok(json!(crate::missions::money(&game.world))),
        "debt" => Ok(json!(game.world.finance.debt)),
        "missions" => {
            crate::missions::ensure_init(&mut game.world);
            let list: Vec<Value> = crate::missions::rows(&game.world)
                .iter()
                .map(|r| {
                    let st = match r.status {
                        crate::missions::MissionStatus::Done => "done",
                        crate::missions::MissionStatus::Active => "active",
                        crate::missions::MissionStatus::Locked => "locked",
                    };
                    json!({ "code": r.code, "title": r.title, "status": st, "objective": r.objective, "reward": r.reward })
                })
                .collect();
            Ok(Value::Array(list))
        }
        _ => Err(trf!("нет функции sys.{}", "no function sys.{}", func)),
    }
}

// ---------------------------------------------------------------- fs

fn call_fs(game: &mut Game, session: u32, func: &str, args: &[Value]) -> R {
    let s = Some(session);
    match func {
        "read" => Ok(json!(game.fs_read(s, &arg_str(args, 0, tr!("путь", "path"))?)?)),
        "write" => {
            game.fs_write(s, &arg_str(args, 0, tr!("путь", "path"))?, &arg_str_or(args, 1, ""))?;
            Ok(Value::Null)
        }
        "append" => {
            let (mid, user) = whoami(game, session);
            let p = game.fs_path(s, &arg_str(args, 0, tr!("путь", "path"))?);
            game.world.machine_mut(mid).fs.append(&p, &arg_str_or(args, 1, ""), &user).map_err(|e| e.to_string())?;
            Ok(Value::Null)
        }
        "list" => {
            let path = arg_str_or(args, 0, ".");
            let list: Vec<Value> = game
                .fs_list(s, &path)?
                .into_iter()
                .map(|e| json!({ "name": e.name, "path": e.path, "is_dir": e.is_dir, "size": e.size, "owner": e.owner, "mode": e.mode, "modified": e.modified }))
                .collect();
            Ok(Value::Array(list))
        }
        "exists" => Ok(json!(game.fs_exists(s, &arg_str(args, 0, tr!("путь", "path"))?))),
        "is_dir" => {
            let (mid, _) = whoami(game, session);
            let p = game.fs_path(s, &arg_str(args, 0, tr!("путь", "path"))?);
            Ok(json!(game.world.machine(mid).fs.is_dir(&p)))
        }
        "mkdir" => {
            game.fs_mkdir(s, &arg_str(args, 0, tr!("путь", "path"))?)?;
            Ok(Value::Null)
        }
        "delete" => {
            game.fs_delete(s, &arg_str(args, 0, tr!("путь", "path"))?)?;
            Ok(Value::Null)
        }
        "move" => {
            game.fs_rename(s, &arg_str(args, 0, tr!("откуда", "source"))?, &arg_str(args, 1, tr!("куда", "destination"))?)?;
            Ok(Value::Null)
        }
        "copy" => {
            game.fs_copy(s, &arg_str(args, 0, tr!("откуда", "source"))?, &arg_str(args, 1, tr!("куда", "destination"))?)?;
            Ok(Value::Null)
        }
        "chmod" => {
            let (mid, user) = whoami(game, session);
            let p = game.fs_path(s, &arg_str(args, 0, tr!("путь", "path"))?);
            let mode_s = arg_str(args, 1, tr!("права", "mode"))?;
            let current = game.world.machine(mid).fs.stat(&p).map(|st| st.mode).unwrap_or(0o644);
            let mode = crate::vfs::parse_mode(&mode_s, current).ok_or_else(|| trf!("непонятные права: {}", "invalid mode: {}", mode_s))?;
            game.world.machine_mut(mid).fs.chmod(&p, mode, &user).map_err(|e| e.to_string())?;
            Ok(Value::Null)
        }
        "chown" => {
            let (mid, user) = whoami(game, session);
            let p = game.fs_path(s, &arg_str(args, 0, tr!("путь", "path"))?);
            let owner = arg_str(args, 1, tr!("владелец", "owner"))?;
            game.world.machine_mut(mid).fs.chown(&p, &owner, &user).map_err(|e| e.to_string())?;
            Ok(Value::Null)
        }
        _ => Err(trf!("нет функции fs.{}", "no function fs.{}", func)),
    }
}

// ---------------------------------------------------------------- net

fn call_net(game: &mut Game, session: u32, func: &str, args: &[Value]) -> R {
    let (mid, _user) = whoami(game, session);
    // при активном рейде исходящие атаки заблокированы
    if matches!(func, "connect" | "exploit" | "crack") && crate::hack::is_locked(&game.world) {
        return Err(trf!("канал под наблюдением после рейда — ещё {} с", "channel under surveillance after the raid — another {} s", crate::hack::lock_remaining(&game.world)));
    }
    match func {
        "my_ip" => {
            let m = game.world.machine(mid);
            Ok(json!(m.lan_ip.clone().unwrap_or_else(|| m.ip.clone())))
        }
        "resolve" => Ok(game.world.resolve(&arg_str(args, 0, tr!("имя", "name"))?).map(|ip| json!(ip)).unwrap_or(Value::Null)),
        "ping" => {
            let addr = arg_str(args, 0, tr!("адрес", "address"))?;
            match game.world.lookup(mid, &addr) {
                Ok(id) => {
                    game.world.mark_known(id);
                    let ms = game.world.rng.range(8, 90);
                    Ok(json!(ms))
                }
                Err(_) => Ok(Value::Null),
            }
        }
        "scan" => {
            let addr = arg_str(args, 0, tr!("адрес", "address"))?;
            let id = game.world.lookup(mid, &addr).map_err(|e| e.to_string())?;
            game.world.mark_scanned(id);
            let list: Vec<Value> = game
                .world
                .machine(id)
                .services
                .iter()
                .map(|sv| json!({ "port": sv.port, "service": sv.name, "version": sv.version, "state": if sv.running { "open" } else { "closed" }, "banner": sv.banner }))
                .collect();
            let text = trf!("сканирование с {}", "scan from {}", game.world.machine(mid).hostname);
            game.world.log(id, LogKind::Net, &text);
            Ok(Value::Array(list))
        }
        "connect" => {
            let addr = arg_str(args, 0, tr!("адрес", "address"))?;
            let user = arg_str(args, 1, tr!("пользователь", "user"))?;
            let pass = arg_str_or(args, 2, "");
            let id = game.world.lookup(mid, &addr).map_err(|e| e.to_string())?;
            let has_ssh = game.world.machine(id).services.iter().any(|s| s.name == "ssh" && s.running);
            if !has_ssh {
                return Err(trf!("{}: порт 22 закрыт", "{}: port 22 closed", addr));
            }
            if crate::hack::defense::firewall_up(&game.world, id) {
                return Err(trf!("{}: узел закрыт файрволом", "{}: host blocked by firewall", addr));
            }
            let from_host = game.world.machine(mid).hostname.clone();
            if game.world.machine(id).users.check_password(&user, &pass) {
                let home = game.world.machine(id).users.home_of(&user);
                if let Some(s) = game.sessions.get_mut(&session) {
                    s.push_hop(id, &user, &home);
                }
                game.world.mark_known(id);
                game.world.log(id, LogKind::Login, &trf!("ssh: вход {} с {}", "ssh: login {} from {}", user, from_host));
                Ok(json!(true))
            } else {
                game.world.log(id, LogKind::LoginFailed, &trf!("ssh: неверный пароль для {} с {}", "ssh: wrong password for {} from {}", user, from_host));
                Ok(json!(false))
            }
        }
        "disconnect" => {
            let popped = {
                let world = &game.world;
                game.sessions.get_mut(&session).and_then(|s| if s.hops.len() > 1 { s.pop_hop(world) } else { None })
            };
            if let Some(hop) = popped {
                game.world.log(hop.machine, LogKind::Logout, &trf!("ssh: выход {}", "ssh: logout {}", hop.user));
                Ok(json!(true))
            } else {
                Ok(json!(false))
            }
        }
        "hops" => {
            let names: Vec<String> = game
                .sessions
                .get(&session)
                .map(|s| s.hops.iter().map(|h| game.world.machine(h.machine).hostname.clone()).collect())
                .unwrap_or_default();
            Ok(json!(names))
        }
        "http" => {
            let url = arg_str(args, 0, tr!("адрес", "address"))?;
            let page = game.world.http(mid, &url);
            Ok(json!({ "status": page.status, "title": page.title, "body": page.body }))
        }
        "access" => crate::hack::access::use_object(game, session, args),
        "research" | "probe" => {
            let addr = arg_str(args, 0, tr!("адрес", "address"))?;
            let selector = args.get(1).and_then(Value::as_str);
            let id = game.world.lookup(mid, &addr).map_err(|e| e.to_string())?;
            let sess = game.sessions.get(&session).ok_or(tr!("нет сеанса", "no session"))?;
            let v = crate::hack::payload::select(&game.world, sess, id, selector)?;
            crate::hack::payload::ensure_reachable(&game.world, sess, id, &v)?;
            if func == "probe" {
                let payload = args.get(2).ok_or(tr!("net.probe(адрес, id, payload): нужен payload", "net.probe(address, id, payload): payload required"))?;
                return crate::hack::payload::attempt(&mut game.world, sess, id, &v, payload, true);
            }
            let mut result = crate::hack::payload::profile(&game.world, id, &v)?;
            result["conditions"]=crate::hack::access::conditions(&game.world,id,&v,Some(sess));
            game.world.mark_scanned(id);
            Ok(result)
        }
        // ---- взлом (второй этап)
        "vulns" => {
            let addr = arg_str(args, 0, tr!("адрес", "address"))?;
            let id = game.world.lookup(mid, &addr).map_err(|e| e.to_string())?;
            game.world.mark_scanned(id);
            let sess = game.sessions.get(&session).ok_or(tr!("нет сеанса", "no session"))?;
            let list: Vec<Value> = crate::hack::payload::available(&game.world, sess, id)
                .iter()
                .filter(|v| !v.local || mid == id)
                .map(|v| json!({ "id": crate::hack::payload::id(v), "kind": crate::hack::payload::kind(&game.world, v),
                    "version": v.version, "service": v.service, "port": v.port, "name": v.name, "access": access_str(&v.access), "result":crate::hack::access::result(&game.world,v), "conditions":crate::hack::access::conditions(&game.world,id,v,Some(sess)), "local": v.local }))
                .collect();
            Ok(Value::Array(list))
        }
        "exploit" => {
            let addr = arg_str(args, 0, tr!("адрес", "address"))?;
            let selector = arg_str(args, 1, tr!("id уязвимости", "vulnerability id"))?;
            let payload = args.get(2).ok_or(tr!("Автоматический взлом отключён: net.exploit(адрес, id, payload). Начни с research и exploit new; готовые инструменты — market.",
                "Automatic hacking is disabled: net.exploit(address, id, payload). Start with research and exploit new; ready-made tools — market."))?;
            let id = game.world.lookup(mid, &addr).map_err(|e| e.to_string())?;
            let sess = game.sessions.get_mut(&session).ok_or_else(|| tr!("нет сеанса", "no session").to_string())?;
            let v = crate::hack::payload::select(&game.world, sess, id, Some(&selector))?;
            if v.local { return Err(tr!("для локальной уязвимости используй net.privesc(payload, id)", "for a local vulnerability use net.privesc(payload, id)").into()); }
            let check = crate::hack::payload::attempt(&mut game.world, sess, id, &v, payload, false)?;
            if check["ok"] != json!(true) { return Ok(check); }
            let r = crate::hack::exploit::exploit_selected(&mut game.world, sess, id, v)?;
            use crate::hack::ExploitOutcome;
            let out = match &r.outcome {
                ExploitOutcome::Creds { user, password } => json!({ "ok": true, "access": "creds", "user": user, "password": password, "host": r.target_host, "monitored": r.monitored }),
                ExploitOutcome::Shell { user } => json!({ "ok": true, "access": "shell", "user": user, "host": r.target_host, "monitored": r.monitored }),
                ExploitOutcome::Read { path, data } => json!({ "ok": true, "access": "read", "path": path, "data": data, "host": r.target_host, "monitored": r.monitored }),
                ExploitOutcome::Object { handle, kind, methods } => json!({"ok":true,"access":kind,"handle":handle,"methods":methods,"host":r.target_host,"expires_in":300}),
                ExploitOutcome::Nothing => json!({ "ok": false, "access": "none", "host": r.target_host, "monitored": r.monitored }),
            };
            Ok(out)
        }
        "crack" => {
            let addr = arg_str(args, 0, tr!("адрес", "address"))?;
            let user = arg_str(args, 1, tr!("пользователь", "user"))?;
            let wl_path = arg_str_or(args, 2, "~/wordlist.txt");
            let id = game.world.lookup(mid, &addr).map_err(|e| e.to_string())?;
            let words = read_wordlist(game, session, &wl_path)?;
            let sess = game.sessions.get(&session).ok_or_else(|| tr!("нет сеанса", "no session").to_string())?;
            let r = crate::hack::crack(&mut game.world, sess, id, &user, &words)?;
            Ok(match r.found {
                Some(p) => json!(p),
                None => Value::Null,
            })
        }
        "privesc" => {
            let sess = game.sessions.get_mut(&session).ok_or_else(|| tr!("нет сеанса", "no session").to_string())?;
            let payload = args.first().ok_or(tr!("Нужен локальный payload: net.privesc(payload, id); исследуй текущую машину через research.",
                "A local payload is required: net.privesc(payload, id); investigate the current machine with research."))?;
            let selector = arg_str(args, 1, tr!("id локальной уязвимости", "local vulnerability id"))?;
            let id = sess.machine_id();
            let v = crate::hack::payload::select(&game.world, sess, id, Some(&selector))?;
            if !v.local { return Err(tr!("privesc требует локальную уязвимость текущей машины", "privesc requires a local vulnerability of the current machine").into()); }
            let check = crate::hack::payload::attempt(&mut game.world, sess, id, &v, payload, false)?;
            if check["ok"] != json!(true) { return Ok(check); }
            let r = crate::hack::privesc::apply_vulnerability(&mut game.world, sess, Some(v))?;
            Ok(json!({"ok": r.ok, "access": if r.ok {"root"} else {"none"}, "host": r.host, "feedback": if r.ok {tr!("Права повышены до root.", "Privileges escalated to root.")} else {tr!("Вы уже root.", "You are already root.")}}))
        }
        "clear_logs" => {
            let pat = args.first().and_then(|v| v.as_str()).map(|s| s.to_string());
            let sess = game.sessions.get(&session).ok_or_else(|| tr!("нет сеанса", "no session").to_string())?;
            let r = crate::hack::clear_logs(&mut game.world, sess, pat.as_deref())?;
            Ok(json!(if r.denied { -1i64 } else { r.removed as i64 }))
        }
        "heat" => Ok(json!(crate::hack::heat(&game.world))),
        "watch" => {
            let inv = &game.world.investigation;
            let lead = inv.lead_org.and_then(|id| game.world.org(id)).map(|o| o.name.clone());
            Ok(json!({ "stage": inv.stage.title(), "progress": inv.progress, "raids": inv.raids, "locked": crate::hack::is_locked(&game.world), "lead": lead }))
        }
        "owned" => {
            let names: Vec<String> = game.world.player.owned.iter().filter_map(|id| game.world.has_machine(*id).then(|| game.world.machine(*id).hostname.clone())).collect();
            Ok(json!(names))
        }
        "proxy" => {
            let sess = game.sessions.get(&session).ok_or_else(|| tr!("нет сеанса", "no session").to_string())?;
            let p = crate::hack::proxy_info(&game.world, sess);
            let chain: Vec<Value> = p.chain.iter().map(|(id, u)| json!({ "host": game.world.machine(*id).hostname, "user": u })).collect();
            Ok(json!({ "chain": chain, "exit_ip": p.exit_ip, "hidden": p.hidden, "monitored": !p.monitored_hops.is_empty() }))
        }
        _ => Err(trf!("нет функции net.{}", "no function net.{}", func)),
    }
}

/// Строка доступа уязвимости для языка.
fn access_str(a: &crate::hack::VulnAccess) -> String {
    use crate::hack::VulnAccess;
    match a {
        VulnAccess::Creds => "creds".into(),
        VulnAccess::Shell => "shell".into(),
        VulnAccess::Root => "root".into(),
        VulnAccess::Read(p) => format!("read:{}", p),
        VulnAccess::Info => "info".into(),
    }
}

/// Прочитать словарь с машины текущего hop'а и разбить на слова.
fn read_wordlist(game: &Game, session: u32, path: &str) -> Result<Vec<String>, String> {
    let text = game.fs_read(Some(session), path)?;
    Ok(text.lines().map(|l| l.trim().to_string()).filter(|l| !l.is_empty()).collect())
}

// ---------------------------------------------------------------- shell

fn call_shell(game: &mut Game, session: u32, func: &str, args: &[Value]) -> R {
    match func {
        "exec" => {
            let line = arg_str(args, 0, tr!("команда", "command"))?;
            let res = game.exec(session, &line);
            let ok = !res.lines.iter().any(|l| l.kind == OutKind::Error);
            let out: Vec<String> = res.lines.iter().map(|l| l.text.clone()).collect();
            // окна и уведомления из команды — оболочке Tauri
            for a in res.actions {
                match a {
                    UiAction::OpenApp { .. } | UiAction::Notify { .. } | UiAction::QuitGame => game.pending_ui.push(a),
                    _ => {}
                }
            }
            Ok(json!({ "out": out.join("\n"), "ok": ok }))
        }
        _ => Err(trf!("нет функции shell.{}", "no function shell.{}", func)),
    }
}
