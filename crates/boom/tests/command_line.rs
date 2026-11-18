//! The command line: its commands, exit codes, JSON answers, no
//! prompts, and repeated writes (command-line).

use std::process::Command;

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
