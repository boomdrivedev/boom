//! `boom mcp` through the binary: it answers each request while its
//! input stays open (agent-tools; record 0011). The other tests of the
//! server run in the crate, in src/tests/mcp.rs.

#[path = "../src/tests/support.rs"]
mod support;

use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};

use serde_json::{json, Value};
use support::{initialize, request, Place};

/// `boom mcp` answers each request while its input stays open: the test
/// waits on each answer, and on no clock
/// (a-gate-test-asserts-no-real-time-bound).
#[test]
fn one_answer_before_the_next_request() {
    let place = Place::new();
    let mut child = Command::new(env!("CARGO_BIN_EXE_boom"))
        .args(["mcp", "--store", &place.store_arg()])
        .env("XDG_DATA_HOME", place.dir.path().join("data"))
        .env("HOME", place.dir.path().join("home"))
        .current_dir(place.dir.path())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("the binary runs");
    // The child ends when the test ends, passed or failed.
    struct Ends(Child);
    impl Drop for Ends {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
    let mut stdin = child.stdin.take().unwrap();
    let mut lines = BufReader::new(child.stdout.take().unwrap()).lines();
    let mut child = Ends(child);
    let mut answer = |line: String| -> Value {
        writeln!(stdin, "{line}").unwrap();
        stdin.flush().unwrap();
        let read = lines
            .next()
            .expect("boom mcp answers before its input closes")
            .unwrap();
        serde_json::from_str(&read).unwrap()
    };
    let first = answer(initialize(1).to_string());
    assert_eq!(first["id"], 1, "{first}");
    assert_eq!(first["result"]["serverInfo"]["name"], "boom", "{first}");
    let second = answer(request(2, "tools/list", json!({})).to_string());
    assert_eq!(second["id"], 2, "{second}");
    assert!(second["result"]["tools"].is_array(), "{second}");
    drop(stdin);
    assert!(lines.next().is_none(), "boom mcp writes nothing else");
    assert_eq!(child.0.wait().unwrap().code(), Some(0));
}
