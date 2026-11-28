//! `boom hook` through the built binary: two hooks that race for one
//! session's record as two processes, and the plugin's hooks as Claude
//! Code runs them, by `sh -c`, with the boom the tree builds first on
//! PATH (session-context, claude-code-plugin). The other tests of the
//! hooks run in the crate, in src/tests/hooks.rs.

#[path = "../src/tests/support.rs"]
mod support;

#[path = "../src/tests/hooks_store.rs"]
mod hooks_store;

use std::path::Path;

use hooks_store::{edit, head, reason, sluice, started, FORGE, ONE_READ, ONE_READ_BODY, RUNS};
use serde_json::Value;
use support::Place;

#[test]
fn two_edits_in_parallel() {
    use std::io::Write;
    use std::process::{Command, Stdio};
    let place = Place::new();
    let (_, folder) = sluice(&place);
    // Each round is a new session, so its two hooks race for one new
    // context; the built boom runs as two processes at one time.
    for round in 0..8 {
        let session = format!("p-{round}");
        let children: Vec<_> = [FORGE, RUNS]
            .into_iter()
            .map(|file| {
                let mut child = Command::new(env!("CARGO_BIN_EXE_boom"))
                    .args(["hook", "pre-tool-use", "--store", &place.store_arg()])
                    .env("XDG_STATE_HOME", place.dir.path().join("state"))
                    .env("HOME", place.dir.path().join("home"))
                    .env_remove("BOOM_REPOSITORY")
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()
                    .unwrap();
                let event = edit(&session, &folder, file, "x").to_string();
                child
                    .stdin
                    .take()
                    .unwrap()
                    .write_all(event.as_bytes())
                    .unwrap();
                child
            })
            .collect();
        let outputs: Vec<_> = children
            .into_iter()
            .map(|child| child.wait_with_output().unwrap())
            .collect();
        for output in &outputs {
            assert_eq!(output.status.code(), Some(0), "{output:?}");
            assert!(output.stderr.is_empty(), "{output:?}");
        }
        let stops: Vec<Value> = outputs
            .iter()
            .filter(|output| !output.stdout.is_empty())
            .map(|output| serde_json::from_slice(&output.stdout).unwrap())
            .collect();
        assert_eq!(stops.len(), 1, "{outputs:?}");
        assert!(reason(&stops[0]).contains(ONE_READ));
    }
}

/// The command of the plugin's one command hook of `event`, read from its
/// hooks.json.
fn plugin_command(event: &str) -> String {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let hooks: Value = serde_json::from_slice(
        &std::fs::read(root.join("plugins/claude-code/hooks/hooks.json")).unwrap(),
    )
    .unwrap();
    let groups = hooks["hooks"][event].as_array().unwrap();
    assert_eq!(groups.len(), 1, "{hooks}");
    let commands = groups[0]["hooks"].as_array().unwrap();
    assert_eq!(commands.len(), 1, "{hooks}");
    assert_eq!(commands[0]["type"], "command");
    commands[0]["command"].as_str().unwrap().to_owned()
}

/// Runs the plugin's hook of `event` by `sh -c` in `folder`, with the
/// boom the tree builds first on PATH, the place's store at the default
/// path of its environment, and `given` on standard input.
fn plugin(place: &Place, event: &str, folder: &Path, given: &Value) -> std::process::Output {
    use std::io::Write;
    use std::process::{Command, Stdio};
    let data = place.dir.path().join("data/boom");
    std::fs::create_dir_all(&data).unwrap();
    std::fs::copy(&place.store, data.join("boom.sqlite")).unwrap();
    let bin = place.dir.path().join("bin");
    if !bin.exists() {
        std::fs::create_dir_all(&bin).unwrap();
        std::os::unix::fs::symlink(env!("CARGO_BIN_EXE_boom"), bin.join("boom")).unwrap();
    }
    let mut child = Command::new("sh")
        .args(["-c", &plugin_command(event)])
        .current_dir(folder)
        .env("PATH", format!("{}:/usr/bin:/bin", bin.display()))
        .env("XDG_DATA_HOME", place.dir.path().join("data"))
        .env("XDG_STATE_HOME", place.dir.path().join("state"))
        .env("HOME", place.dir.path().join("home"))
        .env_remove("BOOM_REPOSITORY")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(given.to_string().as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}

/// The JSON that a run of the plugin's hook printed.
fn printed(output: &std::process::Output) -> Value {
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    serde_json::from_slice(&output.stdout).unwrap()
}

#[test]
fn the_plugin_starts_a_session_in_a_registered_repository() {
    let place = Place::new();
    let (ada, folder) = sluice(&place);
    let entry = head(&place);
    let answer = printed(&plugin(
        &place,
        "SessionStart",
        &folder,
        &started("s-1", &folder, "startup"),
    ));
    let line = format!("Boom: 4 claims for sluice, at entry {entry}");
    assert_eq!(answer["systemMessage"], line);
    assert!(answer["hookSpecificOutput"]["additionalContext"]
        .as_str()
        .unwrap()
        .starts_with(&format!("{line}\n")));
    // Delivery off.
    let off = place.run(&["repo", "set", "sluice", "--session", "off", "--key", &ada]);
    assert_eq!(off.code, 0, "{off:?}");
    let silent = plugin(
        &place,
        "SessionStart",
        &folder,
        &started("s-1", &folder, "startup"),
    );
    assert_eq!(silent.status.code(), Some(0), "{silent:?}");
    assert!(
        silent.stdout.is_empty() && silent.stderr.is_empty(),
        "{silent:?}"
    );
}

#[test]
fn the_plugin_stops_the_first_edit_in_a_context() {
    let place = Place::new();
    let (_, folder) = sluice(&place);
    let first = printed(&plugin(
        &place,
        "PreToolUse",
        &folder,
        &edit("s-1", &folder, FORGE, "x"),
    ));
    assert_eq!(
        first["systemMessage"],
        "Boom: 1 rule and 1 decision for sluice-host"
    );
    assert!(reason(&first).contains(ONE_READ_BODY.lines().nth(2).unwrap()));
    // The edit made again.
    let again = plugin(
        &place,
        "PreToolUse",
        &folder,
        &edit("s-1", &folder, FORGE, "x"),
    );
    assert_eq!(again.status.code(), Some(0), "{again:?}");
    assert!(
        again.stdout.is_empty() && again.stderr.is_empty(),
        "{again:?}"
    );
    // An old word stops a write.
    let old = printed(&plugin(
        &place,
        "PreToolUse",
        &folder,
        &edit("s-1", &folder, FORGE, "the drive log"),
    ));
    assert!(reason(&old).contains("\"drive log\""), "{old}");
    assert!(reason(&old).contains("run log in sluice"), "{old}");
}
