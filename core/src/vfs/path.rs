//! Работа с путями и правами без обращения к дереву: нормализация,
//! разбор режима (`755`, `u+x`, `rwxr-xr-x`), подстановочные знаки `*` и `?`.

/// Привести путь к каноническому абсолютному виду: убрать `.`, `..`, повторные `/`.
pub fn normalize(path: &str) -> String {
    let mut parts: Vec<&str> = Vec::new();
    for p in path.split('/') {
        match p {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            x => parts.push(x),
        }
    }
    if parts.is_empty() {
        "/".to_string()
    } else {
        format!("/{}", parts.join("/"))
    }
}

/// Путь относительно текущей папки → абсолютный нормализованный.
pub fn resolve(cwd: &str, path: &str) -> String {
    if path.is_empty() {
        return normalize(cwd);
    }
    if path.starts_with('/') {
        normalize(path)
    } else {
        normalize(&format!("{}/{}", cwd, path))
    }
}

/// Родительская папка и имя последнего элемента. Для корня — ("/", "").
pub fn split_parent(path: &str) -> (String, String) {
    let n = normalize(path);
    if n == "/" {
        return ("/".to_string(), String::new());
    }
    let i = n.rfind('/').unwrap_or(0);
    let parent = if i == 0 { "/".to_string() } else { n[..i].to_string() };
    (parent, n[i + 1..].to_string())
}

pub fn basename(path: &str) -> String {
    let (_, name) = split_parent(path);
    if name.is_empty() {
        "/".to_string()
    } else {
        name
    }
}

pub fn dirname(path: &str) -> String {
    split_parent(path).0
}

/// Склеить папку и имя.
pub fn join(dir: &str, name: &str) -> String {
    if dir.ends_with('/') {
        format!("{}{}", dir, name)
    } else {
        format!("{}/{}", dir, name)
    }
}

/// Права в виде `drwxr-xr-x`.
pub fn mode_string(mode: u16, is_dir: bool) -> String {
    let mut s = String::with_capacity(10);
    s.push(if is_dir { 'd' } else { '-' });
    for shift in [6u16, 3, 0] {
        let bits = (mode >> shift) & 7;
        s.push(if bits & 4 != 0 { 'r' } else { '-' });
        s.push(if bits & 2 != 0 { 'w' } else { '-' });
        s.push(if bits & 1 != 0 { 'x' } else { '-' });
    }
    s
}

/// Права из девяти символов `rwxr-xr-x` (первый символ `d`/`-` допускается).
pub fn mode_from_rwx(s: &str) -> Option<u16> {
    let chars: Vec<char> = s.chars().collect();
    let body: &[char] = match chars.len() {
        9 => &chars[..],
        10 => &chars[1..],
        _ => return None,
    };
    let mut mode = 0u16;
    for (i, c) in body.iter().enumerate() {
        let want = ['r', 'w', 'x'][i % 3];
        match *c {
            '-' => {}
            c if c == want => mode |= 1 << (8 - i),
            _ => return None,
        }
    }
    Some(mode)
}

/// Разобрать режим для `chmod`: восьмеричное число (`755`, `0644`), символьная
/// запись (`u+x`, `go-w`, `a=r`, `+x`, несколько через запятую) или `rwxr-xr-x`.
pub fn parse_mode(spec: &str, current: u16) -> Option<u16> {
    if spec.is_empty() {
        return None;
    }
    if spec.chars().all(|c| ('0'..='7').contains(&c)) {
        return u16::from_str_radix(spec, 8).ok().filter(|m| *m <= 0o777);
    }
    if let Some(m) = mode_from_rwx(spec) {
        return Some(m);
    }
    let mut mode = current;
    for part in spec.split(',') {
        let op_pos = part.find(|c| c == '+' || c == '-' || c == '=')?;
        let (who, rest) = part.split_at(op_pos);
        let op = rest.chars().next()?;
        let perms = &rest[1..];
        let mut mask = 0u16; // маска владельцев: 0o700, 0o070, 0o007
        let who = if who.is_empty() { "a" } else { who };
        for c in who.chars() {
            match c {
                'u' => mask |= 0o700,
                'g' => mask |= 0o070,
                'o' => mask |= 0o007,
                'a' => mask |= 0o777,
                _ => return None,
            }
        }
        let mut bits = 0u16; // биты rwx, повторённые для всех трёх групп
        for c in perms.chars() {
            match c {
                'r' => bits |= 0o444,
                'w' => bits |= 0o222,
                'x' => bits |= 0o111,
                _ => return None,
            }
        }
        let change = bits & mask;
        match op {
            '+' => mode |= change,
            '-' => mode &= !change,
            '=' => mode = (mode & !mask) | change,
            _ => return None,
        }
    }
    Some(mode)
}

/// Есть ли в строке подстановочные знаки.
pub fn has_glob(s: &str) -> bool {
    s.contains('*') || s.contains('?')
}

/// Сопоставление имени с шаблоном: `*` — любая строка, `?` — любой один символ.
pub fn glob_match(pattern: &str, name: &str) -> bool {
    let p: Vec<char> = pattern.chars().collect();
    let n: Vec<char> = name.chars().collect();
    glob_rec(&p, &n)
}

fn glob_rec(p: &[char], n: &[char]) -> bool {
    match p.first() {
        None => n.is_empty(),
        Some('*') => {
            // звёздочка съедает от нуля символов до всех
            (0..=n.len()).any(|k| glob_rec(&p[1..], &n[k..]))
        }
        Some('?') => !n.is_empty() && glob_rec(&p[1..], &n[1..]),
        Some(c) => !n.is_empty() && n[0] == *c && glob_rec(&p[1..], &n[1..]),
    }
}
