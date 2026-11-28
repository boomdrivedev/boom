//! The old words of the accepted terms through the tools that a change
//! meets them in: the plugin's hook as Claude Code runs it, by `sh -c`,
//! and the gate's task, check-terms.sh, on a git repository, each with
//! the boom the tree builds (term-check). The other tests of the terms
//! run in the crate, in src/tests/terms.rs.

use super::support;

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use serde_json::{json, Value};
use support::{path_arg, Place};

#[test]
fn an_agent_writes_the_old_word_and_the_hook_names_owner() {
    let _slot = super::slot();
    let (matcher, command) = hook_command();
    for tool in ["Write", "Edit", "MultiEdit"] {
        assert!(matcher.split('|').any(|name| name == tool), "{matcher}");
    }
    assert!(!matcher.split('|').any(|name| name == "Bash"), "{matcher}");
    let place = Place::new();
    let (ada, _) = owner_accepted(&place);
    let folder = boom_folder(&place, &ada);
    // The store at the default path of the environment the hook runs in.
    let data = place.dir.path().join("data/boom");
    std::fs::create_dir_all(&data).unwrap();
    std::fs::copy(&place.store, data.join("boom.sqlite")).unwrap();
    let bin = place.dir.path().join("bin");
    std::fs::create_dir_all(&bin).unwrap();
    std::os::unix::fs::symlink(env!("CARGO_BIN_EXE_boom"), bin.join("boom")).unwrap();
    let run = |content: &str| {
        let event = json!({
            "session_id": "s-1",
            "hook_event_name": "PreToolUse",
            "cwd": path_arg(&folder),
            "tool_name": "Write",
            "tool_input": {"file_path": path_arg(&folder.join("notes.md")), "content": content},
        });
        let mut child = Command::new("sh")
            .args(["-c", &command])
            .current_dir(&folder)
            .env("PATH", format!("{}:/usr/bin:/bin", bin.display()))
            .env("XDG_DATA_HOME", place.dir.path().join("data"))
            .env("HOME", place.dir.path().join("home"))
            .env_remove("BOOM_REPOSITORY")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        use std::io::Write;
        child
            .stdin
            .take()
            .unwrap()
            .write_all(event.to_string().as_bytes())
            .unwrap();
        child.wait_with_output().unwrap()
    };
    let stopped = run("The keeper signs.\n"); // boom: allow keeper
    assert_eq!(stopped.status.code(), Some(0), "{stopped:?}");
    let answer: Value = serde_json::from_slice(&stopped.stdout).unwrap();
    assert_eq!(answer["hookSpecificOutput"]["permissionDecision"], "deny");
    let reason = answer["hookSpecificOutput"]["permissionDecisionReason"]
        .as_str()
        .unwrap();
    assert!(reason.contains("\"keeper\""), "{reason}"); // boom: allow keeper
    assert!(reason.contains("owner in boom"), "{reason}");
    let passed = run("The owner signs.\n");
    assert_eq!(passed.status.code(), Some(0));
    assert!(passed.stdout.is_empty() && passed.stderr.is_empty());
}

#[test]
fn a_change_that_adds_an_old_word() {
    let _slot = super::slot();
    let place = Place::new();
    owner_accepted(&place);
    let folder = change(&place, &[("README.md", "# boom\n\nThe keeper signs.\n")]); // boom: allow keeper
    let ran = gate(&place, &folder, &["--repository", "boom"]);
    let stderr = text(&ran.stderr);
    assert!(!ran.status.success(), "{stderr}");
    assert!(stderr.contains("README.md:3"), "{stderr}");
    assert!(stderr.contains("\"keeper\""), "{stderr}"); // boom: allow keeper
    assert!(stderr.contains("owner in boom"), "{stderr}");
}

#[test]
fn an_old_word_that_the_change_does_not_touch() {
    let _slot = super::slot();
    let place = Place::new();
    owner_accepted(&place);
    let folder = change(&place, &[("notes.md", "The owner signs.\n")]);
    let ran = gate(&place, &folder, &["--repository", "boom"]);
    assert!(ran.status.success(), "{}", text(&ran.stderr));
    assert!(
        text(&ran.stdout).contains("1 files of the change hold no old word"),
        "{}",
        text(&ran.stdout)
    );
}

#[test]
fn an_archived_change_and_a_marker() {
    let _slot = super::slot();
    let place = Place::new();
    owner_accepted(&place);
    let folder = change(
        &place,
        &[
            (
                "openspec/changes/archive/2026-11-21-terms/proposal.md",
                "The keeper signs.\n", // boom: allow keeper
            ),
            (
                "docs/history.md",
                "The keeper was the old word. boom: allow keeper\n",
            ),
        ],
    );
    let ran = gate(&place, &folder, &["--repository", "boom"]);
    assert!(ran.status.success(), "{}", text(&ran.stderr));
}

#[test]
fn a_change_that_deletes_a_file_that_holds_an_old_word() {
    let _slot = super::slot();
    let place = Place::new();
    owner_accepted(&place);
    let folder = change(&place, &[]);
    std::fs::remove_file(folder.join("docs/old.md")).unwrap();
    let commit = Command::new("git")
        .args(["commit", "-q", "-a", "-m", "delete"])
        .current_dir(&folder)
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_AUTHOR_NAME", "a")
        .env("GIT_AUTHOR_EMAIL", "a@a")
        .env("GIT_COMMITTER_NAME", "a")
        .env("GIT_COMMITTER_EMAIL", "a@a")
        .status()
        .unwrap();
    assert!(commit.success());
    let ran = gate(&place, &folder, &["--repository", "boom"]);
    assert!(ran.status.success(), "{}", text(&ran.stderr));
    assert!(text(&ran.stdout).contains("adds or modifies no file"));
}

#[test]
fn the_gate_with_no_store() {
    let _slot = super::slot();
    let place = Place::new();
    let folder = change(&place, &[("README.md", "The keeper signs.\n")]); // boom: allow keeper
    let ran = gate(&place, &folder, &["--repository", "boom"]);
    assert!(ran.status.success(), "{}", text(&ran.stderr));
    assert!(text(&ran.stdout).contains("no store"));
}

#[test]
fn the_task_names_the_entry_it_read() {
    let _slot = super::slot();
    let place = Place::new();
    owner_accepted(&place);
    let folder = change(&place, &[("notes.md", "The owner signs.\n")]);
    let ran = gate(&place, &folder, &["--repository", "boom"]);
    assert!(ran.status.success(), "{}", text(&ran.stderr));
    assert!(
        text(&ran.stdout).ends_with("at entry 3 of the store's log\n"),
        "{}",
        text(&ran.stdout)
    );
}

/// A git repository of boom in the place: main holds `docs/old.md` with
/// `keeper` and an archived change with `keeper`; the branch `change` // boom: allow keeper
/// adds `files`. Gives its folder.
fn change(place: &Place, files: &[(&str, &str)]) -> PathBuf {
    let folder = place.dir.path().join("tree");
    let git = |args: &[&str]| {
        let status = Command::new("git")
            .args(args)
            .current_dir(&folder)
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_AUTHOR_NAME", "a")
            .env("GIT_AUTHOR_EMAIL", "a@a")
            .env("GIT_COMMITTER_NAME", "a")
            .env("GIT_COMMITTER_EMAIL", "a@a")
            .stdout(Stdio::null())
            .status()
            .unwrap();
        assert!(status.success(), "git {args:?}");
    };
    let write = |path: &str, text: &str| {
        let path = folder.join(path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    };
    std::fs::create_dir_all(&folder).unwrap();
    git(&["init", "-q", "-b", "main"]);
    write("docs/old.md", "The keeper signs.\n"); // boom: allow keeper
    write("README.md", "# boom\n");
    git(&["add", "."]);
    git(&["commit", "-q", "-m", "main"]);
    git(&["checkout", "-q", "-b", "change"]);
    for (path, text) in files {
        write(path, text);
    }
    git(&["add", "."]);
    git(&["commit", "-q", "-m", "change", "--allow-empty"]);
    folder
}

/// check-terms.sh with `args`, in `folder`, with the boom the tree
/// builds and the place's store at its default path.
fn gate(place: &Place, folder: &Path, args: &[&str]) -> std::process::Output {
    let data = place.dir.path().join("data/boom");
    std::fs::create_dir_all(&data).unwrap();
    if place.store.exists() {
        std::fs::copy(&place.store, data.join("boom.sqlite")).unwrap();
    }
    Command::new("sh")
        .arg(root().join("check-terms.sh"))
        .args(args)
        .current_dir(folder)
        .env("BOOM", env!("CARGO_BIN_EXE_boom"))
        .env("XDG_DATA_HOME", place.dir.path().join("data"))
        .env("HOME", place.dir.path().join("home"))
        .env("TMPDIR", place.dir.path())
        .env_remove("BOOM_REPOSITORY")
        .env_remove("SLUICE_TMP")
        .output()
        .unwrap()
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

/// The root of the tree.
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// The command of the plugin's one `PreToolUse` hook, as Claude Code
/// reads it from the plugin's hooks.json.
fn hook_command() -> (String, String) {
    let hooks: Value = serde_json::from_slice(
        &std::fs::read(root().join("plugins/claude-code/hooks/hooks.json")).unwrap(),
    )
    .unwrap();
    let pre = hooks["hooks"]["PreToolUse"].as_array().unwrap();
    assert_eq!(pre.len(), 1, "{hooks}");
    let command = pre[0]["hooks"].as_array().unwrap();
    assert_eq!(command.len(), 1, "{hooks}");
    assert_eq!(command[0]["type"], "command");
    (
        pre[0]["matcher"].as_str().unwrap().to_owned(),
        command[0]["command"].as_str().unwrap().to_owned(),
    )
}

const OWNER: &str = "A person who grants roles and registers keys in a store.";

/// An agent's proposal of a term with `extra`, giving its answer.
fn term(place: &Place, word: &str, meaning: &str, extra: &[&str]) -> boom::cli::Outcome {
    let mut args = vec!["propose", "term", word, meaning, "--provenance", "run-1"];
    args.extend_from_slice(extra);
    place.run(&args)
}

/// The version of a term proposed with `extra`, asserting it answers.
fn proposed(place: &Place, word: &str, meaning: &str, extra: &[&str]) -> String {
    let outcome = term(place, word, meaning, &[extra, &["--json"]].concat());
    assert_eq!(outcome.code, 0, "{outcome:?}");
    let answer: Value = serde_json::from_str(&outcome.stdout).unwrap();
    answer["version"].as_str().unwrap().to_owned()
}

/// A store whose owner ada accepted the term `owner` of `boom`, which
/// replaces `keeper` and `warden`; gives ada's key and the version. // boom: allow keeper
fn owner_accepted(place: &Place) -> (String, String) {
    let ada = place.made();
    let version = proposed(
        place,
        "owner",
        OWNER,
        &[
            "--repository",
            "boom",
            "--replaces",
            "keeper", // boom: allow keeper
            "--replaces",
            "Warden",
        ],
    );
    place.accepted("owner", &version, &ada);
    (ada, version)
}

/// A folder of the repository `boom`, registered with its remote.
fn boom_folder(place: &Place, ada: &str) -> PathBuf {
    place.repo_added(ada, "boom", "git@github.com:boomdrivedev/boom.git");
    place.repository("boom", Some("https://github.com/boomdrivedev/boom.git"))
}
