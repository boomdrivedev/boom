//! The command line: its commands, exit codes, JSON answers, no
//! prompts, and repeated writes (command-line).

mod support;

use std::process::Command;

use support::Place;

#[test]
fn the_version() {
    let output = Command::new(env!("CARGO_BIN_EXE_boom"))
        .arg("--version")
        .output()
        .expect("the binary runs");
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        String::from_utf8(output.stdout).unwrap().trim_end(),
        format!("boom {}", env!("CARGO_PKG_VERSION"))
    );
}

#[test]
fn the_log_of_a_new_store() {
    let place = Place::new();
    place.made();
    let log = place.run(&["log"]);
    assert_eq!(log.code, 0, "{log:?}");
    let lines: Vec<&str> = log.stdout.lines().collect();
    assert_eq!(lines.len(), 1, "{log:?}");
    assert!(lines[0].starts_with("1 registered by ada"), "{log:?}");
}
