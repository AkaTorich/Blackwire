//! Язык интерфейса игры: русский (по умолчанию) или английский.
//! Один общий переключатель на весь процесс; тексты пишутся парой прямо на месте:
//! `tr!("Файл", "File")` — строка, `trf!("{} файлов", "{} files", n)` — `format!` по выбранному языку.

use std::sync::atomic::{AtomicU8, Ordering};

static LANG: AtomicU8 = AtomicU8::new(0);

/// Выбран английский.
pub fn is_en() -> bool {
    LANG.load(Ordering::Relaxed) == 1
}

/// Выбрать язык по коду: `"en"` (и `en-US` и т. п.) — английский, всё прочее — русский.
pub fn set(code: &str) {
    let en = code.trim().to_ascii_lowercase().starts_with("en");
    LANG.store(if en { 1 } else { 0 }, Ordering::Relaxed);
}

/// Код выбранного языка: `"ru"` или `"en"`.
pub fn code() -> &'static str {
    if is_en() { "en" } else { "ru" }
}

/// Строка по выбранному языку: `tr!("Русский", "English")`.
#[macro_export]
macro_rules! tr {
    ($ru:expr, $en:expr $(,)?) => {
        if $crate::i18n::is_en() { $en } else { $ru }
    };
}

/// `format!` по выбранному языку: `trf!("{} файлов", "{} files", n)`.
#[macro_export]
macro_rules! trf {
    ($ru:literal, $en:literal $(,)?) => {
        if $crate::i18n::is_en() { format!($en) } else { format!($ru) }
    };
    ($ru:literal, $en:literal, $($arg:tt)*) => {
        if $crate::i18n::is_en() { format!($en, $($arg)*) } else { format!($ru, $($arg)*) }
    };
}

#[cfg(test)]
mod tests {
    #[test]
    fn switch() {
        super::set("en-US");
        assert_eq!(tr!("да", "yes"), "yes");
        let n = 3;
        assert_eq!(trf!("{n} файла", "{n} files"), "3 files");
        super::set("ru");
        assert_eq!(trf!("{} файла", "{} files", n), "3 файла");
        assert_eq!(super::code(), "ru");
    }
}
