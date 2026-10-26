//! Команды оболочки на компьютере игрока: ls, cat, echo, конвейеры, перенаправления,
//! пользователи, процессы, подсказка.

use blackwire_core::game::Game;
use blackwire_core::types::{OutKind, UiAction};
use blackwire_core::world::World;

/// Игра с миром из зерна и одним открытым сеансом.
fn game() -> (Game, u32) {
    let mut g = Game::new(World::new(7, "ghost", "pass"));
    let s = g.open_session();
    (g, s)
}

/// Строки вывода без ошибок, склеенные.
fn out(g: &mut Game, s: u32, line: &str) -> String {
    let r = g.exec(s, line);
    r.lines.iter().filter(|l| l.kind != OutKind::Error).map(|l| l.text.clone()).collect::<Vec<_>>().join("\n")
}

/// Весь вывод (включая ошибки).
fn all(g: &mut Game, s: u32, line: &str) -> String {
    let r = g.exec(s, line);
    r.lines.iter().map(|l| l.text.clone()).collect::<Vec<_>>().join("\n")
}

#[test]
fn pwd_and_cd_home() {
    let (mut g, s) = game();
    assert_eq!(out(&mut g, s, "pwd"), "/home/ghost");
    out(&mut g, s, "cd /etc");
    assert_eq!(out(&mut g, s, "pwd"), "/etc");
    out(&mut g, s, "cd");
    assert_eq!(out(&mut g, s, "pwd"), "/home/ghost");
    // cd - возвращает в предыдущую папку
    out(&mut g, s, "cd /etc");
    out(&mut g, s, "cd /tmp");
    out(&mut g, s, "cd -");
    assert_eq!(out(&mut g, s, "pwd"), "/etc");
}

#[test]
fn cd_tilde() {
    let (mut g, s) = game();
    out(&mut g, s, "cd /etc");
    out(&mut g, s, "cd ~");
    assert_eq!(out(&mut g, s, "pwd"), "/home/ghost");
}

#[test]
fn echo_and_redirect() {
    let (mut g, s) = game();
    out(&mut g, s, "echo привет > hi.txt");
    assert_eq!(out(&mut g, s, "cat hi.txt"), "привет");
    out(&mut g, s, "echo ещё >> hi.txt");
    assert_eq!(out(&mut g, s, "cat hi.txt"), "привет\nещё");
}

#[test]
fn echo_variable_expands() {
    let (mut g, s) = game();
    assert_eq!(out(&mut g, s, "echo $USER"), "ghost");
    assert_eq!(out(&mut g, s, "echo ${HOME}"), "/home/ghost");
    assert_eq!(out(&mut g, s, "echo '$USER'"), "$USER");
}

#[test]
fn ls_shows_files() {
    let (mut g, s) = game();
    let o = out(&mut g, s, "ls");
    assert!(o.contains("readme.txt"));
    assert!(o.contains("scripts/"));
}

#[test]
fn ls_long_format() {
    let (mut g, s) = game();
    let o = out(&mut g, s, "ls -l");
    assert!(o.contains("ghost"));
    assert!(o.lines().any(|l| l.starts_with('-') || l.starts_with('d')));
}

#[test]
fn grep_pipe() {
    let (mut g, s) = game();
    out(&mut g, s, "echo -e \"один\\nдва\\nтри\" > nums.txt");
    let o = out(&mut g, s, "cat nums.txt | grep дв");
    assert_eq!(o, "два");
}

#[test]
fn grep_with_file_and_numbers() {
    let (mut g, s) = game();
    out(&mut g, s, "echo -e \"alpha\\nbeta\\nalanine\" > w.txt");
    let o = out(&mut g, s, "grep -n al w.txt");
    assert!(o.contains("1:alpha"));
    assert!(o.contains("3:alanine"));
}

#[test]
fn wc_counts_lines() {
    let (mut g, s) = game();
    out(&mut g, s, "echo -e \"a\\nb\\nc\" > three.txt");
    let o = out(&mut g, s, "wc -l three.txt");
    assert!(o.starts_with('3') || o.contains("3"));
}

#[test]
fn pipe_sort_uniq() {
    let (mut g, s) = game();
    out(&mut g, s, "echo -e \"b\\na\\nb\\nc\" > u.txt");
    let o = out(&mut g, s, "cat u.txt | sort | uniq");
    assert_eq!(o, "a\nb\nc");
}

#[test]
fn mkdir_p_and_rm_r() {
    let (mut g, s) = game();
    out(&mut g, s, "mkdir -p a/b/c");
    assert!(g.fs_exists(Some(s), "/home/ghost/a/b/c"));
    out(&mut g, s, "rm -r a");
    assert!(!g.fs_exists(Some(s), "/home/ghost/a"));
}

#[test]
fn touch_and_rm() {
    let (mut g, s) = game();
    out(&mut g, s, "touch z.txt");
    assert!(g.fs_exists(Some(s), "/home/ghost/z.txt"));
    out(&mut g, s, "rm z.txt");
    assert!(!g.fs_exists(Some(s), "/home/ghost/z.txt"));
}

#[test]
fn and_or_sequencing() {
    let (mut g, s) = game();
    // true && echo ok  → печатает ok
    assert_eq!(out(&mut g, s, "true && echo ok"), "ok");
    // false || echo alt → печатает alt
    assert_eq!(out(&mut g, s, "false || echo alt"), "alt");
    // false && echo no → ничего
    assert_eq!(out(&mut g, s, "false && echo no"), "");
}

#[test]
fn useradd_su_passwd() {
    let (mut g, s) = game();
    out(&mut g, s, "useradd bob secret");
    let o = all(&mut g, s, "su bob secret");
    assert!(!o.contains("неверный"));
    assert_eq!(out(&mut g, s, "whoami"), "bob");
    // неверный пароль при обратном su другого пользователя? вернёмся root-ом
    out(&mut g, s, "exit");
}

#[test]
fn su_wrong_password_fails() {
    let (mut g, s) = game();
    out(&mut g, s, "useradd bob secret");
    let r = g.exec(s, "su bob nope");
    assert!(r.lines.iter().any(|l| l.kind == OutKind::Error));
    assert_eq!(out(&mut g, s, "whoami"), "ghost");
}

#[test]
fn passwd_changes_password() {
    let (mut g, s) = game();
    out(&mut g, s, "useradd bob old");
    out(&mut g, s, "passwd bob new");
    // от имени ghost (админ на своей машине не root — но ghost это владелец-админ)
    // проверим через su с новым паролем, выполнив от root
    out(&mut g, s, "su root pass");
    let o = all(&mut g, s, "su bob new");
    assert!(!o.to_lowercase().contains("неверный"));
}

#[test]
fn ps_lists_processes() {
    let (mut g, s) = game();
    let o = out(&mut g, s, "ps -a");
    assert!(o.contains("init"));
    assert!(o.contains("PID"));
}

#[test]
fn kill_unknown_pid_errors() {
    let (mut g, s) = game();
    let r = g.exec(s, "kill 99999");
    assert!(r.lines.iter().any(|l| l.kind == OutKind::Error));
}

#[test]
fn clear_emits_action() {
    let (mut g, s) = game();
    let r = g.exec(s, "clear");
    assert!(r.actions.iter().any(|a| matches!(a, UiAction::Clear)));
}

#[test]
fn exit_home_emits_exit() {
    let (mut g, s) = game();
    let r = g.exec(s, "exit");
    assert!(r.actions.iter().any(|a| matches!(a, UiAction::Exit)));
}

#[test]
fn edit_opens_app() {
    let (mut g, s) = game();
    let r = g.exec(s, "edit readme.txt");
    assert!(r.actions.iter().any(|a| matches!(a, UiAction::OpenApp { app, .. } if app == "editor")));
}

#[test]
fn unknown_command() {
    let (mut g, s) = game();
    let r = g.exec(s, "нетакой");
    assert!(r.lines.iter().any(|l| l.kind == OutKind::Error));
    assert_eq!(r.prompt.user, "ghost");
}

#[test]
fn complete_commands() {
    let (g, s) = game();
    let v = g.complete(s, "ls");
    assert!(v.iter().any(|c| c == "ls"));
    let v = g.complete(s, "neo");
    assert!(v.iter().any(|c| c == "neofetch"));
}

#[test]
fn complete_filenames() {
    let (g, s) = game();
    let v = g.complete(s, "cat read");
    assert!(v.iter().any(|c| c.contains("readme.txt")));
}

#[test]
fn glob_expands() {
    let (mut g, s) = game();
    out(&mut g, s, "touch one.log two.log three.txt");
    let o = out(&mut g, s, "ls *.log");
    assert!(o.contains("one.log"));
    assert!(o.contains("two.log"));
    assert!(!o.contains("three.txt"));
}

#[test]
fn chmod_via_shell() {
    let (mut g, s) = game();
    out(&mut g, s, "touch run.adr");
    out(&mut g, s, "chmod 755 run.adr");
    let o = out(&mut g, s, "ls -l run.adr");
    assert!(o.contains("rwxr-xr-x"));
}

#[test]
fn bc_calculates() {
    let (mut g, s) = game();
    assert_eq!(out(&mut g, s, "bc \"(2 + 3) * 4\""), "20");
}

#[test]
fn neofetch_runs() {
    let (mut g, s) = game();
    let o = out(&mut g, s, "neofetch");
    assert!(o.contains("ghost-pc") || o.contains("ghost@"));
}

#[test]
fn help_lists_commands() {
    let (mut g, s) = game();
    let o = out(&mut g, s, "help");
    assert!(o.contains("ls"));
    assert!(o.contains("ssh"));
}

#[test]
fn command_help_flag() {
    let (mut g, s) = game();
    let o = out(&mut g, s, "ls --help");
    assert!(o.to_lowercase().contains("использование"));
}
