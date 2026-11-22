//! `boom hook`: the answers to the events of Claude Code, which give an
//! agent its claims in layers (session-context).
//!
//! At session start, the agent gets the rules in full, the other claims
//! by name, and a menu. Before the first edit of a file in a context, the
//! hook stops the edit and gives the rules of that context with their
//! bodies. The agent then makes the edit again. Each hook that gives
//! claims also gives the person one line.
//!
//! The text of each layer comes from three functions: [`session_layer`],
//! [`edit_layer`] and [`menu`]. Each takes the claims and gives text, so
//! another writer of the same layers calls the same functions.
//!
//! A hook reads the store and writes nothing into it. Boom keeps the
//! contexts that a session got in the record of the session, outside
//! each repository and outside the store.

use std::path::{Path, PathBuf};

use serde_json::{json, Value};

use crate::claim::Kind;
use crate::cli::Env;
use crate::judge::{Claim, State, Version};
use crate::store::{self, Wanted};
use crate::{context, repository, terms, Failure};

/// A claim that stands, with its accepted version.
pub type Given<'a> = (&'a Claim, &'a Version);

/// The days after which boom deletes a record that no hook changed.
pub const RECORD_DAYS: u64 = 30;

/// `n` with the word for one or for many.
fn count(n: usize, one: &str, many: &str) -> String {
    if n == 1 {
        format!("1 {one}")
    } else {
        format!("{n} {many}")
    }
}

/// The rules of `claims`, in their order: each invariant, then each
/// term.
fn rules<'a>(claims: &[Given<'a>]) -> Vec<Given<'a>> {
    [Kind::Invariant, Kind::Term]
        .into_iter()
        .flat_map(|kind| claims.iter().filter(move |(claim, _)| claim.kind == kind))
        .copied()
        .collect()
}

/// The claims of `kind` in `claims`.
fn of_kind<'a>(claims: &[Given<'a>], kind: Kind) -> Vec<Given<'a>> {
    claims
        .iter()
        .filter(|(claim, _)| claim.kind == kind)
        .copied()
        .collect()
}

/// The line of a rule: `<kind> <name>@<version>: <statement>`.
fn rule_line((claim, version): Given<'_>) -> String {
    format!(
        "{} {}@{}: {}\n",
        claim.kind.as_str(),
        claim.name,
        version.proposal.version,
        version.proposal.statement
    )
}

/// The lines of the decisions, the assumptions and, where `contexts` is
/// true, the contexts of `claims`: for each kind with one claim at least,
/// its count, then each claim by name.
fn by_name(claims: &[Given<'_>], contexts: bool) -> String {
    let mut text = String::new();
    let mut kinds = vec![
        (Kind::Decision, "Decisions"),
        (Kind::Assumption, "Assumptions"),
    ];
    if contexts {
        kinds.push((Kind::Context, "Contexts"));
    }
    for (kind, heading) in kinds {
        let named = of_kind(claims, kind);
        if named.is_empty() {
            continue;
        }
        text.push_str(&format!("{heading}, {}, by name:\n", named.len()));
        for (claim, _) in named {
            text.push_str(&format!("{} {}\n", claim.kind.as_str(), claim.name));
        }
    }
    text
}

/// The session layer of `repository` (session-context, "Session start
/// gives the rules in full and the other claims by name"): the first
/// line, each rule on one line with its statement, then the decisions,
/// the assumptions and the contexts by name. It gives no scenario and no
/// body, and no line for a kind with no claim.
pub fn session_layer(repository: &str, entry: u64, claims: &[Given<'_>]) -> String {
    let mut text = format!(
        "Boom: {} for {repository}, at entry {entry}\n",
        count(claims.len(), "claim", "claims")
    );
    let rules = rules(claims);
    if !rules.is_empty() {
        text.push_str(&format!(
            "Rules, {}. Each one holds for each change in {repository}:\n",
            rules.len()
        ));
        for rule in rules {
            text.push_str(&rule_line(rule));
        }
    }
    text.push_str(&by_name(claims, true));
    text
}

/// The menu of `repository`: how an agent gets more than a layer gives.
pub fn menu(repository: &str) -> String {
    format!(
        "More:\n\
         - boom show <name>: a claim in full. A rule gives what breaks it, what does not, and its source.\n\
         - boom query --path {repository}:<path>: the claims for a file, with those of its contexts.\n\
         - boom search <words>: the claims whose words hold <words>.\n"
    )
}

/// The counts of the claims that a context gives at the edit: its rules,
/// then its decisions and its assumptions, where it has them.
pub fn edit_counts(claims: &[Given<'_>]) -> String {
    let mut parts = Vec::new();
    let rules = rules(claims).len();
    if rules > 0 {
        parts.push(count(rules, "rule", "rules"));
    }
    for (kind, one, many) in [
        (Kind::Decision, "decision", "decisions"),
        (Kind::Assumption, "assumption", "assumptions"),
    ] {
        let n = of_kind(claims, kind).len();
        if n > 0 {
            parts.push(count(n, one, many));
        }
    }
    repository::words(&parts)
}

/// The edit layer of `context` (session-context, "The rules of a context
/// come before the first edit in it"): the context with its globs, each
/// rule with its body as `boom show` gives it, then the decisions and the
/// assumptions by name. `body` gives the text of the body of a version.
/// Boom reads no heading in a body, so a body in another form comes
/// whole.
pub fn edit_layer(
    (context, version): Given<'_>,
    claims: &[Given<'_>],
    mut body: impl FnMut(&Version) -> Result<Option<String>, Failure>,
) -> Result<String, Failure> {
    let mut text = format!(
        "context {}@{}: {}\n  paths: {}\n",
        context.name,
        version.proposal.version,
        version.proposal.statement,
        version.proposal.paths.join(", ")
    );
    let rules = rules(claims);
    if !rules.is_empty() {
        text.push_str(&format!("Rules, {}:\n", rules.len()));
        for rule in rules {
            text.push_str(&rule_line(rule));
            if let Some(body) = body(rule.1)? {
                for line in body.trim_end().lines() {
                    if line.trim().is_empty() {
                        text.push('\n');
                    } else {
                        text.push_str(&format!("  {line}\n"));
                    }
                }
            }
        }
    }
    text.push_str(&by_name(claims, false));
    Ok(text)
}

/// The claims whose accepted version binds the context `name`, in the
/// order of [`State::standing_for`].
pub fn bound<'a>(state: &'a State, name: &str) -> Vec<Given<'a>> {
    state
        .standing_for(None, &[])
        .into_iter()
        .filter(|(_, version)| context::names(&version.contexts).contains(&name))
        .collect()
}

/// The accepted version of the context `name`, with its claim.
fn context_of<'a>(state: &'a State, name: &str) -> Option<Given<'a>> {
    state
        .named(name)
        .into_iter()
        .filter(|claim| claim.kind == Kind::Context)
        .find_map(|claim| {
            let version = claim.version(claim.standing.as_deref()?)?;
            Some((claim, version))
        })
}

// The record of a session

/// The folder of the records of the sessions:
/// `$XDG_STATE_HOME/boom/sessions`, else `~/.local/state/boom/sessions`;
/// none where neither is set.
pub fn sessions(env: &Env) -> Option<PathBuf> {
    let state = env
        .xdg_state_home
        .clone()
        .or_else(|| env.home.as_ref().map(|home| home.join(".local/state")))?;
    Some(state.join("boom/sessions"))
}

/// True where `session_id` is a name: one character at least, each a
/// letter, a digit, `-` or `_`.
pub fn session_name(session_id: &str) -> bool {
    !session_id.is_empty()
        && session_id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

/// The record of one session: one empty file for each context that the
/// session got, named by the context.
#[derive(Debug, Clone)]
pub struct Record {
    folder: PathBuf,
}

impl Record {
    /// The record of the session `session_id` in `sessions`; none where
    /// `session_id` is not a name.
    pub fn of(sessions: &Path, session_id: &str) -> Option<Record> {
        session_name(session_id).then(|| Record {
            folder: sessions.join(session_id),
        })
    }

    /// The folder of the record.
    pub fn folder(&self) -> &Path {
        &self.folder
    }

    /// Puts the context `name` into the record: true where this call
    /// made its file, false where the file was there. Only the hook that
    /// makes the file gives the rules of the context.
    pub fn take(&self, name: &str) -> Result<bool, Failure> {
        let cannot = |error: std::io::Error| {
            Failure::not_run(format!(
                "the record of the session {} could not be written: {error}",
                self.folder.display()
            ))
        };
        std::fs::create_dir_all(&self.folder).map_err(cannot)?;
        match std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(self.folder.join(name))
        {
            Ok(_) => Ok(true),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => Ok(false),
            Err(error) => Err(cannot(error)),
        }
    }

    /// True where the record holds the context `name`.
    pub fn holds(&self, name: &str) -> bool {
        self.folder.join(name).is_file()
    }

    /// Deletes the record.
    pub fn clear(&self) -> Result<(), Failure> {
        remove(&self.folder)
    }
}

/// Deletes the folder `folder` and what it holds. A folder that is not
/// there is already deleted.
fn remove(folder: &Path) -> Result<(), Failure> {
    match std::fs::remove_dir_all(folder) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(Failure::not_run(format!(
            "the record {} could not be deleted: {error}",
            folder.display()
        ))),
    }
}

/// Deletes each record in `sessions` that no hook changed in
/// [`RECORD_DAYS`] days before `now`, in Unix seconds.
pub fn prune(sessions: &Path, now: u64) -> Result<(), Failure> {
    let entries = match std::fs::read_dir(sessions) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => {
            return Err(Failure::not_run(format!(
                "the records {} could not be read: {error}",
                sessions.display()
            )))
        }
    };
    let oldest = now.saturating_sub(RECORD_DAYS * 24 * 60 * 60);
    for entry in entries.flatten() {
        let changed = entry
            .metadata()
            .ok()
            .filter(std::fs::Metadata::is_dir)
            .and_then(|metadata| metadata.modified().ok())
            .and_then(|modified| modified.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|elapsed| elapsed.as_secs());
        if changed.is_some_and(|changed| changed < oldest) {
            remove(&entry.path())?;
        }
    }
    Ok(())
}

// The events

/// A text field of the event, where it has one.
fn field<'a>(event: &'a Value, name: &str) -> Option<&'a str> {
    event.get(name).and_then(Value::as_str)
}

/// The record of the session of `event`; none where boom has no folder
/// for records or the `session_id` is not a name.
fn record_of(env: &Env, event: &Value) -> Option<Record> {
    Record::of(&sessions(env)?, field(event, "session_id")?)
}

/// Refuses a name from `BOOM_REPOSITORY` that is not kebab case, before
/// any store is opened, so the person sees what to fix.
fn check_launcher(env: &Env) -> Result<(), Failure> {
    if let Some(launched) = env.boom_repository.as_deref() {
        repository::check_name(launched).map_err(|why| {
            Failure::refused(format!(
                "{} names {launched:?}: {why}",
                repository::LAUNCHER
            ))
        })?;
    }
    Ok(())
}

/// `boom hook session-start` (session-context): the answer to a
/// `SessionStart` event, or none. It deletes the record of the session
/// where `source` is `clear` or `compact`, and each record that no hook
/// changed in 30 days. It gives the session layer and the menu of the
/// repository of the session, found from the `cwd` of the event.
pub fn session_start(
    path: Result<PathBuf, Failure>,
    env: &Env,
    event: &Value,
) -> Result<Option<Value>, Failure> {
    check_launcher(env)?;
    if let Some(sessions) = sessions(env) {
        if matches!(field(event, "source"), Some("clear" | "compact")) {
            if let Some(record) = record_of(env, event) {
                record.clear()?;
            }
        }
        prune(&sessions, env.clock.now())?;
    }
    let path = path?;
    if !path.exists() {
        return Ok(None);
    }
    let cwd = field(event, "cwd")
        .map(PathBuf::from)
        .or_else(|| env.cwd.clone());
    store::read_only(&path, Wanted::Accepted, |_, read| {
        let Some(repository) = repository::find(
            None,
            env.boom_repository.as_deref(),
            cwd.as_deref(),
            &read.repositories,
        )?
        else {
            return Ok(None);
        };
        if !delivered(&read.repositories, &repository) {
            return Ok(None);
        }
        let claims = read.state.standing_for(Some(&repository), &[]);
        if claims.is_empty() {
            return Ok(None);
        }
        let text = session_layer(&repository, read.head.0, &claims) + &menu(&repository);
        let line = text.lines().next().unwrap_or_default().to_owned();
        Ok(Some(json!({
            "systemMessage": line,
            "hookSpecificOutput": {
                "hookEventName": "SessionStart",
                "additionalContext": text,
            },
        })))
    })
}

/// True where the delivery at session start of `repository` is on.
fn delivered(registered: &[crate::judge::Repository], repository: &str) -> bool {
    registered
        .iter()
        .find(|registered| registered.name == repository)
        .is_none_or(|registered| registered.session)
}

/// The path of `file` from the root of its worktree, with `/` between
/// its parts; none where boom finds no root above it.
pub fn path_in_worktree(file: &Path) -> Option<String> {
    let root = repository::root(&terms::folder_of(file)?)?;
    let inside = file.strip_prefix(&root).ok()?;
    let parts: Option<Vec<&str>> = inside
        .components()
        .map(|part| match part {
            std::path::Component::Normal(part) => part.to_str(),
            _ => None,
        })
        .collect();
    Some(parts?.join("/")).filter(|path| !path.is_empty())
}

/// A context that the edit gives: its name, the counts of its claims,
/// and its edit layer.
struct Gives {
    name: String,
    counts: String,
    layer: String,
}

/// `boom hook pre-tool-use` (session-context): the answer to a
/// `PreToolUse` event of `Write`, `Edit` or `MultiEdit`, or none. It
/// checks the text that the tool writes for old words, as
/// `boom terms check --hook` does. Then it finds the contexts of the
/// file that the record of the session does not hold and that an
/// accepted claim binds. It stops the edit one time, with the old words
/// first and then the rules, and puts each context that it gives into
/// the record.
pub fn pre_tool_use(
    path: Result<PathBuf, Failure>,
    env: &Env,
    event: &Value,
) -> Result<Option<Value>, Failure> {
    let Some((file, texts)) = terms::written(event)? else {
        return Ok(None);
    };
    check_launcher(env)?;
    let path = path?;
    if !path.exists() {
        return Ok(None);
    }
    store::read_only(&path, Wanted::Accepted, |conn, read| {
        let Some(repository) = terms::repository_of(
            &file,
            None,
            env.boom_repository.as_deref(),
            &read.repositories,
        )?
        else {
            return Ok(None);
        };
        let inside = path_in_worktree(&file);
        let shown = file.display().to_string();

        // The old words of the text that the tool writes.
        let terms = terms::accepted(&read.state, &repository);
        let mut old = 0;
        let mut old_words = String::new();
        for (name, text) in &texts {
            for found in terms::find(text, &terms)? {
                old += 1;
                old_words.push_str(&terms::found_words(&shown, &found, &terms, Some(name)));
            }
        }

        // The rules of each new context of the file.
        let mut gives: Vec<Gives> = Vec::new();
        let record = record_of(env, event);
        if let (true, Some(record), Some(inside)) =
            (delivered(&read.repositories, &repository), &record, &inside)
        {
            for context in context::falls_in_state(&read.state, &repository, inside)? {
                let name = context::names(std::slice::from_ref(&context))[0].to_owned();
                if record.holds(&name) {
                    continue;
                }
                let claims = bound(&read.state, &name);
                let Some(given) = context_of(&read.state, &name) else {
                    continue;
                };
                if claims.is_empty() {
                    continue;
                }
                let layer = edit_layer(given, &claims, |version| {
                    version
                        .proposal
                        .body
                        .as_deref()
                        .map(|fingerprint| {
                            String::from_utf8(store::blob(conn, fingerprint)?).map_err(|_| {
                                Failure::not_run(format!("the blob {fingerprint} is not text"))
                            })
                        })
                        .transpose()
                })?;
                gives.push(Gives {
                    name,
                    counts: edit_counts(&claims),
                    layer,
                });
            }
            // Only the hook that makes the file of a context gives its rules.
            let mut taken = Vec::with_capacity(gives.len());
            for one in gives {
                if record.take(&one.name)? {
                    taken.push(one);
                }
            }
            gives = taken;
        }

        if old == 0 && gives.is_empty() {
            return Ok(None);
        }
        let mut message = Vec::new();
        let mut reason = String::new();
        if old > 0 {
            let place = inside.clone().unwrap_or_else(|| shown.clone());
            message.push(format!(
                "{} in {place}",
                count(old, "old word", "old words")
            ));
            reason.push_str(&format!(
                "Boom: {} of an accepted term: write the term in place of each.\n{old_words}",
                count(old, "old word", "old words")
            ));
        }
        if !gives.is_empty() {
            let names: Vec<String> = gives.iter().map(|one| one.name.clone()).collect();
            let named = repository::words(&names);
            let counts = if gives.len() == 1 {
                gives[0].counts.clone()
            } else {
                let mut all: Vec<Given<'_>> = Vec::new();
                for one in &gives {
                    all.extend(bound(&read.state, &one.name));
                }
                edit_counts(&all)
            };
            message.push(format!("{counts} for {named}"));
            if !reason.is_empty() {
                reason.push('\n');
            }
            let (context_words, rules_words) = if gives.len() == 1 {
                (format!("the context {named}"), "the context")
            } else {
                (format!("the contexts {named}"), "the contexts")
            };
            reason.push_str(&format!(
                "Boom: {counts} for {named}, at entry {}\n\
                 This is the first edit in this session of a file in {context_words}. \
                 Boom stopped it so that you read the rules of {rules_words} first. \
                 Make the edit again, so that it holds these rules.\n",
                read.head.0
            ));
            for one in &gives {
                reason.push_str(&one.layer);
            }
            reason.push_str("More: boom show <name> gives a claim in full.\n");
        }
        Ok(Some(json!({
            "systemMessage": format!("Boom: {}", message.join(", and ")),
            "hookSpecificOutput": {
                "hookEventName": "PreToolUse",
                "permissionDecision": "deny",
                "permissionDecisionReason": reason.trim_end(),
            },
        })))
    })
}
