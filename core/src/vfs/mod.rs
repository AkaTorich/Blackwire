//! Виртуальная файловая система одной машины: дерево каталогов и файлов,
//! владельцы, права rwx, время изменения. Методы с параметром `who` проверяют
//! права (root может всё); методы без него — служебные, для генератора.
//!
//! - `path.rs`     — пути, режимы, подстановочные знаки.
//! - `template.rs` — типичное дерево Unix-машины (`new_unix`).

mod path;
mod template;

pub use path::{
    basename, dirname, glob_match, has_glob, join, mode_from_rwx, mode_string, normalize, parse_mode,
    resolve, split_parent,
};

use std::collections::BTreeMap;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::types::FsEntry;

pub const R: u16 = 4;
pub const W: u16 = 2;
pub const X: u16 = 1;
pub const DIR_SIZE: u64 = 4096;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum FsError {
    NotFound(String),
    NotDir(String),
    IsDir(String),
    Denied(String),
    Exists(String),
    NotEmpty(String),
    Invalid(String),
    Binary(String),
}

impl fmt::Display for FsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FsError::NotFound(p) => write!(f, "нет такого файла или каталога: {}", p),
            FsError::NotDir(p) => write!(f, "не каталог: {}", p),
            FsError::IsDir(p) => write!(f, "это каталог: {}", p),
            FsError::Denied(p) => write!(f, "отказано в доступе: {}", p),
            FsError::Exists(p) => write!(f, "уже существует: {}", p),
            FsError::NotEmpty(p) => write!(f, "каталог не пуст: {}", p),
            FsError::Invalid(p) => write!(f, "недопустимая операция: {}", p),
            FsError::Binary(p) => write!(f, "двоичный файл: {}", p),
        }
    }
}

impl std::error::Error for FsError {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum FileData {
    Text(String),
    Binary(Vec<u8>),
}

impl FileData {
    pub fn len(&self) -> u64 {
        match self {
            FileData::Text(s) => s.len() as u64,
            FileData::Binary(b) => b.len() as u64,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Meta {
    pub owner: String,
    /// Права, например 0o755.
    pub mode: u16,
    /// Игровое время изменения, секунды.
    pub modified: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum NodeKind {
    Dir { children: BTreeMap<String, Node> },
    File { data: FileData },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Node {
    pub meta: Meta,
    pub kind: NodeKind,
}

impl Node {
    fn dir(owner: &str, mode: u16, t: u64) -> Node {
        Node {
            meta: Meta { owner: owner.to_string(), mode, modified: t },
            kind: NodeKind::Dir { children: BTreeMap::new() },
        }
    }

    fn file(owner: &str, mode: u16, t: u64, data: FileData) -> Node {
        Node { meta: Meta { owner: owner.to_string(), mode, modified: t }, kind: NodeKind::File { data } }
    }

    pub fn is_dir(&self) -> bool {
        matches!(self.kind, NodeKind::Dir { .. })
    }

    /// Размер: у файла — байты, у каталога — условные 4096.
    pub fn size(&self) -> u64 {
        match &self.kind {
            NodeKind::Dir { .. } => DIR_SIZE,
            NodeKind::File { data } => data.len(),
        }
    }

    /// Полный размер с содержимым подкаталогов.
    pub fn deep_size(&self) -> u64 {
        match &self.kind {
            NodeKind::Dir { children } => children.values().map(|c| c.deep_size()).sum::<u64>(),
            NodeKind::File { data } => data.len(),
        }
    }

    fn children(&self) -> Option<&BTreeMap<String, Node>> {
        match &self.kind {
            NodeKind::Dir { children } => Some(children),
            _ => None,
        }
    }

    fn children_mut(&mut self) -> Option<&mut BTreeMap<String, Node>> {
        match &mut self.kind {
            NodeKind::Dir { children } => Some(children),
            _ => None,
        }
    }
}

/// Сведения о файле для `stat`, `ls -l`, `file`.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Stat {
    pub path: String,
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
    pub modified: u64,
    pub owner: String,
    pub mode: u16,
    pub is_binary: bool,
}

impl Stat {
    pub fn entry(&self) -> FsEntry {
        FsEntry {
            name: self.name.clone(),
            path: self.path.clone(),
            is_dir: self.is_dir,
            size: self.size,
            modified: self.modified,
            owner: self.owner.clone(),
            mode: mode_string(self.mode, self.is_dir),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Vfs {
    root: Node,
    /// Текущее игровое время — им помечаются изменения.
    clock: u64,
}

fn allowed(meta: &Meta, who: &str, bit: u16) -> bool {
    if who == "root" {
        return true;
    }
    let shift = if meta.owner == who { 6 } else { 0 };
    (meta.mode >> shift) & bit != 0
}

fn comps(path: &str) -> Vec<String> {
    normalize(path).split('/').filter(|s| !s.is_empty()).map(|s| s.to_string()).collect()
}

impl Vfs {
    /// Пустое дерево с одним корнем. Часы — за месяц до начала игры, чтобы файлы
    /// шаблона и генератора выглядели «старыми», а не от 1970 года.
    pub fn empty(owner: &str) -> Vfs {
        let t = crate::world::START_EPOCH - 30 * 86_400;
        Vfs { root: Node::dir(owner, 0o755, t), clock: t }
    }

    /// Типичная Unix-машина с домом пользователя `owner`.
    pub fn new_unix(owner: &str) -> Vfs {
        template::new_unix(owner)
    }

    /// Домашняя папка нового пользователя.
    pub fn add_user_home(&mut self, user: &str) {
        template::add_user_home(self, user);
    }

    pub fn set_clock(&mut self, t: u64) {
        self.clock = t;
    }

    pub fn clock(&self) -> u64 {
        self.clock
    }

    // ---------- навигация без проверки прав ----------

    fn node(&self, path: &str) -> Result<&Node, FsError> {
        let mut cur = &self.root;
        for c in comps(path) {
            cur = match cur.children() {
                Some(ch) => ch.get(&c).ok_or_else(|| FsError::NotFound(normalize(path)))?,
                None => return Err(FsError::NotDir(normalize(path))),
            };
        }
        Ok(cur)
    }

    fn node_mut(&mut self, path: &str) -> Result<&mut Node, FsError> {
        let mut cur = &mut self.root;
        for c in comps(path) {
            cur = match cur.children_mut() {
                Some(ch) => ch.get_mut(&c).ok_or_else(|| FsError::NotFound(normalize(path)))?,
                None => return Err(FsError::NotDir(normalize(path))),
            };
        }
        Ok(cur)
    }

    /// Проверить, что все каталоги на пути (кроме последнего элемента) проходимы (x).
    fn check_traverse(&self, path: &str, who: &str) -> Result<(), FsError> {
        if who == "root" {
            return Ok(());
        }
        let parts = comps(path);
        let mut cur = &self.root;
        let mut walked = String::new();
        for (i, c) in parts.iter().enumerate() {
            // cur — каталог, в котором лежит parts[i]; в него нужно право входа
            if !allowed(&cur.meta, who, X) {
                return Err(FsError::Denied(if walked.is_empty() { "/".to_string() } else { walked }));
            }
            if i + 1 == parts.len() {
                break;
            }
            walked.push('/');
            walked.push_str(c);
            match cur.children().and_then(|ch| ch.get(c)) {
                Some(n) => cur = n,
                None => break,
            }
        }
        Ok(())
    }

    fn need(&self, meta: &Meta, who: &str, bit: u16, path: &str) -> Result<(), FsError> {
        if allowed(meta, who, bit) {
            Ok(())
        } else {
            Err(FsError::Denied(normalize(path)))
        }
    }

    // ---------- служебные операции (без проверки прав) ----------

    /// Создать все недостающие каталоги.
    pub fn mkdir_all(&mut self, path: &str, owner: &str) -> Result<(), FsError> {
        let t = self.clock;
        let mut cur = &mut self.root;
        for c in comps(path) {
            let ch = match cur.children_mut() {
                Some(ch) => ch,
                None => return Err(FsError::NotDir(normalize(path))),
            };
            cur = ch.entry(c).or_insert_with(|| Node::dir(owner, 0o755, t));
        }
        if !cur.is_dir() {
            return Err(FsError::NotDir(normalize(path)));
        }
        Ok(())
    }

    /// Записать текст, создав недостающие каталоги; права 0644, без проверки прав.
    pub fn write_text(&mut self, path: &str, text: &str, owner: &str) -> Result<(), FsError> {
        self.write_data(path, FileData::Text(text.to_string()), owner, 0o644)
    }

    /// Записать двоичные данные без проверки прав.
    pub fn write_binary(&mut self, path: &str, bytes: Vec<u8>, owner: &str, mode: u16) -> Result<(), FsError> {
        self.write_data(path, FileData::Binary(bytes), owner, mode)
    }

    fn write_data(&mut self, path: &str, data: FileData, owner: &str, mode: u16) -> Result<(), FsError> {
        let (parent, name) = split_parent(path);
        if name.is_empty() {
            return Err(FsError::IsDir("/".to_string()));
        }
        self.mkdir_all(&parent, owner)?;
        let t = self.clock;
        let dir = self.node_mut(&parent)?;
        let ch = dir.children_mut().ok_or_else(|| FsError::NotDir(parent.clone()))?;
        match ch.get_mut(&name) {
            Some(n) if n.is_dir() => return Err(FsError::IsDir(normalize(path))),
            Some(n) => {
                n.kind = NodeKind::File { data };
                n.meta.modified = t;
            }
            None => {
                ch.insert(name, Node::file(owner, mode, t, data));
            }
        }
        dir.meta.modified = t;
        Ok(())
    }

    /// Сменить права без проверки.
    pub fn set_mode(&mut self, path: &str, mode: u16) -> Result<(), FsError> {
        self.node_mut(path)?.meta.mode = mode & 0o777;
        Ok(())
    }

    /// Сменить владельца без проверки.
    pub fn set_owner(&mut self, path: &str, owner: &str) -> Result<(), FsError> {
        self.node_mut(path)?.meta.owner = owner.to_string();
        Ok(())
    }

    /// Прочитать текст без проверки прав.
    pub fn read_text(&self, path: &str) -> Result<String, FsError> {
        match &self.node(path)?.kind {
            NodeKind::Dir { .. } => Err(FsError::IsDir(normalize(path))),
            NodeKind::File { data: FileData::Text(s) } => Ok(s.clone()),
            NodeKind::File { data: FileData::Binary(_) } => Err(FsError::Binary(normalize(path))),
        }
    }

    /// Дописать текст без проверки прав (журналы).
    pub fn append_text(&mut self, path: &str, text: &str, owner: &str) -> Result<(), FsError> {
        let mut s = self.read_text(path).unwrap_or_default();
        s.push_str(text);
        self.write_text(path, &s, owner)
    }

    // ---------- запросы ----------

    pub fn exists(&self, path: &str) -> bool {
        self.node(path).is_ok()
    }

    pub fn is_dir(&self, path: &str) -> bool {
        self.node(path).map(|n| n.is_dir()).unwrap_or(false)
    }

    pub fn is_file(&self, path: &str) -> bool {
        self.node(path).map(|n| !n.is_dir()).unwrap_or(false)
    }

    pub fn stat(&self, path: &str) -> Result<Stat, FsError> {
        let n = self.node(path)?;
        let p = normalize(path);
        Ok(Stat {
            name: basename(&p),
            path: p,
            is_dir: n.is_dir(),
            size: n.size(),
            modified: n.meta.modified,
            owner: n.meta.owner.clone(),
            mode: n.meta.mode,
            is_binary: matches!(n.kind, NodeKind::File { data: FileData::Binary(_) }),
        })
    }

    /// Размер с содержимым (для `du`).
    pub fn deep_size(&self, path: &str) -> Result<u64, FsError> {
        Ok(self.node(path)?.deep_size())
    }

    /// Занято всего, байт.
    pub fn total_size(&self) -> u64 {
        self.root.deep_size()
    }

    /// Список каталога без проверки прав (для подсказки Tab).
    pub fn entries_unchecked(&self, path: &str) -> Result<Vec<FsEntry>, FsError> {
        let n = self.node(path)?;
        let p = normalize(path);
        let ch = n.children().ok_or_else(|| FsError::NotDir(p.clone()))?;
        Ok(ch
            .iter()
            .map(|(name, c)| FsEntry {
                name: name.clone(),
                path: join(&p, name),
                is_dir: c.is_dir(),
                size: c.size(),
                modified: c.meta.modified,
                owner: c.meta.owner.clone(),
                mode: mode_string(c.meta.mode, c.is_dir()),
            })
            .collect())
    }

    // ---------- операции с проверкой прав ----------

    /// Список каталога: нужны x на пути и r на самом каталоге.
    pub fn list(&self, path: &str, who: &str) -> Result<Vec<FsEntry>, FsError> {
        self.check_traverse(path, who)?;
        let n = self.node(path)?;
        if !n.is_dir() {
            return Err(FsError::NotDir(normalize(path)));
        }
        self.need(&n.meta, who, R, path)?;
        self.entries_unchecked(path)
    }

    /// Прочитать текстовый файл.
    pub fn read(&self, path: &str, who: &str) -> Result<String, FsError> {
        self.check_traverse(path, who)?;
        let n = self.node(path)?;
        if n.is_dir() {
            return Err(FsError::IsDir(normalize(path)));
        }
        self.need(&n.meta, who, R, path)?;
        self.read_text(path)
    }

    /// Прочитать байты (текст отдаётся как UTF-8).
    pub fn read_bytes(&self, path: &str, who: &str) -> Result<Vec<u8>, FsError> {
        self.check_traverse(path, who)?;
        let n = self.node(path)?;
        self.need(&n.meta, who, R, path)?;
        match &n.kind {
            NodeKind::Dir { .. } => Err(FsError::IsDir(normalize(path))),
            NodeKind::File { data: FileData::Text(s) } => Ok(s.as_bytes().to_vec()),
            NodeKind::File { data: FileData::Binary(b) } => Ok(b.clone()),
        }
    }

    /// Записать текст: существующий файл — w на нём, новый — w на каталоге.
    pub fn write(&mut self, path: &str, text: &str, who: &str) -> Result<(), FsError> {
        self.write_checked(path, FileData::Text(text.to_string()), who)
    }

    pub fn write_bytes(&mut self, path: &str, bytes: Vec<u8>, who: &str) -> Result<(), FsError> {
        self.write_checked(path, FileData::Binary(bytes), who)
    }

    fn write_checked(&mut self, path: &str, data: FileData, who: &str) -> Result<(), FsError> {
        self.check_traverse(path, who)?;
        let (parent, name) = split_parent(path);
        if name.is_empty() {
            return Err(FsError::IsDir("/".to_string()));
        }
        let t = self.clock;
        let dir = self.node_mut(&parent)?;
        if !dir.is_dir() {
            return Err(FsError::NotDir(parent));
        }
        let dir_meta = dir.meta.clone();
        let ch = dir.children_mut().expect("каталог");
        match ch.get_mut(&name) {
            Some(n) if n.is_dir() => Err(FsError::IsDir(normalize(path))),
            Some(n) => {
                if !allowed(&n.meta, who, W) {
                    return Err(FsError::Denied(normalize(path)));
                }
                n.kind = NodeKind::File { data };
                n.meta.modified = t;
                Ok(())
            }
            None => {
                if !allowed(&dir_meta, who, W) {
                    return Err(FsError::Denied(parent));
                }
                ch.insert(name, Node::file(who, 0o644, t, data));
                dir.meta.modified = t;
                Ok(())
            }
        }
    }

    /// Дописать в конец (создаёт файл, если его нет).
    pub fn append(&mut self, path: &str, text: &str, who: &str) -> Result<(), FsError> {
        let mut s = match self.node(path) {
            Ok(_) => self.read(path, who)?,
            Err(_) => String::new(),
        };
        s.push_str(text);
        self.write(path, &s, who)
    }

    /// Создать пустой файл или обновить время.
    pub fn touch(&mut self, path: &str, who: &str) -> Result<(), FsError> {
        if self.exists(path) {
            self.check_traverse(path, who)?;
            let t = self.clock;
            let n = self.node_mut(path)?;
            if !allowed(&n.meta, who, W) {
                return Err(FsError::Denied(normalize(path)));
            }
            n.meta.modified = t;
            Ok(())
        } else {
            self.write(path, "", who)
        }
    }

    /// Создать каталог; с `parents` — все недостающие по пути.
    pub fn mkdir(&mut self, path: &str, who: &str, parents: bool) -> Result<(), FsError> {
        self.check_traverse(path, who)?;
        let p = normalize(path);
        if self.exists(&p) {
            if parents && self.is_dir(&p) {
                return Ok(());
            }
            return Err(FsError::Exists(p));
        }
        let (parent, name) = split_parent(&p);
        if !self.exists(&parent) {
            if parents {
                self.mkdir(&parent, who, true)?;
            } else {
                return Err(FsError::NotFound(parent));
            }
        }
        let t = self.clock;
        let dir = self.node_mut(&parent)?;
        if !dir.is_dir() {
            return Err(FsError::NotDir(parent));
        }
        if !allowed(&dir.meta, who, W) {
            return Err(FsError::Denied(parent));
        }
        dir.meta.modified = t;
        dir.children_mut().expect("каталог").insert(name, Node::dir(who, 0o755, t));
        Ok(())
    }

    /// Удалить файл или каталог (каталог — только с `recursive`, кроме пустого).
    pub fn remove(&mut self, path: &str, who: &str, recursive: bool) -> Result<(), FsError> {
        let p = normalize(path);
        if p == "/" {
            return Err(FsError::Invalid("нельзя удалить корень".to_string()));
        }
        self.check_traverse(&p, who)?;
        let n = self.node(&p)?;
        if let Some(ch) = n.children() {
            if !ch.is_empty() && !recursive {
                return Err(FsError::NotEmpty(p));
            }
        }
        let (parent, name) = split_parent(&p);
        let t = self.clock;
        let dir = self.node_mut(&parent)?;
        if !allowed(&dir.meta, who, W) {
            return Err(FsError::Denied(parent));
        }
        dir.meta.modified = t;
        dir.children_mut().expect("каталог").remove(&name);
        Ok(())
    }

    /// Взять узел целиком (для rename/copy): нужны w на каталоге-источнике.
    fn take(&mut self, path: &str, who: &str) -> Result<(Node, String), FsError> {
        let p = normalize(path);
        if p == "/" {
            return Err(FsError::Invalid("нельзя переместить корень".to_string()));
        }
        self.check_traverse(&p, who)?;
        self.node(&p)?;
        let (parent, name) = split_parent(&p);
        let dir = self.node_mut(&parent)?;
        if !allowed(&dir.meta, who, W) {
            return Err(FsError::Denied(parent));
        }
        let node = dir.children_mut().expect("каталог").remove(&name).expect("узел есть");
        Ok((node, name))
    }

    /// Положить узел по пути `to` (если `to` — существующий каталог, то внутрь него).
    fn put(&mut self, to: &str, name: &str, mut node: Node, who: &str) -> Result<String, FsError> {
        let target = normalize(to);
        let (dir_path, new_name) = if self.is_dir(&target) {
            (target.clone(), name.to_string())
        } else {
            split_parent(&target)
        };
        self.check_traverse(&join(&dir_path, &new_name), who)?;
        let t = self.clock;
        let dir = self.node_mut(&dir_path)?;
        if !dir.is_dir() {
            return Err(FsError::NotDir(dir_path));
        }
        if !allowed(&dir.meta, who, W) {
            return Err(FsError::Denied(dir_path));
        }
        let ch = dir.children_mut().expect("каталог");
        if let Some(existing) = ch.get(&new_name) {
            if existing.is_dir() {
                return Err(FsError::Exists(join(&dir_path, &new_name)));
            }
        }
        node.meta.modified = t;
        ch.insert(new_name.clone(), node);
        dir.meta.modified = t;
        Ok(join(&dir_path, &new_name))
    }

    /// Переименовать или переместить. Возвращает новый путь.
    pub fn rename(&mut self, from: &str, to: &str, who: &str) -> Result<String, FsError> {
        let f = normalize(from);
        let t = normalize(to);
        if t == f {
            return Ok(t);
        }
        if t.starts_with(&format!("{}/", f)) {
            return Err(FsError::Invalid(format!("нельзя переместить {} внутрь самого себя", f)));
        }
        let (node, name) = self.take(&f, who)?;
        match self.put(&t, &name, node.clone(), who) {
            Ok(p) => Ok(p),
            Err(e) => {
                // вернуть на место
                let (parent, _) = split_parent(&f);
                if let Ok(dir) = self.node_mut(&parent) {
                    if let Some(ch) = dir.children_mut() {
                        ch.insert(name, node);
                    }
                }
                Err(e)
            }
        }
    }

    /// Скопировать файл (каталог — только с `recursive`). Возвращает путь копии.
    pub fn copy(&mut self, from: &str, to: &str, who: &str, recursive: bool) -> Result<String, FsError> {
        let f = normalize(from);
        self.check_traverse(&f, who)?;
        let src = self.node(&f)?;
        if src.is_dir() && !recursive {
            return Err(FsError::IsDir(f));
        }
        self.need(&src.meta, who, R, &f)?;
        let mut node = src.clone();
        let t = normalize(to);
        if t.starts_with(&format!("{}/", f)) {
            return Err(FsError::Invalid(format!("нельзя скопировать {} внутрь самого себя", f)));
        }
        node.meta.owner = who.to_string();
        self.put(&t, &basename(&f), node, who)
    }

    /// Сменить права: владелец или root.
    pub fn chmod(&mut self, path: &str, mode: u16, who: &str) -> Result<(), FsError> {
        self.check_traverse(path, who)?;
        let n = self.node_mut(path)?;
        if who != "root" && n.meta.owner != who {
            return Err(FsError::Denied(normalize(path)));
        }
        n.meta.mode = mode & 0o777;
        Ok(())
    }

    /// Сменить владельца: только root.
    pub fn chown(&mut self, path: &str, owner: &str, who: &str) -> Result<(), FsError> {
        if who != "root" {
            return Err(FsError::Denied(normalize(path)));
        }
        self.node_mut(path)?.meta.owner = owner.to_string();
        Ok(())
    }

    /// Найти файлы и каталоги под `root`, имена которых подходят под шаблон
    /// (`*` — все). Возвращает полные пути, включая подходящие каталоги.
    pub fn find(&self, root: &str, pattern: &str) -> Vec<String> {
        let mut out = Vec::new();
        let r = normalize(root);
        if let Ok(n) = self.node(&r) {
            Self::find_rec(n, &r, pattern, &mut out);
        }
        out
    }

    fn find_rec(node: &Node, path: &str, pattern: &str, out: &mut Vec<String>) {
        if let Some(ch) = node.children() {
            for (name, c) in ch {
                let p = join(path, name);
                if glob_match(pattern, name) {
                    out.push(p.clone());
                }
                Self::find_rec(c, &p, pattern, out);
            }
        }
    }

    /// Обойти дерево: f(путь, узел-статистика). Для `tree`, `du`, архивов.
    pub fn walk(&self, root: &str, f: &mut dyn FnMut(&str, &Stat)) {
        let r = normalize(root);
        if let Ok(n) = self.node(&r) {
            self.walk_rec(n, &r, f);
        }
    }

    fn walk_rec(&self, node: &Node, path: &str, f: &mut dyn FnMut(&str, &Stat)) {
        if let Some(ch) = node.children() {
            for (name, c) in ch {
                let p = join(path, name);
                let st = Stat {
                    name: name.clone(),
                    path: p.clone(),
                    is_dir: c.is_dir(),
                    size: c.size(),
                    modified: c.meta.modified,
                    owner: c.meta.owner.clone(),
                    mode: c.meta.mode,
                    is_binary: matches!(c.kind, NodeKind::File { data: FileData::Binary(_) }),
                };
                f(&p, &st);
                self.walk_rec(c, &p, f);
            }
        }
    }

    /// Число файлов во всём дереве.
    pub fn file_count(&self) -> usize {
        let mut n = 0;
        self.walk("/", &mut |_, st| {
            if !st.is_dir {
                n += 1
            }
        });
        n
    }
}
