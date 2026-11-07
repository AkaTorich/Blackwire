//! Команды для файлов и каталогов: ls, cd, cat, mkdir, rm, mv, cp, chmod, find, tree…

use super::util::{columns, human_size, table, Args};
use super::{CmdOutput, Ctx};
use crate::net::machine::LogKind;
use crate::types::{FsEntry, OutKind};
use crate::vfs::{self, mode_string};
use crate::world::GameClock;

fn fmt_size(size: u64, human: bool) -> String {
    if human {
        human_size(size)
    } else {
        size.to_string()
    }
}

fn long_rows(entries: &[FsEntry], human: bool) -> Vec<String> {
    let rows: Vec<Vec<String>> = entries
        .iter()
        .map(|e| {
            vec![
                e.mode.clone(),
                e.owner.clone(),
                fmt_size(e.size, human),
                GameClock::format_short(e.modified),
                if e.is_dir { format!("{}/", e.name) } else { e.name.clone() },
            ]
        })
        .collect();
    table(&rows, &[false, false, true, false, false])
}

pub fn ls(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    let a = Args::parse(args, &[]);
    let long = a.has("l");
    let all = a.has("a");
    let human = a.has("h");
    let targets: Vec<String> = if a.pos.is_empty() { vec![".".to_string()] } else { a.pos.clone() };
    let user = ctx.user();
    let mut out = CmdOutput::ok();
    let many = targets.len() > 1;
    for (i, t) in targets.iter().enumerate() {
        let path = ctx.path(t);
        let st = match ctx.fs().stat(&path) {
            Ok(s) => s,
            Err(e) => {
                out.push_err(format!("ls: {}", e));
                continue;
            }
        };
        let entries: Vec<FsEntry> = if st.is_dir {
            match ctx.fs().list(&path, &user) {
                Ok(mut v) => {
                    if !all {
                        v.retain(|e| !e.name.starts_with('.'));
                    }
                    v.sort_by(|x, y| x.name.to_lowercase().cmp(&y.name.to_lowercase()));
                    v
                }
                Err(e) => {
                    out.push_err(format!("ls: {}", e));
                    continue;
                }
            }
        } else {
            vec![st.entry()]
        };
        if many {
            if i > 0 {
                out.push("");
            }
            out.push_kind(format!("{}:", t), OutKind::Accent);
        }
        if long {
            out.push_lines(long_rows(&entries, human));
        } else {
            let names: Vec<String> =
                entries.iter().map(|e| if e.is_dir { format!("{}/", e.name) } else { e.name.clone() }).collect();
            out.push_lines(columns(&names, 80));
        }
    }
    out
}

pub fn cd(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    let target = match args.first().map(|s| s.as_str()) {
        None | Some("~") => ctx.home(),
        Some("-") => ctx.session.env.get("OLDPWD").cloned().unwrap_or_else(|| ctx.cwd()),
        Some(p) => ctx.path(p),
    };
    let user = ctx.user();
    match ctx.fs().stat(&target) {
        Err(e) => CmdOutput::err(format!("cd: {}", e)),
        Ok(st) if !st.is_dir => CmdOutput::err(trf!("cd: не каталог: {}", "cd: not a directory: {}", target)),
        Ok(_) => {
            // право входа: список каталога доступен или хотя бы проходим
            if let Err(e) = ctx.fs().list(&target, &user) {
                if matches!(e, vfs::FsError::Denied(_)) {
                    return CmdOutput::err(format!("cd: {}", e));
                }
            }
            let old = ctx.cwd();
            ctx.session.env.insert("OLDPWD".to_string(), old);
            ctx.session.set_cwd(&target);
            CmdOutput::ok()
        }
    }
}

pub fn pwd(ctx: &mut Ctx<'_>, _args: &[String]) -> CmdOutput {
    CmdOutput::text(ctx.cwd())
}

/// Содержимое файлов из аргументов или стандартного входа: (имя, текст).
pub fn inputs(ctx: &Ctx<'_>, cmd: &str, files: &[String], out: &mut CmdOutput) -> Vec<(String, String)> {
    let user = ctx.user();
    if files.is_empty() || files == ["-"] {
        return vec![("-".to_string(), ctx.stdin.clone().unwrap_or_default())];
    }
    let mut v = Vec::new();
    for f in files {
        let path = ctx.path(f);
        match ctx.fs().read(&path, &user) {
            Ok(text) => v.push((f.clone(), text)),
            Err(e) => out.push_err(format!("{}: {}", cmd, e)),
        }
    }
    v
}

pub fn cat(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    let a = Args::parse(args, &[]);
    let mut out = CmdOutput::ok();
    let number = a.has("n");
    let mut n = 0;
    for (_, text) in inputs(ctx, "cat", &a.pos, &mut out) {
        let body = text.strip_suffix('\n').unwrap_or(&text);
        if body.is_empty() && text.is_empty() {
            continue;
        }
        for line in body.split('\n') {
            n += 1;
            if number {
                out.push(format!("{:6}  {}", n, line));
            } else {
                out.push(line);
            }
        }
    }
    out
}

pub fn touch(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    if args.is_empty() {
        return CmdOutput::err(tr!("touch: укажите файл", "touch: specify a file"));
    }
    let user = ctx.user();
    let mut out = CmdOutput::ok();
    for f in args {
        let path = ctx.path(f);
        if let Err(e) = ctx.fs_mut().touch(&path, &user) {
            out.push_err(format!("touch: {}", e));
        }
    }
    out
}

pub fn mkdir(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    let a = Args::parse(args, &[]);
    if a.pos.is_empty() {
        return CmdOutput::err(tr!("mkdir: укажите имя папки", "mkdir: specify a directory name"));
    }
    let parents = a.has("p");
    let user = ctx.user();
    let mut out = CmdOutput::ok();
    for d in &a.pos {
        let path = ctx.path(d);
        if let Err(e) = ctx.fs_mut().mkdir(&path, &user, parents) {
            out.push_err(trf!("mkdir: не удалось создать «{}»: {}", "mkdir: cannot create '{}': {}", d, e));
        }
    }
    out
}

pub fn rm(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    let a = Args::parse(args, &[]);
    if a.pos.is_empty() {
        return CmdOutput::err(tr!("rm: укажите, что удалить", "rm: specify what to remove"));
    }
    let recursive = a.has("r") || a.has("R");
    let force = a.has("f");
    let user = ctx.user();
    let mut out = CmdOutput::ok();
    for p in &a.pos {
        let path = ctx.path(p);
        match ctx.fs_mut().remove(&path, &user, recursive) {
            Ok(()) => ctx.log(LogKind::FileChange, &trf!("{} удалил {}", "{} deleted {}", user, path)),
            Err(vfs::FsError::NotFound(_)) if force => {}
            Err(e) => out.push_err(format!("rm: {}", e)),
        }
    }
    out
}

pub fn rmdir(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    if args.is_empty() {
        return CmdOutput::err(tr!("rmdir: укажите папку", "rmdir: specify a directory"));
    }
    let user = ctx.user();
    let mut out = CmdOutput::ok();
    for p in args {
        let path = ctx.path(p);
        if !ctx.fs().is_dir(&path) {
            out.push_err(trf!("rmdir: не каталог: {}", "rmdir: not a directory: {}", path));
            continue;
        }
        if let Err(e) = ctx.fs_mut().remove(&path, &user, false) {
            out.push_err(format!("rmdir: {}", e));
        }
    }
    out
}

pub fn mv(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    if args.len() < 2 {
        return CmdOutput::err(tr!("mv: нужно указать откуда и куда", "mv: specify source and destination"));
    }
    let user = ctx.user();
    let dst = ctx.path(&args[args.len() - 1]);
    let srcs = &args[..args.len() - 1];
    if srcs.len() > 1 && !ctx.fs().is_dir(&dst) {
        return CmdOutput::err(trf!("mv: при нескольких источниках «{}» должен быть каталогом", "mv: with multiple sources '{}' must be a directory", dst));
    }
    let mut out = CmdOutput::ok();
    for s in srcs {
        let src = ctx.path(s);
        match ctx.fs_mut().rename(&src, &dst, &user) {
            Ok(new) => ctx.log(LogKind::FileChange, &trf!("{} переместил {} → {}", "{} moved {} → {}", user, src, new)),
            Err(e) => out.push_err(format!("mv: {}", e)),
        }
    }
    out
}

pub fn cp(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    let a = Args::parse(args, &[]);
    if a.pos.len() < 2 {
        return CmdOutput::err(tr!("cp: нужно указать откуда и куда", "cp: specify source and destination"));
    }
    let recursive = a.has("r") || a.has("R");
    let user = ctx.user();
    let dst = ctx.path(&a.pos[a.pos.len() - 1]);
    let srcs = &a.pos[..a.pos.len() - 1];
    if srcs.len() > 1 && !ctx.fs().is_dir(&dst) {
        return CmdOutput::err(trf!("cp: при нескольких источниках «{}» должен быть каталогом", "cp: with multiple sources '{}' must be a directory", dst));
    }
    let mut out = CmdOutput::ok();
    for s in srcs {
        let src = ctx.path(s);
        match ctx.fs_mut().copy(&src, &dst, &user, recursive) {
            Ok(_) => {}
            Err(vfs::FsError::IsDir(p)) => out.push_err(trf!("cp: {} — каталог, нужен -r", "cp: {} is a directory, -r needed", p)),
            Err(e) => out.push_err(format!("cp: {}", e)),
        }
    }
    out
}

pub fn ln(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    let a = Args::parse(args, &[]);
    if a.pos.len() != 2 {
        return CmdOutput::err(tr!("ln: нужно указать цель и имя", "ln: specify target and link name"));
    }
    let user = ctx.user();
    let (src, dst) = (ctx.path(&a.pos[0]), ctx.path(&a.pos[1]));
    match ctx.fs_mut().copy(&src, &dst, &user, true) {
        Ok(_) => CmdOutput::ok(),
        Err(e) => CmdOutput::err(format!("ln: {}", e)),
    }
}

/// Раскрыть `\n`, `\t`, `\\` в тексте (echo -e).
fn unescape(s: &str) -> String {
    let mut out = String::new();
    let mut it = s.chars().peekable();
    while let Some(c) = it.next() {
        if c == '\\' {
            match it.next() {
                Some('n') => out.push('\n'),
                Some('t') => out.push('\t'),
                Some('\\') => out.push('\\'),
                Some('e') => {}
                Some(other) => {
                    out.push('\\');
                    out.push(other);
                }
                None => out.push('\\'),
            }
        } else {
            out.push(c);
        }
    }
    out
}

pub fn echo(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    let _ = ctx;
    let mut rest = args;
    let mut escapes = false;
    while let Some(first) = rest.first() {
        match first.as_str() {
            "-n" => rest = &rest[1..],
            "-e" => {
                escapes = true;
                rest = &rest[1..];
            }
            "-ne" | "-en" => {
                escapes = true;
                rest = &rest[1..];
            }
            _ => break,
        }
    }
    let text = rest.join(" ");
    let text = if escapes { unescape(&text) } else { text };
    let mut out = CmdOutput::ok();
    out.push(text);
    out
}

pub fn find(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    let mut root = ".".to_string();
    let mut pattern = "*".to_string();
    let mut kind: Option<char> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-name" | "-iname" if i + 1 < args.len() => {
                pattern = args[i + 1].clone();
                i += 1;
            }
            "-type" if i + 1 < args.len() => {
                kind = args[i + 1].chars().next();
                i += 1;
            }
            s if s.starts_with('-') => return CmdOutput::err(trf!("find: неизвестный ключ {}", "find: unknown option {}", s)),
            s => root = s.to_string(),
        }
        i += 1;
    }
    let root_abs = ctx.path(&root);
    if !ctx.fs().exists(&root_abs) {
        return CmdOutput::err(trf!("find: нет такого файла или каталога: {}", "find: no such file or directory: {}", root));
    }
    let mut out = CmdOutput::ok();
    let found = ctx.fs().find(&root_abs, &pattern);
    let shown_root = root.trim_end_matches('/').to_string();
    for p in found {
        let is_dir = ctx.fs().is_dir(&p);
        match kind {
            Some('f') if is_dir => continue,
            Some('d') if !is_dir => continue,
            _ => {}
        }
        let rel = p.strip_prefix(&root_abs).unwrap_or(&p);
        out.push(format!("{}{}", shown_root, rel));
    }
    out
}

pub fn du(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    let a = Args::parse(args, &[]);
    let human = a.has("h");
    let summary = a.has("s");
    let target = ctx.path(a.pos.first().map(|s| s.as_str()).unwrap_or("."));
    let user = ctx.user();
    let mut out = CmdOutput::ok();
    if !ctx.fs().exists(&target) {
        return CmdOutput::err(trf!("du: нет такого файла или каталога: {}", "du: no such file or directory: {}", target));
    }
    let mut rows: Vec<Vec<String>> = Vec::new();
    if !summary && ctx.fs().is_dir(&target) {
        match ctx.fs().list(&target, &user) {
            Ok(entries) => {
                for e in entries.iter().filter(|e| e.is_dir) {
                    let size = ctx.fs().deep_size(&e.path).unwrap_or(0);
                    rows.push(vec![fmt_size(size, human), e.path.clone()]);
                }
            }
            Err(e) => return CmdOutput::err(format!("du: {}", e)),
        }
    }
    let total = ctx.fs().deep_size(&target).unwrap_or(0);
    rows.push(vec![fmt_size(total, human), target]);
    out.push_lines(table(&rows, &[true, false]));
    out
}

pub fn df(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    let a = Args::parse(args, &[]);
    let human = a.has("h");
    let total = ctx.machine().disk_mb as u64 * 1024 * 1024;
    let used = ctx.fs().total_size() + 212 * 1024 * 1024;
    let avail = total.saturating_sub(used);
    let pct = if total > 0 { used * 100 / total } else { 0 };
    let f = |v: u64| if human { human_size(v) } else { (v / 1024).to_string() };
    let rows = vec![
        vec![tr!("Файловая система", "Filesystem").to_string(), if human { tr!("Размер", "Size").into() } else { tr!("1K-блоков", "1K-blocks").into() }, tr!("Занято", "Used").into(), tr!("Свободно", "Avail").into(), tr!("Исп%", "Use%").into(), tr!("Точка", "Mounted on").into()],
        vec!["/dev/sda1".to_string(), f(total), f(used), f(avail), format!("{}%", pct), "/".into()],
        vec!["tmpfs".to_string(), f(ctx.machine().ram_mb as u64 * 1024 * 512), f(0), f(ctx.machine().ram_mb as u64 * 1024 * 512), "0%".into(), "/tmp".into()],
    ];
    let mut out = CmdOutput::ok();
    out.push_lines(table(&rows, &[false, true, true, true, true, false]));
    out
}

pub fn chmod(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    if args.len() < 2 {
        return CmdOutput::err(tr!("chmod: укажите режим и файл (например chmod 755 run.adr)", "chmod: specify mode and file (e.g. chmod 755 run.adr)"));
    }
    let user = ctx.user();
    let spec = &args[0];
    let mut out = CmdOutput::ok();
    for f in &args[1..] {
        let path = ctx.path(f);
        let cur = match ctx.fs().stat(&path) {
            Ok(st) => st.mode,
            Err(e) => {
                out.push_err(format!("chmod: {}", e));
                continue;
            }
        };
        let mode = match vfs::parse_mode(spec, cur) {
            Some(m) => m,
            None => return CmdOutput::err(trf!("chmod: непонятный режим «{}»", "chmod: invalid mode '{}'", spec)),
        };
        if let Err(e) = ctx.fs_mut().chmod(&path, mode, &user) {
            out.push_err(format!("chmod: {}", e));
        }
    }
    out
}

pub fn chown(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    if args.len() < 2 {
        return CmdOutput::err(tr!("chown: укажите пользователя и файл", "chown: specify user and file"));
    }
    let user = ctx.user();
    let owner = args[0].split(':').next().unwrap_or("").to_string();
    if !ctx.machine().users.exists(&owner) {
        return CmdOutput::err(trf!("chown: нет пользователя «{}»", "chown: no such user '{}'", owner));
    }
    let mut out = CmdOutput::ok();
    for f in &args[1..] {
        let path = ctx.path(f);
        if !ctx.fs().exists(&path) {
            out.push_err(trf!("chown: нет такого файла или каталога: {}", "chown: no such file or directory: {}", path));
            continue;
        }
        if let Err(e) = ctx.fs_mut().chown(&path, &owner, &user) {
            out.push_err(format!("chown: {}", e));
        }
    }
    out
}

/// Вид файла по содержимому и расширению.
pub fn describe_file(ctx: &Ctx<'_>, path: &str) -> Result<String, vfs::FsError> {
    let st = ctx.fs().stat(path)?;
    if st.is_dir {
        return Ok(tr!("каталог", "directory").to_string());
    }
    if st.is_binary {
        return Ok(if st.size == 0 { tr!("пустой", "empty").to_string() } else { tr!("двоичный файл (исполняемый)", "binary file (executable)").to_string() });
    }
    if st.size == 0 {
        return Ok(tr!("пустой", "empty").to_string());
    }
    let ext = st.name.rsplit('.').next().unwrap_or("").to_lowercase();
    Ok(match ext.as_str() {
        "adr" => tr!("скрипт Adder, текст UTF-8", "Adder script, UTF-8 text"),
        "log" => tr!("журнал, текст UTF-8", "log, UTF-8 text"),
        "conf" | "cfg" | "ini" => tr!("файл настроек, текст UTF-8", "config file, UTF-8 text"),
        "json" => tr!("данные JSON, текст UTF-8", "JSON data, UTF-8 text"),
        "html" | "htm" => tr!("документ HTML, текст UTF-8", "HTML document, UTF-8 text"),
        "tar" | "zip" => tr!("архив Blackwire", "Blackwire archive"),
        "md" => tr!("документ Markdown, текст UTF-8", "Markdown document, UTF-8 text"),
        "csv" => tr!("таблица CSV, текст UTF-8", "CSV table, UTF-8 text"),
        _ => tr!("текст UTF-8", "UTF-8 text"),
    }
    .to_string())
}

pub fn file(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    if args.is_empty() {
        return CmdOutput::err(tr!("file: укажите файл", "file: specify a file"));
    }
    let mut out = CmdOutput::ok();
    let rows: Vec<Vec<String>> = args
        .iter()
        .map(|f| {
            let path = ctx.path(f);
            let desc = describe_file(ctx, &path).unwrap_or_else(|e| e.to_string());
            vec![format!("{}:", f), desc]
        })
        .collect();
    out.push_lines(table(&rows, &[false, false]));
    out
}

pub fn stat(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    if args.is_empty() {
        return CmdOutput::err(tr!("stat: укажите файл", "stat: specify a file"));
    }
    let mut out = CmdOutput::ok();
    for f in args {
        let path = ctx.path(f);
        match ctx.fs().stat(&path) {
            Ok(st) => {
                out.push_kind(trf!("  Файл: {}", "  File: {}", st.path), OutKind::Accent);
                out.push(trf!("  Вид: {}", "  Type: {}", if st.is_dir { tr!("каталог", "directory") } else if st.is_binary { tr!("двоичный файл", "binary file") } else { tr!("текстовый файл", "text file") }));
                out.push(trf!("  Размер: {} байт", "  Size: {} bytes", st.size));
                out.push(trf!("  Права: {} ({:o})   Владелец: {}", "  Access: {} ({:o})   Owner: {}", mode_string(st.mode, st.is_dir), st.mode, st.owner));
                out.push(trf!("  Изменён: {}", "  Modified: {}", GameClock::format_time(st.modified)));
            }
            Err(e) => out.push_err(format!("stat: {}", e)),
        }
    }
    out
}

pub fn tree(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    let a = Args::parse(args, &["L"]);
    let all = a.has("a");
    let max_depth = a.opt_usize("L", 8);
    let root = ctx.path(a.pos.first().map(|s| s.as_str()).unwrap_or("."));
    let user = ctx.user();
    if !ctx.fs().is_dir(&root) {
        return CmdOutput::err(trf!("tree: не каталог: {}", "tree: not a directory: {}", root));
    }
    let mut out = CmdOutput::ok();
    out.push_kind(a.pos.first().cloned().unwrap_or_else(|| ".".to_string()), OutKind::Accent);
    let mut dirs = 0usize;
    let mut files = 0usize;
    tree_rec(ctx, &root, &user, "", 1, max_depth, all, &mut out, &mut dirs, &mut files);
    out.push("");
    out.push_kind(trf!("{} каталогов, {} файлов", "{} directories, {} files", dirs, files), OutKind::Dim);
    out
}

#[allow(clippy::too_many_arguments)]
fn tree_rec(
    ctx: &Ctx<'_>,
    dir: &str,
    user: &str,
    prefix: &str,
    depth: usize,
    max_depth: usize,
    all: bool,
    out: &mut CmdOutput,
    dirs: &mut usize,
    files: &mut usize,
) {
    if out.lines.len() > 400 {
        return;
    }
    let entries = match ctx.fs().list(dir, user) {
        Ok(mut v) => {
            if !all {
                v.retain(|e| !e.name.starts_with('.'));
            }
            v
        }
        Err(_) => {
            out.push_kind(trf!("{}└── [нет доступа]", "{}└── [access denied]", prefix), OutKind::Dim);
            return;
        }
    };
    let n = entries.len();
    for (i, e) in entries.iter().enumerate() {
        let last = i + 1 == n;
        let branch = if last { "└── " } else { "├── " };
        out.push(format!("{}{}{}", prefix, branch, if e.is_dir { format!("{}/", e.name) } else { e.name.clone() }));
        if e.is_dir {
            *dirs += 1;
            if depth < max_depth {
                let next = format!("{}{}", prefix, if last { "    " } else { "│   " });
                tree_rec(ctx, &e.path, user, &next, depth + 1, max_depth, all, out, dirs, files);
            }
        } else {
            *files += 1;
        }
    }
}
