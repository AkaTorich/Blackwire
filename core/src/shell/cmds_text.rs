//! Текстовые фильтры (работают с файлами или со входом конвейера) и архивы.

use super::cmds_fs::inputs;
use super::util::{base64_decode, base64_encode, fake_md5, table, Args};
use super::{CmdOutput, Ctx};
use crate::vfs::{self, FsError};

/// Строки текста без последнего пустого хвоста после `\n`.
fn lines_of(text: &str) -> Vec<String> {
    let body = text.strip_suffix('\n').unwrap_or(text);
    if body.is_empty() {
        return Vec::new();
    }
    body.split('\n').map(|s| s.to_string()).collect()
}

pub fn head(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    let a = Args::parse(args, &["n"]);
    let n = a.opt_usize("n", 10);
    let mut out = CmdOutput::ok();
    let ins = inputs(ctx, "head", &a.pos, &mut out);
    let many = ins.len() > 1;
    for (name, text) in ins {
        if many {
            out.push_kind(format!("==> {} <==", name), crate::types::OutKind::Accent);
        }
        out.push_lines(lines_of(&text).into_iter().take(n));
    }
    out
}

pub fn tail(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    let a = Args::parse(args, &["n"]);
    let n = a.opt_usize("n", 10);
    let mut out = CmdOutput::ok();
    let ins = inputs(ctx, "tail", &a.pos, &mut out);
    let many = ins.len() > 1;
    for (name, text) in ins {
        if many {
            out.push_kind(format!("==> {} <==", name), crate::types::OutKind::Accent);
        }
        let l = lines_of(&text);
        let skip = l.len().saturating_sub(n);
        out.push_lines(l.into_iter().skip(skip));
    }
    out
}

pub fn grep(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    let a = Args::parse(args, &[]);
    if a.pos.is_empty() {
        return CmdOutput::err("grep: укажите шаблон");
    }
    let icase = a.has("i");
    let numbers = a.has("n");
    let invert = a.has("v");
    let count = a.has("c");
    let recursive = a.has("r") || a.has("R");
    let pattern = if icase { a.pos[0].to_lowercase() } else { a.pos[0].clone() };
    let mut files: Vec<String> = a.pos[1..].to_vec();
    let user = ctx.user();
    if recursive {
        let roots = if files.is_empty() { vec![".".to_string()] } else { files.clone() };
        files = Vec::new();
        for r in roots {
            let abs = ctx.path(&r);
            if ctx.fs().is_dir(&abs) {
                for p in ctx.fs().find(&abs, "*") {
                    if ctx.fs().is_file(&p) && ctx.fs().read(&p, &user).is_ok() {
                        files.push(p);
                    }
                }
            } else {
                files.push(r);
            }
        }
    }
    let mut out = CmdOutput::ok();
    let ins = inputs(ctx, "grep", &files, &mut out);
    let many = ins.len() > 1;
    let mut matched_any = false;
    for (name, text) in ins {
        let mut hits = 0usize;
        for (i, line) in lines_of(&text).iter().enumerate() {
            let hay = if icase { line.to_lowercase() } else { line.clone() };
            let is_match = glob_or_substr(&pattern, &hay) != invert;
            if !is_match {
                continue;
            }
            hits += 1;
            matched_any = true;
            if count {
                continue;
            }
            let mut s = String::new();
            if many {
                s.push_str(&format!("{}:", name));
            }
            if numbers {
                s.push_str(&format!("{}:", i + 1));
            }
            s.push_str(line);
            out.push(s);
        }
        if count {
            out.push(if many { format!("{}:{}", name, hits) } else { hits.to_string() });
        }
    }
    if !matched_any && out.status == 0 {
        out.status = 1;
    }
    out
}

/// Подстрока; если в шаблоне есть `*`/`?` — сопоставление по шаблону внутри строки.
fn glob_or_substr(pattern: &str, hay: &str) -> bool {
    if vfs::has_glob(pattern) {
        vfs::glob_match(&format!("*{}*", pattern), hay)
    } else {
        hay.contains(pattern)
    }
}

pub fn wc(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    let a = Args::parse(args, &[]);
    let (want_l, want_w, want_c) = (a.has("l"), a.has("w"), a.has("c"));
    let all = !(want_l || want_w || want_c);
    let mut out = CmdOutput::ok();
    let ins = inputs(ctx, "wc", &a.pos, &mut out);
    let mut rows: Vec<Vec<String>> = Vec::new();
    let (mut tl, mut tw, mut tc) = (0usize, 0usize, 0usize);
    let many = ins.len() > 1;
    for (name, text) in ins {
        let l = text.matches('\n').count() + usize::from(!text.is_empty() && !text.ends_with('\n'));
        let w = text.split_whitespace().count();
        let c = text.len();
        tl += l;
        tw += w;
        tc += c;
        let mut row = Vec::new();
        if all || want_l {
            row.push(l.to_string());
        }
        if all || want_w {
            row.push(w.to_string());
        }
        if all || want_c {
            row.push(c.to_string());
        }
        if name != "-" {
            row.push(name);
        }
        rows.push(row);
    }
    if many {
        let mut row = Vec::new();
        if all || want_l {
            row.push(tl.to_string());
        }
        if all || want_w {
            row.push(tw.to_string());
        }
        if all || want_c {
            row.push(tc.to_string());
        }
        row.push("итого".to_string());
        rows.push(row);
    }
    out.push_lines(table(&rows, &[true, true, true, false]));
    out
}

pub fn sort(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    let a = Args::parse(args, &[]);
    let mut out = CmdOutput::ok();
    let mut all: Vec<String> = Vec::new();
    for (_, text) in inputs(ctx, "sort", &a.pos, &mut out) {
        all.extend(lines_of(&text));
    }
    if a.has("n") {
        all.sort_by(|x, y| {
            let px = x.trim().split_whitespace().next().and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);
            let py = y.trim().split_whitespace().next().and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);
            px.partial_cmp(&py).unwrap_or(std::cmp::Ordering::Equal).then_with(|| x.cmp(y))
        });
    } else {
        all.sort();
    }
    if a.has("u") {
        all.dedup();
    }
    if a.has("r") {
        all.reverse();
    }
    out.push_lines(all);
    out
}

pub fn uniq(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    let a = Args::parse(args, &[]);
    let count = a.has("c");
    let mut out = CmdOutput::ok();
    let mut all: Vec<String> = Vec::new();
    for (_, text) in inputs(ctx, "uniq", &a.pos, &mut out) {
        all.extend(lines_of(&text));
    }
    let mut result: Vec<(usize, String)> = Vec::new();
    for l in all {
        match result.last_mut() {
            Some((n, prev)) if *prev == l => *n += 1,
            _ => result.push((1, l)),
        }
    }
    for (n, l) in result {
        if count {
            out.push(format!("{:7} {}", n, l));
        } else {
            out.push(l);
        }
    }
    out
}

fn parse_fields(spec: &str) -> Vec<usize> {
    let mut v = Vec::new();
    for part in spec.split(',') {
        if let Some((a, b)) = part.split_once('-') {
            if let (Ok(a), Ok(b)) = (a.parse::<usize>(), b.parse::<usize>()) {
                v.extend(a..=b);
            }
        } else if let Ok(n) = part.parse::<usize>() {
            v.push(n);
        }
    }
    v
}

pub fn cut(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    let a = Args::parse(args, &["d", "f", "c"]);
    let delim = a.opt("d").unwrap_or("\t").to_string();
    let fields = a.opt("f").map(parse_fields);
    let chars = a.opt("c").map(parse_fields);
    if fields.is_none() && chars.is_none() {
        return CmdOutput::err("cut: укажите -f номера (и -d разделитель) или -c позиции");
    }
    let mut out = CmdOutput::ok();
    for (_, text) in inputs(ctx, "cut", &a.pos, &mut out) {
        for line in lines_of(&text) {
            if let Some(f) = &fields {
                let parts: Vec<&str> = line.split(delim.as_str()).collect();
                let picked: Vec<&str> = f.iter().filter_map(|i| parts.get(i.wrapping_sub(1)).copied()).collect();
                out.push(picked.join(&delim));
            } else if let Some(c) = &chars {
                let cs: Vec<char> = line.chars().collect();
                let picked: String = c.iter().filter_map(|i| cs.get(i.wrapping_sub(1))).collect();
                out.push(picked);
            }
        }
    }
    out
}

/// Раскрыть набор символов `a-z` в список.
fn charset(s: &str) -> Vec<char> {
    let cs: Vec<char> = s.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < cs.len() {
        if i + 2 < cs.len() && cs[i + 1] == '-' {
            let (a, b) = (cs[i] as u32, cs[i + 2] as u32);
            if a <= b {
                for c in a..=b {
                    if let Some(ch) = char::from_u32(c) {
                        out.push(ch);
                    }
                }
            }
            i += 3;
        } else {
            out.push(cs[i]);
            i += 1;
        }
    }
    out
}

pub fn tr(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    let a = Args::parse(args, &[]);
    let delete = a.has("d");
    if a.pos.is_empty() || (!delete && a.pos.len() < 2) {
        return CmdOutput::err("tr: укажите набор1 и набор2 (или -d набор)");
    }
    let set1 = charset(&a.pos[0]);
    let set2 = if delete { Vec::new() } else { charset(&a.pos[1]) };
    let text = ctx.stdin.clone().unwrap_or_default();
    let mut out = CmdOutput::ok();
    for line in lines_of(&text) {
        let mut s = String::new();
        for c in line.chars() {
            match set1.iter().position(|x| *x == c) {
                Some(i) => {
                    if !delete {
                        s.push(*set2.get(i).or(set2.last()).unwrap_or(&c));
                    }
                }
                None => s.push(c),
            }
        }
        out.push(s);
    }
    out
}

pub fn rev(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    let mut out = CmdOutput::ok();
    for (_, text) in inputs(ctx, "rev", args, &mut out) {
        for line in lines_of(&text) {
            out.push(line.chars().rev().collect::<String>());
        }
    }
    out
}

pub fn tee(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    let a = Args::parse(args, &[]);
    let text = ctx.stdin.clone().unwrap_or_default();
    let user = ctx.user();
    let mut out = CmdOutput::ok();
    for f in &a.pos {
        let path = ctx.path(f);
        let body = if text.is_empty() { String::new() } else { format!("{}\n", text) };
        let r = if a.has("a") { ctx.fs_mut().append(&path, &body, &user) } else { ctx.fs_mut().write(&path, &body, &user) };
        if let Err(e) = r {
            out.push_err(format!("tee: {}", e));
        }
    }
    out.push_lines(lines_of(&text));
    out
}

pub fn diff(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    if args.len() != 2 {
        return CmdOutput::err("diff: укажите два файла");
    }
    let user = ctx.user();
    let mut texts = Vec::new();
    for f in args {
        let path = ctx.path(f);
        match ctx.fs().read(&path, &user) {
            Ok(t) => texts.push(lines_of(&t)),
            Err(e) => return CmdOutput::err(format!("diff: {}", e)),
        }
    }
    let (a, b) = (&texts[0], &texts[1]);
    let n = a.len().min(600);
    let m = b.len().min(600);
    // таблица длин общих подпоследовательностей
    let mut dp = vec![vec![0usize; m + 1]; n + 1];
    for i in (0..n).rev() {
        for j in (0..m).rev() {
            dp[i][j] = if a[i] == b[j] { dp[i + 1][j + 1] + 1 } else { dp[i + 1][j].max(dp[i][j + 1]) };
        }
    }
    let mut out = CmdOutput::ok();
    let (mut i, mut j) = (0, 0);
    while i < n || j < m {
        if i < n && j < m && a[i] == b[j] {
            i += 1;
            j += 1;
        } else if j < m && (i >= n || dp[i][j + 1] >= dp[i + 1][j]) {
            out.push_kind(format!("+ {}", b[j]), crate::types::OutKind::Success);
            j += 1;
        } else {
            out.push_kind(format!("- {}", a[i]), crate::types::OutKind::Warn);
            i += 1;
        }
    }
    if out.lines.is_empty() {
        out.push_kind("файлы одинаковы", crate::types::OutKind::Dim);
    } else {
        out.status = 1;
    }
    out
}

pub fn md5(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    let a = Args::parse(args, &["s"]);
    let mut out = CmdOutput::ok();
    if let Some(s) = a.opt("s") {
        out.push(format!("{}  \"{}\"", fake_md5(s.as_bytes()), s));
        return out;
    }
    let user = ctx.user();
    if a.pos.is_empty() {
        let text = ctx.stdin.clone().unwrap_or_default();
        out.push(format!("{}  -", fake_md5(text.as_bytes())));
        return out;
    }
    for f in &a.pos {
        let path = ctx.path(f);
        match ctx.fs().read_bytes(&path, &user) {
            Ok(bytes) => out.push(format!("{}  {}", fake_md5(&bytes), f)),
            Err(e) => out.push_err(format!("md5: {}", e)),
        }
    }
    out
}

pub fn base64(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    let a = Args::parse(args, &[]);
    let decode = a.has("d");
    let user = ctx.user();
    let data: Vec<u8> = match a.pos.first() {
        Some(f) => {
            let path = ctx.path(f);
            match ctx.fs().read_bytes(&path, &user) {
                Ok(b) => b,
                Err(e) => return CmdOutput::err(format!("base64: {}", e)),
            }
        }
        None => ctx.stdin.clone().unwrap_or_default().into_bytes(),
    };
    let mut out = CmdOutput::ok();
    if decode {
        let text = String::from_utf8_lossy(&data).to_string();
        match base64_decode(&text) {
            Some(bytes) => out.push(String::from_utf8_lossy(&bytes).to_string()),
            None => return CmdOutput::err("base64: неверные данные"),
        }
    } else {
        let s = base64_encode(&data);
        for chunk in s.as_bytes().chunks(76) {
            out.push(String::from_utf8_lossy(chunk).to_string());
        }
        if s.is_empty() {
            out.push("");
        }
    }
    out
}

pub fn hexdump(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    let a = Args::parse(args, &["n"]);
    let limit = a.opt_usize("n", 512);
    let user = ctx.user();
    let data: Vec<u8> = match a.pos.first() {
        Some(f) => {
            let path = ctx.path(f);
            match ctx.fs().read_bytes(&path, &user) {
                Ok(b) => b,
                Err(e) => return CmdOutput::err(format!("hexdump: {}", e)),
            }
        }
        None => ctx.stdin.clone().unwrap_or_default().into_bytes(),
    };
    let mut out = CmdOutput::ok();
    for (i, chunk) in data.iter().take(limit).collect::<Vec<_>>().chunks(16).enumerate() {
        let hex: Vec<String> = chunk.iter().map(|b| format!("{:02x}", b)).collect();
        let ascii: String = chunk.iter().map(|b| if (0x20..0x7f).contains(*b) { **b as char } else { '.' }).collect();
        out.push(format!("{:08x}  {:<47}  |{}|", i * 16, hex.join(" "), ascii));
    }
    if data.len() > limit {
        out.push_kind(format!("… ещё {} байт", data.len() - limit), crate::types::OutKind::Dim);
    }
    out
}

// ---------------------------------------------------------------- архивы

const ARCHIVE_HEADER: &str = "BWARCHIVE 1\n";

/// Собрать архив: текстовый формат с заголовком на каждую запись.
fn pack(ctx: &Ctx<'_>, paths: &[String]) -> Result<(String, usize), String> {
    let user = ctx.user();
    let mut body = String::from(ARCHIVE_HEADER);
    let mut count = 0usize;
    let mut add = |rel: &str, abs: &str, body: &mut String| -> Result<(), String> {
        let st = ctx.fs().stat(abs).map_err(|e| e.to_string())?;
        if st.is_dir {
            body.push_str(&format!("@@ {} {:o} d 0\n", rel, st.mode));
        } else if st.is_binary {
            let bytes = ctx.fs().read_bytes(abs, &user).map_err(|e| e.to_string())?;
            let hex: String = bytes.iter().map(|b| format!("{:02x}", b)).collect();
            body.push_str(&format!("@@ {} {:o} b {}\n{}\n", rel, st.mode, hex.len(), hex));
        } else {
            let text = ctx.fs().read(abs, &user).map_err(|e| e.to_string())?;
            body.push_str(&format!("@@ {} {:o} f {}\n{}\n", rel, st.mode, text.len(), text));
        }
        count += 1;
        Ok(())
    };
    for p in paths {
        let abs = ctx.path(p);
        let base = vfs::basename(&abs);
        add(&base, &abs, &mut body)?;
        if ctx.fs().is_dir(&abs) {
            let mut inner: Vec<String> = Vec::new();
            ctx.fs().walk(&abs, &mut |path, _| inner.push(path.to_string()));
            for path in inner {
                let rel = format!("{}{}", base, path.strip_prefix(&abs).unwrap_or(""));
                add(&rel, &path, &mut body)?;
            }
        }
    }
    Ok((body, count))
}

/// Записи архива: (путь, режим, вид, данные).
fn entries(text: &str) -> Result<Vec<(String, u16, char, Vec<u8>)>, String> {
    let rest = text.strip_prefix(ARCHIVE_HEADER).ok_or("это не архив Blackwire")?;
    let mut out = Vec::new();
    let mut pos = 0usize;
    while pos < rest.len() {
        let line_end = rest[pos..].find('\n').map(|i| pos + i).unwrap_or(rest.len());
        let header = &rest[pos..line_end];
        pos = (line_end + 1).min(rest.len());
        if header.trim().is_empty() {
            continue;
        }
        let parts: Vec<&str> = header.split(' ').collect();
        if parts.len() != 5 || parts[0] != "@@" {
            return Err("повреждённый архив".to_string());
        }
        let mode = u16::from_str_radix(parts[2], 8).map_err(|_| "повреждённый архив")?;
        let kind = parts[3].chars().next().unwrap_or('f');
        let len: usize = parts[4].parse().map_err(|_| "повреждённый архив")?;
        let data = if kind == 'd' {
            Vec::new()
        } else {
            if pos + len > rest.len() {
                return Err("архив обрезан".to_string());
            }
            let raw = &rest[pos..pos + len];
            pos += len + 1;
            if kind == 'b' {
                (0..raw.len() / 2).filter_map(|i| u8::from_str_radix(&raw[i * 2..i * 2 + 2], 16).ok()).collect()
            } else {
                raw.as_bytes().to_vec()
            }
        };
        out.push((parts[1].to_string(), mode, kind, data));
    }
    Ok(out)
}

fn unpack(ctx: &mut Ctx<'_>, archive: &str, dest: &str, out: &mut CmdOutput) -> Result<(), String> {
    let user = ctx.user();
    let text = ctx.fs().read(archive, &user).map_err(|e| e.to_string())?;
    let list = entries(&text)?;
    for (rel, mode, kind, data) in list {
        let path = format!("{}/{}", dest.trim_end_matches('/'), rel);
        let r: Result<(), FsError> = match kind {
            'd' => ctx.fs_mut().mkdir(&path, &user, true),
            'b' => ctx.fs_mut().write_bytes(&path, data, &user),
            _ => ctx.fs_mut().write(&path, &String::from_utf8_lossy(&data), &user),
        };
        match r {
            Ok(()) => {
                ctx.fs_mut().chmod(&path, mode, &user).ok();
                out.push(rel);
            }
            Err(e) => out.push_err(format!("{}: {}", rel, e)),
        }
    }
    Ok(())
}

fn list_archive(ctx: &Ctx<'_>, archive: &str, out: &mut CmdOutput) -> Result<(), String> {
    let user = ctx.user();
    let text = ctx.fs().read(archive, &user).map_err(|e| e.to_string())?;
    let rows: Vec<Vec<String>> = entries(&text)?
        .into_iter()
        .map(|(rel, mode, kind, data)| vec![vfs::mode_string(mode, kind == 'd'), data.len().to_string(), rel])
        .collect();
    out.push_lines(table(&rows, &[false, true, false]));
    Ok(())
}

pub fn tar(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    let a = Args::parse(args, &[]);
    let create = a.has("c");
    let extract = a.has("x");
    let listing = a.has("t");
    if a.pos.is_empty() || !(create || extract || listing) {
        return CmdOutput::err("tar: использование: tar -c архив.tar файлы… | tar -x архив.tar [папка] | tar -t архив.tar");
    }
    let archive = ctx.path(&a.pos[0]);
    let mut out = CmdOutput::ok();
    if create {
        if a.pos.len() < 2 {
            return CmdOutput::err("tar: укажите, что упаковать");
        }
        match pack(ctx, &a.pos[1..]) {
            Ok((body, n)) => {
                let user = ctx.user();
                if let Err(e) = ctx.fs_mut().write(&archive, &body, &user) {
                    return CmdOutput::err(format!("tar: {}", e));
                }
                out.push_kind(format!("упаковано записей: {} → {}", n, archive), crate::types::OutKind::Dim);
            }
            Err(e) => return CmdOutput::err(format!("tar: {}", e)),
        }
    } else if extract {
        let dest = ctx.path(a.pos.get(1).map(|s| s.as_str()).unwrap_or("."));
        if let Err(e) = unpack(ctx, &archive, &dest, &mut out) {
            return CmdOutput::err(format!("tar: {}", e));
        }
    } else if let Err(e) = list_archive(ctx, &archive, &mut out) {
        return CmdOutput::err(format!("tar: {}", e));
    }
    out
}

pub fn zip(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    let a = Args::parse(args, &[]);
    if a.pos.len() < 2 {
        return CmdOutput::err("zip: использование: zip архив.zip файлы…");
    }
    let archive = ctx.path(&a.pos[0]);
    match pack(ctx, &a.pos[1..]) {
        Ok((body, n)) => {
            let user = ctx.user();
            if let Err(e) = ctx.fs_mut().write(&archive, &body, &user) {
                return CmdOutput::err(format!("zip: {}", e));
            }
            let mut out = CmdOutput::ok();
            out.push_kind(format!("упаковано записей: {} → {}", n, archive), crate::types::OutKind::Dim);
            out
        }
        Err(e) => CmdOutput::err(format!("zip: {}", e)),
    }
}

pub fn unzip(ctx: &mut Ctx<'_>, args: &[String]) -> CmdOutput {
    let a = Args::parse(args, &[]);
    if a.pos.is_empty() {
        return CmdOutput::err("unzip: использование: unzip архив.zip [папка]");
    }
    let archive = ctx.path(&a.pos[0]);
    let mut out = CmdOutput::ok();
    if a.has("l") {
        if let Err(e) = list_archive(ctx, &archive, &mut out) {
            return CmdOutput::err(format!("unzip: {}", e));
        }
        return out;
    }
    let dest = ctx.path(a.pos.get(1).map(|s| s.as_str()).unwrap_or("."));
    if let Err(e) = unpack(ctx, &archive, &dest, &mut out) {
        return CmdOutput::err(format!("unzip: {}", e));
    }
    out
}
