//! Скрипты Adder, окна приложений и украшения: banner, matrix, fortune, cal, bc,
//! neofetch, yes, sleep, seq.

use super::font;
use super::util::Args;
use super::{CmdOutput, Ctx};
use crate::net::machine::{LogKind, MailFolder};
use crate::session::PendingScript;
use crate::types::{OutKind, OutLine, UiAction};
use crate::vfs;
use crate::world::GameClock;

/// Приложения, которые можно открыть командой `open`.
pub const APPS: &[&str] = &["terminal", "editor", "files", "browser", "monitor", "map", "settings", "help", "mail", "scenario", "duel", "plugins"];

// ---------------------------------------------------------------- Adder

/// Зарегистрировать запуск скрипта: процесс, заявка для оболочки Tauri, действие.
fn start_script(ctx: &mut Ctx<'_>, path: &str, src: String, args: Vec<String>) -> CmdOutput {
    let user = ctx.user();
    let name = if path == "<cmd>" { "adder -c".to_string() } else { vfs::basename(path) };
    let session = ctx.session.id;
    let machine = ctx.mid();
    let pid = ctx.machine_mut().procs.spawn_script(&format!("adder {}", name), &user, session);
    ctx.session.pending_scripts.push(PendingScript {
        pid,
        session,
        machine,
        user: user.clone(),
        path: path.to_string(),
        src,
        args,
    });
    ctx.actions.push(UiAction::ScriptStarted { pid, name: name.clone() });
    ctx.log(LogKind::Exec, &format!("{} запустил скрипт {}", user, name));
    let mut out = CmdOutput::ok();
    out.push_kind(format!("запуск {} (pid {})", name, pid), OutKind::Dim);
    out
}

fn read_script(ctx: &Ctx<'_>, arg: &str) -> Result<(String, String), String> {
    let path = ctx.path(arg);
    let user = ctx.user();
    if !ctx.fs().exists(&path) {
        return Err(format!("adder: нет такого файла: {}", arg));
    }
    if ctx.fs().is_dir(&path) {
        return Err(format!("adder: это каталог: {}", arg));
    }
    if !path.ends_with(".adr") {
        return Err(format!("adder: {} — не скрипт Adder (нужно расширение .adr)", arg));
    }
    let src = ctx.fs().read(&path, &user).map_err(|e| format!("adder: {}", e))?;
    Ok((path, src))
}

pub fn adder(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    let first = match args.first() {
        Some(f) => f.as_str(),
        None => return CmdOutput::err("adder: использование: adder файл.adr [аргументы] | adder -c \"код\" | adder --check файл"),
    };
    match first {
        "-c" => {
            if args.len() < 2 {
                return CmdOutput::err("adder: после -c нужен код");
            }
            let code = args[1..].join(" ");
            start_script(ctx, "<cmd>", code, vec!["<cmd>".to_string()])
        }
        "--check" => {
            let file = match args.get(1) {
                Some(f) => f,
                None => return CmdOutput::err("adder: укажите файл для проверки"),
            };
            match read_script(ctx, file) {
                Ok((path, src)) => {
                    ctx.actions.push(UiAction::CheckScript { path: path.clone(), src });
                    let mut out = CmdOutput::ok();
                    out.push_kind(format!("проверка: {}", path), OutKind::Dim);
                    out
                }
                Err(e) => CmdOutput::err(e),
            }
        }
        _ => match read_script(ctx, first) {
            Ok((path, src)) => {
                let mut script_args = vec![path.clone()];
                script_args.extend(args[1..].iter().cloned());
                start_script(ctx, &path, src, script_args)
            }
            Err(e) => CmdOutput::err(e),
        },
    }
}

/// Запуск по пути: `./script.adr args`.
pub fn run_path(ctx: &mut Ctx<'_>, name: &str, args: &[String]) -> CmdOutput {
    let path = ctx.path(name);
    if !ctx.fs().exists(&path) {
        return CmdOutput::err(format!("{}: нет такого файла или каталога", name));
    }
    if ctx.fs().is_dir(&path) {
        return CmdOutput::err(format!("{}: это каталог", name));
    }
    if path.ends_with(".adr") {
        let mut all = vec![path];
        all.extend(args.iter().cloned());
        return adder(ctx, &all);
    }
    CmdOutput::err(format!("{}: не скрипт Adder (нужно расширение .adr)", name))
}

// ---------------------------------------------------------------- окна

fn open_app(ctx: &mut Ctx<'_>, app: &str, arg: Option<String>) -> CmdOutput {
    ctx.actions.push(UiAction::OpenApp { app: app.to_string(), arg: arg.clone() });
    let mut out = CmdOutput::ok();
    match arg {
        Some(a) => out.push_kind(format!("открываю {}: {}", app, a), OutKind::Dim),
        None => out.push_kind(format!("открываю {}", app), OutKind::Dim),
    }
    out
}

pub fn edit(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    let arg = args.first().map(|f| ctx.path(f));
    if let Some(p) = &arg {
        if ctx.fs().is_dir(p) {
            return CmdOutput::err(format!("edit: это каталог: {}", p));
        }
    }
    open_app(ctx, "editor", arg)
}

pub fn open(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    let app = match args.first() {
        Some(a) => a.as_str(),
        None => return CmdOutput::err(format!("open: укажите приложение: {}", APPS.join(", "))),
    };
    if !APPS.contains(&app) {
        return CmdOutput::err(format!("open: нет приложения «{}». Есть: {}", app, APPS.join(", ")));
    }
    let arg = args.get(1).map(|a| match app {
        "editor" | "files" => ctx.path(a),
        _ => a.clone(),
    });
    open_app(ctx, app, arg)
}

pub fn files(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    let path = ctx.path(args.first().map(|s| s.as_str()).unwrap_or("."));
    if !ctx.fs().is_dir(&path) {
        return CmdOutput::err(format!("files: не каталог: {}", path));
    }
    open_app(ctx, "files", Some(path))
}

pub fn browser(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    open_app(ctx, "browser", args.first().cloned())
}

pub fn map(ctx: &mut Ctx<'_>, _args: &[String]) -> CmdOutput {
    open_app(ctx, "map", None)
}

pub fn monitor(ctx: &mut Ctx<'_>, _args: &[String]) -> CmdOutput {
    open_app(ctx, "monitor", None)
}

pub fn settings(ctx: &mut Ctx<'_>, _args: &[String]) -> CmdOutput {
    open_app(ctx, "settings", None)
}

pub fn mail(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    let a = Args::parse(args, &[]);
    if !a.has("l") {
        return open_app(ctx, "mail", None);
    }
    let addr = format!("{}@home.net", ctx.world.player.handle);
    let mut rows: Vec<Vec<String>> = Vec::new();
    let mut msgs: Vec<&crate::net::machine::MailMessage> =
        ctx.world.machines[0].mail.iter().filter(|m| m.to == addr && m.folder == MailFolder::Inbox).collect();
    msgs.sort_by(|a, b| b.time.cmp(&a.time));
    for m in msgs {
        rows.push(vec![
            if m.read { " ".into() } else { "*".into() },
            m.id.to_string(),
            GameClock::format_short(m.time),
            m.from.clone(),
            m.subject.clone(),
        ]);
    }
    let mut out = CmdOutput::ok();
    if rows.is_empty() {
        out.push_kind("писем нет", OutKind::Dim);
        return out;
    }
    out.push_lines(super::util::table(&rows, &[false, true, false, false, false]));
    out
}

// ---------------------------------------------------------------- украшения

pub fn banner(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    let text = if args.is_empty() { ctx.stdin.clone().unwrap_or_default() } else { args.join(" ") };
    if text.trim().is_empty() {
        return CmdOutput::err("banner: укажите текст");
    }
    let mut out = CmdOutput::ok();
    for row in font::render(text.trim()) {
        out.push_kind(row, OutKind::Accent);
    }
    out
}

pub fn matrix(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    let n: usize = args.first().and_then(|s| s.parse().ok()).unwrap_or(12);
    let n = n.clamp(1, 40);
    const CHARS: &[u8] = b"0123456789ABCDEF<>[]{}|/\\=+*:.;";
    let mut out = CmdOutput::ok();
    for _ in 0..n {
        let mut line = String::new();
        for _ in 0..64 {
            if ctx.world.rng.chance(0.35) {
                line.push(' ');
            } else {
                line.push(*ctx.world.rng.pick(CHARS) as char);
            }
        }
        out.lines.push(OutLine::success(line).after(70));
    }
    out
}

const FORTUNES: &[&str] = &[
    "Самый надёжный пароль — тот, который записан на бумажке под клавиатурой. Для того, кто её найдёт.",
    "Любая система защищена ровно настолько, насколько ленив её администратор.",
    "Сканируй дважды, подключайся один раз.",
    "Журнал помнит всё. Вопрос лишь в том, кто его читает.",
    "Нет неуязвимых машин — есть те, до которых ещё не дошли руки.",
    "Открытый порт — это приглашение. Баннер службы — визитка.",
    "Пока ты думаешь, что невидим, кто-то смотрит в auth.log.",
    "Самое слабое звено сети сидит перед монитором.",
    "Хороший скрипт экономит час. Плохой — стирает домашнюю папку.",
    "Если ping молчит, это ещё не значит, что там никого нет.",
    "Чем длиннее цепочка ssh, тем короче память о том, где ты.",
    "admin/admin — не пароль, а традиция.",
    "Терпение — тоже инструмент. Словарь на двести слов — тоже.",
    "Удалив файл, не забудь удалить и запись о том, что ты его удалил.",
    "В сети нет расстояний. Есть только маршруты.",
    "Самое тихое вторжение — то, после которого ничего не изменилось.",
];

pub fn fortune(ctx: &mut Ctx<'_>, _args: &[String]) -> CmdOutput {
    let s = *ctx.world.rng.pick(FORTUNES);
    let mut out = CmdOutput::ok();
    out.push_kind(s, OutKind::Accent);
    out
}

pub fn cal(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    let now = ctx.now();
    let (cy, cm, cd, _, _, _) = GameClock::civil(now);
    let (m, y) = match (args.first().and_then(|s| s.parse::<u32>().ok()), args.get(1).and_then(|s| s.parse::<i64>().ok())) {
        (Some(m), Some(y)) if (1..=12).contains(&m) => (m, y),
        (Some(m), None) if (1..=12).contains(&m) => (m, cy),
        _ => (cm, cy),
    };
    let mut out = CmdOutput::ok();
    let title = format!("{} {}", GameClock::month_name_ru(m), y);
    let title = {
        let mut t = title.chars().collect::<Vec<_>>();
        if let Some(c) = t.first_mut() {
            *c = c.to_uppercase().next().unwrap_or(*c);
        }
        t.into_iter().collect::<String>()
    };
    out.push_kind(format!("{:^27}", title), OutKind::Heading);
    out.push_kind(" Пн  Вт  Ср  Чт  Пт  Сб  Вс", OutKind::Dim);
    let first_wd = GameClock::weekday(y, m, 1);
    let days = GameClock::days_in_month(y, m);
    let mut line = "    ".repeat(first_wd as usize);
    let mut col = first_wd;
    for d in 1..=days {
        if y == cy && m == cm && d == cd {
            line.push_str(&format!("[{:2}]", d));
        } else {
            line.push_str(&format!(" {:2} ", d));
        }
        col += 1;
        if col == 7 {
            out.push(line.trim_end().to_string());
            line = String::new();
            col = 0;
        }
    }
    if !line.trim().is_empty() {
        out.push(line.trim_end().to_string());
    }
    out
}

// ---- калькулятор

struct Calc<'a> {
    toks: Vec<&'a str>,
    pos: usize,
}

impl<'a> Calc<'a> {
    fn peek(&self) -> Option<&'a str> {
        self.toks.get(self.pos).copied()
    }
    fn next(&mut self) -> Option<&'a str> {
        let t = self.peek();
        self.pos += 1;
        t
    }
    fn expr(&mut self) -> Result<f64, String> {
        let mut v = self.term()?;
        while let Some(op) = self.peek() {
            match op {
                "+" => {
                    self.next();
                    v += self.term()?;
                }
                "-" => {
                    self.next();
                    v -= self.term()?;
                }
                _ => break,
            }
        }
        Ok(v)
    }
    fn term(&mut self) -> Result<f64, String> {
        let mut v = self.power()?;
        while let Some(op) = self.peek() {
            match op {
                "*" => {
                    self.next();
                    v *= self.power()?;
                }
                "/" => {
                    self.next();
                    let d = self.power()?;
                    if d == 0.0 {
                        return Err("деление на ноль".to_string());
                    }
                    v /= d;
                }
                "%" => {
                    self.next();
                    let d = self.power()?;
                    if d == 0.0 {
                        return Err("деление на ноль".to_string());
                    }
                    v %= d;
                }
                _ => break,
            }
        }
        Ok(v)
    }
    fn power(&mut self) -> Result<f64, String> {
        let base = self.unary()?;
        if self.peek() == Some("^") {
            self.next();
            let e = self.power()?;
            return Ok(base.powf(e));
        }
        Ok(base)
    }
    fn unary(&mut self) -> Result<f64, String> {
        match self.peek() {
            Some("-") => {
                self.next();
                Ok(-self.unary()?)
            }
            Some("+") => {
                self.next();
                self.unary()
            }
            _ => self.atom(),
        }
    }
    fn atom(&mut self) -> Result<f64, String> {
        match self.next() {
            None => Err("неожиданный конец выражения".to_string()),
            Some("(") => {
                let v = self.expr()?;
                if self.next() != Some(")") {
                    return Err("ожидалась «)»".to_string());
                }
                Ok(v)
            }
            Some(f) if matches!(f, "sqrt" | "abs" | "floor" | "ceil" | "round" | "ln" | "log") => {
                if self.next() != Some("(") {
                    return Err(format!("после {} ожидалась «(»", f));
                }
                let v = self.expr()?;
                if self.next() != Some(")") {
                    return Err("ожидалась «)»".to_string());
                }
                Ok(match f {
                    "sqrt" => v.sqrt(),
                    "abs" => v.abs(),
                    "floor" => v.floor(),
                    "ceil" => v.ceil(),
                    "round" => v.round(),
                    "ln" => v.ln(),
                    _ => v.log10(),
                })
            }
            Some("pi") => Ok(std::f64::consts::PI),
            Some("e") => Ok(std::f64::consts::E),
            Some(t) => {
                if let Some(h) = t.strip_prefix("0x") {
                    return i64::from_str_radix(h, 16).map(|v| v as f64).map_err(|_| format!("не число: {}", t));
                }
                t.parse::<f64>().map_err(|_| format!("не число: {}", t))
            }
        }
    }
}

fn tokenize_expr(s: &str) -> Vec<String> {
    let mut toks = Vec::new();
    let mut cur = String::new();
    for c in s.chars() {
        if c.is_alphanumeric() || c == '.' || c == '_' {
            cur.push(c);
        } else {
            if !cur.is_empty() {
                toks.push(std::mem::take(&mut cur));
            }
            if !c.is_whitespace() {
                toks.push(c.to_string());
            }
        }
    }
    if !cur.is_empty() {
        toks.push(cur);
    }
    toks
}

/// Вычислить арифметическое выражение.
pub fn calc(expr: &str) -> Result<f64, String> {
    let toks = tokenize_expr(expr);
    let refs: Vec<&str> = toks.iter().map(|s| s.as_str()).collect();
    let mut c = Calc { toks: refs, pos: 0 };
    let v = c.expr()?;
    if c.pos < c.toks.len() {
        return Err(format!("лишнее: {}", c.toks[c.pos..].join(" ")));
    }
    Ok(v)
}

pub fn bc(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    let expr = if args.is_empty() { ctx.stdin.clone().unwrap_or_default() } else { args.join(" ") };
    if expr.trim().is_empty() {
        return CmdOutput::err("bc: укажите выражение, например bc \"(2 + 3) * 4\"");
    }
    let mut out = CmdOutput::ok();
    for line in expr.lines().filter(|l| !l.trim().is_empty()) {
        match calc(line) {
            Ok(v) => {
                if v.fract() == 0.0 && v.abs() < 1e15 {
                    out.push(format!("{}", v as i64));
                } else {
                    out.push(format!("{}", (v * 1e6).round() / 1e6));
                }
            }
            Err(e) => out.push_err(format!("bc: {}", e)),
        }
    }
    out
}

pub fn yes(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    let _ = ctx;
    let text = if args.is_empty() { "y".to_string() } else { args.join(" ") };
    let mut out = CmdOutput::ok();
    for _ in 0..20 {
        out.push(text.clone());
    }
    out.push_kind("… (вывод ограничен 20 строками)", OutKind::Dim);
    out
}

pub fn sleep(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    let _ = ctx;
    let secs: f64 = match args.first().and_then(|s| s.parse().ok()) {
        Some(s) => s,
        None => return CmdOutput::err("sleep: укажите число секунд"),
    };
    let ms = (secs.clamp(0.0, 10.0) * 1000.0) as u32;
    let mut out = CmdOutput::ok();
    out.lines.push(OutLine::dim("").after(ms));
    out
}

pub fn seq(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    let _ = ctx;
    let nums: Vec<i64> = args.iter().filter_map(|s| s.parse().ok()).collect();
    let (from, to) = match nums.as_slice() {
        [to] => (1, *to),
        [from, to] => (*from, *to),
        _ => return CmdOutput::err("seq: использование: seq [от] до"),
    };
    let mut out = CmdOutput::ok();
    let mut n = 0;
    let step = if from <= to { 1 } else { -1 };
    let mut i = from;
    loop {
        out.push(i.to_string());
        n += 1;
        if i == to || n >= 500 {
            break;
        }
        i += step;
    }
    out
}

const LOGO: [&str; 9] = [
    "      ▄▄▄▄▄▄▄▄      ",
    "   ▄█▀▀      ▀▀█▄   ",
    "  █▀   ▄▄▄▄▄▄   ▀█  ",
    " █▌   █▀    ▀█   ▐█ ",
    " █▌   █  ▐█  █   ▐█ ",
    " █▌   █▄    ▄█   ▐█ ",
    "  █▄   ▀▀▀▀▀▀   ▄█  ",
    "   ▀█▄▄      ▄▄█▀   ",
    "      ▀▀▀▀▀▀▀▀      ",
];

pub fn neofetch(ctx: &mut Ctx<'_>, _args: &[String]) -> CmdOutput {
    let st = crate::sysinfo::status(ctx.world, ctx.mid());
    let m = ctx.machine();
    let user = ctx.user();
    let info = vec![
        (String::new(), format!("{}@{}", user, m.hostname)),
        (String::new(), "─".repeat(user.len() + m.hostname.len() + 1)),
        ("ОС".to_string(), m.os.clone()),
        ("Вид".to_string(), m.device_type.clone().unwrap_or_else(||m.kind.title().to_string())),
        ("Ядро".to_string(), "6.1.0-bw".to_string()),
        ("Работает".to_string(), GameClock::format_duration(st.uptime)),
        ("Оболочка".to_string(), "bwsh 1.0".to_string()),
        ("Процессор".to_string(), m.cpu.clone()),
        ("Память".to_string(), format!("{} МБ / {} МБ", st.mem_used, st.mem_total)),
        ("Адрес".to_string(), m.ip.clone()),
        ("Процессов".to_string(), st.procs.len().to_string()),
    ];
    let mut out = CmdOutput::ok();
    let rows = LOGO.len().max(info.len());
    for i in 0..rows {
        let logo = LOGO.get(i).copied().unwrap_or("                    ");
        let right = match info.get(i) {
            Some((k, v)) if k.is_empty() => v.clone(),
            Some((k, v)) => format!("{}: {}", k, v),
            None => String::new(),
        };
        out.push_kind(format!("{}   {}", logo, right), if i < 2 { OutKind::Accent } else { OutKind::Normal });
    }
    out
}

// ---------------------------------------------------------------- новости

/// Лента городских событий живого мира.
pub fn news(ctx: &mut Ctx<'_>, _args: &[String]) -> CmdOutput {
    let feed = crate::life::feed(ctx.world);
    let mut out = CmdOutput::ok();
    out.push_kind("Городская лента:", OutKind::Heading);
    if feed.is_empty() {
        out.push_kind("  Пока тихо.", OutKind::Dim);
        return out;
    }
    for item in feed.iter().take(12) {
        let t = GameClock::format_short(item.time);
        out.lines.push(OutLine::new(format!("  {}  {}", t, item.text), OutKind::Normal));
    }
    out
}

pub fn achievements(ctx: &mut Ctx<'_>, _args: &[String]) -> CmdOutput {
    let rows = crate::achievements::rows(ctx.world);
    let (have, total) = crate::achievements::count(ctx.world);
    let mut out = CmdOutput::ok();
    out.push_kind(format!("Достижения: {} из {}", have, total), OutKind::Heading);
    for (id,a) in crate::plugins::catalog::entries(ctx.world,"achievements"){out.push(format!("[{}] {} — {}",if ctx.world.achievements.unlocked.contains(&id){"x"}else{" "},a.name,a.desc));}
    for (a, unlocked) in rows {
        if unlocked {
            out.lines.push(OutLine::new(format!("  [x] {} — {}", a.name, a.desc), OutKind::Accent));
        } else {
            out.lines.push(OutLine::new(format!("  [ ] {} — {}", a.name, a.desc), OutKind::Dim));
        }
    }
    out
}
