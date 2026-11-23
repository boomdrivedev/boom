//! `boom hook`: what boom gives the session of an agent, and when
//! (session-context). At session start, the rules in full and the other
//! claims by name, with a menu. Before the first edit of a file in a
//! context, a stop with the rules of the context and their bodies. One
//! line for the person. The record of a session, outside each repository
//! and outside the store.

mod support;

use std::path::{Path, PathBuf};

use serde_json::{json, Value};
use support::{path_arg, Place};

const SLUICE: &str = "https://github.com/boomdrivedev/sluice";
const HOST: &str = "sluice-host";
const HOST_GLOB: &str = "sluice:crates/sluice/src/host_process/**";
const READER: &str = "sluice-reader";
const SITE: &str = "sluice-site";
const FORGE: &str = "crates/sluice/src/host_process/forge.rs";
const RUNS: &str = "crates/sluice/src/host_process/runs.rs";
const ONE_READ: &str = "one-forge-read-per-repository-per-interval";
const ONE_READ_WORDS: &str =
    "A resident reads the forge one time at most in each interval for each repository.";
const ONE_READ_BODY: &str = "A resident reads the forge one time at most in each interval for each repository.\n\nBreaks the rule:\n- a forge read for each client request\n\nDoes not break it:\n- a test with a paused clock that counts the reads of an idle repository\n";
const QUEUE: &str = "the-queue-lands-one-at-a-time";
const QUEUE_WORDS: &str = "The queue lands one branch at a time.";
const OWNER: &str = "A person who grants roles and registers keys in a store.";

/// The version of an answer.
fn version(answer: &Value) -> String {
    answer["version"].as_str().unwrap().to_owned()
}

/// Proposes with `args` and accepts the version with `key`, giving it.
fn accepted(place: &Place, key: &str, args: &[&str]) -> String {
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
fn sluice(place: &Place) -> (String, PathBuf) {
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

/// `boom hook <event>` on the place's store, with `event` on standard
/// input and the place's environment.
fn hook(place: &Place, event: &str, given: &Value) -> boom::cli::Outcome {
    hook_with(place, &place.env, event, given)
}

/// `boom hook <event>` as [`hook`] runs it, in `env`.
fn hook_with(
    place: &Place,
    env: &boom::cli::Env,
    event: &str,
    given: &Value,
) -> boom::cli::Outcome {
    let store = place.store_arg();
    boom::cli::run_with_input(
        ["hook", event, "--store", &store],
        &place.keys,
        env,
        &mut given.to_string().as_bytes(),
    )
}

/// A `SessionStart` event of the session `session` in `cwd`.
fn started(session: &str, cwd: &Path, source: &str) -> Value {
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
fn edit(session: &str, folder: &Path, file: &str, text: &str) -> Value {
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

/// The answer on standard output, asserting the hook answered with one.
fn answer(outcome: &boom::cli::Outcome) -> Value {
    assert_eq!(outcome.code, 0, "{outcome:?}");
    assert_eq!(outcome.stderr, "", "{outcome:?}");
    assert!(outcome.stdout.ends_with('\n'), "{outcome:?}");
    serde_json::from_str(&outcome.stdout).unwrap()
}

/// Asserts the hook exited 0 with both streams empty.
fn silent(outcome: &boom::cli::Outcome) {
    assert_eq!(outcome.code, 0, "{outcome:?}");
    assert_eq!(
        (outcome.stdout.as_str(), outcome.stderr.as_str()),
        ("", ""),
        "{outcome:?}"
    );
}

/// The reason of a stop.
fn reason(answer: &Value) -> String {
    assert_eq!(answer["hookSpecificOutput"]["hookEventName"], "PreToolUse");
    assert_eq!(answer["hookSpecificOutput"]["permissionDecision"], "deny");
    answer["hookSpecificOutput"]["permissionDecisionReason"]
        .as_str()
        .unwrap()
        .to_owned()
}

/// The folder of the record of the session `session`.
fn record(place: &Place, session: &str) -> PathBuf {
    place.dir.path().join("state/boom/sessions").join(session)
}

/// The number of the log's last entry.
fn head(place: &Place) -> u64 {
    place.log().len() as u64
}

// boom hook answers an event of Claude Code

#[test]
fn json_is_refused() {
    let place = Place::new();
    for event in ["session-start", "pre-tool-use"] {
        let outcome = place.run(&["hook", event, "--json"]);
        assert_eq!(outcome.code, 2, "{outcome:?}");
        assert!(
            outcome
                .stderr
                .contains("a hook always answers in the JSON of Claude Code"),
            "{outcome:?}"
        );
    }
}

#[test]
fn the_store_does_not_change() {
    let place = Place::new();
    let (_, folder) = sluice(&place);
    // Views of an older form: each hook answers from the log.
    place
        .sqlite()
        .execute("UPDATE views_head SET form = 4", [])
        .unwrap();
    let bytes = || {
        let mut wal = place.store.clone().into_os_string();
        wal.push("-wal");
        (
            std::fs::read(&place.store).unwrap(),
            std::fs::read(&wal).unwrap_or_default(),
        )
    };
    let before = bytes();
    let started = answer(&hook(
        &place,
        "session-start",
        &started("s-1", &folder, "startup"),
    ));
    assert!(started["systemMessage"]
        .as_str()
        .unwrap()
        .starts_with("Boom: 4 claims for sluice"));
    assert_eq!(bytes(), before);
    let stopped = answer(&hook(
        &place,
        "pre-tool-use",
        &edit("s-1", &folder, FORGE, "x"),
    ));
    assert!(reason(&stopped).contains(ONE_READ), "{stopped}");
    assert_eq!(bytes(), before);
}

// Session start

#[test]
fn session_start_gives_the_rules_in_full_and_the_other_claims_by_name() {
    let place = Place::new();
    let (_, folder) = sluice(&place);
    let entry = head(&place);
    let answer = answer(&hook(
        &place,
        "session-start",
        &started("s-1", &folder, "startup"),
    ));
    let first = format!("Boom: 4 claims for sluice, at entry {entry}");
    assert_eq!(answer["systemMessage"], first);
    assert_eq!(
        answer["hookSpecificOutput"]["hookEventName"],
        "SessionStart"
    );
    let text = answer["hookSpecificOutput"]["additionalContext"]
        .as_str()
        .unwrap();
    let queue = place.show(QUEUE)["accepted"].as_str().unwrap().to_owned();
    let run_log = place.show("run-log")["accepted"]
        .as_str()
        .unwrap()
        .to_owned();
    let expected = format!(
        "{first}
Rules, 2. Each one holds for each change in sluice:
invariant {QUEUE}@{queue}: {QUEUE_WORDS}
term run-log@{run_log}: The log of a run.
Decisions, 1, by name:
decision one-worker-limit-per-tool
Contexts, 1, by name:
context sluice-host
More:
- boom show <name>: a claim in full. A rule gives what breaks it, what does not, and its source.
- boom query --path sluice:<path>: the claims for a file, with those of its contexts.
- boom search <words>: the claims whose words hold <words>.
"
    );
    assert_eq!(text, expected);
    // No body, no claim bound only to a context, no line for a kind with
    // no claim.
    assert!(!text.contains("## Context"), "{text}");
    assert!(!text.contains(ONE_READ), "{text}");
    assert!(!text.contains("sluices-runs-end"), "{text}");
    assert!(!text.contains("Assumptions"), "{text}");
}

#[test]
fn session_start_of_a_repository_with_no_claim_says_nothing() {
    let place = Place::new();
    let ada = place.made();
    place.repo_added(&ada, "sluice", SLUICE);
    let folder = place.repository("sluice", Some(SLUICE));
    silent(&hook(
        &place,
        "session-start",
        &started("s-1", &folder, "startup"),
    ));
}

#[test]
fn session_start_in_no_registered_repository() {
    let place = Place::new();
    let (_, _) = sluice(&place);
    let shop = place.repository("shop", Some("https://github.com/a/shop"));
    silent(&hook(
        &place,
        "session-start",
        &started("s-1", &shop, "startup"),
    ));
    let outside = place.empty_folder("outside");
    silent(&hook(
        &place,
        "session-start",
        &started("s-1", &outside, "startup"),
    ));
}

#[test]
fn no_store_is_made() {
    let place = Place::new();
    let folder = place.repository("sluice", Some(SLUICE));
    silent(&hook(
        &place,
        "session-start",
        &started("s-1", &folder, "startup"),
    ));
    silent(&hook(
        &place,
        "pre-tool-use",
        &edit("s-1", &folder, FORGE, "x"),
    ));
    assert!(!place.store.exists());
}

#[test]
fn a_launcher_name_not_registered() {
    let place = Place::new();
    let (_, folder) = sluice(&place);
    let mut env = place.env.clone();
    env.boom_repository = Some("nowhere".to_owned());
    let outcome = hook_with(
        &place,
        &env,
        "session-start",
        &started("s-1", &folder, "startup"),
    );
    assert_eq!(outcome.code, 2, "{outcome:?}");
    assert!(outcome.stderr.contains("nowhere"), "{outcome:?}");
    env.boom_repository = Some("Shop".to_owned());
    let outcome = hook_with(
        &place,
        &env,
        "session-start",
        &started("s-1", &folder, "startup"),
    );
    assert_eq!(outcome.code, 2, "{outcome:?}");
    assert!(outcome.stderr.contains("Shop"), "{outcome:?}");
}

// The edit

#[test]
fn the_rules_of_a_context_come_before_the_first_edit_in_it() {
    let place = Place::new();
    let (_, folder) = sluice(&place);
    let entry = head(&place);
    let first = answer(&hook(
        &place,
        "pre-tool-use",
        &edit("s-1", &folder, FORGE, "x"),
    ));
    assert_eq!(
        first["systemMessage"],
        "Boom: 1 rule and 1 decision for sluice-host"
    );
    let host = place.show(HOST)["accepted"].as_str().unwrap().to_owned();
    let rule = place.show(ONE_READ)["accepted"]
        .as_str()
        .unwrap()
        .to_owned();
    let expected = format!(
        "Boom: 1 rule and 1 decision for sluice-host, at entry {entry}
This is the first edit in this session of a file in the context sluice-host. Boom stopped it so that you read the rules of the context first. Make the edit again, so that it holds these rules.
context sluice-host@{host}: The host process of sluice: its forge reads, its runs and its workers.
  paths: {HOST_GLOB}
Rules, 1:
invariant {ONE_READ}@{rule}: {ONE_READ_WORDS}
  {ONE_READ_WORDS}

  Breaks the rule:
  - a forge read for each client request

  Does not break it:
  - a test with a paused clock that counts the reads of an idle repository
Decisions, 1, by name:
decision sluices-runs-end-with-its-host-process
More: boom show <name> gives a claim in full."
    );
    assert_eq!(reason(&first), expected);
    // The record holds the context, and the worktree holds no new file.
    assert!(record(&place, "s-1").join(HOST).is_file());
    let mut files: Vec<String> = std::fs::read_dir(&folder)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect();
    files.sort();
    assert_eq!(files, [".git", "crates"]);
    // The edit made again, and another file of the same context.
    silent(&hook(
        &place,
        "pre-tool-use",
        &edit("s-1", &folder, FORGE, "x"),
    ));
    silent(&hook(
        &place,
        "pre-tool-use",
        &edit("s-1", &folder, RUNS, "x"),
    ));
    // Another session gets the rules again.
    let other = answer(&hook(
        &place,
        "pre-tool-use",
        &edit("s-2", &folder, RUNS, "x"),
    ));
    assert!(reason(&other).contains(ONE_READ));
}

#[test]
fn a_file_in_no_context() {
    let place = Place::new();
    let (_, folder) = sluice(&place);
    silent(&hook(
        &place,
        "pre-tool-use",
        &edit("s-1", &folder, "README.md", "x"),
    ));
    assert!(!record(&place, "s-1").exists());
}

#[test]
fn a_context_with_no_claim() {
    let place = Place::new();
    let (ada, folder) = sluice(&place);
    accepted(
        &place,
        &ada,
        &[
            "propose",
            "context",
            SITE,
            "The site of sluice.",
            "--path",
            "sluice:site/**",
        ],
    );
    silent(&hook(
        &place,
        "pre-tool-use",
        &edit("s-1", &folder, "site/index.md", "x"),
    ));
    assert!(!record(&place, "s-1").join(SITE).exists());
}

#[test]
fn a_file_in_two_contexts() {
    let place = Place::new();
    let (ada, folder) = sluice(&place);
    accepted(
        &place,
        &ada,
        &[
            "propose",
            "context",
            READER,
            "The reader of the forge.",
            "--path",
            "sluice:crates/sluice/src/host_process/forge.rs",
        ],
    );
    accepted(
        &place,
        &ada,
        &[
            "propose",
            "invariant",
            "one-read-in-flight",
            "Many triggers of one read make one read in flight.",
            "--context",
            READER,
        ],
    );
    // The record holds sluice-host: the reason gives sluice-reader only.
    answer(&hook(
        &place,
        "pre-tool-use",
        &edit("s-1", &folder, RUNS, "x"),
    ));
    let both = answer(&hook(
        &place,
        "pre-tool-use",
        &edit("s-1", &folder, FORGE, "x"),
    ));
    assert_eq!(both["systemMessage"], "Boom: 1 rule for sluice-reader");
    let text = reason(&both);
    assert!(text.contains("one-read-in-flight"), "{text}");
    assert!(!text.contains(ONE_READ), "{text}");
    // A new session gets both contexts in one stop.
    let new = answer(&hook(
        &place,
        "pre-tool-use",
        &edit("s-2", &folder, FORGE, "x"),
    ));
    assert_eq!(
        new["systemMessage"],
        "Boom: 2 rules and 1 decision for sluice-host and sluice-reader"
    );
    let text = reason(&new);
    assert!(
        text.contains("a file in the contexts sluice-host and sluice-reader"),
        "{text}"
    );
    assert!(text.contains(ONE_READ) && text.contains("one-read-in-flight"));
}

#[test]
fn another_tool() {
    let place = Place::new();
    let (_, folder) = sluice(&place);
    let bash = json!({
        "session_id": "s-1",
        "cwd": path_arg(&folder),
        "hook_event_name": "PreToolUse",
        "tool_name": "Bash",
        "tool_input": {"command": "echo drive log > crates/sluice/src/host_process/forge.rs"},
    });
    silent(&hook(&place, "pre-tool-use", &bash));
}

// The record of a session

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

#[test]
fn a_session_id_that_is_not_a_name() {
    let place = Place::new();
    let (_, folder) = sluice(&place);
    for session in ["../x", "", "a b", "a/b"] {
        silent(&hook(
            &place,
            "pre-tool-use",
            &edit(session, &folder, FORGE, "x"),
        ));
    }
    assert!(!place.dir.path().join("state/boom/x").exists());
    assert!(!place.dir.path().join("state/boom/sessions").exists());
    // The terms are still checked.
    let stopped = answer(&hook(
        &place,
        "pre-tool-use",
        &edit("../x", &folder, FORGE, "the drive log"),
    ));
    assert!(reason(&stopped).contains("\"drive log\""));
    assert!(!reason(&stopped).contains(ONE_READ));
}

#[test]
fn a_cleared_or_compacted_session_gets_the_rules_again() {
    let place = Place::new();
    let (_, folder) = sluice(&place);
    answer(&hook(
        &place,
        "pre-tool-use",
        &edit("s-1", &folder, FORGE, "x"),
    ));
    // Resumed: the transcript still holds the rules.
    answer(&hook(
        &place,
        "session-start",
        &started("s-1", &folder, "resume"),
    ));
    silent(&hook(
        &place,
        "pre-tool-use",
        &edit("s-1", &folder, FORGE, "x"),
    ));
    for source in ["compact", "clear"] {
        answer(&hook(
            &place,
            "session-start",
            &started("s-1", &folder, source),
        ));
        assert!(!record(&place, "s-1").exists(), "{source}");
        let again = answer(&hook(
            &place,
            "pre-tool-use",
            &edit("s-1", &folder, FORGE, "x"),
        ));
        assert!(reason(&again).contains(ONE_READ), "{source}");
    }
}

#[test]
fn a_record_that_no_hook_changed_in_30_days_is_deleted() {
    let mut place = Place::new();
    let (_, folder) = sluice(&place);
    answer(&hook(
        &place,
        "pre-tool-use",
        &edit("old", &folder, FORGE, "x"),
    ));
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let day = 24 * 60 * 60;
    place.at(now + 29 * day);
    answer(&hook(
        &place,
        "session-start",
        &started("new", &folder, "startup"),
    ));
    assert!(record(&place, "old").exists());
    place.at(now + 31 * day);
    answer(&hook(
        &place,
        "session-start",
        &started("new", &folder, "startup"),
    ));
    assert!(!record(&place, "old").exists());
}

// Old words and rules in one answer

#[test]
fn an_old_word_only() {
    let place = Place::new();
    let (_, folder) = sluice(&place);
    let stopped = answer(&hook(
        &place,
        "pre-tool-use",
        &edit("s-1", &folder, "README.md", "Read the drive log."),
    ));
    assert_eq!(stopped["systemMessage"], "Boom: 1 old word in README.md");
    let file = path_arg(&folder.join("README.md"));
    assert_eq!(
        reason(&stopped),
        format!(
            "Boom: 1 old word of an accepted term: write the term in place of each.\n\
             {file}, line 1 of the new text: \"drive log\" is replaced by run log in sluice: The log of a run."
        )
    );
}

#[test]
fn an_old_word_in_a_new_context() {
    let place = Place::new();
    let (_, folder) = sluice(&place);
    let stopped = answer(&hook(
        &place,
        "pre-tool-use",
        &edit("s-1", &folder, FORGE, "Read the drive log."),
    ));
    assert_eq!(
        stopped["systemMessage"],
        format!("Boom: 1 old word in {FORGE}, and 1 rule and 1 decision for sluice-host")
    );
    let text = reason(&stopped);
    let old = text.find("\"drive log\"").unwrap();
    let rules = text.find(ONE_READ).unwrap();
    assert!(old < rules, "{text}");
    assert!(record(&place, "s-1").join(HOST).is_file());
    // The edit made again with the term: the hook is silent.
    silent(&hook(
        &place,
        "pre-tool-use",
        &edit("s-1", &folder, FORGE, "Read the run log."),
    ));
}

// Silent where nothing applies

#[test]
fn delivery_off() {
    let place = Place::new();
    let (ada, folder) = sluice(&place);
    let off = place.run(&["repo", "set", "sluice", "--session", "off", "--key", &ada]);
    assert_eq!(off.code, 0, "{off:?}");
    silent(&hook(
        &place,
        "session-start",
        &started("s-1", &folder, "startup"),
    ));
    silent(&hook(
        &place,
        "pre-tool-use",
        &edit("s-1", &folder, FORGE, "x"),
    ));
    assert!(!record(&place, "s-1").exists());
    // The terms are still checked.
    let stopped = answer(&hook(
        &place,
        "pre-tool-use",
        &edit("s-1", &folder, FORGE, "the drive log"),
    ));
    assert_eq!(
        stopped["systemMessage"],
        format!("Boom: 1 old word in {FORGE}")
    );
}

#[test]
fn a_file_in_no_registered_repository() {
    let place = Place::new();
    sluice(&place);
    let shop = place.repository("shop", Some("https://github.com/a/shop"));
    silent(&hook(
        &place,
        "pre-tool-use",
        &edit("s-1", &shop, "a.md", "the drive log"),
    ));
}

// A hook names what it cannot read

#[test]
fn a_store_that_cannot_be_read() {
    let place = Place::new();
    let (_, folder) = sluice(&place);
    let conn = place.sqlite();
    conn.execute("UPDATE views_head SET form = 4", []).unwrap();
    let seq: i64 = conn
        .query_row(
            "SELECT seq FROM log WHERE signed LIKE '%one branch%'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    conn.execute(
        "UPDATE log SET signed = replace(signed, 'one branch', 'two branches') WHERE seq = ?1",
        [seq],
    )
    .unwrap();
    let outcome = hook(&place, "pre-tool-use", &edit("s-1", &folder, FORGE, "x"));
    assert_eq!(outcome.code, 3, "{outcome:?}");
    assert!(
        outcome
            .stderr
            .contains(&format!("is altered: entry {seq} ")),
        "{outcome:?}"
    );
    assert_eq!(outcome.stdout, "");
}

#[test]
fn an_event_that_is_not_json() {
    let place = Place::new();
    let store = place.store_arg();
    let outcome = boom::cli::run_with_input(
        ["hook", "pre-tool-use", "--store", &store],
        &place.keys,
        &place.env,
        &mut "not json".as_bytes(),
    );
    assert_eq!(outcome.code, 2, "{outcome:?}");
    assert!(outcome.stderr.contains("not JSON"), "{outcome:?}");
}

#[test]
fn an_owner_term_is_named_with_its_meaning() {
    // The term of boom's own store, as an agent of boom meets it.
    let place = Place::new();
    let ada = place.made();
    place.repo_added(&ada, "boom", "https://github.com/boomdrivedev/boom");
    let folder = place.repository("boom", Some("https://github.com/boomdrivedev/boom"));
    accepted(
        &place,
        &ada,
        &[
            "propose",
            "term",
            "owner",
            OWNER,
            "--repository",
            "boom",
            "--replaces",
            "keeper", // boom: allow keeper
        ],
    );
    let stopped = answer(&hook(
        &place,
        "pre-tool-use",
        &edit("s-1", &folder, "notes.md", "The keeper signs."), // boom: allow keeper
    ));
    let text = reason(&stopped);
    assert!(text.contains("\"keeper\""), "{text}"); // boom: allow keeper
    assert!(text.contains(&format!("owner in boom: {OWNER}")), "{text}");
}

// The plugin's hooks, as Claude Code runs them, with the boom the tree
// builds (claude-code-plugin).

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

#[test]
fn session_start_gives_no_narrative() {
    let place = Place::new();
    let ada = place.made();
    let shop = "https://github.com/example/shop";
    place.repo_added(&ada, "shop", shop);
    let folder = place.repository("shop", Some(shop));
    accepted(
        &place,
        &ada,
        &[
            "propose",
            "invariant",
            "refund-within-payment",
            "A refund never exceeds the payment.",
            "--repository",
            "shop",
        ],
    );
    let narrative = version(&place.narrative(
        "shop-end-to-end",
        "The shop, end to end",
        support::NARRATIVE,
        &["--repository", "shop"],
    ));
    place.accepted("shop-end-to-end", &narrative, &ada);
    let target = format!("shop-end-to-end@{narrative}");
    place.noted(&target, 2, "Say what a claim is.");
    place.noted(&target, 3, "Say who reads the status.");
    let answer = answer(&hook(
        &place,
        "session-start",
        &started("s-1", &folder, "startup"),
    ));
    let text = answer["hookSpecificOutput"]["additionalContext"]
        .as_str()
        .unwrap();
    assert!(
        text.contains("A refund never exceeds the payment."),
        "{text}"
    );
    assert!(text.starts_with("Boom: 1 claim for shop"), "{text}");
    for absent in ["shop-end-to-end", "narrative", "note", "Say what"] {
        assert!(!text.contains(absent), "{absent}: {text}");
    }
}

#[test]
fn session_start_gives_no_outline() {
    let place = Place::new();
    let ada = place.made();
    let shop = "https://github.com/example/shop";
    place.repo_added(&ada, "shop", shop);
    let folder = place.repository("shop", Some(shop));
    accepted(
        &place,
        &ada,
        &[
            "propose",
            "invariant",
            "refund-within-payment",
            "A refund never exceeds the payment.",
            "--system",
        ],
    );
    let narrative = version(&place.narrative(
        "shop-end-to-end",
        "The shop, end to end",
        support::NARRATIVE,
        &["--repository", "shop"],
    ));
    place.accepted("shop-end-to-end", &narrative, &ada);
    accepted(
        &place,
        &ada,
        &[
            "propose",
            "outline",
            "shop-design-book",
            "The design book of the shop",
            "--view",
            "book",
            "--narrative",
            "shop-end-to-end",
        ],
    );
    let answer = answer(&hook(
        &place,
        "session-start",
        &started("s-1", &folder, "startup"),
    ));
    let text = answer["hookSpecificOutput"]["additionalContext"]
        .as_str()
        .unwrap();
    assert!(
        text.contains("A refund never exceeds the payment."),
        "{text}"
    );
    for absent in ["shop-design-book", "outline", "The design book"] {
        assert!(!text.contains(absent), "{absent}: {text}");
    }
}
