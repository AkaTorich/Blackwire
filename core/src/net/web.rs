//! Сборка веб-страниц для http-служб. HTML простой и семантичный, без стилей и
//! скриптов — вид задаёт браузер игры; ссылки относительные. На странице `/`
//! заголовок `h1` несёт имя организации.

use super::orgs::OrgKind;
use super::services::WebPage;

fn page(path: &str, title: &str, html: String) -> WebPage {
    WebPage { path: path.to_string(), title: title.to_string(), html }
}

/// Сотрудник для страницы контактов: отображаемое имя и адрес почты.
pub struct Contact {
    pub name: String,
    pub email: String,
    pub role: String,
}

/// Сайт организации: главная, о нас, контакты (имена и почта — зацепки), вход.
pub fn org_site(kind: OrgKind, name: &str, domain: &str, description: &str, contacts: &[Contact]) -> Vec<WebPage> {
    let nav = "<nav><a href=\"/\">Главная</a> <a href=\"/about\">О нас</a> <a href=\"/contacts\">Контакты</a> <a href=\"/login\">Вход</a></nav>";

    let index = format!(
        "<header><h1>{name}</h1></header>{nav}<main><p>{desc}</p><p>Официальный сайт. {kind_line}</p></main><footer><p>{domain}</p></footer>",
        name = name,
        nav = nav,
        desc = description,
        kind_line = kind_line(kind),
        domain = domain
    );

    let about = format!(
        "<header><h1>О нас</h1></header>{nav}<main><h2>{name}</h2><p>{desc}</p><p>Мы работаем на рынке не первый год и дорожим доверием клиентов и партнёров.</p></main>",
        nav = nav,
        name = name,
        desc = description
    );

    let mut contacts_html = format!(
        "<header><h1>Контакты</h1></header>{nav}<main><p>Общий адрес: <a href=\"mailto:info@{domain}\">info@{domain}</a></p><h2>Сотрудники</h2><ul>",
        nav = nav,
        domain = domain
    );
    for c in contacts {
        contacts_html.push_str(&format!(
            "<li>{} — {} — <a href=\"mailto:{}\">{}</a></li>",
            c.name, c.role, c.email, c.email
        ));
    }
    contacts_html.push_str("</ul></main>");

    let login = format!(
        "<header><h1>Вход</h1></header>{nav}<main><form method=\"post\" action=\"/login\"><p><label>Логин <input name=\"user\"></label></p><p><label>Пароль <input name=\"pass\" type=\"password\"></label></p><p><button type=\"submit\">Войти</button></p></form><p>Только для сотрудников {name}.</p></main>",
        nav = nav,
        name = name
    );

    vec![
        page("/", name, index),
        page("/about", "О нас", about),
        page("/contacts", "Контакты", contacts_html),
        page("/login", "Вход", login),
    ]
}

fn kind_line(kind: OrgKind) -> &'static str {
    match kind {
        OrgKind::Corporation => "Производство и услуги для предприятий.",
        OrgKind::Bank => "Банковские услуги для частных лиц и бизнеса.",
        OrgKind::Hospital => "Запись на приём и справочная — по телефону регистратуры.",
        OrgKind::SecurityCenter => "Мониторинг и защита сетей.",
        OrgKind::CyberPolice => "Приём обращений граждан по киберпреступлениям.",
        OrgKind::Isp => "Доступ в сеть, имена и почта.",
        OrgKind::University => "Образование и наука.",
        OrgKind::Hosting => "Размещение сайтов и приложений.",
        OrgKind::Government => "Государственные услуги населению.",
        OrgKind::Media => "Новости и расследования.",
        OrgKind::Shop => "Товары с доставкой.",
        OrgKind::Home => "Личная страница.",
        OrgKind::Honeypot => "Файлы в общем доступе.",
        OrgKind::Underground => "",
    }
}

/// Страница статуса сети провайдера.
pub fn isp_status(name: &str, domain: &str) -> Vec<WebPage> {
    let html = format!(
        "<header><h1>{name} — статус сети</h1></header><main><table><tr><th>Узел</th><th>Состояние</th></tr><tr><td>Магистраль</td><td>в норме</td></tr><tr><td>Сервер имён</td><td>в норме</td></tr><tr><td>Почта</td><td>в норме</td></tr><tr><td>Биллинг</td><td>плановые работы</td></tr></table><p>Поддержка: <a href=\"mailto:support@{domain}\">support@{domain}</a></p></main>",
        name = name,
        domain = domain
    );
    vec![page("/", name, html)]
}

/// Маленький сайт на хостинге (у каждого узла свой).
pub fn hosting_site(site_name: &str, kind_hint: &str) -> Vec<WebPage> {
    let index = format!(
        "<header><h1>{name}</h1></header><nav><a href=\"/\">Главная</a> <a href=\"/about\">О сайте</a></nav><main><p>{hint}</p><p>Сайт размещён на хостинге «Грифон».</p></main>",
        name = site_name,
        hint = kind_hint
    );
    let about = format!(
        "<header><h1>О сайте</h1></header><nav><a href=\"/\">Главная</a></nav><main><p>{name} — небольшой проект. Связь через форму на главной.</p></main>",
        name = site_name
    );
    vec![page("/", site_name, index), page("/about", "О сайте", about)]
}

/// Страницы скрытой сети hello-world: шлюз `gate.hw`.
/// `invite` — код из письма игрока; `challenge_ip` — адрес домашнего ПК из мира
/// для текста первого испытания.
pub fn underground_gate(invite: &str, challenge_ip: &str) -> Vec<WebPage> {
    let index = "<header><h1>.</h1></header><main><p>Здесь ничего нет.</p></main>".to_string();

    let manifesto = "<header><h1>hello-world</h1></header><nav><a href=\"/manifesto\">Манифест</a> <a href=\"/board\">Доска</a></nav><main><p>Мы были в тени годами. Нас считали выдумкой.</p><p>После крупного дела мы вернулись — чтобы показать, что силы ещё есть, и чтобы вступиться за тех, кого это общество отбросило на обочину.</p><p>Мы не торгуем страхом и не служим сильным. Мы на стороне простых изгоев.</p></main>".to_string();

    let invite_html = format!(
        "<header><h1>hello-world</h1></header><nav><a href=\"/manifesto\">Манифест</a> <a href=\"/board\">Доска</a></nav><main><p>Ты нашёл дверь. Значит, кто-то решил, что тебе стоит её открыть.</p><p>Коротко о нас — на странице <a href=\"/manifesto\">манифеста</a>. Если согласен, вот первое испытание.</p><section><h2>Испытание №1</h2><p>На машине с адресом {ip} лежит один файл. Найди его и пришли содержимое на <a href=\"mailto:null@hw\">null@hw</a>.</p><p>Как войти — твоя забота. Для того и зовём.</p></section><p>Дальше — <a href=\"/board\">доска заданий</a>. Эта ссылка одноразовая, второй раз она не откроется.</p></main>",
        ip = challenge_ip
    );

    let board = "<header><h1>Доска заданий</h1></header><nav><a href=\"/manifesto\">Манифест</a></nav><main><ul><li>Испытание №1 — вход в сеть города</li><li>Остальное — скоро</li></ul></main>".to_string();

    vec![
        page("/", "hello-world", index),
        page(&format!("/invite/{}", invite), "Приглашение", invite_html),
        page("/manifesto", "Манифест", manifesto),
        page("/board", "Доска заданий", board),
    ]
}

/// Почти пустая страница скрытого узла (`board.hw`, `drop.hw`, `relay.hw`).
pub fn underground_node(host: &str) -> Vec<WebPage> {
    let html = format!("<header><h1>{}</h1></header><main><p>Доступ по приглашению.</p></main>", host);
    vec![page("/", host, html)]
}

/// Default router status page, including routers saved before the panel existed.
/// Explicit web pages supplied by a scenario or plugin take precedence.
pub fn router_status(machine: &super::machine::Machine) -> WebPage {
    fn esc(text: &str) -> String {
        text.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
            .replace('"', "&quot;").replace('\'', "&#39;")
    }

    let mut html = format!(
        "<header><h1>{}</h1><p>Состояние маршрутизатора</p></header><main>\
         <h2>Устройство и сеть</h2><table>\
         <tr><th>Система</th><td>{}</td></tr>\
         <tr><th>Адрес узла</th><td><code>{}</code></td></tr>\
         <tr><th>Локальный адрес</th><td><code>{}</code></td></tr>\
         <tr><th>Процессор</th><td>{}</td></tr>\
         <tr><th>Память</th><td>{} МБ</td></tr></table>\
         <h2>Службы</h2><table><tr><th>Порт</th><th>Служба</th><th>Версия</th><th>Состояние</th></tr>",
        esc(&machine.hostname), esc(&machine.os), esc(&machine.ip),
        esc(machine.lan_ip.as_deref().unwrap_or("не назначен")),
        esc(&machine.cpu), machine.ram_mb,
    );
    for service in &machine.services {
        html.push_str(&format!(
            "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
            service.port, esc(&service.name), esc(&service.version),
            if service.running { "работает" } else { "остановлена" },
        ));
    }
    html.push_str("</table><h2>Подключение</h2>");
    if machine.services.iter().any(|s| s.name == "ssh" && s.running) {
        html.push_str("<p>Для управления устройством подключись через игровой терминал с учётной записью маршрутизатора. Справка: <code>man ssh</code>.</p>");
    } else {
        html.push_str("<p>Служба SSH остановлена или отсутствует.</p>");
    }
    html.push_str("</main>");
    page("/", &format!("{} — маршрутизатор", machine.hostname), html)
}
