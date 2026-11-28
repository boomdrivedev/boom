//! The binary's main path: with no terminal, it acts, writes its answer
//! on standard output and exits with its code (command-line). The other
//! tests of the command line run in the crate, in
//! src/tests/command_line.rs.

#[path = "../src/tests/support.rs"]
mod support;

use std::process::Command;

use support::Place;

/// The binary's main path: with no terminal, it acts and writes its
/// answer and its exit code.
#[test]
fn no_terminal() {
    let place = Place::new();
    place.made();
    let store = place.store_arg();
    let mut args = PROPOSAL.to_vec();
    args.extend(["--store", &store, "--json"]);
    let output = Command::new(env!("CARGO_BIN_EXE_boom"))
        .args(&args)
        .stdin(std::process::Stdio::null())
        .output()
        .expect("the binary runs");
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    let answer: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(answer["status"], "proposed", "{answer}");
    assert_eq!(place.acts(), ["registered", "proposed"]);
}

const PROPOSAL: [&str; 6] = [
    "propose",
    "invariant",
    "money-has-currency",
    "Every amount names its currency.",
    "--provenance",
    "run-1",
];
