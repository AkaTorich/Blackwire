use blackwire_core::game::Game;
use blackwire_core::types::{OutKind, UiAction};
use blackwire_core::world::World;

fn game() -> (Game, u32) {
    let mut game = Game::new(World::new(7, "ghost", "pass"));
    let session = game.open_session();
    (game, session)
}

#[test]
fn local_shutdown_requests_quit_and_stops_command_chains() {
    for command in ["shutdown", "shutdown; echo bad > /home/ghost/after.txt", "shutdown | echo bad > /home/ghost/after.txt"] {
        let (mut game, session) = game();
        game.world.tick(5000);
        let uptime = game.world.uptime();
        let result = game.exec(session, command);
        assert!(result.actions.contains(&UiAction::QuitGame));
        assert!(!result.actions.contains(&UiAction::Clear));
        assert!(!result.lines.iter().any(|line| line.kind == OutKind::Error));
        assert_eq!(game.world.uptime(), uptime);
        assert!(game.world.machine(0).up);
        assert!(!game.fs_exists(Some(session), "/home/ghost/after.txt"));
        let restored = Game::from_json(&game.save_json()).unwrap();
        assert!(restored.world.machine(0).up);
    }
}

#[test]
fn reboot_stays_in_game_and_resets_uptime() {
    let (mut game, session) = game();
    game.world.tick(5000);
    assert!(game.world.uptime() > 0);
    let result = game.exec(session, "reboot");
    assert!(!result.actions.contains(&UiAction::QuitGame));
    assert!(result.actions.contains(&UiAction::Clear));
    assert_eq!(game.world.uptime(), 0);
}

#[test]
fn remote_shutdown_disconnects_without_quitting_game() {
    let (mut game, session) = game();
    let router = game.world.machines.iter().find(|m| m.hostname == "home-router").unwrap();
    let id = router.id;
    let command = format!("ssh admin@{} {}", router.ip, router.users.get("admin").unwrap().password);
    let connected = game.exec(session, &command);
    assert!(!connected.lines.iter().any(|line| line.kind == OutKind::Error));
    assert_eq!(game.prompt(session).host, "home-router");
    let result = game.exec(session, "shutdown");
    assert!(!result.actions.contains(&UiAction::QuitGame));
    assert!(!game.world.machine(id).up);
    assert!(game.world.machine(0).up);
    assert_eq!(game.prompt(session).host, "ghost-pc");
}

#[test]
fn unread_count_reaches_zero_after_reading_or_deleting_last_letter() {
    let (mut game, _) = game();
    assert!(game.mail_unread_count() > 0);
    for mail in game.mail_list("inbox") {
        game.mail_read(mail.id).unwrap();
    }
    assert_eq!(game.mail_unread_count(), 0);
    game.mail_send(&game.mail_address(), "New", "Message").unwrap();
    assert_eq!(game.mail_unread_count(), 1);
    let unread = game.mail_list("inbox").into_iter().find(|mail| !mail.read).unwrap();
    game.mail_delete(unread.id).unwrap();
    assert_eq!(game.mail_unread_count(), 0);
    assert_eq!(Game::from_json(&game.save_json()).unwrap().mail_unread_count(), 0);
}
