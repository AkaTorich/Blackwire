//! Простой обратимый шифр для файлов. Ключ — слово или число; из него берётся
//! сдвиг байта. Зашифрованный текст хранится как шестнадцатеричная строка, так
//! что русский текст не ломается. Это игровой шифр, не защита.

/// Сдвиг из ключа (1..=255).
fn key_byte(key: &str) -> u8 {
    let s = key.bytes().fold(0u16, |a, b| a.wrapping_add(b as u16));
    let k = (s % 255) as u8;
    if k == 0 {
        1
    } else {
        k
    }
}

/// Зашифровать текст → шестнадцатеричная строка.
pub fn encrypt(text: &str, key: &str) -> String {
    let k = key_byte(key);
    let mut out = String::with_capacity(text.len() * 2);
    for b in text.bytes() {
        out.push_str(&format!("{:02x}", b.wrapping_add(k)));
    }
    out
}

/// Расшифровать шестнадцатеричную строку → текст (или ошибка).
pub fn decrypt(hex: &str, key: &str) -> Result<String, String> {
    let hex: String = hex.split_whitespace().collect();
    if hex.len() % 2 != 0 || hex.is_empty() {
        return Err("это не похоже на шифртекст".to_string());
    }
    let k = key_byte(key);
    let mut bytes = Vec::with_capacity(hex.len() / 2);
    let raw = hex.as_bytes();
    let mut i = 0;
    while i < raw.len() {
        let hi = hexval(raw[i])?;
        let lo = hexval(raw[i + 1])?;
        bytes.push(((hi << 4) | lo).wrapping_sub(k));
        i += 2;
    }
    String::from_utf8(bytes).map_err(|_| "неверный ключ: получается мусор".to_string())
}

fn hexval(c: u8) -> Result<u8, String> {
    match c {
        b'0'..=b'9' => Ok(c - b'0'),
        b'a'..=b'f' => Ok(c - b'a' + 10),
        b'A'..=b'F' => Ok(c - b'A' + 10),
        _ => Err("это не похоже на шифртекст".to_string()),
    }
}
