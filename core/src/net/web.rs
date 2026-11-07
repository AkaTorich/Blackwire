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
    let nav = tr!(
        "<nav><a href=\"/\">Главная</a> <a href=\"/about\">О нас</a> <a href=\"/contacts\">Контакты</a> <a href=\"/login\">Вход</a></nav>",
        "<nav><a href=\"/\">Home</a> <a href=\"/about\">About</a> <a href=\"/contacts\">Contacts</a> <a href=\"/login\">Log in</a></nav>"
    );

    let index = trf!(
        "<header><h1>{name}</h1></header>{nav}<main><p>{desc}</p><p>Официальный сайт. {kind_line}</p></main><footer><p>{domain}</p></footer>",
        "<header><h1>{name}</h1></header>{nav}<main><p>{desc}</p><p>Official website. {kind_line}</p></main><footer><p>{domain}</p></footer>",
        name = name,
        nav = nav,
        desc = description,
        kind_line = kind_line(kind),
        domain = domain
    );

    let about = trf!(
        "<header><h1>О нас</h1></header>{nav}<main><h2>{name}</h2><p>{desc}</p><p>Мы работаем на рынке не первый год и дорожим доверием клиентов и партнёров.</p></main>",
        "<header><h1>About us</h1></header>{nav}<main><h2>{name}</h2><p>{desc}</p><p>We have been in business for years and value the trust of our clients and partners.</p></main>",
        nav = nav,
        name = name,
        desc = description
    );

    let mut contacts_html = trf!(
        "<header><h1>Контакты</h1></header>{nav}<main><p>Общий адрес: <a href=\"mailto:info@{domain}\">info@{domain}</a></p><h2>Сотрудники</h2><ul>",
        "<header><h1>Contacts</h1></header>{nav}<main><p>General address: <a href=\"mailto:info@{domain}\">info@{domain}</a></p><h2>Staff</h2><ul>",
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

    let login = trf!(
        "<header><h1>Вход</h1></header>{nav}<main><form method=\"post\" action=\"/login\"><p><label>Логин <input name=\"user\"></label></p><p><label>Пароль <input name=\"pass\" type=\"password\"></label></p><p><button type=\"submit\">Войти</button></p></form><p>Только для сотрудников {name}.</p></main>",
        "<header><h1>Log in</h1></header>{nav}<main><form method=\"post\" action=\"/login\"><p><label>Login <input name=\"user\"></label></p><p><label>Password <input name=\"pass\" type=\"password\"></label></p><p><button type=\"submit\">Log in</button></p></form><p>{name} staff only.</p></main>",
        nav = nav,
        name = name
    );

    vec![
        page("/", name, index),
        page("/about", tr!("О нас", "About us"), about),
        page("/contacts", tr!("Контакты", "Contacts"), contacts_html),
        page("/login", tr!("Вход", "Log in"), login),
    ]
}

fn kind_line(kind: OrgKind) -> &'static str {
    match kind {
        OrgKind::Corporation => tr!("Производство и услуги для предприятий.", "Manufacturing and business services."),
        OrgKind::Bank => tr!("Банковские услуги для частных лиц и бизнеса.", "Banking for individuals and businesses."),
        OrgKind::Hospital => tr!("Запись на приём и справочная — по телефону регистратуры.", "Appointments and inquiries — call the front desk."),
        OrgKind::SecurityCenter => tr!("Мониторинг и защита сетей.", "Network monitoring and protection."),
        OrgKind::CyberPolice => tr!("Приём обращений граждан по киберпреступлениям.", "Accepting citizen reports on cybercrime."),
        OrgKind::Isp => tr!("Доступ в сеть, имена и почта.", "Internet access, names and mail."),
        OrgKind::University => tr!("Образование и наука.", "Education and science."),
        OrgKind::Hosting => tr!("Размещение сайтов и приложений.", "Website and application hosting."),
        OrgKind::Government => tr!("Государственные услуги населению.", "Public services for citizens."),
        OrgKind::Media => tr!("Новости и расследования.", "News and investigations."),
        OrgKind::Shop => tr!("Товары с доставкой.", "Goods with delivery."),
        OrgKind::Home => tr!("Личная страница.", "Personal page."),
        OrgKind::Honeypot => tr!("Файлы в общем доступе.", "Publicly shared files."),
        OrgKind::Underground => "",
    }
}

/// Страница статуса сети провайдера.
pub fn isp_status(name: &str, domain: &str) -> Vec<WebPage> {
    let html = trf!(
        "<header><h1>{name} — статус сети</h1></header><main><table><tr><th>Узел</th><th>Состояние</th></tr><tr><td>Магистраль</td><td>в норме</td></tr><tr><td>Сервер имён</td><td>в норме</td></tr><tr><td>Почта</td><td>в норме</td></tr><tr><td>Биллинг</td><td>плановые работы</td></tr></table><p>Поддержка: <a href=\"mailto:support@{domain}\">support@{domain}</a></p></main>",
        "<header><h1>{name} — network status</h1></header><main><table><tr><th>Node</th><th>Status</th></tr><tr><td>Backbone</td><td>normal</td></tr><tr><td>Name server</td><td>normal</td></tr><tr><td>Mail</td><td>normal</td></tr><tr><td>Billing</td><td>scheduled maintenance</td></tr></table><p>Support: <a href=\"mailto:support@{domain}\">support@{domain}</a></p></main>",
        name = name,
        domain = domain
    );
    vec![page("/", name, html)]
}

/// Маленький сайт на хостинге (у каждого узла свой).
pub fn hosting_site(site_name: &str, kind_hint: &str) -> Vec<WebPage> {
    let index = trf!(
        "<header><h1>{name}</h1></header><nav><a href=\"/\">Главная</a> <a href=\"/about\">О сайте</a></nav><main><p>{hint}</p><p>Сайт размещён на хостинге «Грифон».</p></main>",
        "<header><h1>{name}</h1></header><nav><a href=\"/\">Home</a> <a href=\"/about\">About</a></nav><main><p>{hint}</p><p>Hosted by Grifon Hosting.</p></main>",
        name = site_name,
        hint = kind_hint
    );
    let about = trf!(
        "<header><h1>О сайте</h1></header><nav><a href=\"/\">Главная</a></nav><main><p>{name} — небольшой проект. Связь через форму на главной.</p></main>",
        "<header><h1>About</h1></header><nav><a href=\"/\">Home</a></nav><main><p>{name} is a small project. Get in touch via the form on the home page.</p></main>",
        name = site_name
    );
    vec![page("/", site_name, index), page("/about", tr!("О сайте", "About"), about)]
}

/// Страницы скрытой сети hello-world: шлюз `gate.hw`.
/// `invite` — код из письма игрока; `challenge_ip` — адрес домашнего ПК из мира
/// для текста первого испытания.
pub fn underground_gate(invite: &str, challenge_ip: &str) -> Vec<WebPage> {
    let index = tr!(
        "<header><h1>.</h1></header><main><p>Здесь ничего нет.</p></main>",
        "<header><h1>.</h1></header><main><p>Nothing here.</p></main>"
    )
    .to_string();

    let manifesto = tr!(
        "<header><h1>hello-world</h1></header><nav><a href=\"/manifesto\">Манифест</a> <a href=\"/board\">Доска</a></nav><main><p>Мы были в тени годами. Нас считали выдумкой.</p><p>После крупного дела мы вернулись — чтобы показать, что силы ещё есть, и чтобы вступиться за тех, кого это общество отбросило на обочину.</p><p>Мы не торгуем страхом и не служим сильным. Мы на стороне простых изгоев.</p></main>",
        "<header><h1>hello-world</h1></header><nav><a href=\"/manifesto\">Manifesto</a> <a href=\"/board\">Board</a></nav><main><p>We stayed in the shadows for years. People thought we were a myth.</p><p>After a big job we came back — to show we still have strength, and to stand up for those this society has pushed to the margins.</p><p>We don't trade in fear and we don't serve the powerful. We stand with the ordinary outcasts.</p></main>"
    )
    .to_string();

    let invite_html = trf!(
        "<header><h1>hello-world</h1></header><nav><a href=\"/manifesto\">Манифест</a> <a href=\"/board\">Доска</a></nav><main><p>Ты нашёл дверь. Значит, кто-то решил, что тебе стоит её открыть.</p><p>Коротко о нас — на странице <a href=\"/manifesto\">манифеста</a>. Если согласен, вот первое испытание.</p><section><h2>Испытание №1</h2><p>На машине с адресом {ip} лежит один файл. Найди его и пришли содержимое на <a href=\"mailto:null@hw\">null@hw</a>.</p><p>Как войти — твоя забота. Для того и зовём.</p></section><p>Дальше — <a href=\"/board\">доска заданий</a>. Эта ссылка одноразовая, второй раз она не откроется.</p></main>",
        "<header><h1>hello-world</h1></header><nav><a href=\"/manifesto\">Manifesto</a> <a href=\"/board\">Board</a></nav><main><p>You found the door. That means someone decided you should open it.</p><p>About us in short — on the <a href=\"/manifesto\">manifesto</a> page. If you agree, here is the first trial.</p><section><h2>Trial #1</h2><p>The machine at {ip} holds a single file. Find it and send its contents to <a href=\"mailto:null@hw\">null@hw</a>.</p><p>How to get in is your problem. That's why we're calling you.</p></section><p>Next — the <a href=\"/board\">job board</a>. This link is single-use; it won't open a second time.</p></main>",
        ip = challenge_ip
    );

    let board = tr!(
        "<header><h1>Доска заданий</h1></header><nav><a href=\"/manifesto\">Манифест</a></nav><main><ul><li>Испытание №1 — вход в сеть города</li><li>Остальное — скоро</li></ul></main>",
        "<header><h1>Job Board</h1></header><nav><a href=\"/manifesto\">Manifesto</a></nav><main><ul><li>Trial #1 — get into the city network</li><li>More — soon</li></ul></main>"
    )
    .to_string();

    vec![
        page("/", "hello-world", index),
        page(&format!("/invite/{}", invite), tr!("Приглашение", "Invitation"), invite_html),
        page("/manifesto", tr!("Манифест", "Manifesto"), manifesto),
        page("/board", tr!("Доска заданий", "Job Board"), board),
    ]
}

/// Почти пустая страница скрытого узла (`board.hw`, `drop.hw`, `relay.hw`).
pub fn underground_node(host: &str) -> Vec<WebPage> {
    let html = trf!(
        "<header><h1>{}</h1></header><main><p>Доступ по приглашению.</p></main>",
        "<header><h1>{}</h1></header><main><p>Invitation only.</p></main>",
        host
    );
    vec![page("/", host, html)]
}

/// Default router status page, including routers saved before the panel existed.
/// Explicit web pages supplied by a scenario or plugin take precedence.
pub fn router_status(machine: &super::machine::Machine) -> WebPage {
    fn esc(text: &str) -> String {
        text.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
            .replace('"', "&quot;").replace('\'', "&#39;")
    }

    let mut html = trf!(
        "<header><h1>{}</h1><p>Состояние маршрутизатора</p></header><main>\
         <h2>Устройство и сеть</h2><table>\
         <tr><th>Система</th><td>{}</td></tr>\
         <tr><th>Адрес узла</th><td><code>{}</code></td></tr>\
         <tr><th>Локальный адрес</th><td><code>{}</code></td></tr>\
         <tr><th>Процессор</th><td>{}</td></tr>\
         <tr><th>Память</th><td>{} МБ</td></tr></table>\
         <h2>Службы</h2><table><tr><th>Порт</th><th>Служба</th><th>Версия</th><th>Состояние</th></tr>",
        "<header><h1>{}</h1><p>Router status</p></header><main>\
         <h2>Device and network</h2><table>\
         <tr><th>System</th><td>{}</td></tr>\
         <tr><th>Host address</th><td><code>{}</code></td></tr>\
         <tr><th>Local address</th><td><code>{}</code></td></tr>\
         <tr><th>CPU</th><td>{}</td></tr>\
         <tr><th>Memory</th><td>{} MB</td></tr></table>\
         <h2>Services</h2><table><tr><th>Port</th><th>Service</th><th>Version</th><th>State</th></tr>",
        esc(&machine.hostname), esc(&machine.os), esc(&machine.ip),
        esc(machine.lan_ip.as_deref().unwrap_or(tr!("не назначен", "not assigned"))),
        esc(&machine.cpu), machine.ram_mb,
    );
    for service in &machine.services {
        html.push_str(&format!(
            "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
            service.port, esc(&service.name), esc(&service.version),
            if service.running { tr!("работает", "running") } else { tr!("остановлена", "stopped") },
        ));
    }
    html.push_str(tr!("</table><h2>Подключение</h2>", "</table><h2>Connection</h2>"));
    if machine.services.iter().any(|s| s.name == "ssh" && s.running) {
        html.push_str(tr!(
            "<p>Для управления устройством подключись через игровой терминал с учётной записью маршрутизатора. Справка: <code>man ssh</code>.</p>",
            "<p>To manage the device, connect from the game terminal with the router's account. Help: <code>man ssh</code>.</p>"
        ));
    } else {
        html.push_str(tr!("<p>Служба SSH остановлена или отсутствует.</p>", "<p>The SSH service is stopped or missing.</p>"));
    }
    html.push_str("</main>");
    page("/", &trf!("{} — маршрутизатор", "{} — router", machine.hostname), html)
}
