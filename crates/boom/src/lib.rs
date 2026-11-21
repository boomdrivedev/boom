//! boom keeps a store's claims, invariants, decisions and assumptions,
//! in a log on the machine: one SQLite file whose entries are appended,
//! never edited, and chained, so an entry changed or removed is found.
//! An agent proposes a version of a claim; only a person, signing with a
//! security key the store registers to them, accepts or rejects it
//! (record 0006). Every answer is computed from the log (core.1,
//! core.2): the views the store keeps beside it are rebuilt from it
//! whenever they are missing or behind.
//!
//! The binary is a thin command line over [`cli::run`], and `boom mcp`
//! serves an agent's program the commands it may run as tools, through
//! [`mcp::serve`].

pub mod claim;
pub mod cli;
pub mod context;
pub mod entry;
pub mod import;
pub mod judge;
pub mod mcp;
pub mod repository;
pub mod search;
pub mod sign;
pub mod store;
pub mod terms;

/// Why a command did not do what it was asked: it refused, exit 2, or
/// it could not run, exit 3. Either way nothing was written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Failure {
    /// The command refuses: its arguments, or the store's rules, do not
    /// allow what it asks.
    Refused(String),
    /// The command could not run: no store, an altered store, an
    /// unreadable blob, or a key's program that did not sign.
    NotRun(String),
}

impl Failure {
    /// A refusal with these words.
    pub fn refused(why: impl Into<String>) -> Self {
        Failure::Refused(why.into())
    }

    /// A command that could not run, with these words.
    pub fn not_run(why: impl Into<String>) -> Self {
        Failure::NotRun(why.into())
    }
}

/// The lowercase hexadecimal of `bytes`.
pub(crate) fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(DIGITS[usize::from(byte >> 4)] as char);
        out.push(DIGITS[usize::from(byte & 0x0f)] as char);
    }
    out
}

/// The SHA-256 of `bytes`, in lowercase hexadecimal.
pub fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    hex(&Sha256::digest(bytes))
}
