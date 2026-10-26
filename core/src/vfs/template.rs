//! Типичное дерево Unix-машины: системные каталоги, «двоичные» утилиты в /bin,
//! настройки в /etc, журналы в /var/log, дом пользователя. Генератор сети
//! потом дописывает своё поверх (`write_text`).

use super::Vfs;

const BIN_NAMES: &[&str] = &[
    "sh", "bash", "ls", "cat", "cp", "mv", "rm", "mkdir", "echo", "grep", "ps", "kill", "ssh", "scp", "ping", "wget",
    "curl", "tar", "chmod", "chown", "find", "sort", "head", "tail", "wc", "date", "uname", "login", "su", "sudo",
];

const SBIN_NAMES: &[&str] = &["init", "sshd", "cron", "syslogd", "ifconfig", "route", "shutdown", "reboot"];

const USR_BIN_NAMES: &[&str] = &["adder", "nmap", "whois", "traceroute", "nslookup", "top", "free", "tree", "diff"];

/// Условное «двоичное» содержимое утилиты: заголовок и имя.
fn fake_binary(name: &str) -> Vec<u8> {
    let mut v = vec![0x7f, b'E', b'L', b'F', 2, 1, 1, 0];
    v.extend_from_slice(name.as_bytes());
    v.extend_from_slice(&[0, 0, 0, 0]);
    v
}

pub fn new_unix(owner: &str) -> Vfs {
    let mut fs = Vfs::empty("root");
    for d in ["/bin", "/sbin", "/etc", "/etc/ssh", "/home", "/usr", "/usr/bin", "/usr/share", "/usr/share/doc", "/opt",
        "/var", "/var/log", "/var/tmp", "/dev", "/tmp"]
    {
        fs.mkdir_all(d, "root").expect("каталог шаблона");
    }
    fs.set_mode("/tmp", 0o777).ok();
    fs.set_mode("/var/tmp", 0o777).ok();
    fs.mkdir_all("/root", "root").ok();
    fs.set_mode("/root", 0o700).ok();

    for n in BIN_NAMES {
        fs.write_binary(&format!("/bin/{}", n), fake_binary(n), "root", 0o755).ok();
    }
    for n in SBIN_NAMES {
        fs.write_binary(&format!("/sbin/{}", n), fake_binary(n), "root", 0o755).ok();
    }
    for n in USR_BIN_NAMES {
        fs.write_binary(&format!("/usr/bin/{}", n), fake_binary(n), "root", 0o755).ok();
    }
    fs.write_binary("/dev/null", Vec::new(), "root", 0o666).ok();

    let passwd = format!(
        "root:x:0:0:root:/root:/bin/sh\n{0}:x:1000:1000:{0}:/home/{0}:/bin/sh\n",
        owner
    );
    fs.write_text("/etc/passwd", &passwd, "root").ok();
    fs.write_text("/etc/shadow", "root:*:22350:0:99999:7:::\n", "root").ok();
    fs.set_mode("/etc/shadow", 0o600).ok();
    fs.write_text("/etc/hosts", "127.0.0.1\tlocalhost\n::1\t\tlocalhost\n", "root").ok();
    fs.write_text("/etc/hostname", "localhost\n", "root").ok();
    fs.write_text("/etc/motd", "Добро пожаловать. Несанкционированный доступ преследуется.\n", "root").ok();
    fs.write_text(
        "/etc/os-release",
        "NAME=\"Blackwire OS\"\nVERSION=\"1.0\"\nID=blackwire\nPRETTY_NAME=\"Blackwire OS 1.0\"\n",
        "root",
    )
    .ok();
    fs.write_text("/etc/resolv.conf", "nameserver 10.0.0.1\n", "root").ok();
    fs.write_text("/etc/ssh/sshd_config", "Port 22\nPermitRootLogin yes\nPasswordAuthentication yes\n", "root").ok();
    fs.write_text("/etc/crontab", "# m h dom mon dow user command\n0 3 * * * root /sbin/syslogd --rotate\n", "root")
        .ok();
    fs.write_text("/var/log/auth.log", "", "root").ok();
    fs.write_text("/var/log/system.log", "boot: система запущена\n", "root").ok();
    fs.set_mode("/var/log/auth.log", 0o640).ok();
    fs.write_text(
        "/usr/share/doc/readme.txt",
        "Blackwire OS. Справка по командам: help, man <команда>, <команда> --help.\n",
        "root",
    )
    .ok();
    fs.write_text("/root/.bash_history", "", "root").ok();
    fs.set_mode("/root/.bash_history", 0o600).ok();

    add_user_home(&mut fs, owner);
    fs
}

pub fn add_user_home(fs: &mut Vfs, user: &str) {
    if user == "root" {
        fs.mkdir_all("/root", "root").ok();
        fs.set_mode("/root", 0o700).ok();
        return;
    }
    let home = format!("/home/{}", user);
    fs.mkdir_all(&home, user).ok();
    fs.set_owner(&home, user).ok();
    fs.set_mode(&home, 0o750).ok();
    let hist = format!("{}/.bash_history", home);
    if !fs.exists(&hist) {
        fs.write_text(&hist, "", user).ok();
        fs.set_mode(&hist, 0o600).ok();
    }
}
