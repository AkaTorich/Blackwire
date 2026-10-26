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
        Some(Value::Null) | None => Err(format!("нужен аргумент: {}", what)),
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
        Some(Value::Number(n)) => n.as_i64().or_else(|| n.as_f64().map(|f| f as i64)).ok_or_else(|| format!("нужно число: {}", what)),
        Some(Value::String(s)) => s.trim().parse::<i64>().map_err(|_| format!("нужно число: {}", what)),
        _ => Err(format!("нужно число: {}", what)),
    }
}

/// Машина и пользователь, с которыми работает скрипт (текущий hop его сеанса).
fn whoami(game: &Game, session: u32) -> (MachineId, String) {
    game.target(Some(session))
}

/// Вызов игровой функции по полному имени.
pub fn call(game: &mut Game, session: u32, pid: u32, name: &str, args: Vec<Value>) -> R {
    let (module, func) = name.split_once('.').ok_or_else(|| format!("неизвестная функция: {}", name))?;
    let stamp=crate::plugins::events::Stamp::new(&game.world);
    let result = match module {
        "sys" => call_sys(game, session, pid, func, &args),
        "fs" => call_fs(game, session, func, &args),
        "net" => call_net(game, session, func, &args),
        "shell" => call_shell(game, session, func, &args),
        "time" => match func {
            "now" => Ok(json!(game.world.now())),
            _ => Err(format!("нет функции time.{}", func)),
        },
        _ => Err(format!("неизвестный модуль: {}", module)),
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
            let key = arg_str(args, 0, "имя переменной")?;
            Ok(game.sessions.get(&session).and_then(|s| s.env.get(&key)).map(|v| json!(v)).unwrap_or(Value::Null))
        }
        "set_env" => {
            let key = arg_str(args, 0, "имя переменной")?;
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
                return Err("скрипт не может снять сам себя — используйте exit()".into());
            }
            game.world.machine_mut(mid).procs.kill_as(target, &user)?;
            Ok(json!(true))
        }
        "uptime" => Ok(json!(game.world.uptime())),
        "notify" => {
            let title = arg_str(args, 0, "заголовок")?;
            let text = arg_str_or(args, 1, "");
            game.notify(&title, &text);
            Ok(Value::Null)
        }
        "open" => {
            let app = arg_str(args, 0, "имя приложения")?;
            let arg = args.get(1).and_then(|v| v.as_str()).map(|s| s.to_string());
            game.pending_ui.push(UiAction::OpenApp { app, arg });
            Ok(Value::Null)
        }
        "pid" => Ok(json!(pid)),
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
        _ => Err(format!("нет функции sys.{}", func)),
    }
}

// ---------------------------------------------------------------- fs

fn call_fs(game: &mut Game, session: u32, func: &str, args: &[Value]) -> R {
    let s = Some(session);
    match func {
        "read" => Ok(json!(game.fs_read(s, &arg_str(args, 0, "путь")?)?)),
        "write" => {
            game.fs_write(s, &arg_str(args, 0, "путь")?, &arg_str_or(args, 1, ""))?;
            Ok(Value::Null)
        }
        "append" => {
            let (mid, user) = whoami(game, session);
            let p = game.fs_path(s, &arg_str(args, 0, "путь")?);
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
        "exists" => Ok(json!(game.fs_exists(s, &arg_str(args, 0, "путь")?))),
        "is_dir" => {
            let (mid, _) = whoami(game, session);
            let p = game.fs_path(s, &arg_str(args, 0, "путь")?);
            Ok(json!(game.world.machine(mid).fs.is_dir(&p)))
        }
        "mkdir" => {
            game.fs_mkdir(s, &arg_str(args, 0, "путь")?)?;
            Ok(Value::Null)
        }
        "delete" => {
            game.fs_delete(s, &arg_str(args, 0, "путь")?)?;
            Ok(Value::Null)
        }
        "move" => {
            game.fs_rename(s, &arg_str(args, 0, "откуда")?, &arg_str(args, 1, "куда")?)?;
            Ok(Value::Null)
        }
        "copy" => {
            game.fs_copy(s, &arg_str(args, 0, "откуда")?, &arg_str(args, 1, "куда")?)?;
            Ok(Value::Null)
        }
        "chmod" => {
            let (mid, user) = whoami(game, session);
            let p = game.fs_path(s, &arg_str(args, 0, "путь")?);
            let mode_s = arg_str(args, 1, "права")?;
            let current = game.world.machine(mid).fs.stat(&p).map(|st| st.mode).unwrap_or(0o644);
            let mode = crate::vfs::parse_mode(&mode_s, current).ok_or_else(|| format!("непонятные права: {}", mode_s))?;
            game.world.machine_mut(mid).fs.chmod(&p, mode, &user).map_err(|e| e.to_string())?;
            Ok(Value::Null)
        }
        "chown" => {
            let (mid, user) = whoami(game, session);
            let p = game.fs_path(s, &arg_str(args, 0, "путь")?);
            let owner = arg_str(args, 1, "владелец")?;
            game.world.machine_mut(mid).fs.chown(&p, &owner, &user).map_err(|e| e.to_string())?;
            Ok(Value::Null)
        }
        _ => Err(format!("нет функции fs.{}", func)),
    }
}

// ---------------------------------------------------------------- net

fn call_net(game: &mut Game, session: u32, func: &str, args: &[Value]) -> R {
    let (mid, _user) = whoami(game, session);
    // при активном рейде исходящие атаки заблокированы
    if matches!(func, "connect" | "exploit" | "crack") && crate::hack::is_locked(&game.world) {
        return Err(format!("канал под наблюдением после рейда — ещё {} с", crate::hack::lock_remaining(&game.world)));
    }
    match func {
        "my_ip" => {
            let m = game.world.machine(mid);
            Ok(json!(m.lan_ip.clone().unwrap_or_else(|| m.ip.clone())))
        }
        "resolve" => Ok(game.world.resolve(&arg_str(args, 0, "имя")?).map(|ip| json!(ip)).unwrap_or(Value::Null)),
        "ping" => {
            let addr = arg_str(args, 0, "адрес")?;
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
            let addr = arg_str(args, 0, "адрес")?;
            let id = game.world.lookup(mid, &addr).map_err(|e| e.to_string())?;
            game.world.mark_scanned(id);
            let list: Vec<Value> = game
                .world
                .machine(id)
                .services
                .iter()
                .map(|sv| json!({ "port": sv.port, "service": sv.name, "version": sv.version, "state": if sv.running { "open" } else { "closed" }, "banner": sv.banner }))
                .collect();
            let text = format!("сканирование с {}", game.world.machine(mid).hostname);
            game.world.log(id, LogKind::Net, &text);
            Ok(Value::Array(list))
        }
        "connect" => {
            let addr = arg_str(args, 0, "адрес")?;
            let user = arg_str(args, 1, "пользователь")?;
            let pass = arg_str_or(args, 2, "");
            let id = game.world.lookup(mid, &addr).map_err(|e| e.to_string())?;
            let has_ssh = game.world.machine(id).services.iter().any(|s| s.name == "ssh" && s.running);
            if !has_ssh {
                return Err(format!("{}: порт 22 закрыт", addr));
            }
            if crate::hack::defense::firewall_up(&game.world, id) {
                return Err(format!("{}: узел закрыт файрволом", addr));
            }
            let from_host = game.world.machine(mid).hostname.clone();
            if game.world.machine(id).users.check_password(&user, &pass) {
                let home = game.world.machine(id).users.home_of(&user);
                if let Some(s) = game.sessions.get_mut(&session) {
                    s.push_hop(id, &user, &home);
                }
                game.world.mark_known(id);
                game.world.log(id, LogKind::Login, &format!("ssh: вход {} с {}", user, from_host));
                Ok(json!(true))
            } else {
                game.world.log(id, LogKind::LoginFailed, &format!("ssh: неверный пароль для {} с {}", user, from_host));
                Ok(json!(false))
            }
        }
        "disconnect" => {
            let popped = {
                let world = &game.world;
                game.sessions.get_mut(&session).and_then(|s| if s.hops.len() > 1 { s.pop_hop(world) } else { None })
            };
            if let Some(hop) = popped {
                game.world.log(hop.machine, LogKind::Logout, &format!("ssh: выход {}", hop.user));
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
            let url = arg_str(args, 0, "адрес")?;
            let page = game.world.http(mid, &url);
            Ok(json!({ "status": page.status, "title": page.title, "body": page.body }))
        }
        // ---- взлом (второй этап)
        "vulns" => {
            let addr = arg_str(args, 0, "адрес")?;
            let id = game.world.lookup(mid, &addr).map_err(|e| e.to_string())?;
            game.world.mark_scanned(id);
            let list: Vec<Value> = crate::hack::scan_vulns(game.world.machine(id))
                .iter()
                .map(|v| json!({ "service": v.service, "port": v.port, "name": v.name, "access": access_str(&v.access), "local": v.local }))
                .collect();
            Ok(Value::Array(list))
        }
        "exploit" => {
            let addr = arg_str(args, 0, "адрес")?;
            let service = args.get(1).and_then(|v| v.as_str()).map(|s| s.to_string());
            let id = game.world.lookup(mid, &addr).map_err(|e| e.to_string())?;
            let sess = game.sessions.get_mut(&session).ok_or_else(|| "нет сеанса".to_string())?;
            let r = crate::hack::exploit(&mut game.world, sess, id, service.as_deref())?;
            use crate::hack::ExploitOutcome;
            let out = match &r.outcome {
                ExploitOutcome::Creds { user, password } => json!({ "ok": true, "access": "creds", "user": user, "password": password, "host": r.target_host, "monitored": r.monitored }),
                ExploitOutcome::Shell { user } => json!({ "ok": true, "access": "shell", "user": user, "host": r.target_host, "monitored": r.monitored }),
                ExploitOutcome::Read { path, data } => json!({ "ok": true, "access": "read", "path": path, "data": data, "host": r.target_host, "monitored": r.monitored }),
                ExploitOutcome::Nothing => json!({ "ok": false, "access": "none", "host": r.target_host, "monitored": r.monitored }),
            };
            Ok(out)
        }
        "crack" => {
            let addr = arg_str(args, 0, "адрес")?;
            let user = arg_str(args, 1, "пользователь")?;
            let wl_path = arg_str_or(args, 2, "~/wordlist.txt");
            let id = game.world.lookup(mid, &addr).map_err(|e| e.to_string())?;
            let words = read_wordlist(game, session, &wl_path)?;
            let sess = game.sessions.get(&session).ok_or_else(|| "нет сеанса".to_string())?;
            let r = crate::hack::crack(&mut game.world, sess, id, &user, &words)?;
            Ok(match r.found {
                Some(p) => json!(p),
                None => Value::Null,
            })
        }
        "privesc" => {
            let sess = game.sessions.get_mut(&session).ok_or_else(|| "нет сеанса".to_string())?;
            let r = crate::hack::privesc(&mut game.world, sess)?;
            Ok(json!(r.ok))
        }
        "clear_logs" => {
            let pat = args.first().and_then(|v| v.as_str()).map(|s| s.to_string());
            let sess = game.sessions.get(&session).ok_or_else(|| "нет сеанса".to_string())?;
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
            let sess = game.sessions.get(&session).ok_or_else(|| "нет сеанса".to_string())?;
            let p = crate::hack::proxy_info(&game.world, sess);
            let chain: Vec<Value> = p.chain.iter().map(|(id, u)| json!({ "host": game.world.machine(*id).hostname, "user": u })).collect();
            Ok(json!({ "chain": chain, "exit_ip": p.exit_ip, "hidden": p.hidden, "monitored": !p.monitored_hops.is_empty() }))
        }
        _ => Err(format!("нет функции net.{}", func)),
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
            let line = arg_str(args, 0, "команда")?;
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
        _ => Err(format!("нет функции shell.{}", func)),
    }
}
