//! The store: one SQLite file holding the log, the blobs and the views
//! (record-store; design 2, 3).
//!
//! The log is the truth. A blob is content the log names by its
//! fingerprint, checked against that name whenever it is read. The
//! views, persons, claims, versions and each entry's judgment, are
//! computed from the log by [`judge::replay`]: each append updates them
//! in the transaction that appends its entry, and they are rebuilt
//! before any answer wherever they are missing or were built at an
//! entry other than the log's last (core.2).

use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use rusqlite::{params, Connection, OpenFlags, OptionalExtension, TransactionBehavior};

use crate::claim::{Kind, Origin, Proposal};
use crate::entry::{self, Act, Actor, Entry, GENESIS};
use crate::judge::{self, Claim, Judgment, Person, Proof, State, Status, Verdict, Version};
use crate::Failure;

/// The schema's version, the file's `user_version`.
pub const SCHEMA_VERSION: i64 = 1;

/// How long a command waits for another's write before it gives up.
const BUSY_TIMEOUT: Duration = Duration::from_secs(10);

/// The tables that hold the truth: the log, and the blobs it names.
const TRUTH: &str = "
CREATE TABLE log (
    seq INTEGER PRIMARY KEY,
    prev TEXT NOT NULL,
    hash TEXT NOT NULL,
    signed TEXT NOT NULL,
    signature TEXT
);
CREATE TABLE blobs (
    fingerprint TEXT PRIMARY KEY,
    bytes BLOB NOT NULL
);
";

/// The views, computed from the log and never the truth.
const VIEWS: &str = "
CREATE TABLE views_head (
    seq INTEGER NOT NULL,
    hash TEXT NOT NULL,
    form INTEGER NOT NULL
);
CREATE TABLE persons (
    name TEXT PRIMARY KEY,
    key TEXT NOT NULL,
    owner INTEGER NOT NULL,
    registered_in INTEGER NOT NULL
);
CREATE TABLE claims (
    name TEXT NOT NULL,
    term TEXT NOT NULL,
    kind TEXT NOT NULL,
    standing TEXT,
    PRIMARY KEY (name, term)
);
CREATE TABLE versions (
    name TEXT NOT NULL,
    term TEXT NOT NULL,
    version TEXT NOT NULL,
    base TEXT,
    statement TEXT NOT NULL,
    scenarios TEXT NOT NULL,
    body TEXT,
    proposer TEXT NOT NULL,
    proposed_in INTEGER NOT NULL,
    status TEXT NOT NULL,
    decided_by TEXT,
    decided_in INTEGER,
    reason TEXT,
    repositories TEXT NOT NULL,
    contexts TEXT NOT NULL,
    imported TEXT,
    proposal TEXT NOT NULL,
    PRIMARY KEY (name, version)
);
CREATE TABLE repositories (
    name TEXT PRIMARY KEY,
    remote TEXT NOT NULL,
    session INTEGER NOT NULL,
    set_by TEXT NOT NULL,
    set_in INTEGER NOT NULL
);
CREATE TABLE judgments (
    seq INTEGER PRIMARY KEY,
    counted INTEGER NOT NULL,
    reason TEXT
);
CREATE TABLE edges (
    name TEXT NOT NULL,
    term TEXT NOT NULL,
    version TEXT NOT NULL,
    edge TEXT NOT NULL,
    position INTEGER NOT NULL,
    target_name TEXT,
    target_version TEXT,
    artifact TEXT,
    origin TEXT,
    PRIMARY KEY (name, version, edge, position)
);
CREATE INDEX edges_target ON edges (target_name);
CREATE INDEX edges_artifact ON edges (artifact);
CREATE TABLE paths (
    name TEXT NOT NULL,
    term TEXT NOT NULL,
    version TEXT NOT NULL,
    position INTEGER NOT NULL,
    repository TEXT NOT NULL,
    pattern TEXT NOT NULL,
    PRIMARY KEY (name, version, position)
);
CREATE INDEX paths_repository ON paths (repository);
CREATE TABLE bindings (
    name TEXT NOT NULL,
    term TEXT NOT NULL,
    version TEXT NOT NULL,
    context TEXT NOT NULL,
    context_version TEXT NOT NULL,
    PRIMARY KEY (name, version, context)
);
CREATE INDEX bindings_context ON bindings (context);
CREATE TABLE notes (
    name TEXT PRIMARY KEY,
    narrative TEXT NOT NULL,
    version TEXT NOT NULL,
    paragraph INTEGER NOT NULL,
    fingerprint TEXT NOT NULL,
    words TEXT NOT NULL,
    person TEXT NOT NULL,
    at INTEGER NOT NULL,
    noted_in INTEGER NOT NULL
);
CREATE INDEX notes_narrative ON notes (narrative);
";

/// The views' tables, each dropped and made again by a rebuild.
const VIEW_TABLES: [&str; 10] = [
    "views_head",
    "persons",
    "claims",
    "versions",
    "repositories",
    "judgments",
    "edges",
    "paths",
    "bindings",
    "notes",
];

/// The views' form, kept in `views_head`. Views of another form are
/// built again from the whole log.
const VIEWS_FORM: i64 = 8;

thread_local! {
    static ENTRIES_READ: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
    static VERSIONS_READ: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

/// The number of versions this thread has read from a store's views,
/// for a test to count what one command loads.
pub fn versions_read() -> u64 {
    VERSIONS_READ.with(std::cell::Cell::get)
}

/// The number of the log's entries this thread has read from a store,
/// for a test to count what one command reads.
pub fn entries_read() -> u64 {
    ENTRIES_READ.with(std::cell::Cell::get)
}

fn count_read(entries: usize) {
    ENTRIES_READ.with(|read| read.set(read.get() + entries as u64));
}

impl From<rusqlite::Error> for Failure {
    fn from(error: rusqlite::Error) -> Self {
        Failure::not_run(format!("the store could not be read or written: {error}"))
    }
}

/// The path of a store: the one `--store` names, else `boom.sqlite`
/// under `$XDG_DATA_HOME/boom/`, else under `$HOME/.local/share/boom/`.
pub fn resolve(
    named: Option<&Path>,
    xdg_data_home: Option<&Path>,
    home: Option<&Path>,
) -> Result<PathBuf, Failure> {
    fn set(path: Option<&Path>) -> Option<&Path> {
        path.filter(|path| !path.as_os_str().is_empty())
    }
    if let Some(named) = set(named) {
        return Ok(named.to_owned());
    }
    if let Some(data) = set(xdg_data_home) {
        return Ok(data.join("boom").join("boom.sqlite"));
    }
    if let Some(home) = set(home) {
        return Ok(home.join(".local/share/boom/boom.sqlite"));
    }
    Err(Failure::not_run(
        "no store is named: --store, XDG_DATA_HOME and HOME are all unset",
    ))
}

/// One entry of the log, as stored.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    /// Its number, from 1, in the order appended.
    pub seq: u64,
    /// The hash of the entry before it.
    pub prev: String,
    /// Its hash.
    pub hash: String,
    /// Its signed part, as signed.
    pub signed: String,
    /// Its armored signature, where it has one.
    pub signature: Option<String>,
}

/// What an append did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Written {
    /// It appended the entry numbered here.
    Entry(u64),
    /// It appended nothing: the entry repeats the one numbered here.
    Repeats(u64),
}

impl Written {
    /// The entry that holds the act: the one appended, or the one it
    /// repeats.
    pub fn seq(self) -> u64 {
        match self {
            Written::Entry(seq) | Written::Repeats(seq) => seq,
        }
    }
}

/// A file being made by `boom init`, removed unless it is put in place.
struct Making(PathBuf);

impl Drop for Making {
    fn drop(&mut self) {
        for suffix in ["", "-wal", "-shm", "-journal"] {
            let mut path = self.0.clone().into_os_string();
            path.push(suffix);
            let _ = fs::remove_file(path);
        }
    }
}

/// A store, open.
pub struct Store {
    conn: Connection,
    path: PathBuf,
    making: Option<Making>,
}

impl Store {
    /// Opens the store at `path`, making nothing: not run where there is
    /// no store there.
    pub fn open(path: &Path) -> Result<Store, Failure> {
        if !path.is_file() {
            return Err(Failure::not_run(format!(
                "no store at {}: boom init makes one",
                path.display()
            )));
        }
        let conn = Connection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )?;
        conn.busy_timeout(BUSY_TIMEOUT)?;
        let version: i64 = conn.pragma_query_value(None, "user_version", |row| row.get(0))?;
        if version != SCHEMA_VERSION {
            return Err(Failure::not_run(format!(
                "{} is not a boom store of schema {SCHEMA_VERSION}: its user_version is {version}",
                path.display()
            )));
        }
        Ok(Store {
            conn,
            path: path.to_owned(),
            making: None,
        })
    }

    /// Makes a store to be put at `path` by [`Store::finish`], with the
    /// folders above it: in WAL, its `user_version` 1, its tables made
    /// and its log empty. Until then the file is beside `path` under
    /// another name, and removed unless it is finished.
    pub fn create(path: &Path) -> Result<Store, Failure> {
        let cannot = |error: std::io::Error| {
            Failure::not_run(format!(
                "the store {} could not be made: {error}",
                path.display()
            ))
        };
        let parent = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        let name = path
            .file_name()
            .ok_or_else(|| Failure::refused(format!("{} names no file", path.display())))?;
        fs::create_dir_all(parent).map_err(cannot)?;
        let mut making_name = std::ffi::OsString::from(".");
        making_name.push(name);
        making_name.push(format!(".{}.making", std::process::id()));
        let making = Making(parent.join(making_name));
        drop(Making(making.0.clone()));
        let conn = Connection::open(&making.0)?;
        conn.busy_timeout(BUSY_TIMEOUT)?;
        let mode: String = conn.query_row("PRAGMA journal_mode = WAL", [], |row| row.get(0))?;
        if !mode.eq_ignore_ascii_case("wal") {
            return Err(Failure::not_run(format!(
                "the store {} could not be put in WAL: its journal is {mode}",
                path.display()
            )));
        }
        conn.pragma_update(None, "user_version", SCHEMA_VERSION)?;
        conn.execute_batch(TRUTH)?;
        conn.execute_batch(VIEWS)?;
        write_head(&conn, 0, GENESIS)?;
        Ok(Store {
            conn,
            path: path.to_owned(),
            making: Some(making),
        })
    }

    /// Puts a store [`Store::create`] made at its path, refused where a
    /// store is there already.
    pub fn finish(self) -> Result<(), Failure> {
        let Store { conn, path, making } = self;
        let Some(making) = making else {
            return Ok(());
        };
        conn.close().map_err(|(_, error)| Failure::from(error))?;
        match fs::hard_link(&making.0, &path) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => Err(
                Failure::refused(format!("the store at {} exists", path.display())),
            ),
            Err(error) => Err(Failure::not_run(format!(
                "the store {} could not be put in place: {error}",
                path.display()
            ))),
        }
    }

    /// The store's path.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Brings the views to the log's last entry and answers from them
    /// with `answer`. Where the views' head names the log's last entry,
    /// it reads that entry alone; otherwise it catches up first, as a
    /// write does (record-store, "A command answers from the views").
    pub fn read<T>(
        &mut self,
        answer: impl FnOnce(&Connection) -> Result<T, Failure>,
    ) -> Result<T, Failure> {
        let path = self.path.clone();
        {
            let tx = self
                .conn
                .transaction_with_behavior(TransactionBehavior::Deferred)?;
            if fresh(&tx, &path)? {
                let answered = answer(&tx);
                tx.commit()?;
                return answered;
            }
        }
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        catch_up(&tx, &path, Scope::default())?;
        let answered = answer(&tx);
        tx.commit()?;
        answered
    }

    /// The whole state the log's entries build, read from the views once
    /// they are at the log's last entry.
    pub fn state(&mut self) -> Result<State, Failure> {
        self.read(load_state)
    }

    /// The part of the state `scope` names, read from the views once they
    /// are at the log's last entry.
    pub fn state_for(&mut self, scope: &Scope) -> Result<State, Failure> {
        self.read(|conn| load_state_for(conn, scope))
    }

    /// Appends the entry whose signed part is `signed`, with its
    /// signature, and `blobs`, the body and the files it names:
    /// [`Store::append_all`] of one. Answers what it wrote and the state
    /// after it.
    pub fn append(
        &mut self,
        signed: &str,
        signature: Option<&str>,
        blobs: &[&[u8]],
    ) -> Result<(Written, State), Failure> {
        let (written, state) = self.append_all(&[Append {
            signed,
            signature,
            blobs,
        }])?;
        let written = *written.first().expect("one entry is one written");
        Ok((written, state))
    }

    /// Appends `entries`, each with its signature and blob, in one
    /// `IMMEDIATE` transaction: it brings the views to the log's last
    /// entry, judges each entry against the state the views and the
    /// entries before it hold, and updates the rows each one touches
    /// (design 8). A refusal of any rolls the transaction back, so
    /// nothing is written; a repeat adds nothing. Answers what each
    /// wrote, in order, and the state after them.
    pub fn append_all(&mut self, entries: &[Append<'_>]) -> Result<(Vec<Written>, State), Failure> {
        let parsed = entries
            .iter()
            .map(|append| Entry::parse(append.signed).map_err(Failure::Refused))
            .collect::<Result<Vec<_>, _>>()?;
        let path = self.path.clone();
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let mut state = catch_up(&tx, &path, Scope::of(&parsed))?;
        let (mut seq, mut prev) = views_head(&tx)?;
        let mut written = Vec::with_capacity(entries.len());
        for (append, entry) in entries.iter().zip(&parsed) {
            match judge::judge(&state, entry, append.signed, Proof::of(append.signature)) {
                Verdict::Counts => {}
                Verdict::Refused(why) => return Err(Failure::Refused(why)),
                Verdict::Repeats(repeated) => {
                    written.push(Written::Repeats(repeated));
                    continue;
                }
            }
            seq += 1;
            let hash = entry::hash(&prev, append.signed, append.signature);
            tx.execute(
                "INSERT INTO log (seq, prev, hash, signed, signature) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![sql_int(seq), prev, hash, append.signed, append.signature],
            )?;
            for bytes in append.blobs {
                tx.execute(
                    "INSERT OR IGNORE INTO blobs (fingerprint, bytes) VALUES (?1, ?2)",
                    params![crate::sha256_hex(bytes), bytes],
                )?;
            }
            judge::apply(&mut state, seq, entry);
            read_body_of(&tx, &mut state, entry);
            write_judgment(&tx, seq, &Judgment::counted())?;
            write_touched(&tx, &state, entry)?;
            write_head(&tx, seq, &hash)?;
            prev = hash;
            written.push(Written::Entry(seq));
        }
        tx.commit()?;
        Ok((written, state))
    }

    /// Checks the whole chain, replays the whole log and builds every
    /// view again from it, naming each view table whose rows differed
    /// (record-store, "boom verify checks the log and rebuilds the
    /// views"). Where the chain does not hold, not run, naming the first
    /// entry whose chain fails, and nothing written.
    pub fn verify(&mut self) -> Result<Verified, Failure> {
        let path = self.path.clone();
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let rows = read_log(&tx)?;
        check_chain(&path, &rows)?;
        let before = VIEW_TABLES
            .iter()
            .map(|table| snapshot(&tx, table))
            .collect::<Result<Vec<_>, _>>()?;
        let (state, judgments) = replay(&tx, &rows);
        write_views(&tx, &state, &judgments, &rows)?;
        let mut rebuilt = Vec::new();
        for (table, was) in VIEW_TABLES.iter().zip(before) {
            if was.as_ref() != Some(&snapshot(&tx, table)?.unwrap_or_default()) {
                rebuilt.push((*table).to_owned());
            }
        }
        let (seq, hash) = rows
            .last()
            .map_or((0, GENESIS.to_owned()), |row| (row.seq, row.hash.clone()));
        tx.commit()?;
        Ok(Verified { seq, hash, rebuilt })
    }
}

/// What a reader that writes nothing reads from a store: the log's last
/// entry, the repositories it registers, and the claims that it asks for
/// (term-check; session-context).
#[derive(Debug, Clone)]
pub struct ReadOnly {
    /// The log's last entry, `(number, hash)`; `(0, GENESIS)` for an
    /// empty log.
    pub head: (u64, String),
    /// The repositories the store registers.
    pub repositories: Vec<judge::Repository>,
    /// A state that holds each claim that the reader asks for, and maybe
    /// other claims.
    pub state: State,
}

/// The claims that a reader that writes nothing asks for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Wanted {
    /// Each claim whose accepted version is a term.
    Terms,
    /// Each claim that has an accepted version.
    Accepted,
}

/// Reads the accepted terms of the store at `path` and writes nothing,
/// not even the views: [`read_only`] of the terms.
pub fn read_terms(path: &Path) -> Result<ReadOnly, Failure> {
    read_only(path, Wanted::Terms, |_, read| Ok(read))
}

/// Reads the store at `path` and writes nothing, not even the views, and
/// answers with `answer` on the connection that read it, which can also
/// read the blobs. Where the views are not at the log's last entry, the
/// log is replayed in memory. Where SQLite cannot open the store to read
/// beside it, as in a sandbox that lets nothing write the store's folder,
/// the store is read as a file that does not change, which holds only
/// while no entry waits in its write-ahead log (term-check, the gate
/// task; session-context).
pub fn read_only<T>(
    path: &Path,
    wanted: Wanted,
    answer: impl FnOnce(&Connection, ReadOnly) -> Result<T, Failure>,
) -> Result<T, Failure> {
    if !path.is_file() {
        return Err(Failure::not_run(format!(
            "no store at {}: boom init makes one",
            path.display()
        )));
    }
    let conn = open_to_read(path)?;
    let version: i64 = conn.pragma_query_value(None, "user_version", |row| row.get(0))?;
    if version != SCHEMA_VERSION {
        return Err(Failure::not_run(format!(
            "{} is not a boom store of schema {SCHEMA_VERSION}: its user_version is {version}",
            path.display()
        )));
    }
    let tx = conn.unchecked_transaction()?;
    let read = if fresh(&tx, path)? {
        let mut state = State::default();
        let filter = match wanted {
            Wanted::Terms => "WHERE kind = 'term' AND standing IS NOT NULL",
            Wanted::Accepted => "WHERE standing IS NOT NULL",
        };
        load_claims(&tx, &mut state, filter, [])?;
        ReadOnly {
            head: views_head(&tx)?,
            repositories: repositories(&tx)?,
            state,
        }
    } else {
        let rows = read_log(&tx)?;
        check_chain(path, &rows)?;
        let (state, _) = replay(&tx, &rows);
        let head = rows
            .last()
            .map_or((0, GENESIS.to_owned()), |row| (row.seq, row.hash.clone()));
        ReadOnly {
            head,
            repositories: state.repositories.values().cloned().collect(),
            state,
        }
    };
    let answered = answer(&tx, read);
    tx.finish()?;
    answered
}

/// A connection that only reads the store at `path`: an ordinary one
/// where SQLite can share the store's memory file, else one that takes
/// the file as one that does not change, refused while its write-ahead
/// log holds a write.
fn open_to_read(path: &Path) -> Result<Connection, Failure> {
    let flags = OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX;
    let ordinary = Connection::open_with_flags(path, flags).and_then(|conn| {
        conn.busy_timeout(BUSY_TIMEOUT)?;
        conn.query_row("SELECT count(*) FROM sqlite_master", [], |_| Ok(()))?;
        Ok(conn)
    });
    let error = match ordinary {
        Ok(conn) => return Ok(conn),
        Err(error) => error,
    };
    let mut wal = path.as_os_str().to_owned();
    wal.push("-wal");
    if fs::metadata(&wal).is_ok_and(|wal| wal.len() > 0) {
        return Err(Failure::not_run(format!(
            "the store at {} could not be read: {error}; and a write waits in its write-ahead log, so it cannot be read as a file that does not change",
            path.display()
        )));
    }
    let uri = format!("file:{}?immutable=1", uri_path(path));
    let conn = Connection::open_with_flags(uri, flags | OpenFlags::SQLITE_OPEN_URI)?;
    Ok(conn)
}

/// A path as the path of an SQLite URI: each byte other than a letter,
/// a digit and `/-_.~` written as `%XX`.
fn uri_path(path: &Path) -> String {
    let mut written = String::new();
    for byte in path.as_os_str().as_encoded_bytes() {
        if byte.is_ascii_alphanumeric() || b"/-_.~".contains(byte) {
            written.push(char::from(*byte));
        } else {
            written.push_str(&format!("%{byte:02X}"));
        }
    }
    written
}

/// The part of the state a command needs: every person, the claims it
/// names, and, for an import, every claim with a version that came in
/// from a file. A command loads only these rows from the views.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Scope {
    /// The claims named.
    pub names: std::collections::BTreeSet<String>,
    /// Whether every claim with an imported version is needed too.
    pub imported: bool,
    /// The notes named, by their names, which an answer names
    /// (narrative-notes).
    pub notes: std::collections::BTreeSet<String>,
}

impl Scope {
    /// The claims `names` names.
    pub fn names<I: IntoIterator<Item = S>, S: Into<String>>(names: I) -> Scope {
        Scope {
            names: names.into_iter().map(Into::into).collect(),
            imported: false,
            notes: std::collections::BTreeSet::new(),
        }
    }

    /// What judging `entries` needs.
    pub fn of<'a>(entries: impl IntoIterator<Item = &'a Entry>) -> Scope {
        let mut scope = Scope::default();
        for entry in entries {
            match &entry.act {
                Act::Registered { .. } => {}
                Act::Proposed {
                    claim,
                    imported,
                    attach,
                } => {
                    scope.names.insert(claim.name.clone());
                    scope.imported |= imported.is_some();
                    // The claims its edges and its contexts name, for the
                    // judge to find the versions they meant (claim-edges;
                    // context-claims).
                    scope
                        .notes
                        .extend(claim.answers.iter().map(|answer| answer.note.clone()));
                    scope.names.extend(
                        claim
                            .source
                            .iter()
                            .chain(&claim.changes)
                            .chain(&claim.includes)
                            .chain(crate::judge::contexts(attach.as_ref()))
                            .filter_map(|edge| edge.split_once('@'))
                            .map(|(name, _)| name.to_owned()),
                    );
                }
                Act::Accepted { versions } => {
                    scope
                        .names
                        .extend(versions.iter().map(|named| named.name.clone()));
                }
                Act::Rejected { name, .. } | Act::Noted { name, .. } => {
                    scope.names.insert(name.clone());
                }
                Act::Repository { .. } => {}
            }
        }
        scope
    }

    fn join(&mut self, other: Scope) {
        self.names.extend(other.names);
        self.imported |= other.imported;
        self.notes.extend(other.notes);
    }
}

/// What `boom verify` found: the log's last entry, and the view tables
/// it built again because their rows differed from the replay.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Verified {
    /// The log's last entry's number, 0 for an empty log.
    pub seq: u64,
    /// Its hash.
    pub hash: String,
    /// The view tables whose rows differed, in the views' order.
    pub rebuilt: Vec<String>,
}

/// An entry to append: its signed part, its signature where it has
/// one, and the blobs it names: its body and the files it came from.
#[derive(Debug, Clone, Copy)]
pub struct Append<'a> {
    /// Its signed part, as signed.
    pub signed: &'a str,
    /// Its armored signature.
    pub signature: Option<&'a str>,
    /// The body and the files it names, each kept under its fingerprint.
    pub blobs: &'a [&'a [u8]],
}

fn sql_int(value: u64) -> i64 {
    i64::try_from(value).expect("an entry's number fits SQLite's integer")
}

fn read_log(conn: &Connection) -> Result<Vec<Row>, Failure> {
    let mut statement =
        conn.prepare("SELECT seq, prev, hash, signed, signature FROM log ORDER BY seq")?;
    let rows = statement
        .query_map([], |row| {
            Ok(Row {
                seq: u64::try_from(row.get::<_, i64>(0)?).unwrap_or(0),
                prev: row.get(1)?,
                hash: row.get(2)?,
                signed: row.get(3)?,
                signature: row.get(4)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    count_read(rows.len());
    Ok(rows)
}

fn row_from(row: &rusqlite::Row<'_>) -> rusqlite::Result<Row> {
    Ok(Row {
        seq: u64::try_from(row.get::<_, i64>(0)?).unwrap_or(0),
        prev: row.get(1)?,
        hash: row.get(2)?,
        signed: row.get(3)?,
        signature: row.get(4)?,
    })
}

/// The log's last entry, where it has one: one entry read.
fn last_row(conn: &Connection) -> Result<Option<Row>, Failure> {
    let row = conn
        .query_row(
            "SELECT seq, prev, hash, signed, signature FROM log ORDER BY seq DESC LIMIT 1",
            [],
            row_from,
        )
        .optional()?;
    count_read(usize::from(row.is_some()));
    Ok(row)
}

/// The entry numbered `seq`, where there is one: one entry read.
pub fn row_at(conn: &Connection, seq: u64) -> Result<Option<Row>, Failure> {
    let row = conn
        .query_row(
            "SELECT seq, prev, hash, signed, signature FROM log WHERE seq = ?1",
            [sql_int(seq)],
            row_from,
        )
        .optional()?;
    count_read(usize::from(row.is_some()));
    Ok(row)
}

/// The entries after the one numbered `seq`, in order.
fn rows_after(conn: &Connection, seq: u64) -> Result<Vec<Row>, Failure> {
    let mut statement = conn.prepare(
        "SELECT seq, prev, hash, signed, signature FROM log WHERE seq > ?1 ORDER BY seq",
    )?;
    let rows = statement
        .query_map([sql_int(seq)], row_from)?
        .collect::<Result<Vec<_>, _>>()?;
    count_read(rows.len());
    Ok(rows)
}

fn altered(path: &Path, seq: u64, what: &str) -> Failure {
    Failure::not_run(format!(
        "the store at {} is altered: entry {seq} {what}",
        path.display()
    ))
}

/// Checks that `row` still has the hash `hash` the views' head names,
/// and that its words hash to it.
fn check_head(path: &Path, row: Option<&Row>, seq: u64, hash: &str) -> Result<(), Failure> {
    if seq == 0 {
        return if hash == GENESIS {
            Ok(())
        } else {
            Err(Failure::not_run(format!(
                "the store at {} is altered: its views name an entry its log lacks",
                path.display()
            )))
        };
    }
    let Some(row) = row else {
        return Err(altered(path, seq, "is missing: the views were built at it"));
    };
    if row.hash != hash {
        return Err(altered(
            path,
            seq,
            "does not match the hash the views were built at",
        ));
    }
    if row.hash != entry::hash(&row.prev, &row.signed, row.signature.as_deref()) {
        return Err(altered(path, seq, "does not match its hash"));
    }
    Ok(())
}

/// Checks that each of `rows` follows the one before it, from the entry
/// numbered `seq` whose hash is `prev`.
fn check_chain_from(path: &Path, rows: &[Row], seq: u64, prev: &str) -> Result<(), Failure> {
    let mut prev = prev;
    for (at, row) in rows.iter().enumerate() {
        if row.prev != prev {
            return Err(altered(
                path,
                row.seq,
                "does not follow the entry before it: its previous hash does not match",
            ));
        }
        if row.hash != entry::hash(&row.prev, &row.signed, row.signature.as_deref()) {
            return Err(altered(path, row.seq, "does not match its hash"));
        }
        if row.seq != seq + at as u64 + 1 {
            return Err(altered(path, row.seq, "is numbered out of order"));
        }
        prev = &row.hash;
    }
    Ok(())
}

/// The views' head, `(seq, hash, form)`, where every view table is there
/// and the head can be read.
fn head(conn: &Connection) -> Result<Option<(u64, String, i64)>, Failure> {
    for table in VIEW_TABLES {
        let found: Option<i64> = conn
            .query_row(
                "SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = ?1",
                [table],
                |row| row.get(0),
            )
            .optional()?;
        if found.is_none() {
            return Ok(None);
        }
    }
    let read = conn
        .query_row("SELECT seq, hash, form FROM views_head", [], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i64>(2)?,
            ))
        })
        .optional();
    match read {
        Ok(Some((seq, hash, form))) => Ok(Some((u64::try_from(seq).unwrap_or(0), hash, form))),
        // A head of an older form, with no `form`, is built again.
        Ok(None) | Err(_) => Ok(None),
    }
}

/// The entry the views were built at, `(seq, hash)`.
fn views_head(conn: &Connection) -> Result<(u64, String), Failure> {
    let (seq, hash): (i64, String) =
        conn.query_row("SELECT seq, hash FROM views_head", [], |row| {
            Ok((row.get(0)?, row.get(1)?))
        })?;
    Ok((u64::try_from(seq).unwrap_or(0), hash))
}

/// Whether the views are of this form and at the log's last entry, read
/// from that entry alone; not run where that entry is the views' head
/// and no longer hashes to what they name.
fn fresh(conn: &Connection, path: &Path) -> Result<bool, Failure> {
    let Some((seq, hash, form)) = head(conn)? else {
        return Ok(false);
    };
    if form != VIEWS_FORM {
        return Ok(false);
    }
    let last = last_row(conn)?;
    match &last {
        Some(row) if row.seq == seq => {
            check_head(path, last.as_ref(), seq, &hash)?;
            Ok(true)
        }
        None if seq == 0 => {
            check_head(path, None, seq, &hash)?;
            Ok(true)
        }
        _ => Ok(false),
    }
}

/// Brings the views to the log's last entry, inside a write's
/// transaction, and answers the part of the state they then hold that
/// `scope` and the entries applied name. Views missing or of
/// another form are built again from the whole log, its whole chain
/// checked; otherwise the views' head entry is checked, and each entry
/// after it is checked, judged and applied, its rows written.
fn catch_up(conn: &Connection, path: &Path, mut scope: Scope) -> Result<State, Failure> {
    let Some((seq, hash, form)) = head(conn)?.filter(|(_, _, form)| *form == VIEWS_FORM) else {
        let rows = read_log(conn)?;
        check_chain(path, &rows)?;
        let (state, judgments) = replay(conn, &rows);
        write_views(conn, &state, &judgments, &rows)?;
        return Ok(state);
    };
    debug_assert_eq!(form, VIEWS_FORM);
    let at = if seq == 0 { None } else { row_at(conn, seq)? };
    check_head(path, at.as_ref(), seq, &hash)?;
    let rows = rows_after(conn, seq)?;
    check_chain_from(path, &rows, seq, &hash)?;
    let parsed: Vec<Entry> = rows
        .iter()
        .filter_map(|row| Entry::parse(&row.signed).ok())
        .collect();
    scope.join(Scope::of(&parsed));
    let mut state = load_state_for(conn, &scope)?;
    for row in &rows {
        let judgment = judge::step(&mut state, row.seq, &row.signed, row.signature.as_deref());
        write_judgment(conn, row.seq, &judgment)?;
        if judgment.counted {
            let entry = Entry::parse(&row.signed).map_err(Failure::not_run)?;
            read_body_of(conn, &mut state, &entry);
            write_touched(conn, &state, &entry)?;
        }
        write_head(conn, row.seq, &row.hash)?;
    }
    Ok(state)
}

/// Writes the view rows an entry that counts touched.
fn write_touched(conn: &Connection, state: &State, entry: &Entry) -> Result<(), Failure> {
    match &entry.act {
        Act::Registered { person, .. } => write_person(conn, &state.persons[person]),
        Act::Proposed { claim, .. } => {
            write_claim(conn, holding(state, &claim.name, &claim.version))
        }
        Act::Accepted { versions } => {
            for named in versions {
                write_claim(conn, holding(state, &named.name, &named.version))?;
            }
            Ok(())
        }
        Act::Rejected { name, version, .. } => write_claim(conn, holding(state, name, version)),
        Act::Repository { name, .. } => write_repository(conn, &state.repositories[name]),
        Act::Noted {
            name,
            version,
            paragraph,
            words,
            ..
        } => {
            let person = match &entry.actor {
                Actor::Person(person) => person.as_str(),
                Actor::Agent { .. } => "",
            };
            let note = crate::narrative::note_name(name, version, *paragraph, words, person);
            write_note(conn, &state.notes[&note])
        }
    }
}

fn write_note(conn: &Connection, note: &judge::Note) -> Result<(), Failure> {
    conn.execute(
        "INSERT OR REPLACE INTO notes (name, narrative, version, paragraph, fingerprint, words, \
         person, at, noted_in) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            note.name,
            note.narrative,
            note.version,
            sql_int(note.paragraph),
            note.fingerprint,
            note.words,
            note.person,
            sql_int(note.at),
            sql_int(note.noted_in)
        ],
    )?;
    Ok(())
}

/// Reads into `state` the notes `filter` selects.
fn load_notes<P: rusqlite::Params>(
    conn: &Connection,
    state: &mut State,
    filter: &str,
    params: P,
) -> Result<(), Failure> {
    let mut statement = conn.prepare_cached(&format!(
        "SELECT name, narrative, version, paragraph, fingerprint, words, person, at, noted_in \
         FROM notes {filter}"
    ))?;
    let rows = statement
        .query_map(params, |row| {
            let number = |at: usize| row.get::<_, i64>(at).map(|n| u64::try_from(n).unwrap_or(0));
            Ok(judge::Note {
                name: row.get(0)?,
                narrative: row.get(1)?,
                version: row.get(2)?,
                paragraph: number(3)?,
                fingerprint: row.get(4)?,
                words: row.get(5)?,
                person: row.get(6)?,
                at: number(7)?,
                noted_in: number(8)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    for note in rows {
        state.notes.insert(note.name.clone(), note);
    }
    Ok(())
}

/// The claim of `state` that holds `name@version`, an entry that counts
/// named.
fn holding<'a>(state: &'a State, name: &str, version: &str) -> &'a Claim {
    state
        .holding(name, version)
        .expect("a counted entry's version is in the state")
}

/// A view table's rows, each as its columns' text, in order; none where
/// the table is missing.
fn snapshot(conn: &Connection, table: &str) -> Result<Option<Vec<Vec<String>>>, Failure> {
    let Ok(mut statement) = conn.prepare(&format!("SELECT * FROM {table}")) else {
        return Ok(None);
    };
    let columns = statement.column_count();
    let mut rows = statement
        .query_map([], |row| {
            (0..columns)
                .map(|at| {
                    Ok(match row.get_ref(at)? {
                        rusqlite::types::ValueRef::Null => "null".to_owned(),
                        rusqlite::types::ValueRef::Integer(value) => value.to_string(),
                        rusqlite::types::ValueRef::Real(value) => value.to_string(),
                        rusqlite::types::ValueRef::Text(text) => {
                            format!("'{}", String::from_utf8_lossy(text))
                        }
                        rusqlite::types::ValueRef::Blob(bytes) => format!("{bytes:?}"),
                    })
                })
                .collect::<rusqlite::Result<Vec<_>>>()
        })?
        .collect::<Result<Vec<_>, _>>()?;
    rows.sort();
    Ok(Some(rows))
}

/// Checks each entry's `prev` against the hash of the one before it, and
/// its `hash` against its words: not run, naming the first entry where
/// either fails (record-log, "A changed or removed entry is found").
fn check_chain(path: &Path, rows: &[Row]) -> Result<(), Failure> {
    let mut prev = GENESIS;
    for (at, row) in rows.iter().enumerate() {
        let altered = |what: &str| {
            Failure::not_run(format!(
                "the store at {} is altered: entry {} {what}",
                path.display(),
                row.seq
            ))
        };
        if row.prev != prev {
            return Err(altered(
                "does not follow the entry before it: its previous hash does not match",
            ));
        }
        if row.hash != entry::hash(&row.prev, &row.signed, row.signature.as_deref()) {
            return Err(altered("does not match its hash"));
        }
        if row.seq != at as u64 + 1 {
            return Err(altered("is numbered out of order"));
        }
        prev = &row.hash;
    }
    Ok(())
}

/// Replays `rows` through the judge, reading the body of each narrative
/// proposed for the notes after it.
fn replay(conn: &Connection, rows: &[Row]) -> (State, Vec<(u64, Judgment)>) {
    judge::replay_with(
        rows.iter()
            .map(|row| (row.seq, row.signed.as_str(), row.signature.as_deref())),
        |state, entry| read_body_of(conn, state, entry),
    )
}

/// Where `entry` proposes a version of a narrative, reads its body's
/// paragraphs into `state`, for the judge to check a note on it.
fn read_body_of(conn: &Connection, state: &mut State, entry: &Entry) {
    if let Act::Proposed { claim, .. } = &entry.act {
        if claim.kind == Kind::Narrative {
            if let Some(body) = &claim.body {
                read_paragraphs(conn, state, body);
            }
        }
    }
}

/// Reads the paragraphs of the body `fingerprint` into `state`, where the
/// store holds it as text; a body it does not hold has none, and a note on
/// it is refused.
fn read_paragraphs(conn: &Connection, state: &mut State, fingerprint: &str) {
    if state.paragraphs.contains_key(fingerprint) {
        return;
    }
    if let Some(text) = blob(conn, fingerprint)
        .ok()
        .and_then(|bytes| String::from_utf8(bytes).ok())
    {
        state.paragraphs.insert(
            fingerprint.to_owned(),
            crate::narrative::fingerprints(&text),
        );
    }
}

/// Reads the paragraphs of each version of each narrative of `state`.
fn read_all_paragraphs(conn: &Connection, state: &mut State) {
    let bodies: Vec<String> = state
        .claims
        .values()
        .filter(|claim| claim.kind == Kind::Narrative)
        .flat_map(|claim| claim.versions.iter())
        .filter_map(|version| version.proposal.body.clone())
        .collect();
    for body in bodies {
        read_paragraphs(conn, state, &body);
    }
}

/// Drops the views and writes them again from a replay of the log.
fn write_views(
    conn: &Connection,
    state: &State,
    judgments: &[(u64, Judgment)],
    rows: &[Row],
) -> Result<(), Failure> {
    for table in VIEW_TABLES {
        conn.execute_batch(&format!("DROP TABLE IF EXISTS {table}"))?;
    }
    conn.execute_batch(VIEWS)?;
    for person in state.persons.values() {
        write_person(conn, person)?;
    }
    for claim in state.claims.values() {
        write_claim(conn, claim)?;
    }
    for repository in state.repositories.values() {
        write_repository(conn, repository)?;
    }
    for note in state.notes.values() {
        write_note(conn, note)?;
    }
    for (seq, judgment) in judgments {
        write_judgment(conn, *seq, judgment)?;
    }
    let (seq, hash) = rows
        .last()
        .map_or((0, GENESIS), |row| (row.seq, row.hash.as_str()));
    write_head(conn, seq, hash)
}

fn write_head(conn: &Connection, seq: u64, hash: &str) -> Result<(), Failure> {
    conn.execute("DELETE FROM views_head", [])?;
    conn.execute(
        "INSERT INTO views_head (seq, hash, form) VALUES (?1, ?2, ?3)",
        params![sql_int(seq), hash, VIEWS_FORM],
    )?;
    Ok(())
}

fn write_judgment(conn: &Connection, seq: u64, judgment: &Judgment) -> Result<(), Failure> {
    conn.execute(
        "INSERT OR REPLACE INTO judgments (seq, counted, reason) VALUES (?1, ?2, ?3)",
        params![sql_int(seq), judgment.counted, judgment.reason],
    )?;
    Ok(())
}

fn write_repository(conn: &Connection, repository: &judge::Repository) -> Result<(), Failure> {
    conn.execute(
        "INSERT OR REPLACE INTO repositories (name, remote, session, set_by, set_in) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            repository.name,
            repository.remote,
            repository.session,
            repository.set_by,
            sql_int(repository.set_in)
        ],
    )?;
    Ok(())
}

/// The repositories the views hold, by name (repository-settings).
pub fn repositories(conn: &Connection) -> Result<Vec<judge::Repository>, Failure> {
    let mut rows = conn
        .prepare("SELECT name, remote, session, set_by, set_in FROM repositories ORDER BY name")?;
    let found = rows
        .query_map([], |row| {
            Ok(judge::Repository {
                name: row.get(0)?,
                remote: row.get(1)?,
                session: row.get(2)?,
                set_by: row.get(3)?,
                set_in: u64::try_from(row.get::<_, i64>(4)?).unwrap_or(0),
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(found)
}

fn write_person(conn: &Connection, person: &Person) -> Result<(), Failure> {
    conn.execute(
        "INSERT OR REPLACE INTO persons (name, key, owner, registered_in) VALUES (?1, ?2, ?3, ?4)",
        params![
            person.name,
            crate::sign::key_line(&person.key),
            person.owner,
            sql_int(person.registered_in)
        ],
    )?;
    Ok(())
}

fn to_json<T: serde::Serialize>(value: &T) -> String {
    serde_json::to_string(value).expect("a view's field is JSON")
}

fn write_claim(conn: &Connection, claim: &Claim) -> Result<(), Failure> {
    let term = claim.repository.as_deref().unwrap_or_default();
    conn.execute(
        "DELETE FROM versions WHERE name = ?1 AND term = ?2",
        [&claim.name, term],
    )?;
    for table in ["edges", "paths", "bindings"] {
        conn.execute(
            &format!("DELETE FROM {table} WHERE name = ?1 AND term = ?2"),
            [&claim.name, term],
        )?;
    }
    conn.execute(
        "INSERT OR REPLACE INTO claims (name, term, kind, standing) VALUES (?1, ?2, ?3, ?4)",
        params![claim.name, term, claim.kind.as_str(), claim.standing],
    )?;
    for version in &claim.versions {
        conn.execute(
            "INSERT INTO versions (name, term, version, base, statement, scenarios, body, \
             proposer, proposed_in, status, decided_by, decided_in, reason, repositories, \
             contexts, imported, proposal) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)",
            params![
                claim.name,
                term,
                version.proposal.version,
                version.proposal.base,
                version.proposal.statement,
                to_json(&version.proposal.scenarios),
                version.proposal.body,
                to_json(&version.proposer),
                sql_int(version.proposed_in),
                version.status.as_str(),
                version.decided_by,
                version.decided_in.map(sql_int),
                version.reason,
                to_json(&version.repositories),
                to_json(&version.contexts),
                version.imported.as_ref().map(to_json),
                to_json(&version.proposal),
            ],
        )?;
        write_edges(conn, &claim.name, term, &version.proposal)?;
        write_scope(conn, &claim.name, term, version)?;
    }
    Ok(())
}

/// A row for each glob of a context version, with its repository and its
/// pattern, and a row for each context a version binds, with the version
/// of the context that the binding meant (context-claims).
fn write_scope(
    conn: &Connection,
    name: &str,
    term: &str,
    version: &Version,
) -> Result<(), Failure> {
    let mut path = conn.prepare_cached(
        "INSERT INTO paths (name, term, version, position, repository, pattern) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
    )?;
    for (position, glob) in version.proposal.paths.iter().enumerate() {
        let (repository, pattern) = glob.split_once(':').unwrap_or(("", glob));
        path.execute(params![
            name,
            term,
            version.proposal.version,
            sql_int(position as u64),
            repository,
            pattern,
        ])?;
    }
    let mut binding = conn.prepare_cached(
        "INSERT INTO bindings (name, term, version, context, context_version) \
         VALUES (?1, ?2, ?3, ?4, ?5)",
    )?;
    for stored in &version.contexts {
        let (context, meant) = stored.split_once('@').unwrap_or((stored, ""));
        binding.execute(params![
            name,
            term,
            version.proposal.version,
            context,
            meant
        ])?;
    }
    Ok(())
}

/// The contexts the version `name@version` binds, each
/// `<name>@<version>`; none where the views hold no such version.
pub fn contexts_of(conn: &Connection, name: &str, version: &str) -> Result<Vec<String>, Failure> {
    let contexts: Option<String> = conn
        .query_row(
            "SELECT contexts FROM versions WHERE name = ?1 AND version = ?2",
            [name, version],
            |row| row.get(0),
        )
        .optional()?;
    contexts.as_deref().map_or(Ok(Vec::new()), from_json)
}

/// The globs of the accepted version of each context that name
/// `repository`, as `(name, version, pattern)`, in the order of the
/// contexts' names and then their globs.
pub fn accepted_globs(
    conn: &Connection,
    repository: &str,
) -> Result<Vec<(String, String, String)>, Failure> {
    let mut statement = conn.prepare_cached(
        "SELECT p.name, p.version, p.pattern FROM paths p \
         JOIN claims c ON c.name = p.name AND c.term = p.term AND c.standing = p.version \
         WHERE p.repository = ?1 ORDER BY p.name, p.position",
    )?;
    let rows = statement
        .query_map([repository], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

/// The names of the contexts whose accepted version has a glob in
/// `repository`, sorted.
pub fn contexts_in(conn: &Connection, repository: &str) -> Result<Vec<String>, Failure> {
    let mut names: Vec<String> = accepted_globs(conn, repository)?
        .into_iter()
        .map(|(name, _, _)| name)
        .collect();
    names.dedup();
    Ok(names)
}

/// A claim whose accepted version binds a context, as the views hold it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bound {
    /// The claim.
    pub name: String,
    /// Its accepted version.
    pub version: String,
    /// The version of the context that the binding meant.
    pub meant: String,
}

/// Each claim whose accepted version binds the context `context`, in the
/// order of their names (context-claims, "boom show gives a context and a
/// binding").
pub fn bound_to(conn: &Connection, context: &str) -> Result<Vec<Bound>, Failure> {
    let mut statement = conn.prepare(
        "SELECT b.name, b.version, b.context_version FROM bindings b \
         JOIN claims c ON c.name = b.name AND c.term = b.term AND c.standing = b.version \
         WHERE b.context = ?1 ORDER BY b.name, b.term",
    )?;
    let rows = statement
        .query_map([context], |row| {
            Ok(Bound {
                name: row.get(0)?,
                version: row.get(1)?,
                meant: row.get(2)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

/// A row for each edge of a version, in its order: each `source` and
/// `changes` with the claim and the version it meant, each `origin` with
/// its form and, for a file, its fingerprint (claim-edges). Its base is
/// the `base` of its row in `versions`.
fn write_edges(
    conn: &Connection,
    name: &str,
    term: &str,
    proposal: &Proposal,
) -> Result<(), Failure> {
    let mut insert = conn.prepare_cached(
        "INSERT INTO edges (name, term, version, edge, position, target_name, target_version, \
         artifact, origin) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
    )?;
    for (edge, edges) in [
        ("source", &proposal.source),
        ("changes", &proposal.changes),
        ("includes", &proposal.includes),
    ] {
        for (position, stored) in edges.iter().enumerate() {
            let (target_name, target_version) = stored.split_once('@').unwrap_or((stored, ""));
            insert.execute(params![
                name,
                term,
                proposal.version,
                edge,
                sql_int(position as u64),
                target_name,
                target_version,
                None::<String>,
                None::<String>,
            ])?;
        }
    }
    for (position, origin) in proposal.origin.iter().enumerate() {
        let artifact = match origin {
            Origin::File { fingerprint, .. } => Some(fingerprint.as_str()),
            Origin::Git { .. } => None,
        };
        insert.execute(params![
            name,
            term,
            proposal.version,
            "origin",
            sql_int(position as u64),
            None::<String>,
            None::<String>,
            artifact,
            to_json(origin),
        ])?;
    }
    Ok(())
}

/// A version whose `source` or `changes` names a claim, as the views
/// hold it (claim-edges, "boom show gives the claims that rest on a
/// claim").
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resting {
    /// The edge: `source` or `changes`.
    pub edge: String,
    /// The claim whose version holds the edge.
    pub name: String,
    /// That version.
    pub version: String,
    /// The version of the named claim that the edge meant.
    pub meant: String,
}

/// Each edge `source` or `changes` to the claim `name` held by a version
/// that counts for its claim: its accepted version, or its newest where
/// none is accepted. In the order of the claims' names, then their
/// versions, then the edges.
pub fn resting_on(conn: &Connection, name: &str) -> Result<Vec<Resting>, Failure> {
    let mut statement = conn.prepare(
        "SELECT e.edge, e.name, e.version, e.target_version FROM edges e \
         JOIN claims c ON c.name = e.name AND c.term = e.term \
         WHERE e.target_name = ?1 AND e.version = COALESCE(c.standing, \
           (SELECT v.version FROM versions v WHERE v.name = c.name AND v.term = c.term \
            ORDER BY v.proposed_in DESC LIMIT 1)) \
         ORDER BY e.name, e.term, e.edge DESC, e.position",
    )?;
    let rows = statement
        .query_map([name], |row| {
            Ok(Resting {
                edge: row.get(0)?,
                name: row.get(1)?,
                version: row.get(2)?,
                meant: row.get(3)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

/// A version as an edge meant it: its statement, and the accepted version
/// of its claim now, where one is; none where the views hold no such
/// version.
pub fn meant(
    conn: &Connection,
    name: &str,
    version: &str,
) -> Result<Option<(String, Option<String>)>, Failure> {
    Ok(conn
        .query_row(
            "SELECT v.statement, c.standing FROM versions v \
             JOIN claims c ON c.name = v.name AND c.term = v.term \
             WHERE v.name = ?1 AND v.version = ?2",
            [name, version],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?)
}

/// The name of the file an origin edge keeps under `fingerprint`, where
/// one does.
pub fn artifact_name(conn: &Connection, fingerprint: &str) -> Result<Option<String>, Failure> {
    let origin: Option<String> = conn
        .query_row(
            "SELECT origin FROM edges WHERE artifact = ?1 ORDER BY name, version, position LIMIT 1",
            [fingerprint],
            |row| row.get(0),
        )
        .optional()?;
    match origin.as_deref().map(from_json::<Origin>).transpose()? {
        Some(Origin::File { name, .. }) => Ok(Some(name)),
        _ => Ok(None),
    }
}

/// The entry the views were built at, `(number, hash)`: the log's last
/// entry inside [`Store::read`].
pub fn head_entry(conn: &Connection) -> Result<(u64, String), Failure> {
    views_head(conn)
}

/// The state the views hold, built from `persons`, `claims` and
/// `versions` with no entry of the log read (design 1). It is the state a
/// replay of the log up to the views' head builds.
pub fn load_state(conn: &Connection) -> Result<State, Failure> {
    let mut state = load_persons(conn)?;
    load_claims(conn, &mut state, "", [])?;
    load_notes(conn, &mut state, "", [])?;
    read_all_paragraphs(conn, &mut state);
    Ok(state)
}

/// The part of the state the views hold that `scope` names: every
/// person, the claims it names, and, where it asks, every claim with an
/// imported version. No other claim's rows are read.
pub fn load_state_for(conn: &Connection, scope: &Scope) -> Result<State, Failure> {
    let mut state = load_persons(conn)?;
    for name in &scope.names {
        load_claims(conn, &mut state, "WHERE name = ?1", [name.as_str()])?;
    }
    if scope.imported {
        load_claims(
            conn,
            &mut state,
            "WHERE name IN (SELECT DISTINCT name FROM versions WHERE imported IS NOT NULL)",
            [],
        )?;
    }
    // The notes on each narrative named, and each note an answer names
    // (narrative-notes).
    for name in &scope.names {
        load_notes(conn, &mut state, "WHERE narrative = ?1", [name.as_str()])?;
    }
    for note in &scope.notes {
        load_notes(conn, &mut state, "WHERE name = ?1", [note.as_str()])?;
    }
    read_all_paragraphs(conn, &mut state);
    Ok(state)
}

/// Each narrative of the store with its versions, and every note
/// (narrative-page, "The list of narratives").
pub fn narratives(conn: &Connection) -> Result<State, Failure> {
    let mut state = load_persons(conn)?;
    load_claims(conn, &mut state, "WHERE kind = 'narrative'", [])?;
    load_notes(conn, &mut state, "", [])?;
    Ok(state)
}

/// Each claim's accepted version, with the claim, for `Some(repository)`
/// those that bind it, a context named in `contexts` or the whole system,
/// in the order of [`State::standing_for`]: no other version is read.
pub fn accepted_for(
    conn: &Connection,
    repository: Option<&str>,
    contexts: &[String],
) -> Result<Vec<(Claim, Version)>, Failure> {
    let mut state = State::default();
    load_claims(conn, &mut state, "WHERE standing IS NOT NULL", [])?;
    // Only each claim's accepted version is read: load_claims reads the
    // versions of the claims it loads, which a join narrows to those.
    Ok(state
        .standing_for(repository, contexts)
        .into_iter()
        .map(|(claim, version)| {
            let mut claim = claim.clone();
            claim.versions.clear();
            (claim, version.clone())
        })
        .collect())
}

fn load_persons(conn: &Connection) -> Result<State, Failure> {
    let mut state = State::default();
    let mut persons = conn.prepare("SELECT name, key, owner, registered_in FROM persons")?;
    let rows = persons
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, bool>(2)?,
                row.get::<_, i64>(3)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    let mut first: Option<(u64, String)> = None;
    for (name, key, owner, registered_in) in rows {
        let key = ssh_key::PublicKey::from_openssh(&key)
            .map_err(|error| Failure::not_run(format!("a view could not be read: {error}")))?;
        let registered_in = u64::try_from(registered_in).unwrap_or(0);
        if first.as_ref().is_none_or(|(seq, _)| registered_in < *seq) {
            first = Some((registered_in, name.clone()));
        }
        state.persons.insert(
            name.clone(),
            Person {
                name,
                key,
                owner,
                registered_in,
            },
        );
    }
    state.first_owner = first.map(|(_, name)| name);
    for repository in repositories(conn)? {
        state
            .repositories
            .insert(repository.name.clone(), repository);
    }
    Ok(state)
}

/// Reads into `state` the claims `filter` selects from `claims`, with
/// their versions; with `WHERE standing IS NOT NULL`, each one's accepted
/// version alone.
fn load_claims<P: rusqlite::Params + Copy>(
    conn: &Connection,
    state: &mut State,
    filter: &str,
    params: P,
) -> Result<(), Failure> {
    let mut claims = conn.prepare(&format!(
        "SELECT name, term, kind, standing FROM claims {filter}"
    ))?;
    let rows = claims
        .query_map(params, |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, Option<String>>(3)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    let accepted_only = filter.contains("standing IS NOT NULL");
    for (name, term, kind, standing) in rows {
        let repository = Some(term.clone()).filter(|term| !term.is_empty());
        let key = judge::key(&name, repository.as_deref());
        if state.claims.contains_key(&key) {
            continue;
        }
        let kind: Kind = serde_json::from_value(serde_json::Value::String(kind))
            .map_err(|error| Failure::not_run(format!("a view could not be read: {error}")))?;
        let versions = if accepted_only {
            read_versions(
                conn,
                "WHERE name = ?1 AND term = ?2 AND version = ?3",
                [
                    name.as_str(),
                    term.as_str(),
                    standing.as_deref().unwrap_or_default(),
                ],
            )?
        } else {
            read_versions(
                conn,
                "WHERE name = ?1 AND term = ?2",
                [name.as_str(), term.as_str()],
            )?
        };
        state.claims.insert(
            key,
            Claim {
                name,
                kind,
                repository,
                standing,
                versions,
            },
        );
    }
    Ok(())
}

fn read_versions<P: rusqlite::Params>(
    conn: &Connection,
    filter: &str,
    params: P,
) -> Result<Vec<Version>, Failure> {
    let mut versions = conn.prepare_cached(&format!(
        "SELECT proposer, proposed_in, status, decided_by, decided_in, reason, \
         repositories, contexts, imported, proposal FROM versions {filter} ORDER BY proposed_in"
    ))?;
    let rows = versions
        .query_map(params, |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, Option<i64>>(4)?,
                row.get::<_, Option<String>>(5)?,
                (row.get::<_, String>(6)?, row.get::<_, String>(7)?),
                row.get::<_, Option<String>>(8)?,
                row.get::<_, String>(9)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    VERSIONS_READ.with(|read| read.set(read.get() + rows.len() as u64));
    rows.into_iter()
        .map(
            |(
                proposer,
                proposed_in,
                status,
                decided_by,
                decided_in,
                reason,
                (repositories, contexts),
                imported,
                proposal,
            )| {
                Ok(Version {
                    proposal: from_json::<Proposal>(&proposal)?,
                    repositories: from_json(&repositories)?,
                    contexts: from_json(&contexts)?,
                    proposer: from_json(&proposer)?,
                    proposed_in: u64::try_from(proposed_in).unwrap_or(0),
                    status: Status::from_name(&status).ok_or_else(|| {
                        Failure::not_run(format!("a view could not be read: no status {status}"))
                    })?,
                    decided_by,
                    decided_in: decided_in.and_then(|seq| u64::try_from(seq).ok()),
                    reason,
                    imported: imported.as_deref().map(from_json).transpose()?,
                })
            },
        )
        .collect()
}

/// An entry of the log as `boom log` gives it: as stored, with its
/// judgment.
#[derive(Debug, Clone)]
pub struct LogEntry {
    /// The entry as stored.
    pub row: Row,
    /// Whether it counts.
    pub counted: bool,
    /// Why it does not.
    pub reason: Option<String>,
}

/// The log's entries in order, each with its judgment, the whole chain
/// checked as they are read: not run, naming the first entry whose
/// chain fails, where it does not hold.
pub fn log_entries(conn: &Connection, path: &Path) -> Result<Vec<LogEntry>, Failure> {
    let mut judgments = conn.prepare("SELECT counted, reason FROM judgments WHERE seq = ?1")?;
    let rows = read_log(conn)?;
    check_chain(path, &rows)?;
    rows.into_iter()
        .map(|row| {
            let (counted, reason) = judgments
                .query_row([sql_int(row.seq)], |found| {
                    Ok((found.get::<_, bool>(0)?, found.get::<_, Option<String>>(1)?))
                })
                .optional()?
                .unwrap_or((false, Some("not judged".to_owned())));
            Ok(LogEntry {
                row,
                counted,
                reason,
            })
        })
        .collect()
}

/// A claim as the views hold it.
#[derive(Debug, Clone)]
pub struct ClaimView {
    /// Its name.
    pub name: String,
    /// Its kind.
    pub kind: String,
    /// A term's repository; none for the other kinds.
    pub repository: Option<String>,
    /// Its standing version.
    pub standing: Option<String>,
    /// Its versions in the order proposed.
    pub versions: Vec<VersionView>,
}

/// A version as the views hold it.
#[derive(Debug, Clone)]
pub struct VersionView {
    /// The version.
    pub version: String,
    /// The version it amends.
    pub base: Option<String>,
    /// Its statement.
    pub statement: String,
    /// Its scenarios.
    pub scenarios: Vec<String>,
    /// Its body's fingerprint.
    pub body: Option<String>,
    /// The old words a term's version replaces.
    pub replaces: Vec<String>,
    /// The claims it rests on, each `<name>@<version>` (claim-edges).
    pub source: Vec<String>,
    /// The claims a decision changes, each `<name>@<version>`.
    pub changes: Vec<String>,
    /// The files and commits it came from.
    pub origin: Vec<Origin>,
    /// The claims a narrative includes, each `<name>@<version>`
    /// (narratives).
    pub includes: Vec<String>,
    /// The notes a version of a narrative answers, with its answers
    /// (narrative-notes).
    pub answers: Vec<crate::claim::Answer>,
    /// The globs of a context version (context-claims).
    pub paths: Vec<String>,
    /// The repositories its proposal binds.
    pub repositories: Vec<String>,
    /// The contexts its proposal binds, each `<name>@<version>` of the
    /// version the binding meant. With no repository and no context, it
    /// binds the whole system.
    pub contexts: Vec<String>,
    /// Who proposed it.
    pub proposer: Actor,
    /// The entry that proposed it.
    pub proposed_in: u64,
    /// Its status.
    pub status: String,
    /// The person who accepted or rejected it.
    pub decided_by: Option<String>,
    /// The entry that accepted or rejected it.
    pub decided_in: Option<u64>,
    /// Why it was rejected.
    pub reason: Option<String>,
}

fn from_json<T: serde::de::DeserializeOwned>(text: &str) -> Result<T, Failure> {
    serde_json::from_str(text)
        .map_err(|error| Failure::not_run(format!("a view could not be read: {error}")))
}

/// The claims named `name`: none, one, or for a word with a term in
/// several repositories one for each, in the order of their
/// repositories.
pub fn claim_views(conn: &Connection, name: &str) -> Result<Vec<ClaimView>, Failure> {
    let mut claims =
        conn.prepare("SELECT term, kind, standing FROM claims WHERE name = ?1 ORDER BY term")?;
    let found = claims
        .query_map([name], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<String>>(2)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    let mut views = Vec::with_capacity(found.len());
    for (term, kind, standing) in found {
        views.push(ClaimView {
            name: name.to_owned(),
            versions: version_views(conn, name, &term)?,
            repository: Some(term).filter(|term| !term.is_empty()),
            kind,
            standing,
        });
    }
    Ok(views)
}

fn version_views(conn: &Connection, name: &str, term: &str) -> Result<Vec<VersionView>, Failure> {
    let mut statement = conn.prepare(
        "SELECT version, base, statement, scenarios, body, proposer, proposed_in, status, \
         decided_by, decided_in, reason, repositories, contexts, proposal FROM versions \
         WHERE name = ?1 AND term = ?2 ORDER BY proposed_in",
    )?;
    let rows = statement
        .query_map([name, term], |row| {
            Ok((
                (
                    row.get::<_, String>(0)?,
                    row.get::<_, Option<String>>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, Option<String>>(4)?,
                    row.get::<_, String>(5)?,
                ),
                (
                    row.get::<_, i64>(6)?,
                    row.get::<_, String>(7)?,
                    row.get::<_, Option<String>>(8)?,
                    row.get::<_, Option<i64>>(9)?,
                    row.get::<_, Option<String>>(10)?,
                    (row.get::<_, String>(11)?, row.get::<_, String>(12)?),
                    row.get::<_, String>(13)?,
                ),
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    let mut versions = Vec::with_capacity(rows.len());
    for (
        (version, base, words, scenarios, body, proposer),
        (proposed_in, status, by, decided_in, reason, (repositories, contexts), proposal),
    ) in rows
    {
        let proposal = from_json::<Proposal>(&proposal)?;
        versions.push(VersionView {
            version,
            base,
            statement: words,
            scenarios: from_json(&scenarios)?,
            body,
            replaces: proposal.replaces,
            source: proposal.source,
            changes: proposal.changes,
            origin: proposal.origin,
            includes: proposal.includes,
            answers: proposal.answers,
            paths: proposal.paths,
            repositories: from_json(&repositories)?,
            contexts: from_json(&contexts)?,
            proposer: from_json(&proposer)?,
            proposed_in: u64::try_from(proposed_in).unwrap_or(0),
            status,
            decided_by: by,
            decided_in: decided_in.and_then(|seq| u64::try_from(seq).ok()),
            reason,
        });
    }
    Ok(versions)
}

/// The signed part of the entry numbered `seq`, where there is one.
pub fn signed_at(conn: &Connection, seq: u64) -> Result<Option<String>, Failure> {
    Ok(row_at(conn, seq)?.map(|row| row.signed))
}

/// The bytes of the blob named `fingerprint`, checked against it: not
/// run where they do not hash to it, or there are none.
pub fn blob(conn: &Connection, fingerprint: &str) -> Result<Vec<u8>, Failure> {
    let bytes: Option<Vec<u8>> = conn
        .query_row(
            "SELECT bytes FROM blobs WHERE fingerprint = ?1",
            [fingerprint],
            |row| row.get(0),
        )
        .optional()?;
    match bytes {
        Some(bytes) if crate::sha256_hex(&bytes) == fingerprint => Ok(bytes),
        Some(_) => Err(Failure::not_run(format!(
            "the blob {fingerprint} is unreadable: its bytes no longer match its fingerprint"
        ))),
        None => Err(Failure::not_run(format!(
            "the blob {fingerprint} is unreadable: the store does not hold it"
        ))),
    }
}
