//! Каталог магазина: что можно купить за ₵ и что это даёт. Часть покупок
//! действует сразу (словарь, карта, железо), часть — постоянным эффектом,
//! который читают другие части (vpn — тише подозрение, scrubber — быстрее спад,
//! rootkit — повышение прав всегда удаётся).

use crate::net::machine::MachineKind;
use crate::world::World;

/// Товар.
#[derive(Clone, Copy, Debug)]
pub struct ShopItem {
    pub id: &'static str,
    pub name: &'static str,
    pub price: i64,
    pub desc: &'static str,
}

pub const ITEMS: &[ShopItem] = &[
    ShopItem { id: "wordlist", name: "Большой словарь", price: 300, desc: "пополняет ~/wordlist.txt — подбор берёт больше паролей" },
    ShopItem { id: "map", name: "Карта провайдера", price: 400, desc: "открывает на карте ещё несколько узлов города" },
    ShopItem { id: "rig", name: "Железо", price: 500, desc: "апгрейд процессора и памяти твоей машины" },
    ShopItem { id: "scrubber", name: "Чистильщик", price: 600, desc: "подозрение спадает заметно быстрее" },
    ShopItem { id: "vpn", name: "Туннель", price: 800, desc: "подозрение за атаки растёт вдвое медленнее" },
    ShopItem { id: "rootkit", name: "Руткит", price: 1000, desc: "privesc (повышение прав) удаётся всегда" },
];

pub fn item(id: &str) -> Option<&'static ShopItem> {
    ITEMS.iter().find(|i| i.id == id)
}

/// Дополнительные частые пароли, которые добавляет «Большой словарь».
const EXTRA_WORDS: &[&str] = &[
    "qazwsx", "121212", "trustno1", "sunshine", "iloveyou", "princess", "admin123", "welcome1", "passw0rd", "letmein1",
    "zaq12wsx", "qwertyuiop", "1qaz2wsx", "michael1", "superman1", "batman01", "liverpool", "arsenal1", "spartak", "zenit",
    "matrix", "neo", "root1234", "toortoor", "service", "default", "system32", "backup1", "secret1", "hunter2",
    "monkey12", "dragon01", "master01", "shadow01", "ninja007", "falcon1", "phoenix1", "orion12", "vector1", "cipher",
    "firewall", "sandbox", "payload", "exploit1", "override", "kernel01", "daemon12", "sysop123", "operator", "console1",
];

/// Сразу действующий эффект покупки. Для постоянных эффектов — пусто (достаточно
/// факта владения в `player.upgrades`). Возвращает короткую строку результата.
pub fn apply(world: &mut World, id: &str) -> String {
    match id {
        "wordlist" => {
            let home = world.machines[0].users.home_of(&world.player.handle);
            let path = format!("{}/wordlist.txt", home);
            let prev = world.machines[0].fs.read_text(&path).unwrap_or_default();
            let mut text = prev;
            if !text.ends_with('\n') && !text.is_empty() {
                text.push('\n');
            }
            text.push_str(&EXTRA_WORDS.join("\n"));
            text.push('\n');
            let _ = world.machines[0].fs.write_text(&path, &text, &world.player.handle);
            format!("Словарь пополнен: +{} паролей.", EXTRA_WORDS.len())
        }
        "map" => {
            let public: Vec<_> = world
                .machines
                .iter()
                .filter(|m| m.id != 0 && m.up && m.is_public() && !matches!(m.kind, MachineKind::Honeypot))
                .map(|m| m.id)
                .collect();
            let mut added = 0;
            for id in public.into_iter().take(10) {
                if !world.player.known_machines.contains(&id) {
                    world.player.known_machines.push(id);
                    added += 1;
                }
            }
            format!("Открыто узлов: {}. Посмотри карту сети.", added)
        }
        "rig" => {
            let m = &mut world.machines[0];
            m.cpu = "Ryzen 9 (разогнан)".to_string();
            m.ram_mb = m.ram_mb.max(8192) * 2;
            "Железо обновлено: смотри монитор.".to_string()
        }
        "scrubber" => "Чистильщик активен: подозрение спадает быстрее.".to_string(),
        "vpn" => "Туннель поднят: следы тише.".to_string(),
        "rootkit" => "Руткит в деле: privesc теперь всегда проходит.".to_string(),
        _ => String::new(),
    }
}
