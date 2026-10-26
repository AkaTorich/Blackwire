//! Файловая система: пути, права, операции.

use blackwire_core::vfs::{self, FsError, Vfs};

#[test]
fn normalize_and_resolve() {
    assert_eq!(vfs::normalize("/a/b/../c"), "/a/c");
    assert_eq!(vfs::normalize("/a/./b/"), "/a/b");
    assert_eq!(vfs::normalize("/"), "/");
    assert_eq!(vfs::normalize("/a/../.."), "/");
    assert_eq!(vfs::resolve("/home/ghost", "docs/a.txt"), "/home/ghost/docs/a.txt");
    assert_eq!(vfs::resolve("/home/ghost", "/etc/passwd"), "/etc/passwd");
    assert_eq!(vfs::resolve("/home/ghost", "../root"), "/home/root");
}

#[test]
fn split_and_basename() {
    assert_eq!(vfs::split_parent("/a/b/c.txt"), ("/a/b".to_string(), "c.txt".to_string()));
    assert_eq!(vfs::basename("/a/b/c.txt"), "c.txt");
    assert_eq!(vfs::dirname("/a/b/c.txt"), "/a/b");
    assert_eq!(vfs::basename("/"), "/");
}

#[test]
fn mode_string_and_parse() {
    assert_eq!(vfs::mode_string(0o755, true), "drwxr-xr-x");
    assert_eq!(vfs::mode_string(0o644, false), "-rw-r--r--");
    assert_eq!(vfs::parse_mode("755", 0), Some(0o755));
    assert_eq!(vfs::parse_mode("u+x", 0o644), Some(0o744));
    assert_eq!(vfs::parse_mode("a-w", 0o666), Some(0o444));
    assert_eq!(vfs::parse_mode("rwxr-xr-x", 0), Some(0o755));
    assert_eq!(vfs::parse_mode("go-rwx", 0o777), Some(0o700));
}

#[test]
fn glob_matching() {
    assert!(vfs::glob_match("*.txt", "notes.txt"));
    assert!(vfs::glob_match("a?c", "abc"));
    assert!(!vfs::glob_match("a?c", "ac"));
    assert!(vfs::glob_match("*", "anything"));
    assert!(!vfs::glob_match("*.adr", "file.txt"));
}

#[test]
fn new_unix_has_typical_tree() {
    let fs = Vfs::new_unix("ghost");
    assert!(fs.is_dir("/bin"));
    assert!(fs.is_dir("/etc"));
    assert!(fs.is_dir("/home/ghost"));
    assert!(fs.is_dir("/var/log"));
    assert!(fs.exists("/etc/passwd"));
    assert!(fs.exists("/etc/hosts"));
    assert!(fs.exists("/etc/motd"));
    assert!(fs.read_text("/etc/passwd").unwrap().contains("ghost"));
}

#[test]
fn write_read_with_owner() {
    let mut fs = Vfs::new_unix("ghost");
    fs.write("/home/ghost/a.txt", "привет", "ghost").unwrap();
    assert_eq!(fs.read("/home/ghost/a.txt", "ghost").unwrap(), "привет");
}

#[test]
fn permission_denied_read() {
    let mut fs = Vfs::new_unix("ghost");
    fs.write("/home/ghost/secret.txt", "тайна", "ghost").unwrap();
    fs.set_mode("/home/ghost/secret.txt", 0o600).unwrap();
    // другой пользователь не прочитает
    let e = fs.read("/home/ghost/secret.txt", "bob").unwrap_err();
    assert!(matches!(e, FsError::Denied(_)));
    // root прочитает
    assert_eq!(fs.read("/home/ghost/secret.txt", "root").unwrap(), "тайна");
}

#[test]
fn permission_denied_write_to_others_file() {
    let mut fs = Vfs::new_unix("ghost");
    fs.write("/home/ghost/a.txt", "x", "ghost").unwrap();
    fs.set_mode("/home/ghost/a.txt", 0o644).unwrap();
    let e = fs.write("/home/ghost/a.txt", "y", "bob").unwrap_err();
    assert!(matches!(e, FsError::Denied(_)));
}

#[test]
fn root_dir_write_denied_for_user() {
    let mut fs = Vfs::new_unix("ghost");
    // /etc принадлежит root, права 755 — обычный пользователь писать не может
    let e = fs.write("/etc/newfile", "x", "ghost").unwrap_err();
    assert!(matches!(e, FsError::Denied(_)));
    fs.write("/etc/newfile", "x", "root").unwrap();
    assert!(fs.exists("/etc/newfile"));
}

#[test]
fn mkdir_and_mkdir_p() {
    let mut fs = Vfs::new_unix("ghost");
    assert!(fs.mkdir("/home/ghost/a/b/c", "ghost", false).is_err());
    fs.mkdir("/home/ghost/a/b/c", "ghost", true).unwrap();
    assert!(fs.is_dir("/home/ghost/a/b/c"));
    // повторный mkdir -p не ошибка
    fs.mkdir("/home/ghost/a/b/c", "ghost", true).unwrap();
}

#[test]
fn remove_file_and_recursive() {
    let mut fs = Vfs::new_unix("ghost");
    fs.mkdir("/home/ghost/d", "ghost", false).unwrap();
    fs.write("/home/ghost/d/x.txt", "1", "ghost").unwrap();
    // непустую папку без -r удалить нельзя
    assert!(matches!(fs.remove("/home/ghost/d", "ghost", false), Err(FsError::NotEmpty(_))));
    fs.remove("/home/ghost/d", "ghost", true).unwrap();
    assert!(!fs.exists("/home/ghost/d"));
}

#[test]
fn rename_and_copy() {
    let mut fs = Vfs::new_unix("ghost");
    fs.write("/home/ghost/a.txt", "данные", "ghost").unwrap();
    fs.rename("/home/ghost/a.txt", "/home/ghost/b.txt", "ghost").unwrap();
    assert!(!fs.exists("/home/ghost/a.txt"));
    assert_eq!(fs.read("/home/ghost/b.txt", "ghost").unwrap(), "данные");
    fs.copy("/home/ghost/b.txt", "/home/ghost/c.txt", "ghost", false).unwrap();
    assert_eq!(fs.read("/home/ghost/c.txt", "ghost").unwrap(), "данные");
}

#[test]
fn rename_into_directory() {
    let mut fs = Vfs::new_unix("ghost");
    fs.write("/home/ghost/a.txt", "x", "ghost").unwrap();
    fs.mkdir("/home/ghost/box", "ghost", false).unwrap();
    let new = fs.rename("/home/ghost/a.txt", "/home/ghost/box", "ghost").unwrap();
    assert_eq!(new, "/home/ghost/box/a.txt");
}

#[test]
fn chmod_chown_rules() {
    let mut fs = Vfs::new_unix("ghost");
    fs.write("/home/ghost/a.txt", "x", "ghost").unwrap();
    fs.chmod("/home/ghost/a.txt", 0o600, "ghost").unwrap();
    assert_eq!(fs.stat("/home/ghost/a.txt").unwrap().mode, 0o600);
    // chown только root
    assert!(fs.chown("/home/ghost/a.txt", "bob", "ghost").is_err());
    fs.chown("/home/ghost/a.txt", "root", "root").unwrap();
    assert_eq!(fs.stat("/home/ghost/a.txt").unwrap().owner, "root");
}

#[test]
fn find_and_list() {
    let mut fs = Vfs::new_unix("ghost");
    fs.write("/home/ghost/a.adr", "1", "ghost").unwrap();
    fs.write("/home/ghost/b.adr", "2", "ghost").unwrap();
    fs.write("/home/ghost/c.txt", "3", "ghost").unwrap();
    let found = fs.find("/home/ghost", "*.adr");
    assert_eq!(found.len(), 2);
    let entries = fs.list("/home/ghost", "ghost").unwrap();
    assert!(entries.iter().any(|e| e.name == "a.adr"));
}

#[test]
fn fserror_display_russian() {
    let e = FsError::NotFound("/x".to_string());
    assert!(e.to_string().contains("нет такого файла"));
    let e = FsError::Denied("/y".to_string());
    assert!(e.to_string().contains("отказано"));
}
