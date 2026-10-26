//! Разбор командной строки: кавычки, разделители, конвейеры, перенаправления.

use blackwire_core::shell::parse::{parse, Sep};

#[test]
fn simple_words() {
    let jobs = parse("ls -l /home").unwrap();
    assert_eq!(jobs.len(), 1);
    let cmd = &jobs[0].pipeline.cmds[0];
    let words: Vec<String> = cmd.words.iter().map(|w| w.raw()).collect();
    assert_eq!(words, vec!["ls", "-l", "/home"]);
}

#[test]
fn double_quotes_keep_spaces() {
    let jobs = parse("echo \"привет мир\"").unwrap();
    let cmd = &jobs[0].pipeline.cmds[0];
    assert_eq!(cmd.words.len(), 2);
    assert_eq!(cmd.words[1].raw(), "привет мир");
}

#[test]
fn single_quotes_literal() {
    let jobs = parse("echo '$HOME'").unwrap();
    let w = &jobs[0].pipeline.cmds[0].words[1];
    assert_eq!(w.raw(), "$HOME");
    assert!(w.fully_quoted());
}

#[test]
fn escaped_space() {
    let jobs = parse("cat a\\ b.txt").unwrap();
    assert_eq!(jobs[0].pipeline.cmds[0].words[1].raw(), "a b.txt");
}

#[test]
fn comment_is_dropped() {
    let jobs = parse("ls # это комментарий").unwrap();
    assert_eq!(jobs[0].pipeline.cmds[0].words.len(), 1);
}

#[test]
fn pipeline_splits() {
    let jobs = parse("cat f | grep x | wc -l").unwrap();
    assert_eq!(jobs.len(), 1);
    assert_eq!(jobs[0].pipeline.cmds.len(), 3);
}

#[test]
fn separators_and_or() {
    let jobs = parse("a && b || c ; d").unwrap();
    assert_eq!(jobs.len(), 4);
    assert_eq!(jobs[0].sep, Sep::Seq);
    assert_eq!(jobs[1].sep, Sep::And);
    assert_eq!(jobs[2].sep, Sep::Or);
    assert_eq!(jobs[3].sep, Sep::Seq);
}

#[test]
fn redirections() {
    let jobs = parse("echo hi > out.txt").unwrap();
    let s = &jobs[0].pipeline.cmds[0];
    assert!(s.redirs.stdout.is_some());
    assert_eq!(s.redirs.stdout.as_ref().unwrap().1, false);

    let jobs = parse("echo hi >> out.txt").unwrap();
    assert_eq!(jobs[0].pipeline.cmds[0].redirs.stdout.as_ref().unwrap().1, true);

    let jobs = parse("cmd 2> err.txt").unwrap();
    assert!(jobs[0].pipeline.cmds[0].redirs.stderr.is_some());

    let jobs = parse("cmd < in.txt").unwrap();
    assert!(jobs[0].pipeline.cmds[0].redirs.stdin.is_some());
}

#[test]
fn unterminated_quote_errors() {
    assert!(parse("echo \"no end").is_err());
}

#[test]
fn dangling_redirection_errors() {
    assert!(parse("echo hi >").is_err());
}

#[test]
fn empty_line_ok() {
    assert_eq!(parse("").unwrap().len(), 0);
    assert_eq!(parse("   ").unwrap().len(), 0);
}
