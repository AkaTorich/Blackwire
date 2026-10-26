//! Оболочка: исполнение строки (конвейеры, перенаправления, подстановки),
//! подсказка по Tab, строка приглашения, справка.
//!
//! - `parse`      — разбор строки.
//! - `util`       — флаги, таблицы, размеры, `CmdOutput`.
//! - `table`      — список всех команд с группами и кратким описанием.
//! - `help`       — тексты `--help`/`man`, `shell_doc()`.
//! - `cmds_fs`    — файлы и каталоги.
//! - `cmds_text`  — текстовые фильтры (grep, sort, wc, …) и архивы.
//! - `cmds_sys`   — пользователи, процессы, окружение, системные сведения.
//! - `cmds_net`   — сеть: ping, scan, ssh, scp, wget…
//! - `cmds_misc`  — Adder, окна приложений, украшения (banner, matrix, neofetch…).

pub mod cmds_base;
pub mod cmds_defense;
pub mod cmds_fs;
pub mod cmds_hack;
pub mod cmds_jobs;
pub mod cmds_money;
pub mod cmds_misc;
pub mod cmds_net;
pub mod cmds_sys;
pub mod cmds_text;
mod font;
pub mod help;
pub mod parse;
pub mod table;
pub mod util;

pub use util::CmdOutput;

use crate::net::machine::{LogKind, Machine, MachineId};
use crate::session::Session;
use crate::types::{OutKind, Prompt, ShellResult, UiAction};
use crate::vfs::{self, Vfs};
use crate::world::World;

/// Окружение одной команды: мир, сеанс, вход из конвейера, собранные действия.
pub struct Ctx<'a> {
    pub world: &'a mut World,
    pub session: &'a mut Session,
    pub stdin: Option<String>,
    pub actions: Vec<UiAction>,
    /// Команда исполняется под `sudo`: действующий пользователь — root.
    pub sudo: bool,
}

impl<'a> Ctx<'a> {
    /// Действующий пользователь.
    pub fn user(&self) -> String {
        if self.sudo {
            "root".to_string()
        } else {
            self.session.user().to_string()
        }
    }

    pub fn mid(&self) -> MachineId {
        self.session.machine_id()
    }

    pub fn machine(&self) -> &Machine {
        self.world.machine(self.mid())
    }

    pub fn machine_mut(&mut self) -> &mut Machine {
        let id = self.mid();
        self.world.machine_mut(id)
    }

    pub fn fs(&self) -> &Vfs {
        &self.machine().fs
    }

    pub fn fs_mut(&mut self) -> &mut Vfs {
        &mut self.machine_mut().fs
    }

    pub fn cwd(&self) -> String {
        self.session.cwd.clone()
    }

    /// Домашняя папка действующего пользователя на текущей машине.
    pub fn home(&self) -> String {
        let u = self.user();
        self.machine().users.home_of(&u)
    }

    /// Абсолютный путь из аргумента команды (`~` и относительные пути).
    pub fn path(&self, p: &str) -> String {
        let p = if p == "~" {
            self.home()
        } else if let Some(rest) = p.strip_prefix("~/") {
            format!("{}/{}", self.home(), rest)
        } else {
            p.to_string()
        };
        vfs::resolve(&self.session.cwd, &p)
    }

    pub fn now(&self) -> u64 {
        self.world.now()
    }

    pub fn is_admin(&self) -> bool {
        let u = self.user();
        self.machine().users.is_admin(&u)
    }

    pub fn log(&mut self, kind: LogKind, text: &str) {
        let id = self.mid();
        self.world.log(id, kind, text);
    }

    /// Строки стандартного входа (из конвейера или `<`).
    pub fn stdin_lines(&self) -> Vec<String> {
        match &self.stdin {
            Some(s) => s.lines().map(|l| l.to_string()).collect(),
            None => Vec::new(),
        }
    }
}

/// Исполнить строку в сеансе.
pub fn exec(world: &mut World, session: &mut Session, line: &str) -> ShellResult {
    let mut res = ShellResult::default();
    let trimmed = line.trim();
    if !trimmed.is_empty() {
        session.history.push(trimmed.to_string());
        if session.history.len() > 500 {
            session.history.remove(0);
        }
        record_history(world, session, trimmed);
    }
    match parse::parse(line) {
        Err(e) => {
            res.lines.push(crate::types::OutLine::error(format!("ошибка разбора: {}", e)));
            session.last_status = 2;
        }
        Ok(jobs) => {
            for job in jobs {
                let run = match job.sep {
                    parse::Sep::Seq => true,
                    parse::Sep::And => session.last_status == 0,
                    parse::Sep::Or => session.last_status != 0,
                };
                if !run {
                    continue;
                }
                let (lines, actions) = run_pipeline(world, session, &job.pipeline);
                res.lines.extend(lines);
                let stop = actions.iter().any(|a| matches!(a, UiAction::Exit | UiAction::QuitGame));
                res.actions.extend(actions);
                if stop {
                    break;
                }
            }
        }
    }
    res.prompt = prompt(world, session);
    res
}

/// След команды: `.bash_history` пользователя и журнал чужой машины.
fn record_history(world: &mut World, session: &Session, line: &str) {
    let id = session.machine_id();
    let user = session.user().to_string();
    let home = world.machine(id).users.home_of(&user);
    let hist = format!("{}/.bash_history", home);
    world.machine_mut(id).fs.append_text(&hist, &format!("{}\n", line), &user).ok();
    if !session.at_home() {
        world.log(id, LogKind::Exec, &format!("{}: {}", user, line));
    }
}

/// Раскрыть шаблон `*`/`?` по файловой системе относительно текущей папки.
pub fn glob_expand(fs: &Vfs, cwd: &str, pattern: &str) -> Vec<String> {
    let (dir_part, name_part) = match pattern.rfind('/') {
        Some(i) => (&pattern[..=i], &pattern[i + 1..]),
        None => ("", pattern),
    };
    let dir_abs = if dir_part.is_empty() { cwd.to_string() } else { vfs::resolve(cwd, dir_part) };
    let mut out: Vec<String> = match fs.entries_unchecked(&dir_abs) {
        Ok(entries) => entries
            .iter()
            .filter(|e| name_part.starts_with('.') || !e.name.starts_with('.'))
            .filter(|e| vfs::glob_match(name_part, &e.name))
            .map(|e| format!("{}{}", dir_part, e.name))
            .collect(),
        Err(_) => Vec::new(),
    };
    out.sort();
    out
}

fn run_pipeline(world: &mut World, session: &mut Session, pipeline: &parse::Pipeline) -> (Vec<crate::types::OutLine>, Vec<UiAction>) {
    let mut lines = Vec::new();
    let mut actions = Vec::new();
    let mut stdin: Option<String> = None;
    let n = pipeline.cmds.len();
    for (i, simple) in pipeline.cmds.iter().enumerate() {
        // подстановки — при неизменяемом мире
        let (mut words, redir_out, redir_err, redir_in) = {
            let mid = session.machine_id();
            let fs = &world.machine(mid).fs;
            let cwd = session.cwd.clone();
            let home = world.machine(mid).users.home_of(session.user());
            let glob = |p: &str| glob_expand(fs, &cwd, p);
            let ex = parse::Expander { env: &session.env, home: &home, status: session.last_status, glob: &glob };
            let mut words: Vec<String> = Vec::new();
            for w in &simple.words {
                words.extend(parse::expand_word(w, &ex));
            }
            let out = simple.redirs.stdout.as_ref().map(|(w, a)| (parse::expand_literal(w, &ex), *a));
            let err = simple.redirs.stderr.as_ref().map(|w| parse::expand_literal(w, &ex));
            let inp = simple.redirs.stdin.as_ref().map(|w| parse::expand_literal(w, &ex));
            (words, out, err, inp)
        };
        if words.is_empty() {
            continue;
        }
        // псевдонимы
        if let Some(alias) = session.aliases.get(&words[0]).cloned() {
            let mut replaced: Vec<String> = alias.split_whitespace().map(|s| s.to_string()).collect();
            replaced.extend(words.drain(1..));
            words = replaced;
        }
        let mut ctx = Ctx { world, session, stdin: stdin.take(), actions: Vec::new(), sudo: false };
        let mut out = CmdOutput::ok();
        if let Some(path) = redir_in {
            let abs = ctx.path(&path);
            let user = ctx.user();
            match ctx.fs().read(&abs, &user) {
                Ok(text) => ctx.stdin = Some(text),
                Err(e) => {
                    out.push_err(format!("{}: {}", words[0], e));
                }
            }
        }
        if out.status == 0 {
            out = run_simple(&mut ctx, &words);
        }
        actions.extend(ctx.actions.drain(..));
        let user = ctx.user();
        let (mut out_lines, mut err_lines): (Vec<_>, Vec<_>) = out.lines.into_iter().partition(|l| l.kind != OutKind::Error);
        if let Some((path, append)) = redir_out {
            let abs = ctx.path(&path);
            let text = join_lines(&out_lines);
            let r = if append { ctx.fs_mut().append(&abs, &text, &user) } else { ctx.fs_mut().write(&abs, &text, &user) };
            out_lines.clear();
            if let Err(e) = r {
                err_lines.push(crate::types::OutLine::error(format!("{}: {}", words[0], e)));
                out.status = 1;
            }
        }
        if let Some(path) = redir_err {
            let abs = ctx.path(&path);
            let text = join_lines(&err_lines);
            err_lines.clear();
            if let Err(e) = ctx.fs_mut().write(&abs, &text, &user) {
                err_lines.push(crate::types::OutLine::error(format!("{}: {}", words[0], e)));
            }
        }
        if i + 1 < n {
            stdin = Some(out_lines.iter().map(|l| l.text.clone()).collect::<Vec<_>>().join("\n"));
            out_lines.clear();
        }
        lines.extend(err_lines);
        lines.extend(out_lines);
        session.last_status = out.status;
        if actions.iter().any(|a| matches!(a, UiAction::QuitGame)) {
            break;
        }
    }
    (lines, actions)
}

fn join_lines(lines: &[crate::types::OutLine]) -> String {
    if lines.is_empty() {
        return String::new();
    }
    let mut s = lines.iter().map(|l| l.text.as_str()).collect::<Vec<_>>().join("\n");
    s.push('\n');
    s
}

/// Исполнить одну простую команду по словам (уже после подстановок).
pub fn run_simple(ctx: &mut Ctx<'_>, words: &[String]) -> CmdOutput {
    if words.is_empty() {
        return CmdOutput::ok();
    }
    let name = words[0].as_str();
    let args = &words[1..];
    if args.iter().any(|a| a == "--help") {
        if let Some(info) = table::lookup(name) {
            return help::command_help(info);
        }
    }
    if name.contains('/') {
        return cmds_misc::run_path(ctx, name, args);
    }
    match table::lookup(name) {
        Some(info) => (info.run)(ctx, args),
        None => {
            let mut out = CmdOutput::err(format!("{}: команда не найдена", name));
            out.status = 127;
            let mut best: Option<(usize, &str)> = None;
            for c in table::COMMANDS {
                let d = util::levenshtein(name, c.name);
                if d <= 2 && best.map(|(bd, _)| d < bd).unwrap_or(true) {
                    best = Some((d, c.name));
                }
            }
            if let Some((_, s)) = best {
                out.push_kind(format!("возможно, вы имели в виду: {}", s), OutKind::Dim);
            }
            out.push_kind("список команд: help", OutKind::Dim);
            out
        }
    }
}

/// Строка приглашения для сеанса.
pub fn prompt(world: &World, session: &Session) -> Prompt {
    let m = world.machine(session.machine_id());
    let user = session.user().to_string();
    let home = m.users.home_of(&user);
    let cwd = if session.cwd == home {
        "~".to_string()
    } else if let Some(rest) = session.cwd.strip_prefix(&format!("{}/", home)) {
        format!("~/{}", rest)
    } else {
        session.cwd.clone()
    };
    Prompt {
        is_root: user == "root",
        user,
        host: m.hostname.clone(),
        cwd,
        hops: session.hops.iter().map(|h| world.machine(h.machine).hostname.clone()).collect(),
    }
}

/// Подсказка по Tab: варианты для последнего слова строки.
pub fn complete(world: &World, session: &Session, line: &str) -> Vec<String> {
    let ends_with_space = line.ends_with(' ');
    let mut words: Vec<&str> = line.split_whitespace().collect();
    if ends_with_space || words.is_empty() {
        words.push("");
    }
    let last = words[words.len() - 1];
    let mut out: Vec<String> = Vec::new();
    if words.len() == 1 {
        for c in table::COMMANDS {
            if c.name.starts_with(last) {
                out.push(c.name.to_string());
            }
        }
        for a in session.aliases.keys() {
            if a.starts_with(last) {
                out.push(a.clone());
            }
        }
        out.sort();
        out.dedup();
        return out;
    }
    let cmd = words[0];
    let m = world.machine(session.machine_id());
    match cmd {
        "su" | "passwd" | "userdel" | "chown" if words.len() == 2 || cmd == "chown" && words.len() == 2 => {
            out.extend(m.users.names().into_iter().filter(|n| n.starts_with(last)));
        }
        "ssh" | "scp" | "ping" | "scan" | "nmap" | "traceroute" | "whois" | "nslookup" | "wget" | "curl" if words.len() == 2 => {
            let (prefix, tail) = match last.rfind('@') {
                Some(i) => (&last[..=i], &last[i + 1..]),
                None => ("", last),
            };
            for id in std::iter::once(&0u32).chain(world.player.known_machines.iter()) {
                if let Some(k) = world.machines.iter().find(|x| x.id == *id) {
                    for cand in [k.ip.as_str(), k.hostname.as_str()] {
                        if cand.starts_with(tail) && !tail.is_empty() || tail.is_empty() {
                            out.push(format!("{}{}", prefix, cand));
                        }
                    }
                    if let Some(d) = &k.domain {
                        if d.starts_with(tail) {
                            out.push(format!("{}{}", prefix, d));
                        }
                    }
                }
            }
        }
        "open" if words.len() == 2 => {
            for app in cmds_misc::APPS {
                if app.starts_with(last) {
                    out.push(app.to_string());
                }
            }
        }
        "help" | "man" if words.len() == 2 => {
            for c in table::COMMANDS {
                if c.name.starts_with(last) {
                    out.push(c.name.to_string());
                }
            }
        }
        _ => {
            let home = m.users.home_of(session.user());
            let expanded = if last == "~" {
                home.clone()
            } else if let Some(rest) = last.strip_prefix("~/") {
                format!("{}/{}", home, rest)
            } else {
                last.to_string()
            };
            let (dir_part, name_part) = match expanded.rfind('/') {
                Some(i) => (expanded[..=i].to_string(), expanded[i + 1..].to_string()),
                None => (String::new(), expanded.clone()),
            };
            let dir_abs = if dir_part.is_empty() { session.cwd.clone() } else { vfs::resolve(&session.cwd, &dir_part) };
            let shown_dir = if last.starts_with("~/") || last == "~" {
                // вернуть тильду как ввёл пользователь
                let rel = dir_part.strip_prefix(&home).unwrap_or(&dir_part);
                format!("~{}", rel)
            } else {
                dir_part.clone()
            };
            if let Ok(entries) = m.fs.entries_unchecked(&dir_abs) {
                for e in entries {
                    if e.name.starts_with(&name_part) && (name_part.starts_with('.') || !e.name.starts_with('.')) {
                        let suffix = if e.is_dir { "/" } else { "" };
                        out.push(format!("{}{}{}", shown_dir, e.name, suffix));
                    }
                }
            }
        }
    }
    out.sort();
    out.dedup();
    out
}

/// Справка по всем командам в Markdown — для окна «Справка».
pub fn shell_doc() -> String {
    help::shell_doc()
}
