//! Письма: начальный ящик игрока (завязка сюжета) и рабочая почта сотрудников
//! организаций (с зацепками — паролями «на всякий случай», адресами серверов,
//! графиками). Тексты по-русски, без детского тона.

use super::machine::{fmt_date_ru, MailFolder, MailMessage};
use super::names::Person;
use crate::rng::Rng;

const DAY: u64 = 86_400;

fn msg(from: &str, to: &str, subject: &str, body: String, time: u64, read: bool, folder: MailFolder) -> MailMessage {
    MailMessage { id: 0, from: from.to_string(), to: to.to_string(), subject: subject.to_string(), body, time, read, folder }
}

/// Начальный ящик игрока: 7 входящих писем с разным временем за последние дни.
/// `epoch` — старт игрового времени; письма датируются назад от него.
pub fn player_inbox(
    handle: &str,
    player_ip: &str,
    bank_name: &str,
    bank_domain: &str,
    corp_name: &str,
    corp_domain: &str,
    isp_domain: &str,
    invite: &str,
    epoch: u64,
) -> Vec<MailMessage> {
    let me = format!("{}@home.net", handle);

    // Банк — просрочка и изъятие половины имущества.
    let t_bank = epoch - 9 * DAY;
    let bank = msg(
        &format!("noreply@{}", bank_domain),
        &me,
        &format!("{}: просрочка по кредиту и опись имущества", bank_name),
        format!(
            "Уважаемый клиент!\n\nПо договору потребительского кредита у Вас образовалась длительная просрочка. Досудебный порядок исчерпан.\n\nВ счёт погашения половины задолженности произведено изъятие имущества по адресу регистрации. Опись изъятого:\n\n  - автомобиль;\n  - телевизор и домашний кинотеатр;\n  - холодильник и стиральная машина;\n  - рабочий ноутбук;\n  - мебель из гостиной (комплект).\n\nОстаток долга сохраняется. Для реструктуризации обратитесь в отделение.\n\n{}\nОтдел взыскания",
            bank_name
        ),
        t_bank,
        false,
        MailFolder::Inbox,
    );

    // Бывший работодатель — расторжение договора.
    let t_corp = epoch - 14 * DAY;
    let corp = msg(
        &format!("hr@{}", corp_domain),
        &me,
        "Расторжение трудового договора",
        format!(
            "Здравствуйте.\n\nУведомляем, что трудовой договор с Вами расторгнут с текущей даты по инициативе работодателя в связи с сокращением. Доступ к рабочим системам отключён.\n\nОкончательный расчёт перечислен на карту. Трудовую книжку и справки можно получить в отделе кадров.\n\nБлагодарим за работу.\n{}\nОтдел кадров",
            corp_name
        ),
        t_corp,
        true,
        MailFolder::Inbox,
    );

    // Провайдер — тариф и внешний адрес.
    let t_isp = epoch - 20 * DAY;
    let isp = msg(
        &format!("support@{}", isp_domain),
        &me,
        "Ваше подключение и тариф",
        format!(
            "Здравствуйте, {}!\n\nПодключение активно. Тариф «Домашний», до 100 Мбит/с.\nВаш внешний адрес: {}\nСервер имён и почта — на нашей стороне. Домашний почтовый адрес вида имя@home.net.\n\nПо вопросам отвечайте на это письмо.\nПровайдер «Меридиан»",
            handle, player_ip
        ),
        t_isp,
        true,
        MailFolder::Inbox,
    );

    // Два спама.
    let spam1 = msg(
        "win@mega-prize.net",
        &me,
        "Вы выиграли! Осталось получить приз",
        "Поздравляем! Ваш адрес выбран среди миллионов. Чтобы получить приз, перейдите по ссылке и укажите данные карты для перевода.\n\nНе упустите шанс! Предложение ограничено.".to_string(),
        epoch - 3 * DAY,
        false,
        MailFolder::Inbox,
    );
    let spam2 = msg(
        "doctor@cure-all.net",
        &me,
        "Средство от усталости — минус 70% только сегодня",
        "Новейшая формула! Энергия на весь день. Заказывайте прямо сейчас со скидкой. Доставка в любой город.".to_string(),
        epoch - DAY - 3600,
        false,
        MailFolder::Inbox,
    );

    // hello-world — загадочное письмо со ссылкой-приглашением.
    let t_hw = epoch - 6 * 3600;
    let hw = msg(
        "null@hw",
        &me,
        "—",
        format!(
            "Ты нас не знаешь. Это к лучшему.\n\nТы потерял половину дома из-за бумажки с печатью. Таких, как ты, много. Мы давно смотрим.\n\nhttp://gate.hw/invite/{}\n\nОдин переход. Второй раз ссылка не откроется. Решай сам.\n\nhello-world",
            invite
        ),
        t_hw,
        false,
        MailFolder::Inbox,
    );

    let _ = fmt_date_ru; // дата может пригодиться оформлению писем позже
    vec![bank, corp, isp, spam1, spam2, hw]
        .into_iter()
        .chain(std::iter::once(welcome_note(&me, epoch)))
        .collect()
}

/// Седьмое письмо — старое приветствие провайдера при регистрации почты.
fn welcome_note(me: &str, epoch: u64) -> MailMessage {
    msg(
        "noreply@meridian.net",
        me,
        "Почтовый ящик создан",
        "Ваш домашний почтовый ящик создан и готов к работе. Это письмо можно удалить.\n\nПровайдер «Меридиан»".to_string(),
        epoch - 20 * DAY - 60,
        true,
        MailFolder::Inbox,
    )
}

/// Рабочие письма сотрудника (1–4), складываются в его ящик. С зацепками:
/// адрес сервера, пароль «на всякий случай», график.
pub fn employee_mail(
    rng: &mut Rng,
    person: &Person,
    org_name: &str,
    domain: &str,
    colleague: Option<&Person>,
    leak_ip: Option<&str>,
    leak_password: Option<&str>,
    epoch: u64,
) -> Vec<MailMessage> {
    let me = format!("{}@{}", person.login, domain);
    let mut out: Vec<MailMessage> = Vec::new();
    let n = rng.range(1, 5);

    for i in 0..n {
        let t = epoch - rng.range(1, 30) as u64 * DAY - rng.range(0, DAY as i64) as u64;
        let (from, subject, body) = match i {
            0 => (
                format!("it@{}", domain),
                "Плановые работы".to_string(),
                format!(
                    "Коллеги, в выходные плановое обслуживание серверов {}. Возможны перерывы в работе почты и общих дисков. Просьба сохранить документы заранее.\nСлужба ИТ",
                    org_name
                ),
            ),
            1 => {
                let colleague_name = colleague.map(|c| format!("{} {}", c.first, c.last)).unwrap_or_else(|| "коллега".into());
                let hook = match leak_ip {
                    Some(ip) => format!("\n\nКстати, сервер с отчётами теперь по адресу {} — сохрани себе.", ip),
                    None => String::new(),
                };
                (
                    colleague.map(|c| format!("{}@{}", c.login, domain)).unwrap_or_else(|| format!("info@{}", domain)),
                    "По проекту".to_string(),
                    format!(
                        "Привет, {first}!\nПосмотри, пожалуйста, последние правки. {who} обещал прислать свою часть к пятнице.{hook}\n\nСпасибо.",
                        first = person.first,
                        who = colleague_name,
                        hook = hook
                    ),
                )
            }
            2 => {
                let hook = match leak_password {
                    Some(pw) => format!("\n\nЧтобы не бегать каждый раз к администратору, пароль от общей учётки записал здесь: {}. Только не потеряй.", pw),
                    None => "\n\nПароль от общей учётки спроси у администратора.".to_string(),
                };
                (
                    format!("admin@{}", domain),
                    "Доступ к общему ресурсу".to_string(),
                    format!("Здравствуйте.\nДоступ выдан.{}\nАдминистратор", hook),
                )
            }
            _ => (
                format!("hr@{}", domain),
                "График на неделю".to_string(),
                "Коллеги, прикладываю график смен на следующую неделю. Изменения — до среды.\nОтдел кадров".to_string(),
            ),
        };
        out.push(msg(&from, &me, &subject, body, t, rng.chance(0.5), MailFolder::Inbox));
    }
    out
}
