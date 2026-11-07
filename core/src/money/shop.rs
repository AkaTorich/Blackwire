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
    pub name_en: &'static str,
    pub desc_en: &'static str,
}

impl ShopItem {
    /// Название на текущем языке.
    pub fn name(&self) -> &'static str {
        tr!(self.name, self.name_en)
    }
    /// Описание на текущем языке.
    pub fn desc(&self) -> &'static str {
        tr!(self.desc, self.desc_en)
    }
}

pub const ITEMS: &[ShopItem] = &[
    ShopItem { id: "wordlist", name: "Большой словарь", price: 300, desc: "пополняет ~/wordlist.txt — подбор берёт больше паролей",
        name_en: "Big wordlist", desc_en: "extends ~/wordlist.txt — cracking tries more passwords" },
    ShopItem { id: "map", name: "Карта провайдера", price: 400, desc: "открывает на карте ещё несколько узлов города",
        name_en: "ISP map", desc_en: "reveals several more city hosts on the map" },
    ShopItem { id: "rig", name: "Железо", price: 500, desc: "апгрейд процессора и памяти твоей машины",
        name_en: "Hardware", desc_en: "CPU and memory upgrade for your machine" },
    ShopItem { id: "scrubber", name: "Чистильщик", price: 600, desc: "подозрение спадает заметно быстрее",
        name_en: "Scrubber", desc_en: "heat drops noticeably faster" },
    ShopItem { id: "vpn", name: "Туннель", price: 800, desc: "подозрение за атаки растёт вдвое медленнее",
        name_en: "Tunnel", desc_en: "heat from attacks grows half as fast" },
    ShopItem { id: "rootkit", name: "Руткит", price: 1000, desc: "локальный эксплоит rootkit.adr; работает и на свежем ядре",
        name_en: "Rootkit", desc_en: "local exploit rootkit.adr; works even on a fresh kernel" },
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
            trf!("Словарь пополнен: +{} паролей.", "Wordlist extended: +{} passwords.", EXTRA_WORDS.len())
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
            trf!("Открыто узлов: {}. Посмотри карту сети.", "Hosts revealed: {}. Check the network map.", added)
        }
        "rig" => {
            let m = &mut world.machines[0];
            m.cpu = tr!("Ryzen 9 (разогнан)", "Ryzen 9 (overclocked)").to_string();
            m.ram_mb = m.ram_mb.max(8192) * 2;
            tr!("Железо обновлено: смотри монитор.", "Hardware upgraded: check the system monitor.").to_string()
        }
        "scrubber" => tr!("Чистильщик активен: подозрение спадает быстрее.", "Scrubber active: heat drops faster.").to_string(),
        "vpn" => tr!("Туннель поднят: следы тише.", "Tunnel up: your traces are quieter.").to_string(),
        "rootkit" => {
            let user = world.player.handle.clone();
            let home = world.machines[0].users.home_of(&user);
            let path = format!("{}/scripts/rootkit.adr", home);
            if !world.machines[0].fs.exists(&path) {
                let _ = world.machines[0].fs.mkdir_all(&format!("{}/scripts", home), &user);
                let _ = world.machines[0].fs.write_text(&path, tr!(include_str!("../hack/exploits/race.adr"), include_str!("../hack/exploits/en/race.adr")), &user);
            }
            tr!("Rootkit установлен: privesc ~/scripts/rootkit.adr id. Найди id: probe текущий_хост; подсказки — research.",
                "Rootkit installed: privesc ~/scripts/rootkit.adr id. Find the id: probe current_host; hints — research.").to_string()
        },
        _ => String::new(),
    }
}
