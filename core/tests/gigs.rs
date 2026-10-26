//! Подработки: открытые заказы, выполнение (доступ/доставка), оплата и
//! пополнение доски.

use blackwire_core::gigs::{self, GigGoal};
use blackwire_core::net::machine::{MailFolder, MailMessage};
use blackwire_core::world::World;

fn world() -> World {
    World::new(7, "ghost", "secret")
}

fn push_sent(w: &mut World, to: &str, body: &str) {
    let now = w.now();
    w.machines[0].mail.push(MailMessage {
        id: 777,
        from: "ghost@home.net".to_string(),
        to: to.to_string(),
        subject: "добыча".to_string(),
        body: body.to_string(),
        time: now,
        read: true,
        folder: MailFolder::Sent,
    });
}

#[test]
fn gigs_open_on_new_world() {
    let w = world();
    assert!(!w.gigs.gigs.is_empty(), "на доске есть подработки");
    for g in &w.gigs.gigs {
        let t = match &g.goal {
            GigGoal::Deliver { target, .. } | GigGoal::Own { target } => *target,
        };
        assert!(w.has_machine(t), "цель заказа — настоящая машина");
        assert!(g.reward > 0);
    }
}

#[test]
fn own_gig_pays_and_refills() {
    let mut w = world();
    let idx = w.gigs.gigs.iter().position(|g| matches!(g.goal, GigGoal::Own { .. })).expect("есть заказ на доступ");
    let (target, reward) = match &w.gigs.gigs[idx].goal {
        GigGoal::Own { target } => (*target, w.gigs.gigs[idx].reward),
        _ => unreachable!(),
    };
    let count = w.gigs.gigs.len();
    let money0 = w.player.money;
    w.player.owned.push(target);
    gigs::check(&mut w);
    assert_eq!(w.player.money, money0 + reward, "за заказ заплатили");
    assert!(w.gigs.done >= 1);
    assert_eq!(w.gigs.gigs.len(), count, "доска снова полна");
    assert!(w.machines[0].mail.iter().any(|m| m.from == "broker@hw"), "скупщик прислал оплату");
}

#[test]
fn deliver_gig_pays() {
    let mut w = world();
    let idx = w.gigs.gigs.iter().position(|g| matches!(g.goal, GigGoal::Deliver { .. })).expect("есть заказ на доставку");
    let (needle, reward) = match &w.gigs.gigs[idx].goal {
        GigGoal::Deliver { needle, .. } => (needle.clone(), w.gigs.gigs[idx].reward),
        _ => unreachable!(),
    };
    let money0 = w.player.money;
    push_sent(&mut w, "broker@hw", &format!("файл:\n{}\n", needle));
    gigs::check(&mut w);
    assert_eq!(w.player.money, money0 + reward, "доставка оплачена");
}

#[test]
fn own_gig_not_already_owned() {
    // заказ на доступ не должен указывать на уже взломанную машину
    let w = world();
    for g in &w.gigs.gigs {
        if let GigGoal::Own { target } = &g.goal {
            assert!(!w.player.owned.contains(target));
        }
    }
}
