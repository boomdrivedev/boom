//! The batches that wait on a person, computed from the views and kept
//! nowhere: a narrative with each proposed version that it includes, a
//! revision with its cause, and each other proposed version alone, a
//! ruling. Each batch is one command that a person's key signs one time
//! (narrative-page, "The batches that wait on a person";
//! `views-computed-from-the-ledger`).

use std::collections::BTreeSet;

use rusqlite::Connection;

use crate::claim::Kind;
use crate::entry::Actor;
use crate::judge::{State, Status, Version};
use crate::store::{self, Scope};
use crate::Failure;

/// The shape of a batch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shape {
    /// A proposed narrative, with each proposed version it includes.
    Narrative,
    /// A cause, with each proposed version that names it.
    Revision,
    /// One proposed version alone.
    Ruling,
}

impl Shape {
    /// The shape in one word.
    pub fn as_str(self) -> &'static str {
        match self {
            Shape::Narrative => "narrative",
            Shape::Revision => "revision",
            Shape::Ruling => "ruling",
        }
    }
}

/// A version that a batch names: its claim, its version, its kind and its
/// status now.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Named {
    /// The claim's name.
    pub name: String,
    /// The version.
    pub version: String,
    /// The claim's kind.
    pub kind: Kind,
    /// Its status now.
    pub status: Status,
    /// Its statement.
    pub statement: String,
    /// The version it amends, where it amends one.
    pub base: Option<String>,
}

impl Named {
    fn of(name: &str, version: &Version) -> Named {
        Named {
            name: name.to_owned(),
            version: version.proposal.version.clone(),
            kind: version.proposal.kind,
            status: version.status,
            statement: version.proposal.statement.clone(),
            base: version.proposal.base.clone(),
        }
    }

    /// `<name>@<version>`.
    pub fn target(&self) -> String {
        format!("{}@{}", self.name, self.version)
    }
}

/// What one accept takes, as a batch or a command computes it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Takes {
    /// Each proposed version that the accept signs, in its order.
    pub versions: Vec<Named>,
    /// Each version it names that is accepted already, which the accept
    /// leaves as it is.
    pub accepted: Vec<Named>,
    /// Each rejected, superseded or deprecated version that refuses the
    /// accept.
    pub closed: Vec<Named>,
}

impl Takes {
    fn add(&mut self, named: Named) {
        let seen = |list: &[Named]| list.contains(&named);
        if seen(&self.versions) || seen(&self.accepted) || seen(&self.closed) {
            return;
        }
        match named.status {
            Status::Proposed => self.versions.push(named),
            Status::Accepted => self.accepted.push(named),
            Status::Rejected | Status::Superseded | Status::Deprecated => self.closed.push(named),
        }
    }

    /// Each version of `other`, after its own.
    pub fn join(&mut self, other: Takes) {
        for named in other
            .versions
            .into_iter()
            .chain(other.accepted)
            .chain(other.closed)
        {
            self.add(named);
        }
    }
}

/// A closed version as a refusal names it: `<name>@<version> is <status>`.
pub fn closed_words(closed: &[Named]) -> String {
    closed
        .iter()
        .map(|named| format!("{} is {}", named.target(), named.status.as_str()))
        .collect::<Vec<_>>()
        .join(", ")
}

/// The version `name@version` of `state`, where it holds it.
fn found<'a>(state: &'a State, name: &str, version: &str) -> Option<&'a Version> {
    state
        .holding(name, version)
        .and_then(|claim| claim.version(version))
}

/// What the accept of the narrative `name@version` takes: the narrative,
/// then each version it includes, in its order
/// (`the-accept-of-a-narrative-accepts-the-claims-it-includes`).
pub fn of_narrative(state: &State, name: &str, version: &Version) -> Takes {
    let mut takes = Takes::default();
    takes.add(Named::of(name, version));
    for stored in &version.proposal.includes {
        let Some((included, meant)) = stored.split_once('@') else {
            continue;
        };
        if let Some(found) = found(state, included, meant) {
            takes.add(Named::of(included, found));
        }
    }
    takes
}

/// What the accept of the cause `name@version` takes: the cause, then
/// each version of `state` whose edge `cause` names it, proposed or
/// accepted, in the order proposed. A superseded or rejected version that
/// names it is left out: only the newest draft of a claim is taken.
pub fn of_cause(state: &State, name: &str, version: &Version) -> Takes {
    let target = format!("{name}@{}", version.proposal.version);
    let mut takes = Takes::default();
    takes.add(Named::of(name, version));
    let mut naming: Vec<(&str, &Version)> = state
        .claims
        .values()
        .flat_map(|claim| {
            claim
                .versions
                .iter()
                .map(move |version| (claim.name.as_str(), version))
        })
        .filter(|(_, version)| version.proposal.cause.as_deref() == Some(target.as_str()))
        .filter(|(_, version)| matches!(version.status, Status::Proposed | Status::Accepted))
        .collect();
    naming.sort_by_key(|(_, version)| version.proposed_in);
    for (claim, version) in naming {
        takes.add(Named::of(claim, version));
    }
    takes
}

/// A batch that waits on a person.
#[derive(Debug, Clone)]
pub struct Batch {
    /// Its shape.
    pub shape: Shape,
    /// What it is named for: the narrative, the cause or the one claim.
    pub head: Named,
    /// The statement of its head: its title.
    pub title: String,
    /// Who proposed its head.
    pub proposer: Actor,
    /// The repositories its head binds; none for the whole system.
    pub repositories: Vec<String>,
    /// What its accept takes.
    pub takes: Takes,
    /// The entry that proposed the oldest version it takes: its order.
    pub since: u64,
}

impl Batch {
    /// The command that accepts it, where Boom does not refuse it.
    pub fn command(&self) -> Option<String> {
        if !self.takes.closed.is_empty() {
            return None;
        }
        Some(match self.shape {
            Shape::Revision => format!("boom accept --cause {} --key <key>", self.head.target()),
            Shape::Narrative | Shape::Ruling => {
                format!("boom accept {} --key <key>", self.head.target())
            }
        })
    }

    /// Its counts in words: what it takes, and what of it is accepted
    /// already.
    pub fn counts(&self) -> String {
        let proposed = self.takes.versions.len();
        let mut words = match self.shape {
            Shape::Narrative => {
                let included = proposed.saturating_sub(1);
                format!(
                    "1 narrative · {}",
                    count(
                        included,
                        "claim it includes is proposed",
                        "claims it includes are proposed"
                    )
                )
            }
            Shape::Revision => {
                let cause = usize::from(self.head.status == Status::Proposed);
                let mut words = count(
                    proposed - cause,
                    "new version names its cause",
                    "new versions name their cause",
                );
                if cause == 1 {
                    words.push_str(" · and the cause, proposed");
                }
                words
            }
            Shape::Ruling => format!("1 {}", self.head.kind.as_str()),
        };
        let accepted = self
            .takes
            .accepted
            .iter()
            .filter(|named| *named != &self.head)
            .count();
        if accepted > 0 {
            words.push_str(&format!(" · {accepted} already accepted"));
        }
        words
    }
}

/// `n` and the noun of one or of many.
fn count(n: usize, one: &str, many: &str) -> String {
    format!("{n} {}", if n == 1 { one } else { many })
}

/// Each batch that waits on a person, computed from the views on `conn`,
/// the oldest first. Only the proposed versions, and the claims they
/// include or name as their cause, are read.
pub fn waiting(conn: &Connection) -> Result<Vec<Batch>, Failure> {
    let state = waiting_state(conn)?;
    Ok(batches(&state))
}

/// The part of the views the batches read: each claim with a proposed
/// version, and each claim that one of them includes or names as its
/// cause.
fn waiting_state(conn: &Connection) -> Result<State, Failure> {
    let names = store::proposed_names(conn)?;
    let first = store::load_state_for(conn, &Scope::names(names.iter().cloned()))?;
    let mut more: BTreeSet<String> = names;
    for claim in first.claims.values() {
        for version in claim
            .versions
            .iter()
            .filter(|version| version.status == Status::Proposed)
        {
            more.extend(
                version
                    .proposal
                    .includes
                    .iter()
                    .chain(&version.proposal.cause)
                    .filter_map(|edge| edge.split_once('@'))
                    .map(|(name, _)| name.to_owned()),
            );
        }
    }
    store::load_state_for(conn, &Scope::names(more))
}

/// The batches of `state`, the oldest first.
pub fn batches(state: &State) -> Vec<Batch> {
    let proposed: Vec<(&str, &Version)> = state
        .claims
        .values()
        .flat_map(|claim| {
            claim
                .versions
                .iter()
                .map(move |version| (claim.name.as_str(), version))
        })
        .filter(|(_, version)| version.status == Status::Proposed)
        .collect();
    let batch = |shape: Shape, name: &str, version: &Version, takes: Takes| {
        let since = takes
            .versions
            .iter()
            .chain(std::iter::once(&Named::of(name, version)))
            .filter_map(|named| found(state, &named.name, &named.version))
            .map(|version| version.proposed_in)
            .min()
            .unwrap_or(version.proposed_in);
        Batch {
            shape,
            head: Named::of(name, version),
            title: version.proposal.statement.clone(),
            proposer: version.proposer.clone(),
            repositories: version.repositories.clone(),
            takes,
            since,
        }
    };
    let mut all = Vec::new();
    for (name, version) in &proposed {
        if version.proposal.kind == Kind::Narrative {
            all.push(batch(
                Shape::Narrative,
                name,
                version,
                of_narrative(state, name, version),
            ));
        }
    }
    let causes: BTreeSet<&str> = proposed
        .iter()
        .filter_map(|(_, version)| version.proposal.cause.as_deref())
        .collect();
    for cause in causes {
        let Some((name, meant)) = cause.split_once('@') else {
            continue;
        };
        if let Some(version) = found(state, name, meant) {
            all.push(batch(
                Shape::Revision,
                name,
                version,
                of_cause(state, name, version),
            ));
        }
    }
    let taken: BTreeSet<(String, String)> = all
        .iter()
        .flat_map(|batch| batch.takes.versions.iter().chain(&batch.takes.closed))
        .map(|named| (named.name.clone(), named.version.clone()))
        .collect();
    for (name, version) in &proposed {
        let key = ((*name).to_owned(), version.proposal.version.clone());
        if !taken.contains(&key) {
            let mut takes = Takes::default();
            takes.add(Named::of(name, version));
            all.push(batch(Shape::Ruling, name, version, takes));
        }
    }
    all.sort_by_key(|batch| (batch.since, batch.head.target()));
    all
}
