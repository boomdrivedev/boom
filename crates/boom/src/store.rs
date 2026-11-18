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

use crate::entry::{self, Act, Actor, Entry, GENESIS};
use crate::judge::{self, Claim, Judgment, Person, Proof, State, Verdict};
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
    hash TEXT NOT NULL
);
CREATE TABLE persons (
    name TEXT PRIMARY KEY,
    key TEXT NOT NULL,
    owner INTEGER NOT NULL,
    registered_in INTEGER NOT NULL
);
CREATE TABLE claims (
    name TEXT PRIMARY KEY,
    kind TEXT NOT NULL,
    standing TEXT
);
CREATE TABLE versions (
    name TEXT NOT NULL,
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
    PRIMARY KEY (name, version)
);
CREATE TABLE judgments (
    seq INTEGER PRIMARY KEY,
    counted INTEGER NOT NULL,
    reason TEXT
);
";

/// The views' tables, each dropped and made again by a rebuild.
const VIEW_TABLES: [&str; 5] = ["views_head", "persons", "claims", "versions", "judgments"];

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

    /// Checks the log's chain and brings the views to its last entry,
    /// then answers from them with `answer`, in one read of the store.
    pub fn read<T>(
        &mut self,
        answer: impl FnOnce(&Connection) -> Result<T, Failure>,
    ) -> Result<T, Failure> {
        let path = self.path.clone();
        {
            let tx = self
                .conn
                .transaction_with_behavior(TransactionBehavior::Deferred)?;
            let rows = read_log(&tx)?;
            check_chain(&path, &rows)?;
            if views_fresh(&tx, &rows)? {
                let answered = answer(&tx);
                tx.commit()?;
                return answered;
            }
        }
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let rows = read_log(&tx)?;
        check_chain(&path, &rows)?;
        if !views_fresh(&tx, &rows)? {
            let (state, judgments) = replay(&rows);
            write_views(&tx, &state, &judgments, &rows)?;
        }
        let answered = answer(&tx);
        tx.commit()?;
        answered
    }

    /// The state the log's entries build, its chain checked.
    pub fn state(&mut self) -> Result<State, Failure> {
        let path = self.path.clone();
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Deferred)?;
        let rows = read_log(&tx)?;
        check_chain(&path, &rows)?;
        tx.commit()?;
        Ok(replay(&rows).0)
    }

    /// Appends the entry whose signed part is `signed`, with its
    /// signature, and `blob`, a body it names, in one `IMMEDIATE`
    /// transaction that judges it again and updates the views with it.
    /// Answers what it wrote and the state after it.
    pub fn append(
        &mut self,
        signed: &str,
        signature: Option<&str>,
        blob: Option<&[u8]>,
    ) -> Result<(Written, State), Failure> {
        let entry = Entry::parse(signed).map_err(Failure::Refused)?;
        let path = self.path.clone();
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let rows = read_log(&tx)?;
        check_chain(&path, &rows)?;
        let (mut state, judgments) = replay(&rows);
        if !views_fresh(&tx, &rows)? {
            write_views(&tx, &state, &judgments, &rows)?;
        }
        match judge::judge(&state, &entry, signed, Proof::of(signature)) {
            Verdict::Counts => {}
            Verdict::Refused(why) => return Err(Failure::Refused(why)),
            Verdict::Repeats(seq) => {
                tx.commit()?;
                return Ok((Written::Repeats(seq), state));
            }
        }
        let seq = rows.last().map_or(1, |row| row.seq + 1);
        let prev = rows.last().map_or(GENESIS, |row| row.hash.as_str());
        let hash = entry::hash(prev, signed, signature);
        tx.execute(
            "INSERT INTO log (seq, prev, hash, signed, signature) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![sql_int(seq), prev, hash, signed, signature],
        )?;
        if let Some(bytes) = blob {
            tx.execute(
                "INSERT OR IGNORE INTO blobs (fingerprint, bytes) VALUES (?1, ?2)",
                params![crate::sha256_hex(bytes), bytes],
            )?;
        }
        judge::apply(&mut state, seq, &entry);
        write_judgment(&tx, seq, &Judgment::counted())?;
        match &entry.act {
            Act::Registered { person, .. } => write_person(&tx, &state.persons[person])?,
            Act::Proposed { claim } => write_claim(&tx, &state.claims[&claim.name])?,
            Act::Accepted { name, .. } | Act::Rejected { name, .. } => {
                write_claim(&tx, &state.claims[name])?
            }
        }
        write_head(&tx, seq, &hash)?;
        tx.commit()?;
        Ok((Written::Entry(seq), state))
    }
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
    Ok(rows)
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

fn replay(rows: &[Row]) -> (State, Vec<(u64, Judgment)>) {
    judge::replay(
        rows.iter()
            .map(|row| (row.seq, row.signed.as_str(), row.signature.as_deref())),
    )
}

/// Whether every view's table is there and the views were built at the
/// log's last entry.
fn views_fresh(conn: &Connection, rows: &[Row]) -> Result<bool, Failure> {
    for table in VIEW_TABLES {
        let found: Option<i64> = conn
            .query_row(
                "SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = ?1",
                [table],
                |row| row.get(0),
            )
            .optional()?;
        if found.is_none() {
            return Ok(false);
        }
    }
    let head: Option<(i64, String)> = conn
        .query_row("SELECT seq, hash FROM views_head", [], |row| {
            Ok((row.get(0)?, row.get(1)?))
        })
        .optional()?;
    let last = rows
        .last()
        .map_or((0, GENESIS), |row| (sql_int(row.seq), row.hash.as_str()));
    Ok(head.is_some_and(|(seq, hash)| seq == last.0 && hash == last.1))
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
        "INSERT INTO views_head (seq, hash) VALUES (?1, ?2)",
        params![sql_int(seq), hash],
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
    conn.execute("DELETE FROM versions WHERE name = ?1", [&claim.name])?;
    conn.execute(
        "INSERT OR REPLACE INTO claims (name, kind, standing) VALUES (?1, ?2, ?3)",
        params![claim.name, claim.kind.as_str(), claim.standing],
    )?;
    for version in &claim.versions {
        conn.execute(
            "INSERT INTO versions (name, version, base, statement, scenarios, body, proposer, \
             proposed_in, status, decided_by, decided_in, reason) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                claim.name,
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
            ],
        )?;
    }
    Ok(())
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

/// The log's entries in order, each with its judgment.
pub fn log_entries(conn: &Connection) -> Result<Vec<LogEntry>, Failure> {
    let mut judgments = conn.prepare("SELECT counted, reason FROM judgments WHERE seq = ?1")?;
    read_log(conn)?
        .into_iter()
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

/// The claim named `name`, where there is one.
pub fn claim_view(conn: &Connection, name: &str) -> Result<Option<ClaimView>, Failure> {
    let Some((kind, standing)) = conn
        .query_row(
            "SELECT kind, standing FROM claims WHERE name = ?1",
            [name],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?)),
        )
        .optional()?
    else {
        return Ok(None);
    };
    let mut statement = conn.prepare(
        "SELECT version, base, statement, scenarios, body, proposer, proposed_in, status, \
         decided_by, decided_in, reason FROM versions WHERE name = ?1 ORDER BY proposed_in",
    )?;
    let rows = statement
        .query_map([name], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, Option<String>>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, Option<String>>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, i64>(6)?,
                row.get::<_, String>(7)?,
                row.get::<_, Option<String>>(8)?,
                row.get::<_, Option<i64>>(9)?,
                row.get::<_, Option<String>>(10)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    let mut versions = Vec::with_capacity(rows.len());
    for (
        version,
        base,
        words,
        scenarios,
        body,
        proposer,
        proposed_in,
        status,
        by,
        decided_in,
        reason,
    ) in rows
    {
        versions.push(VersionView {
            version,
            base,
            statement: words,
            scenarios: from_json(&scenarios)?,
            body,
            proposer: from_json(&proposer)?,
            proposed_in: u64::try_from(proposed_in).unwrap_or(0),
            status,
            decided_by: by,
            decided_in: decided_in.and_then(|seq| u64::try_from(seq).ok()),
            reason,
        });
    }
    Ok(Some(ClaimView {
        name: name.to_owned(),
        kind,
        standing,
        versions,
    }))
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
