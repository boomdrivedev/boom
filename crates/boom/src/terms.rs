//! `boom terms check`: the old words of the accepted terms in a file, or
//! in the text that an agent writes (term-check).
//!
//! The check reads the accepted terms of the repository of each file
//! from the store and writes nothing. It reads a file for the old words
//! of those terms and for nothing else. It matches a word as `boom
//! search` matches one, with SQLite's FTS5 tokenizer, in a table filled
//! in memory for one check and dropped with it.

use std::path::{Path, PathBuf};

use rusqlite::Connection;
use serde_json::{json, Value};

use crate::claim::Kind;
use crate::judge::{Repository, State};
use crate::{repository, Failure};

/// The tokenizer of `boom search`: `unicode61` splits at each mark that
/// is not a letter or a digit and folds case and accents, and `porter`
/// takes each word to its stem.
const TABLE: &str = "CREATE VIRTUAL TABLE lines USING fts5(
    text,
    tokenize = 'porter unicode61 remove_diacritics 2'
)";

/// The mark before and after each match that FTS5's `highlight` writes.
const OPEN: char = '\u{1}';
const CLOSE: char = '\u{2}';

/// The mark that `split_names` writes between the two words of a name
/// in code that a change of case joins, as in `KeeperStore`. FTS5 reads // boom: allow keeper
/// it as a mark between two words.
const CASE_JOIN: char = '\u{3}';

/// The marks that join the words of an old word that holds a hyphen:
/// a hyphen, an underscore and a change of case.
const JOINS: [char; 3] = ['-', '_', CASE_JOIN];

/// The marker that allows an old word on its line.
pub const MARKER: &str = "boom: allow ";

/// An accepted term of a repository, as the check uses it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Term {
    /// Its word, as its name.
    pub name: String,
    /// Its accepted version.
    pub version: String,
    /// Its meaning.
    pub meaning: String,
    /// The repository it binds, or none for the whole system.
    pub repository: Option<String>,
    /// The old words it replaces.
    pub replaces: Vec<String>,
}

/// An old word found on a line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Found {
    /// The line, from 1.
    pub line: usize,
    /// The old word, as its term names it.
    pub word: String,
    /// The old word as the line writes it.
    pub written: String,
    /// The term that replaces it, by its place in the terms checked.
    pub term: usize,
}

/// The accepted terms that stand for `repository` in `state`, by name:
/// its own, and those of the whole system that no term of its own stands
/// over (term-claims).
pub fn accepted(state: &State, repository: &str) -> Vec<Term> {
    state
        .standing_for(Some(repository), &[])
        .into_iter()
        .filter(|(claim, _)| claim.kind == Kind::Term)
        .map(|(claim, version)| Term {
            name: claim.name.clone(),
            version: version.proposal.version.clone(),
            meaning: version.proposal.statement.clone(),
            repository: claim.repository.clone(),
            replaces: version.proposal.replaces.clone(),
        })
        .collect()
}

/// The words that name a plan, not the system: a part or a point of
/// the work in time, a work item, a form that later work replaces, a
/// case named by the plan's hope, or a status of work. `boom propose`
/// warns of each one in a statement or a body, and never refuses it
/// (claim-versions, "The plan words").
pub const PLAN_WORDS: [&str; 22] = [
    "wave",
    "phase",
    "milestone",
    "sprint",
    "epic",
    "user story",
    "roadmap",
    "MVP",
    "stretch goal",
    "nice to have",
    "stopgap",
    "interim",
    "for now",
    "until then",
    "not built yet",
    "happy path",
    "sad path",
    "unhappy path",
    "TODO",
    "TBD",
    "WIP",
    "FIXME",
];

/// Each plan word in `text`, matched as an old word of a term matches
/// (`find`): `word` is the plan word as the list names it.
pub fn plan_words(text: &str) -> Result<Vec<Found>, Failure> {
    let plan = Term {
        name: String::new(),
        version: String::new(),
        meaning: String::new(),
        repository: None,
        replaces: PLAN_WORDS.iter().map(|&word| word.to_owned()).collect(),
    };
    let mut found = find(text, std::slice::from_ref(&plan))?;
    found.sort_by(|a, b| (a.line, &a.written).cmp(&(b.line, &b.written)));
    Ok(found)
}

/// `line` with `CASE_JOIN` at each change from a lowercase letter or a
/// digit to an uppercase letter, so a name in code splits into its
/// words: `KeeperStore` is `Keeper` and `Store`. FTS5 splits at `_`, `-` // boom: allow keeper
/// and `.`.
fn split_names(line: &str) -> String {
    let mut split = String::with_capacity(line.len() + 8);
    let mut before: Option<char> = None;
    for c in line.chars() {
        if c.is_uppercase() && before.is_some_and(|b| b.is_lowercase() || b.is_ascii_digit()) {
            split.push(CASE_JOIN);
        }
        split.push(c);
        before = Some(c);
    }
    split
}

/// Each old word of `terms` in `text`, by line and then by the order of
/// the terms: a word in another form, case or accent, an old word of
/// several words in their order on one line, and never an old word
/// inside a longer word. An old word that holds a hyphen matches only
/// its words joined as one name (`joined`). A line that holds
/// `boom: allow <old word>` gives that old word as found nowhere on it.
pub fn find(text: &str, terms: &[Term]) -> Result<Vec<Found>, Failure> {
    let olds: Vec<(usize, &str)> = terms
        .iter()
        .enumerate()
        .flat_map(|(at, term)| term.replaces.iter().map(move |old| (at, old.as_str())))
        .collect();
    if olds.is_empty() {
        return Ok(Vec::new());
    }
    let failed =
        |error: rusqlite::Error| Failure::not_run(format!("the check could not run: {error}"));
    let conn = Connection::open_in_memory().map_err(failed)?;
    conn.execute(TABLE, []).map_err(failed)?;
    let lines: Vec<&str> = text.lines().collect();
    {
        let mut insert = conn
            .prepare("INSERT INTO lines (rowid, text) VALUES (?1, ?2)")
            .map_err(failed)?;
        for (at, line) in lines.iter().enumerate() {
            let rowid = i64::try_from(at + 1).expect("a line's number fits a rowid");
            insert
                .execute(rusqlite::params![rowid, split_names(line)])
                .map_err(failed)?;
        }
    }
    let mut matches = conn
        .prepare(
            "SELECT rowid, highlight(lines, 0, char(1), char(2)) FROM lines
             WHERE lines MATCH ?1 ORDER BY rowid",
        )
        .map_err(failed)?;
    let mut found = Vec::new();
    for (term, old) in olds {
        let query = format!("\"{}\"", old.replace('"', "\"\""));
        let rows = matches
            .query_map([query], |row| {
                Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(failed)?;
        for row in rows {
            let (rowid, marked) = row.map_err(failed)?;
            let line = usize::try_from(rowid).expect("a rowid is a line's number");
            if lines[line - 1].contains(&format!("{MARKER}{old}")) {
                continue;
            }
            let hyphenated = old.contains('-');
            for span in marked_spans(&marked) {
                if hyphenated && !joined(&span) {
                    continue;
                }
                found.push(Found {
                    line,
                    word: old.to_owned(),
                    written: span.replace(CASE_JOIN, ""),
                    term,
                });
            }
        }
    }
    found.sort_by(|a, b| (a.line, a.term, &a.word).cmp(&(b.line, b.term, &b.word)));
    Ok(found)
}

/// True when one mark of `JOINS` joins each two words of `span`, and no
/// other mark is between them: `came-from`, `came_from` and `cameFrom` // boom: allow came-from
/// are joined, and `came from` and `came.from` are not.
fn joined(span: &str) -> bool {
    let mut gap = String::new();
    for c in span.chars() {
        if c.is_alphanumeric() {
            if !gap.is_empty() && !(gap.chars().count() == 1 && gap.starts_with(JOINS)) {
                return false;
            }
            gap.clear();
        } else {
            gap.push(c);
        }
    }
    true
}

/// The text between each pair of marks in `marked`.
fn marked_spans(marked: &str) -> Vec<String> {
    let mut spans = Vec::new();
    let mut rest = marked;
    while let Some(open) = rest.find(OPEN) {
        let after = &rest[open + OPEN.len_utf8()..];
        let Some(close) = after.find(CLOSE) else {
            break;
        };
        spans.push(after[..close].to_owned());
        rest = &after[close + CLOSE.len_utf8()..];
    }
    spans
}

/// The file that a tool writes to, and each text it writes there, with
/// its name: `the content`, `the new text`, or `the new text of edit 2`.
pub type Written = (PathBuf, Vec<(String, String)>);

/// The text that one Claude Code `PreToolUse` event writes: the file it
/// writes to, and each text with its name, or none for a tool that
/// writes no file's text (term-check, "A check run by a writer's hook").
pub fn written(event: &Value) -> Result<Option<Written>, Failure> {
    let tool = event.get("tool_name").and_then(Value::as_str).unwrap_or("");
    let input = event.get("tool_input").cloned().unwrap_or(Value::Null);
    let text = |field: &str| input.get(field).and_then(Value::as_str).map(str::to_owned);
    let texts = match tool {
        "Write" => vec![("the content".to_owned(), text("content"))],
        "Edit" => vec![("the new text".to_owned(), text("new_string"))],
        "MultiEdit" => input
            .get("edits")
            .and_then(Value::as_array)
            .map(|edits| {
                edits
                    .iter()
                    .enumerate()
                    .map(|(at, edit)| {
                        (
                            format!("the new text of edit {}", at + 1),
                            edit.get("new_string")
                                .and_then(Value::as_str)
                                .map(str::to_owned),
                        )
                    })
                    .collect()
            })
            .unwrap_or_default(),
        _ => return Ok(None),
    };
    let file = input
        .get("file_path")
        .and_then(Value::as_str)
        .filter(|file| !file.is_empty())
        .ok_or_else(|| {
            Failure::refused(format!(
                "the {tool} event names no file_path: boom terms check --hook reads one"
            ))
        })?;
    let mut file = PathBuf::from(file);
    if file.is_relative() {
        if let Some(cwd) = event.get("cwd").and_then(Value::as_str) {
            file = Path::new(cwd).join(file);
        }
    }
    Ok(Some((
        file,
        texts
            .into_iter()
            .filter_map(|(name, text)| Some((name, text?)))
            .collect(),
    )))
}

/// The nearest folder at or above the folder of `file` that exists.
pub fn folder_of(file: &Path) -> Option<PathBuf> {
    let absolute = if file.is_absolute() {
        file.to_owned()
    } else {
        std::env::current_dir().ok()?.join(file)
    };
    absolute
        .parent()
        .and_then(|parent| parent.ancestors().find(|folder| folder.is_dir()))
        .map(Path::to_owned)
}

/// The repository of `file`, as `boom query` finds the repository of a
/// session, with the folder of the file as the working folder.
pub fn repository_of(
    file: &Path,
    named: Option<&str>,
    launcher: Option<&str>,
    registered: &[Repository],
) -> Result<Option<String>, Failure> {
    repository::find(named, launcher, folder_of(file).as_deref(), registered)
}

/// One old word found, as `refused.found` gives it.
pub fn found_json(file: &str, found: &Found, terms: &[Term], text: Option<&str>) -> Value {
    let term = &terms[found.term];
    let mut object = json!({
        "file": file,
        "line": found.line,
        "word": found.word,
        "written": found.written,
        "term": term.name,
        "version": term.version,
        "meaning": term.meaning,
        "repository": term.repository,
    });
    if let Some(text) = text {
        object["text"] = json!(text);
    }
    object
}

/// One old word found, in words.
pub fn found_words(file: &str, found: &Found, terms: &[Term], text: Option<&str>) -> String {
    let term = &terms[found.term];
    let place = match text {
        Some(text) => format!("{file}, line {} of {text}", found.line),
        None => format!("{file}:{}", found.line),
    };
    let written = if found.written.to_lowercase() == found.word {
        format!("\"{}\"", found.written)
    } else {
        format!("\"{}\", the old word \"{}\",", found.written, found.word)
    };
    format!(
        "{place}: {written} is replaced by {} in {}: {}\n",
        term.name.replace('-', " "),
        crate::judge::binding_words(term.repository.as_deref()),
        term.meaning
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn owner() -> Vec<Term> {
        vec![
            Term {
                name: "owner".to_owned(),
                version: "aaaaaaaaaaaa".to_owned(),
                meaning: "A person who grants roles and registers keys in a store.".to_owned(),
                repository: Some("boom".to_owned()),
                replaces: vec!["keeper".to_owned(), "warden".to_owned()], // boom: allow keeper
            },
            Term {
                name: "run-log".to_owned(),
                version: "bbbbbbbbbbbb".to_owned(),
                meaning: "The log of a run.".to_owned(),
                repository: Some("boom".to_owned()),
                replaces: vec!["drive log".to_owned()],
            },
            Term {
                name: "origin".to_owned(),
                version: "cccccccccccc".to_owned(),
                meaning: "The file or the commit that a claim came from.".to_owned(),
                repository: Some("boom".to_owned()),
                replaces: vec!["came-from".to_owned()], // boom: allow came-from
            },
        ]
    }

    fn words(text: &str) -> Vec<(usize, String, String)> {
        find(text, &owner())
            .unwrap()
            .into_iter()
            .map(|found| (found.line, found.word, found.written))
            .collect()
    }

    #[test]
    fn an_old_word_in_another_form() {
        assert_eq!(
            words("one\ntwo\nThe keeper signs.\nKeepers and KEEPER\n"), // boom: allow keeper
            [
                (3, "keeper".to_owned(), "keeper".to_owned()), // boom: allow keeper
                (4, "keeper".to_owned(), "Keepers".to_owned()), // boom: allow keeper
                (4, "keeper".to_owned(), "KEEPER".to_owned()), // boom: allow keeper
            ]
        );
        assert_eq!(words("A kéeper.").len(), 1); // boom: allow keeper
    }

    #[test]
    fn a_name_in_code() {
        assert_eq!(
            words("let keeper_id = KeeperStore::new(); x.keeper-name") // boom: allow keeper
                .iter()
                .map(|(_, _, written)| written.as_str())
                .collect::<Vec<_>>(),
            ["keeper", "Keeper", "keeper"] // boom: allow keeper
        );
    }

    #[test]
    fn not_inside_a_longer_word() {
        assert!(words("The gatekeeper opens. Keeperless.").is_empty()); // boom: allow keeper
    }

    #[test]
    fn two_words_in_order_on_one_line() {
        assert_eq!(
            words("the Drive Log\nlog drive\ndrive\nlog"),
            [(1, "drive log".to_owned(), "Drive Log".to_owned())]
        );
    }

    #[test]
    fn a_hyphenated_old_word_matches_its_joined_names() {
        assert_eq!(
            words("came-from came_from cameFrom CameFrom CAME_FROM Came-From x.came_from_id") // boom: allow came-from
                .iter()
                .map(|(_, word, written)| (word.as_str(), written.as_str()))
                .collect::<Vec<_>>(),
            [
                ("came-from", "came-from"), // boom: allow came-from
                ("came-from", "came_from"), // boom: allow came-from
                ("came-from", "cameFrom"),  // boom: allow came-from
                ("came-from", "CameFrom"),  // boom: allow came-from
                ("came-from", "CAME_FROM"), // boom: allow came-from
                ("came-from", "Came-From"), // boom: allow came-from
                ("came-from", "came_from"), // boom: allow came-from
            ]
        );
    }

    #[test]
    fn a_hyphenated_old_word_never_matches_the_plain_words() {
        assert!(words(
            "It came from main.\nCame From\ncame.from\ncame - from\ncame--from\nbecame-from"
        )
        .is_empty());
    }

    #[test]
    fn a_name_in_code_is_written_as_the_line_writes_it() {
        assert_eq!(
            words("let DriveLog = 1;"),
            [(1, "drive log".to_owned(), "DriveLog".to_owned())]
        );
    }

    #[test]
    fn a_marker_allows_its_word_on_its_line() {
        assert!(words(
            "// boom: allow keeper. The old field keeper_id stays for the format of 2026."
        )
        .is_empty());
        assert_eq!(
            words("boom: allow warden, and a keeper"), // boom: allow keeper
            [(1, "keeper".to_owned(), "keeper".to_owned())]  // boom: allow keeper
        );
        assert_eq!(words("boom: allow keeper\nkeeper").len(), 1);
    }

    #[test]
    fn no_term_no_word() {
        assert!(find("keeper", &[]).unwrap().is_empty()); // boom: allow keeper
    }

    #[test]
    fn the_text_that_a_tool_writes() {
        let write =
            json!({"tool_name": "Write", "tool_input": {"file_path": "/r/a.md", "content": "x"}});
        let (file, texts) = written(&write).unwrap().unwrap();
        assert_eq!(file, PathBuf::from("/r/a.md"));
        assert_eq!(texts, [("the content".to_owned(), "x".to_owned())]);
        let multi = json!({"tool_name": "MultiEdit", "tool_input": {"file_path": "/r/a.md",
            "edits": [{"old_string": "a", "new_string": "b"}, {"old_string": "c", "new_string": "d"}]}});
        let (_, texts) = written(&multi).unwrap().unwrap();
        assert_eq!(texts.len(), 2);
        assert_eq!(texts[1].1, "d");
        let bash = json!({"tool_name": "Bash", "tool_input": {"command": "echo keeper > a"}}); // boom: allow keeper
        assert!(written(&bash).unwrap().is_none());
    }
}
