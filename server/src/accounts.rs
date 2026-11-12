//! Учётные записи игроков: имя и пароль. Пароль хранится только солёным
//! многократным SHA-256, в открытом виде нигде не лежит.

use std::collections::hash_map::RandomState;
use std::hash::{BuildHasher, Hasher};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const ROUNDS: u32 = 60_000;
const RESERVED: &[&str] = &["nobody", "root", "admin", "server", "offline", "system", "hello-world", "null", "broker"];

#[derive(Serialize, Deserialize, Clone)]
pub struct Account {
    pub salt: String,
    pub hash: String,
    pub created: u64,
    pub last_seen: u64,
}

/// Случайное число из системного источника (ключи SipHash берутся у ОС).
pub fn random_u64() -> u64 {
    let mut h = RandomState::new().build_hasher();
    h.write_u128(std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0));
    h.finish()
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

fn digest(salt: &str, password: &str) -> String {
    let mut h = Sha256::digest(format!("{}:{}", salt, password).as_bytes());
    for _ in 0..ROUNDS {
        let mut s = Sha256::new();
        s.update(h);
        s.update(salt.as_bytes());
        h = s.finalize();
    }
    hex(&h)
}

pub fn now() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

impl Account {
    pub fn new(password: &str) -> Account {
        let salt = format!("{:016x}{:016x}", random_u64(), random_u64());
        let hash = digest(&salt, password);
        Account { salt, hash, created: now(), last_seen: now() }
    }

    pub fn check(&self, password: &str) -> bool {
        let got = digest(&self.salt, password);
        // сравнение без раннего выхода
        got.len() == self.hash.len() && got.bytes().zip(self.hash.bytes()).fold(0u8, |acc, (a, b)| acc | (a ^ b)) == 0
    }
}

/// Имя игрока: латиница, цифры, «_» и «-», от 3 до 16 знаков. Возвращает
/// имя в нижнем регистре (оно же имя компьютера в сети).
pub fn clean_name(name: &str) -> Result<String, String> {
    let n = name.trim().to_ascii_lowercase();
    if n.len() < 3 || n.len() > 16 || !n.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') {
        return Err("имя — латиница, цифры, «_» и «-», от 3 до 16 знаков / name: Latin letters, digits, _ and -, 3–16 chars".into());
    }
    if RESERVED.contains(&n.as_str()) {
        return Err("это имя занято / this name is reserved".into());
    }
    Ok(n)
}

pub fn check_password_rules(p: &str) -> Result<(), String> {
    if p.chars().count() < 6 || p.len() > 128 {
        return Err("пароль — от 6 знаков / password: at least 6 characters".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn password_check() {
        let a = Account::new("secret-1");
        assert!(a.check("secret-1"));
        assert!(!a.check("secret-2"));
        assert_ne!(Account::new("x12345").salt, Account::new("x12345").salt);
    }

    #[test]
    fn names() {
        assert_eq!(clean_name(" Neo_1 ").unwrap(), "neo_1");
        assert!(clean_name("ab").is_err());
        assert!(clean_name("root").is_err());
        assert!(clean_name("bad name").is_err());
    }
}
