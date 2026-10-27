//! Player-written exploit payloads for the simulated Blackwire network.
//! Profiles are stable across save/reload and tied to the exact device/version.
use serde_json::{json, Value};
use crate::{World, session::Session};
use super::{VulnInfo, VulnAccess};

pub const KINDS: &[&str] = &["integer_overflow", "signedness", "length_confusion", "use_after_free", "double_free", "type_confusion", "object_reference", "mass_assignment", "token_confusion", "request_smuggling", "cache_poisoning", "header_injection", "unsafe_deserialization", "prototype_pollution", "template_injection", "xml_entity", "regex_backtracking", "fragment_overlap", "weak_rng", "permission_mask", "overflow", "traversal", "auth_replay", "query", "format", "protocol", "race", "command_chain"];

fn hash(text: &str) -> u64 {
    text.bytes().fold(0xcbf29ce484222325, |n, b| (n ^ b as u64).wrapping_mul(0x100000001b3))
}
pub fn id(v: &VulnInfo) -> String {
    format!("bw-{:016x}", hash(&format!("{}:{}:{}:{}:{}", v.service, v.port, v.version, v.name, v.local)))
}
fn custom_kind(s:&str)->bool {
    fn ident(v:&str)->bool{!v.is_empty()&&v.len()<=64&&v.bytes().enumerate().all(|(i,c)|c.is_ascii_alphabetic()||c==b'_'||(i>0&&c.is_ascii_digit()))}
    s.split_once(':').is_some_and(|(a,b)|ident(a)&&ident(b))
}
pub fn validate_spec(spec: &Value) -> Result<(), String> {
    let obj = spec.as_object().ok_or("exploit должен быть словарём")?;
    if obj.keys().any(|k| !["kind", "difficulty", "hint", "market", "params"].contains(&k.as_str())) {
        return Err("неизвестное поле exploit: допустимы kind, difficulty, hint, market, params".into());
    }
    if !KINDS.contains(&spec["kind"].as_str().unwrap_or("")) && !custom_kind(spec["kind"].as_str().unwrap_or("")) {
        return Err(format!("неизвестная механика exploit; доступны {}", KINDS.join(", ")));
    }
    if let Some(params)=spec.get("params"){if !params.is_object()||params.to_string().len()>16384{return Err("params должен быть словарём до 16 КиБ".into());}}
    if let Some(d) = spec.get("difficulty") {
        if !d.as_u64().is_some_and(|d| (1..=5).contains(&d)) { return Err("difficulty должна быть от 1 до 5".into()); }
    }
    if let Some(market) = spec.get("market") {
        if !market.as_str().is_some_and(|s| !s.is_empty() && s.len() <= 129 && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == ':')) {
            return Err("market должен быть id товара встроенного маркета или плагина".into());
        }
    }
    if let Some(h) = spec.get("hint") {
        if !h.as_str().is_some_and(|h| h.len() <= 2048) { return Err("hint должна быть строкой до 2048 байт".into()); }
    }
    Ok(())
}
pub fn kind(world: &World, v: &VulnInfo) -> String {
    if let Some(k) = world.plugins.vulnerabilities.get(&v.name).and_then(|s| s.get("exploit")).and_then(|s| s["kind"].as_str()) {
        if KINDS.contains(&k) || custom_kind(k) { return k.into(); }
    }
    for rule in crate::net::services::known_vulns(){
        if rule.name==v.name && rule.service==v.service && v.version.starts_with(&rule.version_prefix){
            if let Some(kind)=rule.effect.strip_prefix("payload:"){if KINDS.contains(&kind){return kind.into();}}
        }
    }
    if v.local { return "race".into(); }
    match v.service.as_str() {
        "ssh" | "vnc" => "auth_replay",
        "ftp" | "badge" => "command_chain",
        "telnet" | "mqtt" | "dns" => "protocol",
        "smtp" | "imap" | "monitor" => "format",
        "mysql" | "postgresql" | "ledger" => "query",
        "http" | "https" => "traversal",
        "smb" | "rdp" | "hl7" | "atm-link" => "overflow",
        _ => match v.access { VulnAccess::Read(_) => "traversal", VulnAccess::Creds => "auth_replay", VulnAccess::Info => "format", _ => "overflow" }
    }.into()
}
pub fn available(world: &World, session: &Session, target: u32) -> Vec<VulnInfo> {
    let mut list = super::remote_vulns(world.machine(target));
    if session.machine_id() == target {
        list.extend(super::local_vulns(world.machine(target)));
        if world.player.upgrades.contains("rootkit") && !list.iter().any(|v| v.local) {
            list.push(VulnInfo { service: "kernel".into(), port: 0, version: world.machine(target).os.clone(),
                name: "Купленный rootkit".into(), access: VulnAccess::Root, local: true });
        }
    }
    list
}
pub fn select(world: &World, session: &Session, target: u32, selector: Option<&str>) -> Result<VulnInfo, String> {
    let list = available(world, session, target);
    let matches: Vec<_> = list.iter().filter(|v| selector.map(|s| s == id(v) || s == v.name || s == v.service).unwrap_or(true)).collect();
    match matches.as_slice() {
        [v] => Ok((*v).clone()),
        [] => Err("Уязвимость не найдена. Выполни probe адрес; локальные уязвимости исследуются после входа на машину.".into()),
        _ => Err(format!("Выбери конкретную уязвимость: {}", matches.iter().map(|v| format!("{} ({})", id(v), v.name)).collect::<Vec<_>>().join(", ")))
    }
}
pub fn ensure_reachable(world: &World, session: &Session, target: u32, v: &VulnInfo) -> Result<(), String> {
    if !world.machine(target).up { return Err("узел не отвечает".into()); }
    if v.local {
        if session.machine_id() != target { return Err("локальный эксплоит запускается на текущей машине".into()); }
    } else {
        if super::is_locked(world) { return Err(format!("канал заблокирован после рейда: {} с", super::lock_remaining(world))); }
        if super::defense::firewall_up(world, target) { return Err("служба закрыта файрволом; сначала исследуй защиту".into()); }
        world.lookup(session.machine_id(), &world.address_for(session.machine_id(), target)).map_err(|e| e.to_string())?;
    }
    Ok(())
}
pub fn profile(world: &World, target: u32, v: &VulnInfo) -> Result<Value,String> {
    let m = world.machine(target);
    let fingerprint = format!("{:016x}", hash(&format!("{}:{}:{}:{}:{}", world.seed, target, m.hostname, m.ip, format!("{}:{}",id(v),world.plugins.vulnerabilities.get(&v.name).unwrap_or(&Value::Null)))));
    let n = hash(&fingerprint);
    let spec = world.plugins.vulnerabilities.get(&v.name).and_then(|s| s.get("exploit"));
    let difficulty = spec.and_then(|s| s["difficulty"].as_u64()).unwrap_or(2).clamp(1, 5);
    let kind = kind(world, v);
    let user = m.users.list().into_iter().find(|u| !u.is_admin).map(|u| u.name.clone()).unwrap_or_else(|| "root".into());
    let path = match &v.access { VulnAccess::Read(p) => p.as_str(), _ => "/etc/passwd" };
    let nonce = format!("BW-{:08x}", n as u32);
    let channel = match v.access { VulnAccess::Root => "root", VulnAccess::Shell => "shell", VulnAccess::Creds => "accounts", _ => "records" };
    let (layout, hints, fields) = if let Some(puzzle)=super::payload_extra::profile(&kind,n,difficulty,&user,path,&nonce){puzzle}else{match kind.as_str() {
        "overflow" => (json!({"frame_words": 8 + n % (8 * difficulty), "word_size": 4, "code_base": 4096 + (n % 1024) * 16,
            "entry_offset": 16 + n % 64, "canary": format!("{:04x}", (n >> 24) as u16)}),
            vec!["Нужно переполнить буфер до границы кадра, сохранив контрольную метку.",
                "Длина заполнения равна frame_words × word_size. Адрес входа — code_base + entry_offset.",
                "Формат buffer: заполнение буквами A, затем |адрес_в_десятичной_форме|canary."],
            vec!["buffer"]),
        "traversal" => (json!({"mounts": (0..(2 + n % difficulty.max(2))).map(|i| format!("mount{}", i)).collect::<Vec<_>>(), "target_file": path}),
            vec!["Служба неверно обрабатывает выход из вложенных каталогов.",
                "На каждый элемент mounts нужен один переход ../.",
                "После переходов добавь target_file без начальной косой черты."],
            vec!["path"]),
        "auth_replay" => (json!({"account": user, "challenge": nonce, "salt": format!("{:x}", n >> 32), "modulus": 65521}),
            vec!["Проверка авторизации использует слабую игровую контрольную сумму.",
                "Подписываемая строка — challenge + двоеточие + salt.",
                "response — сумма (индекс символа + 1) × ord(символ), по модулю modulus. user возьми из account."],
            vec!["user", "response"]),
        "query" => (json!({"table": format!("vault_{}", n % 97), "columns": ["login", format!("secret_{}", n % 13).as_str()]}),
            vec!["Игровой обработчик запросов допускает обход фильтра.",
                "Собери FIND <table> FIELDS <columns через запятую>.",
                "Добавь к запросу пробел и WHEN key='' OR BW_TRUE. Это синтаксис вымышленной службы Blackwire."],
            vec!["query"]),
        "format" => (json!({"stack_depth": 3 + n % 7, "argument_base": 2 + n % 3, "region_base": 8192 + n % 512, "record_offset": 8 + n % 128}),
            vec!["Служба интерпретирует пользовательский формат как указатель на запись.",
                "Номер аргумента — stack_depth + argument_base. Адрес — region_base + record_offset.",
                "format имеет вид %<номер>$bw; address — вычисленный целочисленный адрес."],
            vec!["format", "address"]),
        "protocol" => (json!({"nonce": nonce, "capabilities": [{"mode": "public", "channel": "status"}, {"mode": "maintenance", "channel": channel}]}),
            vec!["Нужно составить допустимую последовательность кадров игрового протокола.",
                "Начни с HELLO и nonce, выбери capability режима maintenance.",
                "frames — список: HELLO <nonce>, MODE <mode>, OPEN <channel>. Каждый кадр — строка."],
            vec!["frames"]),
        "race" => (json!({"resource": format!("/tmp/bw-check-{}", n % 1000), "window_ms": 12 / difficulty + 1}),
            vec!["Между проверкой ресурса и его использованием есть игровое окно гонки.",
                "steps — три словаря с op, resource и at: check, replace, use.",
                "Метки at целочисленные: 0 ≤ check < replace < use ≤ window_ms. resource должен совпасть на всех шагах; реальное ожидание не нужно."],
            vec!["steps"]),
        "command_chain" => (json!({"command": "status", "parser": (["semicolon", "and_then", "pipe"][(n % 3) as usize]), "channel": channel, "nonce": nonce}),
            vec!["Аргумент команды проходит через небезопасный игровой разборщик.",
                "Для parser semicolon разделитель ;, для and_then — &&, для pipe — |.",
                "argument: command + разделитель + bw-open:<channel>:<nonce>. Команды относятся только к модели Blackwire."],
            vec!["argument"]),
        _ => (json!({}),vec!["Авторский тип: исследуй подсказки его плагина."],vec!["payload"])
    }};
    let mut hints: Vec<String> = hints.into_iter().map(String::from).collect();
    if let Some(hint) = spec.and_then(|s| s["hint"].as_str()) { hints.insert(0, hint.into()); }
    let market = spec.and_then(|s| s["market"].as_str()).filter(|id| {
        crate::market::item(id).is_some() || world.plugins.catalogs.get("market").is_some_and(|items| items.contains_key(*id))
    }).map(String::from).or_else(|| (!custom_kind(&kind)).then(||format!("exp_{}",kind)));
    let result=json!({"id": id(v), "name": v.name, "service": v.service, "port": v.port, "version": v.version,
        "local": v.local, "kind": kind, "difficulty": difficulty, "fingerprint": fingerprint,
        "layout": layout, "hints": hints, "fields": fields, "market": market,
        "result": super::access::result(world,v), "conditions":super::access::conditions(world,target,v,None),
        "next": "Составь payload с fingerprint и нужными полями; net.probe проверяет, net.exploit применяет. Для локального взлома — net.privesc."});
    if custom_kind(&kind){crate::plugins::exploit_types::profile(world,target,v,result)}else{Ok(result)}
}
fn number(v: &Value, name: &str) -> Result<i64, String> {
    v[name].as_i64().ok_or_else(|| format!("поле {} должно быть целым числом", name))
}
fn envelope(profile:&Value,payload:&Value)->Result<(),String>{
    if !payload.is_object() { return Err("payload должен быть словарём Adder".into()); }
    if payload.to_string().len() > 65536 { return Err("payload превышает 64 КиБ".into()); }
    if payload["fingerprint"].as_str() != profile["fingerprint"].as_str() {
        return Err("fingerprint не соответствует устройству или версии; повтори net.research".into());
    }
    Ok(())
}
pub fn validate(profile:&Value,payload:&Value)->Result<(),String>{
    envelope(profile,payload)?;
    if super::payload_extra::KINDS.contains(&profile["kind"].as_str().unwrap_or("")){return super::payload_extra::validate(profile,payload);}
    let l = &profile["layout"];
    let error = match profile["kind"].as_str().unwrap_or("") {
        "overflow" => {
            let length = l["frame_words"].as_u64().unwrap() * 4;
            let address = l["code_base"].as_u64().unwrap() + l["entry_offset"].as_u64().unwrap();
            let wanted = format!("{}|{}|{}", "A".repeat(length as usize), address, l["canary"].as_str().unwrap());
            (payload["buffer"].as_str() != Some(&wanted)).then_some("Неверны длина buffer, адрес входа или canary.")
        }
        "traversal" => {
            let wanted = "../".repeat(l["mounts"].as_array().unwrap().len()) + l["target_file"].as_str().unwrap().trim_start_matches('/');
            (payload["path"].as_str() != Some(&wanted)).then_some("Путь не вышел через все mounts к target_file.")
        }
        "auth_replay" => {
            let text = format!("{}:{}", l["challenge"].as_str().unwrap(), l["salt"].as_str().unwrap());
            let sum: u64 = text.chars().enumerate().map(|(i, c)| (i as u64 + 1) * c as u64).sum();
            (payload["user"] != l["account"] || payload["response"].as_u64() != Some(sum % 65521)).then_some("Неверный user или response; проверь строку, веса символов и modulus.")
        }
        "query" => {
            let columns = l["columns"].as_array().unwrap().iter().map(|v| v.as_str().unwrap()).collect::<Vec<_>>().join(",");
            let wanted = format!("FIND {} FIELDS {} WHEN key='' OR BW_TRUE", l["table"].as_str().unwrap(), columns);
            (payload["query"].as_str() != Some(&wanted)).then_some("Неверные таблица, порядок столбцов или условие игрового запроса.")
        }
        "format" => {
            let wanted = format!("%{}$bw", l["stack_depth"].as_u64().unwrap() + l["argument_base"].as_u64().unwrap());
            (payload["format"].as_str() != Some(&wanted) || payload["address"].as_u64() != Some(l["region_base"].as_u64().unwrap() + l["record_offset"].as_u64().unwrap())).then_some("Неверны позиционный format или вычисленный address.")
        }
        "protocol" => {
            let cap = &l["capabilities"][1];
            let wanted = json!([format!("HELLO {}", l["nonce"].as_str().unwrap()), format!("MODE {}", cap["mode"].as_str().unwrap()), format!("OPEN {}", cap["channel"].as_str().unwrap())]);
            (payload["frames"] != wanted).then_some("Нарушен порядок HELLO → MODE → OPEN или неверны параметры кадров.")
        }
        "race" => {
            let steps = payload["steps"].as_array().ok_or("steps должен быть списком из трёх шагов")?;
            if steps.len() != 3 { return Err("нужны ровно три шага: check, replace, use".into()); }
            let mut previous = -1;
            for (step, op) in steps.iter().zip(["check", "replace", "use"]) {
                let at = number(step, "at")?;
                if step["op"] != op || step["resource"] != l["resource"] || at <= previous || at > l["window_ms"].as_i64().unwrap() {
                    return Err("Неверны порядок шагов, resource или метки внутри window_ms.".into());
                }
                previous = at;
            }
            None
        }
        "command_chain" => {
            let separator = match l["parser"].as_str().unwrap() { "semicolon" => ";", "and_then" => "&&", _ => "|" };
            let wanted = format!("{}{}bw-open:{}:{}", l["command"].as_str().unwrap(), separator, l["channel"].as_str().unwrap(), l["nonce"].as_str().unwrap());
            (payload["argument"].as_str() != Some(&wanted)).then_some("Разборщик отверг argument: проверь разделитель, канал и nonce.")
        }
        _ => Some("Неизвестная механика уязвимости.")
    };
    error.map_or(Ok(()), |s| Err(s.into()))
}
pub fn attempt(world: &mut World, session: &Session, target: u32, v: &VulnInfo, payload: &Value, dry_run: bool) -> Result<Value, String> {
    ensure_reachable(world, session, target, v)?;
    let p = profile(world, target, v)?;
    let result = super::access::check(world,session,target,v).and_then(|_|envelope(&p,payload)).and_then(|_|{
        if custom_kind(p["kind"].as_str().unwrap_or("")){crate::plugins::exploit_types::validate(world,target,v,&p,payload)}else{validate(&p,payload)}
    });
    let source = super::tracks::source_ip(world, session);
    world.log(target, crate::net::machine::LogKind::Warn, &format!("{} payload для {} с {}: {}", if dry_run {"проверка"} else {"попытка"}, id(v), source, if result.is_ok() {"принят"} else {"отклонён"}));
    // Rejected attempts and probes are visible to defenders too.
    if dry_run || result.is_err() {
        if super::tracks::is_monitored(world, target) { super::tracks::attack_heat(world, session, target); }
        if super::defense::is_trap(world, target) { super::defense::spring_trap(world, target); }
    }
    Ok(json!({"ok": result.is_ok(), "applied": false, "id": id(v), "kind": p["kind"],
        "feedback": result.err().unwrap_or_else(|| if dry_run {"Payload подходит. Примени его через net.exploit или net.privesc.".into()} else {"Payload принят.".into()}),
        "hint": p["hints"][0]}))
}
pub fn starter(address: &str, selector: &str) -> String {
    format!("# Мой эксплоит: исследуй цель, составь payload и проверь результат.\n# Запуск: adder этот_файл.adr [адрес] [id]\naddress = sys.args[1] if len(sys.args) > 1 else {}\nselector = sys.args[2] if len(sys.args) > 2 else {}\np = net.research(address, selector)\nprint(p[\"name\"], p[\"kind\"])\nprint(p[\"layout\"])\nprint(\"Результат:\", p[\"result\"])\nfor condition in p[\"conditions\"]:\n    print(condition)\nfor hint in p[\"hints\"]:\n    print(hint)\n# TODO: добавь нужные поля из p[\"fields\"]. Готовые решения: market buy exp_<kind>.\npayload = {{\"fingerprint\": p[\"fingerprint\"]}}\ncheck = net.probe(address, selector, payload)\nprint(check[\"feedback\"])\nif check[\"ok\"]:\n    if p[\"local\"]:\n        print(net.privesc(payload, selector))\n    else:\n        print(net.exploit(address, selector, payload))\n", serde_json::to_string(address).unwrap(), serde_json::to_string(selector).unwrap())
}
