//! The store of sluice's claims that the tests of `boom hook` read, in
//! the crate and through the plugin, with the events that they give the
//! hooks. No one test file uses all of it.
#![allow(dead_code)]

use std::path::{Path, PathBuf};

use serde_json::{json, Value};

use super::support::{path_arg, Place};

pub const SLUICE: &str = "https://github.com/boomdrivedev/sluice";

pub const HOST: &str = "sluice-host";

pub const HOST_GLOB: &str = "sluice:crates/sluice/src/host_process/**";

pub const FORGE: &str = "crates/sluice/src/host_process/forge.rs";

pub const RUNS: &str = "crates/sluice/src/host_process/runs.rs";

pub const ONE_READ: &str = "one-forge-read-per-repository-per-interval";

pub const ONE_READ_WORDS: &str =
    "A resident reads the forge one time at most in each interval for each repository.";

pub const ONE_READ_BODY: &str = "A resident reads the forge one time at most in each interval for each repository.\n\nBreaks the rule:\n- a forge read for each client request\n\nDoes not break it:\n- a test with a paused clock that counts the reads of an idle repository\n";

pub const QUEUE: &str = "the-queue-lands-one-at-a-time";

pub const QUEUE_WORDS: &str = "The queue lands one branch at a time.";

/// The version of an answer.
pub fn version(answer: &Value) -> String {
    answer["version"].as_str().unwrap().to_owned()
}

/// Proposes with `args` and accepts the version with `key`, giving it.
pub fn accepted(place: &Place, key: &str, args: &[&str]) -> String {
    let mut all = args.to_vec();
    all.extend(["--provenance", "run-1"]);
    let version = version(&place.json(&all));
    place.accepted(args[2], &version, key);
    version
}

/// A store of ada's that registers `sluice`, with a worktree of it:
/// - the invariant `the-queue-lands-one-at-a-time`, bound to `sluice`;
/// - the term `run-log` of `sluice`, which replaces `drive log`;
/// - the decision `one-worker-limit-per-tool`, bound to `sluice`;
/// - the context `sluice-host`, with one glob;
/// - the invariant `one-forge-read-...`, with a body, bound only to
///   `sluice-host`;
/// - the decision `sluices-runs-end-with-its-host-process`, bound only to
///   `sluice-host`.
///
/// It gives ada's key and the folder of the worktree.
pub fn sluice(place: &Place) -> (String, PathBuf) {
    let ada = place.made();
    place.repo_added(&ada, "sluice", SLUICE);
    let folder = place.repository("sluice", Some(SLUICE));
    std::fs::create_dir_all(folder.join("crates/sluice/src/host_process")).unwrap();
    accepted(
        place,
        &ada,
        &[
            "propose",
            "invariant",
            QUEUE,
            QUEUE_WORDS,
            "--repository",
            "sluice",
        ],
    );
    accepted(
        place,
        &ada,
        &[
            "propose",
            "term",
            "run-log",
            "The log of a run.",
            "--repository",
            "sluice",
            "--replaces",
            "drive log",
        ],
    );
    let body = place.decision_body();
    accepted(
        place,
        &ada,
        &[
            "propose",
            "decision",
            "one-worker-limit-per-tool",
            "Each tool has one limit of workers on a host.",
            "--body",
            &body,
            "--repository",
            "sluice",
        ],
    );
    accepted(
        place,
        &ada,
        &[
            "propose",
            "context",
            HOST,
            "The host process of sluice: its forge reads, its runs and its workers.",
            "--path",
            HOST_GLOB,
        ],
    );
    let rule_body = place.file("rule.md", ONE_READ_BODY);
    accepted(
        place,
        &ada,
        &[
            "propose",
            "invariant",
            ONE_READ,
            ONE_READ_WORDS,
            "--body",
            &rule_body,
            "--context",
            HOST,
        ],
    );
    accepted(
        place,
        &ada,
        &[
            "propose",
            "decision",
            "sluices-runs-end-with-its-host-process",
            "The runs of sluice end with its host process.",
            "--body",
            &body,
            "--context",
            HOST,
        ],
    );
    (ada, folder)
}

/// A `SessionStart` event of the session `session` in `cwd`.
pub fn started(session: &str, cwd: &Path, source: &str) -> Value {
    json!({
        "session_id": session,
        "transcript_path": "/dev/null",
        "cwd": path_arg(cwd),
        "hook_event_name": "SessionStart",
        "source": source,
    })
}

/// An `Edit` event of the session `session` of `file` in `folder`, whose
/// new text is `text`.
pub fn edit(session: &str, folder: &Path, file: &str, text: &str) -> Value {
    json!({
        "session_id": session,
        "transcript_path": "/dev/null",
        "cwd": path_arg(folder),
        "hook_event_name": "PreToolUse",
        "tool_name": "Edit",
        "tool_input": {
            "file_path": path_arg(&folder.join(file)),
            "old_string": "a",
            "new_string": text,
        },
    })
}

/// The reason of a stop.
pub fn reason(answer: &Value) -> String {
    assert_eq!(answer["hookSpecificOutput"]["hookEventName"], "PreToolUse");
    assert_eq!(answer["hookSpecificOutput"]["permissionDecision"], "deny");
    answer["hookSpecificOutput"]["permissionDecisionReason"]
        .as_str()
        .unwrap()
        .to_owned()
}

/// The number of the log's last entry.
pub fn head(place: &Place) -> u64 {
    place.log().len() as u64
}
