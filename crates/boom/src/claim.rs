//! Claims: their kinds, their names, their statements and scenarios,
//! a version as the fingerprint of its content, the history a version
//! that comes in from an import may carry (claim-versions; design 5),
//! and the edges a version stores: its sources, the claims a decision
//! changes, and its origins (claim-edges), the globs of a context
//! (context-claims), the claims a narrative includes and the notes its
//! version answers (narratives; narrative-notes), and the view of an
//! outline and the narratives it holds (views).

use serde::{Deserialize, Serialize};

/// The kind of a claim, fixed with its name at its first proposal.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, clap::ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    /// Something that holds of the work at every moment.
    Invariant,
    /// A choice made, and why.
    Decision,
    /// Something taken as true and not checked.
    Assumption,
    /// A word and its meaning in one repository or the whole system, with
    /// the old words it replaces (term-claims).
    Term,
    /// A part of the system that owns path globs, each in one repository
    /// (context-claims).
    Context,
    /// An explanation in its author's words, with a title and a body of
    /// Markdown text, that includes claims by name; a person judges it,
    /// and no one checks it (narratives).
    Narrative,
    /// The order of a view's narratives: a title, the view it computes and
    /// the narratives it holds, in order; a person judges it, and no one
    /// checks it (views).
    Outline,
}

impl Kind {
    /// The kind's name as an entry writes it.
    pub fn as_str(self) -> &'static str {
        match self {
            Kind::Invariant => "invariant",
            Kind::Decision => "decision",
            Kind::Assumption => "assumption",
            Kind::Term => "term",
            Kind::Context => "context",
            Kind::Narrative => "narrative",
            Kind::Outline => "outline",
        }
    }
}

/// What a record's file said of it where it no longer holds: a record
/// that comes in as history (record-import; record 0007, point 3).
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum HistoryStatus {
    /// Replaced by another record, its successor.
    Superseded,
    /// No longer followed, and replaced by none.
    Deprecated,
    /// Never followed.
    Rejected,
}

impl HistoryStatus {
    /// The status as an entry writes it.
    pub fn as_str(self) -> &'static str {
        match self {
            HistoryStatus::Superseded => "superseded",
            HistoryStatus::Deprecated => "deprecated",
            HistoryStatus::Rejected => "rejected",
        }
    }
}

/// A version's history: what its record's file said of it, and the
/// claim that succeeded it, where one did. A version with a history is
/// accepted as history and never stands (design 6).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct History {
    /// Superseded, deprecated or rejected.
    pub status: HistoryStatus,
    /// The successor's name, or none.
    pub by: Option<String>,
}

/// A version of a claim, as a `proposed` entry holds it.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Proposal {
    /// The claim's name, kebab case.
    pub name: String,
    /// The claim's kind.
    pub kind: Kind,
    /// The version: the fingerprint of the rest.
    pub version: String,
    /// The version it amends, or none for a first version.
    pub base: Option<String>,
    /// The claim in one line.
    pub statement: String,
    /// Its scenarios: a version in the old form has one at least; a
    /// version in the new form has none, and its entry writes none.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub scenarios: Vec<String>,
    /// The fingerprint of its body, where it has one.
    pub body: Option<String>,
    /// The old words a term replaces, each one time, in lowercase and
    /// in order; written only where there is one (term-claims).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub replaces: Vec<String>,
    /// The globs a context owns, each `<repository>:<pattern>`, each one
    /// time and sorted; written only where there is one (context-claims).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub paths: Vec<String>,
    /// Its history, where it comes in as history; written only where
    /// present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub history: Option<History>,
    /// The claims it rests on, each `<name>@<version>`, in the order
    /// given; written only where there is one (claim-edges).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source: Vec<String>,
    /// For a decision, the claims it changes, each `<name>@<version>`, in
    /// the order given; written only where there is one (claim-edges).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub changes: Vec<String>,
    /// The files and the commits it came from, in the order given;
    /// written only where there is one (claim-edges).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub origin: Vec<Origin>,
    /// For a narrative, the claims it includes, each `<name>@<version>`,
    /// in the order given; written only where there is one (narratives).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub includes: Vec<String>,
    /// For a narrative, the notes this version answers, each with its
    /// answer, in the order given; written only where there is one
    /// (narrative-notes).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub answers: Vec<Answer>,
    /// For an outline, the view it computes; written only where there is
    /// one (views).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub view: Option<String>,
    /// For an outline, the narratives it holds, each `<name>@<version>`,
    /// in the order given; written only where there is one (views).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub narratives: Vec<String>,
    /// The version of another claim that forced this one, `<name>@<version>`,
    /// which a revision names; written only where there is one
    /// (claim-edges, "A version names its cause").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cause: Option<String>,
}

/// The one view that boom computes: the design book (views).
pub const BOOK: &str = "book";

/// A note that a version of a narrative answers, and its answer
/// (narrative-notes).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Answer {
    /// The note's name, 12 hexadecimal characters.
    pub note: String,
    /// The answer, one line.
    pub words: String,
}

impl Answer {
    /// An answer read from `<note>=<words>`.
    pub fn parse(given: &str) -> Result<Answer, String> {
        let (note, words) = given.split_once('=').ok_or_else(|| {
            format!("an answer names its note and its words, <note>=<words>: {given} does not")
        })?;
        let answer = Answer {
            note: note.trim().to_owned(),
            words: words.trim().to_owned(),
        };
        answer.check()?;
        Ok(answer)
    }

    /// The form of an answer: a note's name, and one line of words.
    pub fn check(&self) -> Result<(), String> {
        if !hexadecimal(&self.note, 12) {
            return Err(format!(
                "an answer names a note by its name, 12 hexadecimal characters: {} is not",
                self.note
            ));
        }
        if self.words.contains(['\n', '\r']) {
            return Err(format!(
                "an answer is one line: the answer to {} holds a line break",
                self.note
            ));
        }
        if self.words.trim().is_empty() {
            return Err(format!(
                "an answer holds words: the answer to {} is empty",
                self.note
            ));
        }
        Ok(())
    }
}

/// Where a version came from: a file boom keeps by the fingerprint of
/// its bytes, or a file in a commit of a repository (claim-edges, "The
/// origin of a claim").
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase", deny_unknown_fields)]
pub enum Origin {
    /// A file boom keeps as a blob.
    File {
        /// The file's name, with no folder.
        name: String,
        /// The SHA-256 of its bytes, 64 hexadecimal characters.
        fingerprint: String,
    },
    /// A file in a commit, `git:<repository>@<commit>:<path>`.
    Git {
        /// The repository, as boom names repositories.
        repository: String,
        /// The commit's full name, 40 hexadecimal characters.
        commit: String,
        /// The file's path in the commit, relative.
        path: String,
    },
}

/// The prefix of an origin that names a file in a commit.
pub const GIT: &str = "git:";

impl Origin {
    /// A file in a commit, read from `git:<repository>@<commit>:<path>`.
    pub fn git(reference: &str) -> Result<Origin, String> {
        let rest = reference.strip_prefix(GIT).ok_or_else(|| {
            format!(
                "a reference to a commit is git:<repository>@<commit>:<path>: {reference} is not"
            )
        })?;
        let (repository, rest) = rest.split_once('@').ok_or_else(|| {
            format!("a reference to a commit is git:<repository>@<commit>:<path>: {reference} names no commit")
        })?;
        let (commit, path) = rest.split_once(':').ok_or_else(|| {
            format!(
                "a reference to a commit names the path of a file in it, git:<repository>@<commit>:<path>: {reference} names no path"
            )
        })?;
        let origin = Origin::Git {
            repository: repository.to_owned(),
            commit: commit.to_owned(),
            path: path.to_owned(),
        };
        origin.check()?;
        Ok(origin)
    }

    /// The origin as a person reads it: `<name>, sha256 <fingerprint>`,
    /// or `git:<repository>@<commit>:<path>`.
    pub fn words(&self) -> String {
        match self {
            Origin::File { name, fingerprint } => format!("{name}, sha256 {fingerprint}"),
            Origin::Git {
                repository,
                commit,
                path,
            } => format!("{GIT}{repository}@{commit}:{path}"),
        }
    }

    /// The form of an origin: a file's name with no folder and its
    /// fingerprint; a commit's repository, its full name and a relative
    /// path with no `..`.
    pub fn check(&self) -> Result<(), String> {
        match self {
            Origin::File { name, fingerprint } => {
                if name.is_empty()
                    || name.contains(['/', '\\', '\n', '\r'])
                    || name == "."
                    || name == ".."
                {
                    return Err(format!(
                        "an origin file is named by its name alone, one line with no folder: {name:?} is not"
                    ));
                }
                if !hexadecimal(fingerprint, 64) {
                    return Err(format!(
                        "an origin file is kept by its fingerprint, 64 hexadecimal characters: {fingerprint} is not"
                    ));
                }
                Ok(())
            }
            Origin::Git {
                repository,
                commit,
                path,
            } => {
                crate::repository::check_name(repository)?;
                if !hexadecimal(commit, 40) {
                    return Err(format!(
                        "a commit has 40 hexadecimal characters: {commit} has {}",
                        commit.chars().count()
                    ));
                }
                if path.is_empty() {
                    return Err(format!(
                        "a reference to a commit names the path of a file in it: {GIT}{repository}@{commit} names none"
                    ));
                }
                if path.starts_with('/')
                    || path.contains(['\\', '\n', '\r'])
                    || path
                        .split('/')
                        .any(|part| part.is_empty() || part == "." || part == "..")
                {
                    return Err(format!(
                        "the path of a file in a commit is relative, with no `..`: {path} is not"
                    ));
                }
                Ok(())
            }
        }
    }
}

/// Whether `text` is `length` lowercase hexadecimal characters.
fn hexadecimal(text: &str, length: usize) -> bool {
    text.len() == length
        && text
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

/// An edge to a claim as an entry stores it, `<name>@<version>`, split.
pub fn edge(stored: &str) -> Result<(&str, &str), String> {
    stored
        .split_once('@')
        .filter(|(name, version)| check_name(name).is_ok() && hexadecimal(version, 12))
        .ok_or_else(|| {
            format!("an edge names a claim of the store, <name>@<version>: {stored} does not")
        })
}

/// Each edge of `edges` one time, in the order first given.
pub fn once<T: PartialEq + Clone>(edges: &[T]) -> Vec<T> {
    let mut kept: Vec<T> = Vec::with_capacity(edges.len());
    for edge in edges {
        if !kept.contains(edge) {
            kept.push(edge.clone());
        }
    }
    kept
}

/// The content a version is the fingerprint of, in the order it is
/// written: its history, a term's repository, a term's old words, the
/// globs of a context and its edges last, and each only where present, so
/// a version with none keeps the fingerprint it had before they were.
#[derive(Serialize)]
struct Content<'a> {
    kind: Kind,
    name: &'a str,
    statement: &'a str,
    scenarios: &'a [String],
    body: Option<&'a str>,
    base: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    history: Option<&'a History>,
    #[serde(skip_serializing_if = "Option::is_none")]
    repository: Option<&'a str>,
    #[serde(skip_serializing_if = "<[String]>::is_empty")]
    replaces: &'a [String],
    #[serde(skip_serializing_if = "<[String]>::is_empty")]
    paths: &'a [String],
    #[serde(skip_serializing_if = "<[String]>::is_empty")]
    source: &'a [String],
    #[serde(skip_serializing_if = "<[String]>::is_empty")]
    changes: &'a [String],
    #[serde(skip_serializing_if = "<[Origin]>::is_empty")]
    origin: &'a [Origin],
    #[serde(skip_serializing_if = "<[String]>::is_empty")]
    includes: &'a [String],
    #[serde(skip_serializing_if = "<[Answer]>::is_empty")]
    answers: &'a [Answer],
    #[serde(skip_serializing_if = "Option::is_none")]
    view: Option<&'a str>,
    #[serde(skip_serializing_if = "<[String]>::is_empty")]
    narratives: &'a [String],
    #[serde(skip_serializing_if = "Option::is_none")]
    cause: Option<&'a str>,
}

impl Proposal {
    /// A version with its fingerprint computed from its content.
    pub fn new(
        kind: Kind,
        name: &str,
        statement: &str,
        scenarios: Vec<String>,
        body: Option<String>,
        base: Option<String>,
        history: Option<History>,
    ) -> Proposal {
        let mut proposal = Proposal {
            name: name.to_owned(),
            kind,
            version: String::new(),
            base,
            statement: statement.to_owned(),
            scenarios,
            body,
            replaces: Vec::new(),
            paths: Vec::new(),
            history,
            source: Vec::new(),
            changes: Vec::new(),
            origin: Vec::new(),
            includes: Vec::new(),
            answers: Vec::new(),
            view: None,
            narratives: Vec::new(),
            cause: None,
        };
        proposal.version = proposal.fingerprint(None);
        proposal
    }

    /// A version of a term: its word, its meaning, its body, its base,
    /// the one repository it binds, or none for the whole system, and the
    /// old words it replaces, kept as [`replaces`] keeps them
    /// (term-claims).
    pub fn term(
        name: &str,
        statement: &str,
        body: Option<String>,
        base: Option<String>,
        repository: Option<&str>,
        replaced: &[String],
    ) -> Proposal {
        let mut proposal = Proposal::new(Kind::Term, name, statement, Vec::new(), body, base, None);
        proposal.replaces = replaces(replaced);
        proposal.version = proposal.fingerprint(repository);
        proposal
    }

    /// A version of a context: its name, its statement, its body, its
    /// base and the globs it owns, kept as [`crate::context::globs`] keeps
    /// them (context-claims).
    pub fn context(
        name: &str,
        statement: &str,
        body: Option<String>,
        base: Option<String>,
        globs: &[String],
    ) -> Proposal {
        let mut proposal =
            Proposal::new(Kind::Context, name, statement, Vec::new(), body, base, None);
        proposal.paths = crate::context::globs(globs);
        proposal.version = proposal.fingerprint(None);
        proposal
    }

    /// The first 12 hexadecimal characters of the SHA-256 of its kind,
    /// name, statement, scenarios, body's fingerprint, base and, where
    /// it has them, its history, the repository of a term, given as
    /// `repository` and none for a term of the whole system, the old words of a term, the globs of a context, its
    /// edges `source`, `changes`, `origin` and `includes`, the answers
    /// of a narrative, the view and the narratives of an outline, and its
    /// cause (claim-versions; claim-edges; context-claims; narratives;
    /// views).
    pub fn fingerprint(&self, repository: Option<&str>) -> String {
        let content = serde_json::to_string(&Content {
            kind: self.kind,
            name: &self.name,
            statement: &self.statement,
            scenarios: &self.scenarios,
            body: self.body.as_deref(),
            base: self.base.as_deref(),
            history: self.history.as_ref(),
            repository,
            replaces: &self.replaces,
            paths: &self.paths,
            source: &self.source,
            changes: &self.changes,
            origin: &self.origin,
            includes: &self.includes,
            answers: &self.answers,
            view: self.view.as_deref(),
            narratives: &self.narratives,
            cause: self.cause.as_deref(),
        })
        .expect("a version's content is JSON");
        crate::sha256_hex(content.as_bytes())[..12].to_owned()
    }

    /// The rules of a version's form: its name, its statement, what its
    /// kind needs, its history, and its version the fingerprint of its
    /// content. A version with scenarios is in the old form, valid as it
    /// was written, none of them empty. A version with none is in the new
    /// form: a decision needs a body. A version with a history has no
    /// base, and names as its successor another claim (design 6). A
    /// term binds one repository of `repositories`, the ones its entry
    /// binds, or the whole system where they are none, and only a term
    /// replaces words (term-claims). A context
    /// owns one glob at least, each one time and sorted, and binds the
    /// repositories of its globs; only a context owns globs
    /// (context-claims). A narrative has no scenarios and no history, and
    /// only a narrative includes claims and answers notes (narratives).
    pub fn check(&self, repositories: &[String]) -> Result<(), String> {
        check_name(&self.name)?;
        check_statement(&self.statement)?;
        match &self.history {
            None if self.scenarios.is_empty() => check_kind(self.kind, self.body.as_deref())?,
            None => check_scenarios(&self.scenarios)?,
            Some(history) => check_history(self, history)?,
        }
        let repository = match (self.kind, repositories) {
            (Kind::Term, [one]) => Some(one.as_str()),
            (Kind::Term, [_, _, ..]) => {
                return Err(format!(
                    "a term binds one repository or the whole system: {} binds {}",
                    self.name,
                    crate::repository::words(repositories)
                ))
            }
            _ => None,
        };
        if self.kind == Kind::Term {
            if !self.scenarios.is_empty() || self.history.is_some() {
                return Err("a term has no scenarios and no history".to_owned());
            }
            check_replaces(&self.name, &self.replaces)?;
        } else if !self.replaces.is_empty() {
            return Err(format!(
                "only a term replaces words: {} is {} {}",
                self.name,
                article(self.kind),
                self.kind.as_str()
            ));
        }
        if self.kind == Kind::Narrative && (!self.scenarios.is_empty() || self.history.is_some()) {
            return Err("a narrative has no scenarios and no history".to_owned());
        }
        self.check_paths(repositories)?;
        self.check_edges()?;
        self.check_narrative()?;
        self.check_outline()?;
        let fingerprint = self.fingerprint(repository);
        if self.version != fingerprint {
            return Err(format!(
                "its version {} is not the fingerprint of its content, {fingerprint}",
                self.version
            ));
        }
        Ok(())
    }
}

impl Proposal {
    /// The globs of a context: one at least, each in its form, each one
    /// time and sorted, and its repositories the ones its entry binds.
    fn check_paths(&self, repositories: &[String]) -> Result<(), String> {
        if self.kind != Kind::Context {
            if self.paths.is_empty() {
                return Ok(());
            }
            return Err(format!(
                "only a context owns paths: {} is {} {}",
                self.name,
                article(self.kind),
                self.kind.as_str()
            ));
        }
        if !self.scenarios.is_empty() || self.history.is_some() {
            return Err("a context has no scenarios and no history".to_owned());
        }
        if self.paths.is_empty() {
            return Err(format!(
                "a context owns one glob at least, with --path <repository>:<pattern>: {} owns none",
                self.name
            ));
        }
        for glob in &self.paths {
            crate::context::glob(glob)?;
        }
        if self.paths != crate::context::globs(&self.paths) {
            return Err("a context keeps each glob one time, sorted".to_owned());
        }
        let named = crate::context::repositories(&self.paths);
        if repositories != named.as_slice() {
            return Err(format!(
                "a context binds the repositories of its globs: {} binds {}, and its globs name {}",
                self.name,
                crate::repository::words(repositories),
                crate::repository::words(&named)
            ));
        }
        Ok(())
    }

    /// The form of its edges: each edge to a claim `<name>@<version>` of
    /// another claim, each one time; `changes` only on a decision; each
    /// origin in its form, each one time (claim-edges).
    fn check_edges(&self) -> Result<(), String> {
        if !self.changes.is_empty() && self.kind != Kind::Decision {
            return Err(format!(
                "only a decision changes claims: {} is {} {}",
                self.name,
                article(self.kind),
                self.kind.as_str()
            ));
        }
        let cause: Vec<String> = self.cause.iter().cloned().collect();
        for (flag, edges) in [
            ("source", &self.source),
            ("changes", &self.changes),
            ("includes", &self.includes),
            ("cause", &cause),
        ] {
            for (at, stored) in edges.iter().enumerate() {
                let (name, _) = edge(stored)?;
                if name == self.name {
                    return Err(format!(
                        "a version names a version of its own claim with --amends, not with --{flag}: {stored}"
                    ));
                }
                if edges[..at].contains(stored) {
                    return Err(format!(
                        "a version keeps each edge one time: --{flag} {stored} is twice"
                    ));
                }
            }
        }
        for (at, origin) in self.origin.iter().enumerate() {
            origin.check()?;
            if self.origin[..at].contains(origin) {
                return Err(format!(
                    "a version keeps each edge one time: --origin {} is twice",
                    origin.words()
                ));
            }
        }
        Ok(())
    }

    /// What only a narrative has: the claims it includes, and the notes it
    /// answers, each note one time and each answer in its form
    /// (narratives; narrative-notes).
    fn check_narrative(&self) -> Result<(), String> {
        if self.kind != Kind::Narrative {
            if !self.includes.is_empty() {
                return Err(format!(
                    "only a narrative includes claims: {} is {} {}",
                    self.name,
                    article(self.kind),
                    self.kind.as_str()
                ));
            }
            if !self.answers.is_empty() {
                return Err(format!(
                    "only a narrative answers notes: {} is {} {}",
                    self.name,
                    article(self.kind),
                    self.kind.as_str()
                ));
            }
            return Ok(());
        }
        for (at, answer) in self.answers.iter().enumerate() {
            answer.check()?;
            if self.answers[..at]
                .iter()
                .any(|before| before.note == answer.note)
            {
                return Err(format!(
                    "a version answers each note one time: it answers {} twice",
                    answer.note
                ));
            }
        }
        Ok(())
    }
}

impl Proposal {
    /// What only an outline has: the view it computes, `book`, and the
    /// narratives it holds, one at least, each `<name>@<version>` of
    /// another claim and each narrative one time (views).
    fn check_outline(&self) -> Result<(), String> {
        if self.kind != Kind::Outline {
            if !self.narratives.is_empty() {
                return Err(format!(
                    "only an outline holds narratives: {} is {} {}",
                    self.name,
                    article(self.kind),
                    self.kind.as_str()
                ));
            }
            if self.view.is_some() {
                return Err(format!(
                    "only an outline names a view: {} is {} {}",
                    self.name,
                    article(self.kind),
                    self.kind.as_str()
                ));
            }
            return Ok(());
        }
        if !self.scenarios.is_empty() || self.history.is_some() {
            return Err("an outline has no scenarios and no history".to_owned());
        }
        match self.view.as_deref() {
            None => {
                return Err(format!(
                    "an outline names its view, with --view {BOOK}: {} names none",
                    self.name
                ))
            }
            Some(BOOK) => {}
            Some(other) => {
                return Err(format!(
                    "the view of an outline is {BOOK}: boom computes no view {other}"
                ))
            }
        }
        if self.narratives.is_empty() {
            return Err(format!(
                "an outline holds one narrative at least, with --narrative: {} holds none",
                self.name
            ));
        }
        let mut held: Vec<&str> = Vec::with_capacity(self.narratives.len());
        for stored in &self.narratives {
            let (name, _) = edge(stored)?;
            if name == self.name {
                return Err(format!(
                    "a version names a version of its own claim with --amends, not with --narrative: {stored}"
                ));
            }
            if held.contains(&name) {
                return Err(format!("the outline holds {name} two times"));
            }
            held.push(name);
        }
        Ok(())
    }
}

/// The article of a kind's name, `an invariant` or `a term`.
pub fn article(kind: Kind) -> &'static str {
    match kind {
        Kind::Invariant | Kind::Assumption | Kind::Outline => "an",
        Kind::Decision | Kind::Term | Kind::Context | Kind::Narrative => "a",
    }
}

/// Old words as a term keeps them: each with its runs of whitespace one
/// space, in lowercase, one time, in the order of the alphabet.
pub fn replaces(words: &[String]) -> Vec<String> {
    let mut kept: Vec<String> = words
        .iter()
        .map(|word| {
            word.split([' ', '\t'])
                .filter(|part| !part.is_empty())
                .collect::<Vec<_>>()
                .join(" ")
                .to_lowercase()
        })
        .collect();
    kept.sort();
    kept.dedup();
    kept
}

/// The word of a term, as a file writes it: its name's words joined by
/// spaces, `drive log` for `drive-log`.
pub fn word(name: &str) -> String {
    name.replace('-', " ")
}

/// A term's old words: none empty, each one line, as [`replaces`] keeps
/// them, and none the word of the term.
pub fn check_replaces(name: &str, replaces: &[String]) -> Result<(), String> {
    for old in replaces {
        if old.contains(['\n', '\r']) {
            return Err(format!(
                "an old word is one line: {old:?} holds a line break"
            ));
        }
        if !old.chars().any(char::is_alphanumeric) {
            return Err("an old word holds a letter or a digit: one is empty".to_owned());
        }
        if word(old) == word(name) {
            return Err(format!(
                "a term does not replace its own word: {name} replaces {old:?}"
            ));
        }
    }
    if replaces != self::replaces(replaces).as_slice() {
        return Err(
            "a term keeps its old words each one time, in lowercase, in the order of the alphabet"
                .to_owned(),
        );
    }
    Ok(())
}

/// A version that comes in as history: no base, no empty scenario, and
/// a successor that is a name other than its own.
fn check_history(proposal: &Proposal, history: &History) -> Result<(), String> {
    let name = &proposal.name;
    if let Some(base) = &proposal.base {
        return Err(format!(
            "a version that comes in as history has no base: {name} amends {base}"
        ));
    }
    if proposal
        .scenarios
        .iter()
        .any(|scenario| scenario.trim().is_empty())
    {
        return Err("a scenario of an old version is empty".to_owned());
    }
    if let Some(by) = &history.by {
        check_name(by).map_err(|why| format!("its successor: {why}"))?;
        if by == name {
            return Err(format!(
                "a version that comes in as history names a successor other than itself: {name} names {by}"
            ));
        }
    }
    Ok(())
}

/// A name is kebab case: lowercase letters and digits in words joined by
/// single hyphens.
pub fn check_name(name: &str) -> Result<(), String> {
    let kebab = !name.is_empty()
        && name.split('-').all(|word| {
            !word.is_empty()
                && word
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
        });
    if kebab {
        Ok(())
    } else {
        Err(format!(
            "a name is kebab case, lowercase letters and digits in words joined by single hyphens: {name:?} is not"
        ))
    }
}

/// A statement is one line of words.
pub fn check_statement(statement: &str) -> Result<(), String> {
    if statement.contains(['\n', '\r']) {
        Err("a statement is one line: this one holds a line break".to_owned())
    } else if statement.trim().is_empty() {
        Err("a statement is one line of words: this one is empty".to_owned())
    } else {
        Ok(())
    }
}

/// What a version in the new form needs for its kind: every kind a
/// statement; a decision a body too, its context, options, outcome and
/// consequences; a narrative a body, its Markdown text. An invariant's
/// body and an assumption's are optional.
pub fn check_kind(kind: Kind, body: Option<&str>) -> Result<(), String> {
    match (kind, body) {
        (Kind::Decision, None) => Err(
            "a decision needs a body: its context, options, outcome and consequences, with --body"
                .to_owned(),
        ),
        (Kind::Narrative, None) => {
            Err("a narrative needs a body: its Markdown text, with --body".to_owned())
        }
        _ => Ok(()),
    }
}

/// An old version's scenarios: none of them is empty.
pub fn check_scenarios(scenarios: &[String]) -> Result<(), String> {
    if scenarios.iter().any(|scenario| scenario.trim().is_empty()) {
        Err("a scenario of an old version is empty".to_owned())
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn money(statement: &str, base: Option<&str>) -> Proposal {
        Proposal::new(
            Kind::Invariant,
            "money-has-currency",
            statement,
            vec!["Given an amount, it names its currency.".to_owned()],
            None,
            base.map(str::to_owned),
            None,
        )
    }

    fn superseded(by: Option<&str>) -> History {
        History {
            status: HistoryStatus::Superseded,
            by: by.map(str::to_owned),
        }
    }

    fn floats(scenarios: Vec<String>, history: Option<History>) -> Proposal {
        Proposal::new(
            Kind::Decision,
            "amounts-as-floats",
            "Amounts are floats.",
            scenarios,
            None,
            None,
            history,
        )
    }

    #[test]
    fn a_name_is_kebab_case() {
        for name in ["money-has-currency", "billing", "rule-2", "a1-b2-c3"] {
            assert_eq!(check_name(name), Ok(()), "{name}");
        }
        for name in [
            "Refund_Within",
            "refund_within",
            "Money",
            "-money",
            "money-",
            "money--currency",
            "",
            "money currency",
        ] {
            let refused = check_name(name).unwrap_err();
            assert!(
                refused.starts_with("a name is kebab case"),
                "{name}: {refused}"
            );
        }
    }

    #[test]
    fn a_statement_is_one_line() {
        assert_eq!(check_statement("Every amount names its currency."), Ok(()));
        for statement in ["Two\nlines.", "Two\r\nlines.", "Cr\rlines."] {
            let refused = check_statement(statement).unwrap_err();
            assert!(refused.starts_with("a statement is one line"), "{refused}");
        }
        assert!(check_statement("  ").is_err());
    }

    #[test]
    fn a_new_version_needs_what_its_kind_needs() {
        // A new version has no scenarios: an invariant needs a statement.
        let invariant = Proposal::new(
            Kind::Invariant,
            "money-has-currency",
            "Every amount names its currency.",
            Vec::new(),
            None,
            None,
            None,
        );
        assert_eq!(invariant.check(&[]), Ok(()));
        let written = serde_json::to_string(&invariant).unwrap();
        assert!(!written.contains("scenarios"), "{written}");
        // A decision needs a body too.
        let refused = floats(Vec::new(), None).check(&[]).unwrap_err();
        assert!(refused.starts_with("a decision needs a body"), "{refused}");
        let mut with_body = floats(Vec::new(), None);
        with_body.body = Some("a".repeat(64));
        with_body.version = with_body.fingerprint(None);
        assert_eq!(with_body.check(&[]), Ok(()));
        // An old version keeps its scenarios, none of them empty.
        assert!(check_scenarios(&[String::new()]).is_err());
        assert_eq!(check_scenarios(&["Given x, then y.".to_owned()]), Ok(()));
        assert_eq!(
            floats(vec!["Given x, then y.".to_owned()], None).check(&[]),
            Ok(())
        );
    }

    #[test]
    fn a_version_is_the_fingerprint_of_its_content() {
        let first = money("Every amount names its currency.", None);
        assert_eq!(first.version.len(), 12);
        assert!(first.version.chars().all(|c| c.is_ascii_hexdigit()));
        let expected = crate::sha256_hex(
            br#"{"kind":"invariant","name":"money-has-currency","statement":"Every amount names its currency.","scenarios":["Given an amount, it names its currency."],"body":null,"base":null}"#,
        );
        assert_eq!(first.version, expected[..12]);
        assert_eq!(first, money("Every amount names its currency.", None));
        assert_ne!(
            first.version,
            money("Every amount names one currency.", None).version
        );
        assert_eq!(first.check(&[]), Ok(()));
        let mut forged = first.clone();
        forged.version = "000000000000".to_owned();
        assert!(forged
            .check(&[])
            .unwrap_err()
            .contains("not the fingerprint"));
    }

    #[test]
    fn the_base_is_part_of_a_versions_content() {
        let words = "Every amount names its currency.";
        let first = money(words, None);
        let on_base = money(words, Some(&first.version));
        assert_ne!(first.version, on_base.version);
        assert_ne!(on_base.version, money(words, Some("aaaaaaaaaaaa")).version);
    }

    #[test]
    fn history_is_part_of_a_versions_content() {
        let scenarios = vec!["Given an amount, then it is a float.".to_owned()];
        let standing = floats(scenarios.clone(), None);
        let history = floats(
            scenarios.clone(),
            Some(superseded(Some("amounts-in-cents"))),
        );
        assert_ne!(standing.version, history.version);
        let deprecated = floats(
            scenarios.clone(),
            Some(History {
                status: HistoryStatus::Deprecated,
                by: None,
            }),
        );
        assert_ne!(history.version, deprecated.version);
        let by_another = floats(scenarios, Some(superseded(Some("amounts-in-pence"))));
        assert_ne!(history.version, by_another.version);
        let expected = crate::sha256_hex(
            br#"{"kind":"decision","name":"amounts-as-floats","statement":"Amounts are floats.","scenarios":["Given an amount, then it is a float."],"body":null,"base":null,"history":{"status":"superseded","by":"amounts-in-cents"}}"#,
        );
        assert_eq!(history.version, expected[..12]);
        assert_eq!(history.check(&[]), Ok(()));
    }

    #[test]
    fn no_history_keeps_the_fingerprint_it_had() {
        let version = money("Every amount names its currency.", None);
        let expected = crate::sha256_hex(
            br#"{"kind":"invariant","name":"money-has-currency","statement":"Every amount names its currency.","scenarios":["Given an amount, it names its currency."],"body":null,"base":null}"#,
        );
        assert_eq!(version.version, expected[..12]);
        let written = serde_json::to_string(&version).unwrap();
        assert!(!written.contains("history"), "{written}");
        assert_eq!(serde_json::from_str::<Proposal>(&written).unwrap(), version);
    }

    #[test]
    fn a_history_version_needs_no_scenario() {
        let history = floats(Vec::new(), Some(superseded(Some("amounts-in-cents"))));
        assert!(history.scenarios.is_empty());
        assert_eq!(history.check(&[]), Ok(()));
        let rejected = floats(
            Vec::new(),
            Some(History {
                status: HistoryStatus::Rejected,
                by: None,
            }),
        );
        assert_eq!(rejected.check(&[]), Ok(()));
        let accepted = floats(Vec::new(), None);
        assert!(accepted
            .check(&[])
            .unwrap_err()
            .starts_with("a decision needs a body"));
        let empty = floats(vec![" ".to_owned()], Some(superseded(None)));
        assert!(empty.check(&[]).is_err());
        for by in ["Amounts_In_Cents", "amounts-as-floats"] {
            let refused = floats(Vec::new(), Some(superseded(Some(by))))
                .check(&[])
                .unwrap_err();
            assert!(refused.contains(by), "{refused}");
        }
    }

    fn with_edges(source: &[&str], changes: &[&str], origin: Vec<Origin>) -> Proposal {
        let mut decision = floats(Vec::new(), None);
        decision.body = Some("a".repeat(64));
        decision.source = source.iter().map(|edge| (*edge).to_owned()).collect();
        decision.changes = changes.iter().map(|edge| (*edge).to_owned()).collect();
        decision.origin = origin;
        decision.version = decision.fingerprint(None);
        decision
    }

    fn git(path: &str) -> Origin {
        Origin::Git {
            repository: "sluice".to_owned(),
            commit: "0123456789abcdef0123456789abcdef01234567".to_owned(),
            path: path.to_owned(),
        }
    }

    #[test]
    fn the_edges_are_part_of_a_versions_content() {
        let none = with_edges(&[], &[], Vec::new());
        let source = with_edges(&["money-has-currency@aaaaaaaaaaaa"], &[], Vec::new());
        let changes = with_edges(&[], &["money-has-currency@aaaaaaaaaaaa"], Vec::new());
        let origin = with_edges(&[], &[], vec![git("design/overview.svg")]);
        let versions = [&none, &source, &changes, &origin].map(|v| v.version.clone());
        for (at, version) in versions.iter().enumerate() {
            assert!(!versions[..at].contains(version), "{versions:?}");
        }
        for version in [&none, &source, &changes, &origin] {
            assert_eq!(version.check(&[]), Ok(()), "{version:?}");
        }
        // A version with no edge writes none, and keeps its content.
        let written = serde_json::to_string(&none).unwrap();
        for edge in ["source", "changes", "origin"] {
            assert!(!written.contains(edge), "{written}");
        }
        let written = serde_json::to_string(&origin).unwrap();
        assert!(
            written.ends_with(r#""origin":[{"git":{"repository":"sluice","commit":"0123456789abcdef0123456789abcdef01234567","path":"design/overview.svg"}}]}"#),
            "{written}"
        );
        assert_eq!(serde_json::from_str::<Proposal>(&written).unwrap(), origin);
    }

    #[test]
    fn an_edge_to_a_claim_in_its_form() {
        for edge in [
            "money-has-currency",
            "Money@aaaaaaaaaaaa",
            "money@AAAAAAAAAAAA",
            "money@aaa",
            "https://example.com/model",
        ] {
            let refused = with_edges(&[edge], &[], Vec::new()).check(&[]).unwrap_err();
            assert!(
                refused.starts_with("an edge names a claim of the store"),
                "{edge}: {refused}"
            );
        }
        let twice = with_edges(
            &["money@aaaaaaaaaaaa", "money@aaaaaaaaaaaa"],
            &[],
            Vec::new(),
        );
        assert!(twice.check(&[]).unwrap_err().contains("each edge one time"));
        let own = with_edges(&["amounts-as-floats@aaaaaaaaaaaa"], &[], Vec::new());
        assert!(own.check(&[]).unwrap_err().contains("with --amends"));
        let mut invariant = money("Every amount names its currency.", None);
        invariant.changes = vec!["amounts-as-floats@aaaaaaaaaaaa".to_owned()];
        invariant.version = invariant.fingerprint(None);
        assert!(invariant
            .check(&[])
            .unwrap_err()
            .starts_with("only a decision changes claims"));
    }

    #[test]
    fn an_origin_in_its_form() {
        let commit = "0123456789abcdef0123456789abcdef01234567";
        assert_eq!(
            Origin::git(&format!("git:sluice@{commit}:design/overview.svg")),
            Ok(git("design/overview.svg"))
        );
        for (reference, words) in [
            ("sluice@x:y", "a reference to a commit is"),
            ("git:sluice", "names no commit"),
            (&format!("git:sluice@{commit}") as &str, "names no path"),
            (
                "git:sluice@1a2b3c:a.svg",
                "a commit has 40 hexadecimal characters",
            ),
            (&format!("git:Sluice@{commit}:a.svg"), "kebab case"),
            (&format!("git:sluice@{commit}:"), "names the path of a file"),
            (
                &format!("git:sluice@{commit}:../a.svg"),
                "relative, with no `..`",
            ),
            (
                &format!("git:sluice@{commit}:/a.svg"),
                "relative, with no `..`",
            ),
            (
                &format!("git:sluice@{commit}:a//b.svg"),
                "relative, with no `..`",
            ),
        ] {
            let refused = Origin::git(reference).unwrap_err();
            assert!(refused.contains(words), "{reference}: {refused}");
        }
        let file = |name: &str, fingerprint: &str| Origin::File {
            name: name.to_owned(),
            fingerprint: fingerprint.to_owned(),
        };
        let fingerprint = crate::sha256_hex(b"<svg/>");
        assert_eq!(file("mockup.svg", &fingerprint).check(), Ok(()));
        assert!(file("a/mockup.svg", &fingerprint).check().is_err());
        assert!(file("", &fingerprint).check().is_err());
        assert!(file("mockup.svg", &fingerprint[..12]).check().is_err());
        let twice = with_edges(&[], &[], vec![git("a.svg"), git("a.svg")]);
        assert!(twice.check(&[]).unwrap_err().contains("each edge one time"));
    }

    fn overview(globs: &[&str]) -> Proposal {
        let globs: Vec<String> = globs.iter().map(|glob| (*glob).to_owned()).collect();
        Proposal::context(
            "sluice-overview",
            "The overview that a reader of the queue sees.",
            None,
            None,
            &globs,
        )
    }

    #[test]
    fn the_globs_of_a_context_are_part_of_its_content() {
        let one = overview(&["sluice:src/overview*.rs"]);
        let expected = crate::sha256_hex(
            br#"{"kind":"context","name":"sluice-overview","statement":"The overview that a reader of the queue sees.","scenarios":[],"body":null,"base":null,"paths":["sluice:src/overview*.rs"]}"#,
        );
        assert_eq!(one.version, expected[..12]);
        let two = overview(&["sluice:src/overview*.rs", "flume:src/status.rs"]);
        assert_ne!(one.version, two.version);
        assert_eq!(
            two.version,
            overview(&["flume:src/status.rs", "sluice:src/overview*.rs"]).version
        );
        assert_eq!(one.check(&["sluice".to_owned()]), Ok(()));
        assert_eq!(
            two.check(&["flume".to_owned(), "sluice".to_owned()]),
            Ok(())
        );
        let refused = one.check(&["flume".to_owned()]).unwrap_err();
        assert!(
            refused.starts_with("a context binds the repositories of its globs"),
            "{refused}"
        );
        let written = serde_json::to_string(&one).unwrap();
        assert!(
            written.contains(r#""paths":["sluice:src/overview*.rs"]"#),
            "{written}"
        );
        assert_eq!(serde_json::from_str::<Proposal>(&written).unwrap(), one);
    }

    #[test]
    fn a_context_owns_one_glob_at_least_and_only_a_context_owns_globs() {
        let none = overview(&[]);
        assert!(none
            .check(&[])
            .unwrap_err()
            .starts_with("a context owns one glob at least"));
        let mut unsorted = overview(&["sluice:a.rs"]);
        unsorted.paths = vec!["sluice:b.rs".to_owned(), "sluice:a.rs".to_owned()];
        unsorted.version = unsorted.fingerprint(None);
        assert!(unsorted
            .check(&["sluice".to_owned()])
            .unwrap_err()
            .contains("each glob one time, sorted"));
        let mut invariant = money("Every amount names its currency.", None);
        invariant.paths = vec!["sluice:a.rs".to_owned()];
        invariant.version = invariant.fingerprint(None);
        assert!(invariant
            .check(&[])
            .unwrap_err()
            .starts_with("only a context owns paths"));
        // A version with no glob writes none, and keeps its content.
        let written =
            serde_json::to_string(&money("Every amount names its currency.", None)).unwrap();
        assert!(!written.contains("paths"), "{written}");
    }

    #[test]
    fn a_history_version_has_no_base() {
        let history = Proposal::new(
            Kind::Decision,
            "amounts-as-floats",
            "Amounts are floats.",
            Vec::new(),
            None,
            Some("aaaaaaaaaaaa".to_owned()),
            Some(superseded(Some("amounts-in-cents"))),
        );
        let refused = history.check(&[]).unwrap_err();
        assert!(refused.contains("has no base"), "{refused}");
    }
}
