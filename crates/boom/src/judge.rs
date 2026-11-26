//! The rules that decide whether an entry counts, and the state a
//! replay of the log builds (design 6).
//!
//! One judge, asked before a write signs, again inside the write's
//! transaction, and for every entry whenever the views are rebuilt, so
//! an entry no write would have made counts for nothing
//! (`validated-on-read`). Its rules, in order: the entry's form; for a
//! person's entry, a signature that verifies by the key registered to
//! that person before it, with presence (for the store's first entry,
//! the key it registers); for an agent's, an act that is `proposed` or
//! `withdrawn`, and a provenance; then the act's own rules. A person's
//! note or withdrawal alone carries the person's name and no signature
//! (narrative-notes; claim-versions).
//!
//! An entry is judged by the rules it was written under, never by a rule
//! that came into force after it: each entry names its rule set, and a
//! rule that came in with a later set does not hold for it (record-log,
//! "An entry is judged by the rules it was written under").

use std::collections::BTreeMap;

use ssh_key::PublicKey;

use crate::claim::{self, Kind, Proposal};
use crate::entry::{Act, Actor, Attach, Entry, Form, Imported};
use crate::{repository, sign};

/// The rule set this boom writes each new entry under, and the newest
/// rule set it judges by. An entry that names no rule set was written
/// under rule set 0: the rules that each boom held before entries named
/// their rule set (record-log, "An entry is judged by the rules it was
/// written under").
pub const RULES: u32 = 1;

/// The rule set from which a narrative binds one level (narratives, "A
/// narrative binds one level").
const ONE_LEVEL: u32 = 1;

/// The rule set from which a draft that a newer draft superseded is not
/// accepted, rejected or amended again (claim-versions, "A new draft
/// supersedes the draft it amends"). Under an earlier rule set, no draft
/// supersedes another, so that draft still waits.
const ONE_DRAFT_LINE: u32 = 1;

/// A person the store registers.
#[derive(Debug, Clone)]
pub struct Person {
    /// Their name, kebab case.
    pub name: String,
    /// Their security key's public half.
    pub key: PublicKey,
    /// Whether they are an owner, who registers persons.
    pub owner: bool,
    /// The entry that registered them.
    pub registered_in: u64,
}

/// A repository a store registers, with its settings, as the newest
/// counted `repository` entry for it gives them (repository-settings).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Repository {
    /// Its name, kebab case.
    pub name: String,
    /// Its remote, in one form.
    pub remote: String,
    /// Whether a session in it gets its context at session start.
    pub session: bool,
    /// The owner who set it.
    pub set_by: String,
    /// The entry that set it.
    pub set_in: u64,
}

/// A version's status, read from the log (claim-versions).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// Waiting on a person.
    Proposed,
    /// Accepted, and no later version of its claim is: it is in force.
    Accepted,
    /// Accepted, and a later version of its claim is; or, for a record
    /// that came in as history, superseded as its file says.
    Superseded,
    /// Rejected by a person; or, for a record that came in as history,
    /// rejected as its file says.
    Rejected,
    /// Retired with nothing in its place. Only a record that came in as
    /// history, deprecated as its file says, has it; no act makes it yet.
    Deprecated,
    /// Withdrawn by its author, with a reason and no signature
    /// (claim-versions, "The author withdraws a proposed version").
    Withdrawn,
}

impl Status {
    /// The status an answer names `name`, where there is one.
    pub fn from_name(name: &str) -> Option<Status> {
        [
            Status::Proposed,
            Status::Accepted,
            Status::Superseded,
            Status::Rejected,
            Status::Deprecated,
            Status::Withdrawn,
        ]
        .into_iter()
        .find(|status| status.as_str() == name)
    }

    /// The status as an answer names it.
    pub fn as_str(self) -> &'static str {
        match self {
            Status::Proposed => "proposed",
            Status::Accepted => "accepted",
            Status::Superseded => "superseded",
            Status::Rejected => "rejected",
            Status::Deprecated => "deprecated",
            Status::Withdrawn => "withdrawn",
        }
    }
}

/// A version of a claim and what the log says of it.
#[derive(Debug, Clone)]
pub struct Version {
    /// The version as proposed.
    pub proposal: Proposal,
    /// The repositories its proposal binds.
    pub repositories: Vec<String>,
    /// The contexts its proposal binds, each `<name>@<version>` of the
    /// version that the binding meant (context-claims). With no
    /// repository and no context, a version binds the whole system.
    pub contexts: Vec<String>,
    /// Who proposed it.
    pub proposer: Actor,
    /// The entry that proposed it.
    pub proposed_in: u64,
    /// Its status.
    pub status: Status,
    /// The person who accepted, rejected or withdrew it; none for a
    /// version that an agent withdrew.
    pub decided_by: Option<String>,
    /// The entry that accepted, rejected or withdrew it.
    pub decided_in: Option<u64>,
    /// Why it was rejected or withdrawn.
    pub reason: Option<String>,
    /// Where it came in from, for a version an import proposed.
    pub imported: Option<Imported>,
}

/// A claim: its kind, its standing version, and its versions in the
/// order proposed.
#[derive(Debug, Clone)]
pub struct Claim {
    /// Its name.
    pub name: String,
    /// Its kind.
    pub kind: Kind,
    /// The repository of a term, where one word can have a term in each
    /// repository; none for the other kinds (term-claims).
    pub repository: Option<String>,
    /// The version that stands, where one does.
    pub standing: Option<String>,
    /// Its versions, in the order proposed.
    pub versions: Vec<Version>,
}

impl Version {
    /// Whether it binds the whole system: no repository and no context.
    pub fn binds_the_whole_system(&self) -> bool {
        self.repositories.is_empty() && self.contexts.is_empty()
    }

    /// Whether it binds `repository`, or a context named in `contexts`,
    /// or the whole system.
    pub fn binds(&self, repository: &str, contexts: &[String]) -> bool {
        self.binds_the_whole_system()
            || self.repositories.iter().any(|name| name == repository)
            || crate::context::names(&self.contexts)
                .iter()
                .any(|name| contexts.iter().any(|context| context == name))
    }

    /// The act that decided it, in words: `withdrawn`, `rejected` or
    /// `accepted`, a record that came in as history among the accepted.
    pub fn decided_word(&self) -> &'static str {
        if self.status == Status::Withdrawn {
            "withdrawn"
        } else if self.reason.is_some() {
            "rejected"
        } else {
            "accepted"
        }
    }

    /// Whether a person accepted it to stand: it is accepted, or a later
    /// accept superseded it. A draft that a newer draft superseded, a
    /// rejected version and a version from history never stood.
    pub fn stood(&self) -> bool {
        self.proposal.history.is_none()
            && self.decided_in.is_some()
            && matches!(self.status, Status::Accepted | Status::Superseded)
    }
}

impl Claim {
    /// The version named `version`.
    pub fn version(&self, version: &str) -> Option<&Version> {
        self.versions
            .iter()
            .find(|candidate| candidate.proposal.version == version)
    }

    /// The status of `found`, a version of this claim, under the rule set
    /// `rules`: before rule set [`ONE_DRAFT_LINE`], a draft that a newer
    /// draft superseded still waits, and is proposed.
    fn status_under(&self, found: &Version, rules: u32) -> Status {
        if rules < ONE_DRAFT_LINE && self.newer_draft(&found.proposal.version).is_some() {
            Status::Proposed
        } else {
            found.status
        }
    }

    /// The newer draft that superseded the draft `version`: the version
    /// that amends it, where no person accepted or rejected it
    /// (claim-versions, "A new draft supersedes the draft it amends").
    /// None for any other version.
    pub fn newer_draft(&self, version: &str) -> Option<&Version> {
        let found = self.version(version)?;
        if found.status != Status::Superseded || found.decided_in.is_some() {
            return None;
        }
        self.versions
            .iter()
            .find(|newer| newer.proposal.base.as_deref() == Some(version))
    }

    /// The base that an accept of `version` checks: the nearest version
    /// in its line of bases that a person accepted to stand, or none
    /// (claim-versions, "An accept stands only on its base").
    pub fn accepted_base<'a>(&'a self, version: &'a Version) -> Option<&'a str> {
        let mut base = version.proposal.base.as_deref();
        while let Some(at) = base {
            let found = self.version(at)?;
            if found.stood() {
                return Some(at);
            }
            base = found.proposal.base.as_deref();
        }
        None
    }

    fn version_mut(&mut self, version: &str) -> Option<&mut Version> {
        self.versions
            .iter_mut()
            .find(|candidate| candidate.proposal.version == version)
    }
}

/// A person's note on one paragraph of a version of a narrative
/// (narrative-notes).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Note {
    /// Its name: [`crate::narrative::note_name`] of what it names.
    pub name: String,
    /// The narrative.
    pub narrative: String,
    /// The version it is on.
    pub version: String,
    /// The paragraph's number.
    pub paragraph: u64,
    /// The paragraph's fingerprint.
    pub fingerprint: String,
    /// Its words.
    pub words: String,
    /// The person who wrote it.
    pub person: String,
    /// When, in Unix seconds, as the maker's clock read.
    pub at: u64,
    /// The entry that holds it.
    pub noted_in: u64,
}

/// A counted entry, as a link finds it by its act hash (record-log, "A
/// link names an entry that the log holds").
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Acted {
    /// Its number.
    pub seq: u64,
    /// Its act, `proposed` or another tool's `<tool>.<type>`.
    pub act: String,
}

/// A counted approval of a plan (plan-approvals).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Approval {
    /// The entry that holds it.
    pub seq: u64,
    /// The SHA-256 of the plan's document.
    pub plan: String,
    /// The claims it covers, each `<name>@<version>`.
    pub covers: Vec<String>,
}

/// The key of a claim among a state's claims: its name, and the
/// repository of a term or nothing for the other kinds.
pub type Key = (String, String);

/// What a term binds, in words: its repository, or the whole system.
pub fn binding_words(repository: Option<&str>) -> String {
    repository.map_or_else(|| "the whole system".to_owned(), str::to_owned)
}

/// The key of the claim `name`, a term of `repository` where one is
/// given.
pub fn key(name: &str, repository: Option<&str>) -> Key {
    (name.to_owned(), repository.unwrap_or_default().to_owned())
}

impl Claim {
    /// Its key among a state's claims.
    pub fn key(&self) -> Key {
        key(&self.name, self.repository.as_deref())
    }
}

/// What the counted entries of a log, replayed in order, build.
#[derive(Debug, Clone, Default)]
pub struct State {
    /// The persons registered, by name.
    pub persons: BTreeMap<String, Person>,
    /// The claims, by name and, for a term, its repository.
    pub claims: BTreeMap<Key, Claim>,
    /// The store's first owner, the person its first counted entry
    /// registers.
    pub first_owner: Option<String>,
    /// The repositories the store registers, by name.
    pub repositories: BTreeMap<String, Repository>,
    /// The notes on the narratives, by name (narrative-notes).
    pub notes: BTreeMap<String, Note>,
    /// The fingerprints of the paragraphs of each body of a narrative, by
    /// the fingerprint of the body: what the store read from its blobs,
    /// for the judge to check a note against (narratives).
    pub paragraphs: BTreeMap<String, Vec<String>>,
    /// The counted entries, by their act hash: the first counted entry
    /// of each act hash (record-log, "Each entry has an act hash").
    pub acts: BTreeMap<String, Acted>,
    /// The counted approvals of plans, by their act hash.
    pub approvals: BTreeMap<String, Approval>,
}

impl State {
    /// The notes on the narrative `name`, in the order written.
    pub fn notes_of(&self, name: &str) -> Vec<&Note> {
        let mut notes: Vec<&Note> = self
            .notes
            .values()
            .filter(|note| note.narrative == name)
            .collect();
        notes.sort_by_key(|note| note.noted_in);
        notes
    }

    /// The version of its narrative that answers `note`, with the answer:
    /// the first, in the order proposed, that a person did not reject and
    /// its author did not withdraw. None where the note is open.
    pub fn answer_of(&self, note: &Note) -> Option<(&Version, &str)> {
        let claim = self.named(&note.narrative).into_iter().next()?;
        claim
            .versions
            .iter()
            .filter(|version| !matches!(version.status, Status::Rejected | Status::Withdrawn))
            .find_map(|version| {
                version
                    .proposal
                    .answers
                    .iter()
                    .find(|answer| answer.note == note.name)
                    .map(|answer| (version, answer.words.as_str()))
            })
    }

    /// The elaboration that the version `note` is on came from, which
    /// that version's origin names; None where it names none
    /// (narrative-notes).
    pub fn elaboration_of(&self, note: &Note) -> Option<&crate::claim::Origin> {
        self.named(&note.narrative)
            .into_iter()
            .flat_map(|claim| &claim.versions)
            .find(|version| version.proposal.version == note.version)
            .and_then(|version| version.proposal.elaboration())
    }

    /// The notes on the narrative `name` that no version answers.
    pub fn open_notes(&self, name: &str) -> Vec<&Note> {
        self.notes_of(name)
            .into_iter()
            .filter(|note| self.answer_of(note).is_none())
            .collect()
    }

    /// The person `key` is registered to.
    pub fn person_by_key(&self, key: &PublicKey) -> Option<&Person> {
        self.persons
            .values()
            .find(|person| person.key.key_data() == key.key_data())
    }

    /// The claims named `name`: one, or for a term one in each
    /// repository, in the order of their repositories.
    pub fn named(&self, name: &str) -> Vec<&Claim> {
        self.claims
            .range(key(name, None)..)
            .take_while(|((named, _), _)| named == name)
            .map(|(_, claim)| claim)
            .collect()
    }

    /// The claim named `name` that holds `version`.
    pub fn holding(&self, name: &str, version: &str) -> Option<&Claim> {
        self.named(name)
            .into_iter()
            .find(|claim| claim.version(version).is_some())
    }

    fn holding_mut(&mut self, name: &str, version: &str) -> Option<&mut Claim> {
        let key = self.holding(name, version)?.key();
        self.claims.get_mut(&key)
    }

    /// Whether a standing term of `repository` with the word of `claim`,
    /// a term of the whole system, stands over it there (term-claims,
    /// "The term of a repository stands over the term of the whole
    /// system"). For no repository, nothing stands over it.
    fn stands_over(&self, claim: &Claim, repository: Option<&str>) -> bool {
        let Some(repository) = repository else {
            return false;
        };
        claim.kind == Kind::Term
            && claim.repository.is_none()
            && self
                .claims
                .get(&key(&claim.name, Some(repository)))
                .is_some_and(|own| own.kind == Kind::Term && own.standing.is_some())
    }

    /// Each claim that stands with its standing version: for
    /// `Some(repository)`, those whose standing version binds that
    /// repository, a context named in `contexts`, or the whole system,
    /// less each term of the whole system that a term of the repository
    /// stands over; for `None`, every one. In the order invariants,
    /// decisions, assumptions, terms, contexts, narratives and outlines,
    /// each kind by name and a term then by its repository (claim-search;
    /// design 4; term-claims; context-claims; narratives; views).
    pub fn standing_for(
        &self,
        repository: Option<&str>,
        contexts: &[String],
    ) -> Vec<(&Claim, &Version)> {
        let mut standing = Vec::new();
        for kind in [
            Kind::Invariant,
            Kind::Decision,
            Kind::Assumption,
            Kind::Term,
            Kind::Context,
            Kind::Narrative,
            Kind::Outline,
        ] {
            for claim in self.claims.values().filter(|claim| claim.kind == kind) {
                let Some(version) = claim
                    .standing
                    .as_deref()
                    .and_then(|version| claim.version(version))
                else {
                    continue;
                };
                let bound = repository.is_none_or(|repository| version.binds(repository, contexts));
                if bound && !self.stands_over(claim, repository) {
                    standing.push((claim, version));
                }
            }
        }
        standing
    }
}

/// What an entry carries, or will carry, as its signature.
#[derive(Debug, Clone, Copy)]
pub enum Proof<'a> {
    /// No signature: an agent's entry.
    Unsigned,
    /// A signature this key is about to make: the judge asked before a
    /// person is asked to touch their key.
    ToBeSigned(&'a PublicKey),
    /// An armored SSH signature over the entry's signed part.
    Signed(&'a str),
}

impl<'a> Proof<'a> {
    /// The proof an entry's stored signature gives.
    pub fn of(signature: Option<&'a str>) -> Self {
        signature.map_or(Proof::Unsigned, Proof::Signed)
    }
}

/// The judge's answer for one entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Verdict {
    /// It counts.
    Counts,
    /// It does not count, and why.
    Refused(String),
    /// It repeats a counted entry, numbered here, and adds nothing.
    Repeats(u64),
}

/// Whether an entry of the log counts, and why not where it does not.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Judgment {
    /// Whether it counts.
    pub counted: bool,
    /// Why it does not.
    pub reason: Option<String>,
}

impl Judgment {
    fn of(verdict: Verdict) -> Self {
        match verdict {
            Verdict::Counts => Judgment {
                counted: true,
                reason: None,
            },
            Verdict::Refused(why) => Judgment {
                counted: false,
                reason: Some(why),
            },
            Verdict::Repeats(seq) => Judgment {
                counted: false,
                reason: Some(format!("repeats entry {seq}")),
            },
        }
    }

    /// The judgment of an entry that counts.
    pub fn counted() -> Self {
        Judgment::of(Verdict::Counts)
    }
}

/// Judges `entry`, whose signed part is `signed`, against the state
/// built so far.
pub fn judge(state: &State, entry: &Entry, signed: &str, proof: Proof<'_>) -> Verdict {
    match check(state, entry, signed, proof) {
        Ok(verdict) => verdict,
        Err(why) => Verdict::Refused(why),
    }
}

fn check(state: &State, entry: &Entry, signed: &str, proof: Proof<'_>) -> Result<Verdict, String> {
    if entry.rules > RULES {
        return Err(format!(
            "it was written under rule set {}, and this boom judges by rule set {RULES} at most: a newer boom judges it",
            entry.rules
        ));
    }
    check_form(&entry.act)?;
    if let Act::Other { act, .. } = &entry.act {
        // Only a person's accept, reject, registration and setting carry
        // a signature (lineage, "Boom judges another tool's entry by its
        // form, its actor and its links").
        if !matches!(proof, Proof::Unsigned) {
            return Err(format!(
                "the key signs no entry of another tool: {act} carries a signature"
            ));
        }
    }
    match &entry.actor {
        Actor::Person(name) => check_person(state, entry, name, signed, proof)?,
        Actor::Agent { provenance } => check_agent(&entry.act, provenance, proof)?,
    }
    check_links(state, entry.act.links())?;
    if let Act::Other { .. } = &entry.act {
        if let Some(acted) = state.acts.get(&crate::entry::act_hash(signed)) {
            return Ok(Verdict::Repeats(acted.seq));
        }
    }
    check_act(state, entry)
}

/// Each link names a counted entry before it, by its act hash (record-log,
/// "A link names an entry that the log holds").
fn check_links(state: &State, links: &[String]) -> Result<(), String> {
    for link in links {
        if !state.acts.contains_key(link) {
            return Err(format!(
                "a link names an entry that the log holds before it: no counted entry has the act hash {link}"
            ));
        }
    }
    Ok(())
}

/// The form of the links of an entry: each the 64 lowercase hexadecimal
/// characters of an act hash, and each once.
pub fn check_link_form(links: &[String]) -> Result<(), String> {
    for (at, link) in links.iter().enumerate() {
        let hash = link.len() == 64
            && link
                .chars()
                .all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c));
        if !hash {
            return Err(format!(
                "a link is the 64 lowercase hexadecimal characters of an act hash: {link} is not"
            ));
        }
        if links[..at].contains(link) {
            return Err(format!(
                "an entry names each link once: it names {link} twice"
            ));
        }
    }
    Ok(())
}

/// The form of another tool's type: two kebab names joined by a dot, and
/// a tool other than boom (lineage, "Another tool appends its entry").
pub fn check_other_type(act: &str) -> Result<(), String> {
    let Some((tool, kind)) = act.split_once('.') else {
        return Err(format!(
            "a type of another tool is <tool>.<type>: {act} names no tool"
        ));
    };
    if tool == "boom" {
        return Err(format!(
            "boom names no other tool: a type of boom has no dot, and {act} is not another tool's"
        ));
    }
    for name in [tool, kind] {
        claim::check_name(name).map_err(|_| {
            format!("a type of another tool is two kebab names joined by a dot: {act} is not")
        })?;
    }
    Ok(())
}

/// An act's form: what holds of it whatever the store holds.
fn check_form(act: &Act) -> Result<(), String> {
    match act {
        Act::Registered { person, key, .. } => {
            claim::check_name(person)?;
            let key = registered_key(key)?;
            sign::check_key_type(&key)
        }
        Act::Proposed { claim, attach, .. } => {
            check_attach(attach.as_ref())?;
            let bound = contexts(attach.as_ref());
            if !bound.is_empty() {
                match claim.kind {
                    Kind::Term => {
                        return Err(format!(
                            "a term binds one repository or the whole system, and never a context: {} binds {}",
                            claim.name,
                            crate::repository::words(bound)
                        ))
                    }
                    Kind::Context => {
                        return Err(format!(
                            "a context binds the repositories of its globs, and no context: {} binds {}",
                            claim.name,
                            crate::repository::words(bound)
                        ))
                    }
                    _ => {}
                }
            }
            claim.check(repositories(attach.as_ref()))
        }
        Act::Accepted { versions } => {
            for (at, named) in versions.iter().enumerate() {
                claim::check_name(&named.name)?;
                if versions[..at].iter().any(|before| before == named) {
                    return Err(format!(
                        "an accept names each version one time: it names {}@{} twice",
                        named.name, named.version
                    ));
                }
            }
            if versions.is_empty() {
                return Err("an accept names a version".to_owned());
            }
            Ok(())
        }
        Act::Repository { name, remote, .. } => {
            crate::repository::check_name(name)?;
            // A remote kept in its one form reads back as itself.
            match crate::repository::remote_form(&format!("https://{remote}")) {
                Some(form) if &form == remote => Ok(()),
                _ => Err(format!(
                    "a repository's remote is kept in one form, its host and path: {remote} is not"
                )),
            }
        }
        Act::Rejected { versions, reason } => check_versions("reject", versions, reason),
        Act::Withdrawn { versions, reason } => check_versions("withdrawal", versions, reason),
        Act::Noted {
            name,
            version,
            paragraph,
            fingerprint,
            words,
        } => {
            claim::edge(&format!("{name}@{version}")).map_err(|_| {
                format!("a note names a version, <name>@<version>: {name}@{version} is not one")
            })?;
            if *paragraph == 0 {
                return Err("a note names a paragraph by its number, from 1".to_owned());
            }
            if claim::edge(&format!("a@{fingerprint}")).is_err() {
                return Err(format!(
                    "a note names its paragraph's fingerprint, 12 hexadecimal characters: {fingerprint} is not"
                ));
            }
            if words.trim().is_empty() {
                return Err("a note holds words: this one is empty".to_owned());
            }
            Ok(())
        }
        Act::Approved {
            plan,
            links,
            outputs,
            covers,
        } => {
            let fingerprint = plan.len() == 64
                && plan
                    .chars()
                    .all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c));
            if !fingerprint {
                return Err(format!(
                    "an approval names its plan by the SHA-256 of its document, 64 lowercase hexadecimal characters: {plan} is not"
                ));
            }
            check_link_form(links)?;
            for output in outputs {
                claim::Origin::git(output)?;
            }
            for (at, covered) in covers.iter().enumerate() {
                let (name, _) = claim::edge(covered)?;
                if covers[..at]
                    .iter()
                    .any(|before| before.split_once('@').map(|(n, _)| n) == Some(name))
                {
                    return Err(format!(
                        "an approval covers each claim once: it covers {name} twice"
                    ));
                }
            }
            Ok(())
        }
        Act::Other {
            act,
            links,
            content,
        } => {
            check_other_type(act)?;
            check_link_form(links)?;
            match serde_json::from_str::<serde_json::Value>(content) {
                Ok(serde_json::Value::Object(_)) => Ok(()),
                _ => Err(format!(
                    "the content of {act} is one JSON object: it is not"
                )),
            }
        }
    }
}

/// The form of a reject or a withdrawal: one version at least, each
/// named one time, and its reason.
fn check_versions(act: &str, versions: &[crate::entry::Named], reason: &str) -> Result<(), String> {
    if versions.is_empty() {
        return Err(format!("a {act} names a version"));
    }
    for (at, named) in versions.iter().enumerate() {
        claim::check_name(&named.name)?;
        if versions[..at].contains(named) {
            return Err(format!(
                "a {act} names each version one time: it names {}@{} twice",
                named.name, named.version
            ));
        }
    }
    if reason.trim().is_empty() {
        return Err(format!("a {act} gives its reason"));
    }
    Ok(())
}

/// An attachment's form: a repository or a context at least; each
/// repository kebab case, in ascending order and none twice; each context
/// `<name>@<version>`, in the order of their names and each name once
/// (claim-attachments; design 2; context-claims).
fn check_attach(attach: Option<&Attach>) -> Result<(), String> {
    let Some(attach) = attach else {
        return Ok(());
    };
    if attach.repositories.is_empty() && attach.contexts.is_empty() {
        return Err(
            "an attach names a repository or a context at least: one of the whole system holds no attach"
                .to_owned(),
        );
    }
    for stored in &attach.contexts {
        claim::edge(stored).map_err(|_| {
            format!("an attach names a context as <name>@<version>: {stored} does not")
        })?;
    }
    let names = crate::context::names(&attach.contexts);
    for pair in names.windows(2) {
        if pair[0] == pair[1] {
            return Err(format!(
                "an attach names each context once: {} is named twice",
                pair[0]
            ));
        }
        if pair[0] > pair[1] {
            return Err(format!(
                "an attach names its contexts in the order of their names: {} comes before {}",
                pair[1], pair[0]
            ));
        }
    }
    for name in &attach.repositories {
        repository::check_name(name)?;
    }
    for pair in attach.repositories.windows(2) {
        if pair[0] == pair[1] {
            return Err(format!(
                "an attach names each repository once: {} is named twice",
                pair[0]
            ));
        }
        if pair[0] > pair[1] {
            return Err(format!(
                "an attach names its repositories in order: {} comes before {}",
                pair[1], pair[0]
            ));
        }
    }
    Ok(())
}

/// The repositories an attachment names.
pub fn repositories(attach: Option<&Attach>) -> &[String] {
    attach.map_or(&[], |attach| attach.repositories.as_slice())
}

/// The contexts an attachment names, each `<name>@<version>`.
pub fn contexts(attach: Option<&Attach>) -> &[String] {
    attach.map_or(&[], |attach| attach.contexts.as_slice())
}

/// Whether two bindings are the same: the same repositories, and the
/// same contexts by name, whatever version of each they meant
/// (context-claims, "The same binding by name").
pub fn same_binding(repositories: &[String], contexts: &[String], attach: Option<&Attach>) -> bool {
    repositories == self::repositories(attach)
        && crate::context::names(contexts) == crate::context::names(self::contexts(attach))
}

fn registered_key(key: &str) -> Result<PublicKey, String> {
    PublicKey::from_openssh(key)
        .map_err(|error| format!("its key is not an OpenSSH public key: {error}"))
}

/// A person's entry: signed by the key the store registered to them
/// before it, or, for the store's first entry, by the key it registers.
fn check_person(
    state: &State,
    entry: &Entry,
    name: &str,
    signed: &str,
    proof: Proof<'_>,
) -> Result<(), String> {
    let unsigned = match &entry.act {
        Act::Noted { .. } => Some("a note"),
        Act::Withdrawn { .. } => Some("a withdrawal"),
        Act::Other { .. } => Some("another tool's entry"),
        _ => None,
    };
    if let Some(act) = unsigned {
        // A note or a withdrawal binds nothing: it is the person's name
        // and no signature (narrative-notes, "Only a note or a withdrawal
        // counts with no signature"). So is another tool's entry of a
        // person (lineage).
        if !state.persons.contains_key(name) {
            return Err(format!("no person {name} is registered"));
        }
        return match proof {
            Proof::Unsigned => Ok(()),
            _ => Err(format!("{act} carries the person's name and no signature")),
        };
    }
    let key = if state.persons.is_empty() {
        match &entry.act {
            Act::Registered { person, key, owner } if person == name && *owner => {
                registered_key(key)?
            }
            _ => return Err(
                "a store's first entry registers its first owner, signed by the key it registers"
                    .to_owned(),
            ),
        }
    } else {
        match state.persons.get(name) {
            Some(person) => person.key.clone(),
            None => return Err(format!("no person {name} is registered")),
        }
    };
    match proof {
        Proof::Unsigned => Err("a person's act needs their signature".to_owned()),
        Proof::ToBeSigned(signer) if signer.key_data() == key.key_data() => Ok(()),
        Proof::ToBeSigned(_) => Err(format!("the key is not the one registered to {name}")),
        Proof::Signed(signature) => {
            sign::verify(&key, signed.as_bytes(), signature).map_err(|why| {
                if why != "its signature is by another key" {
                    return why;
                }
                match sign::signing_key(signature)
                    .as_ref()
                    .and_then(|signer| state.person_by_key(signer))
                {
                    Some(other) => format!("it is signed by {}'s key, not {name}'s", other.name),
                    None => "its key is not registered".to_owned(),
                }
            })
        }
    }
}

/// An agent's entry: unsigned, a proposal, naming its provenance.
fn check_agent(act: &Act, provenance: &str, proof: Proof<'_>) -> Result<(), String> {
    if !matches!(proof, Proof::Unsigned) {
        return Err("an agent's entry carries no signature".to_owned());
    }
    let what = match act {
        Act::Proposed { .. } => "proposal",
        Act::Withdrawn { .. } => "withdrawal",
        Act::Other { .. } => "entry",
        Act::Accepted { .. } => {
            return Err("only a person accepts: a person's act needs their signature".to_owned())
        }
        Act::Rejected { .. } => {
            return Err("only a person rejects: a person's act needs their signature".to_owned())
        }
        Act::Registered { .. } => {
            return Err(
                "only an owner registers a person: a person's act needs their signature".to_owned(),
            )
        }
        Act::Repository { .. } => {
            return Err(
                "only an owner sets a repository: a person's act needs their signature".to_owned(),
            )
        }
        Act::Noted { .. } => return Err("only a person writes a note".to_owned()),
        Act::Approved { .. } => {
            return Err(
                "only a person approves a plan: a person's act needs their signature".to_owned(),
            )
        }
    };
    if provenance.trim().is_empty() {
        return Err(format!(
            "an agent's {what} names its provenance, with --provenance"
        ));
    }
    Ok(())
}

fn actor_name(actor: &Actor) -> Option<&str> {
    match actor {
        Actor::Person(name) => Some(name),
        Actor::Agent { .. } => None,
    }
}

/// The act's own rules, against the state built so far.
fn check_act(state: &State, entry: &Entry) -> Result<Verdict, String> {
    match &entry.act {
        Act::Registered { person, key, owner } => {
            check_registration(state, &entry.actor, person, key, *owner)
        }
        Act::Proposed {
            claim,
            imported,
            attach,
        } => {
            if let Some(imported) = imported {
                check_imported(state, claim, imported)?;
            }
            check_proposal(state, claim, attach.as_ref(), entry.rules)
        }
        Act::Accepted { versions } => {
            // One act, judged whole: each version counts, or each is
            // accepted already by this person and the act repeats.
            let accepter = actor_name(&entry.actor).unwrap_or_default();
            let mut verdicts = Vec::with_capacity(versions.len());
            let mut claims: Vec<Key> = Vec::with_capacity(versions.len());
            for named in versions {
                if let Some(claim) = state.holding(&named.name, &named.version) {
                    if claims.contains(&claim.key()) {
                        return Err(format!(
                            "an accept names each claim one time: it names two versions of {}",
                            named.name
                        ));
                    }
                    claims.push(claim.key());
                }
                verdicts.push(
                    check_accept(state, &named.name, &named.version, accepter, entry.rules)
                        .map_err(|why| naming(&named.name, &named.version, why))?,
                );
            }
            whole("accept", "accepted", versions, &verdicts)
        }
        Act::Rejected { versions, .. } => {
            let rejecter = actor_name(&entry.actor).unwrap_or_default();
            let mut verdicts = Vec::with_capacity(versions.len());
            for named in versions {
                verdicts.push(
                    check_reject(state, &named.name, &named.version, rejecter, entry.rules)
                        .map_err(|why| naming(&named.name, &named.version, why))?,
                );
            }
            whole("reject", "rejected", versions, &verdicts)
        }
        Act::Withdrawn { versions, .. } => {
            let mut verdicts = Vec::with_capacity(versions.len());
            for named in versions {
                verdicts.push(
                    check_withdraw(state, &named.name, &named.version, &entry.actor)
                        .map_err(|why| naming(&named.name, &named.version, why))?,
                );
            }
            whole("withdrawal", "withdrawn", versions, &verdicts)
        }
        Act::Repository {
            name,
            remote,
            session,
        } => check_repository(state, &entry.actor, name, remote, *session),
        Act::Noted {
            name,
            version,
            paragraph,
            fingerprint,
            words,
        } => {
            let person = actor_name(&entry.actor).unwrap_or_default();
            let meant = Meant {
                name,
                version,
                paragraph: *paragraph,
                fingerprint,
            };
            check_note(state, &meant, words, person)
        }
        // Boom judges another tool's entry by its form, its actor and its
        // links alone, checked before.
        Act::Other { .. } => Ok(Verdict::Counts),
        Act::Approved {
            plan,
            links,
            covers,
            ..
        } => check_approval(state, plan, links, covers),
    }
}

/// The verdict of one act on several versions, judged whole: it counts
/// where each version counts, and repeats where each repeats; a mix of
/// the two is refused, naming the first version that repeats.
fn whole(
    act: &str,
    done: &str,
    versions: &[crate::entry::Named],
    verdicts: &[Verdict],
) -> Result<Verdict, String> {
    match verdicts {
        [Verdict::Repeats(seq), rest @ ..]
            if rest.iter().all(|v| matches!(v, Verdict::Repeats(_))) =>
        {
            Ok(Verdict::Repeats(*seq))
        }
        _ => match versions
            .iter()
            .zip(verdicts)
            .find(|(_, verdict)| matches!(verdict, Verdict::Repeats(_)))
        {
            Some((named, _)) => Err(format!(
                "{}@{} is {done} already: an {act} of several versions names only versions that wait",
                named.name, named.version
            )),
            None => Ok(Verdict::Counts),
        },
    }
}

/// What a note names: a version of a narrative, and one of its
/// paragraphs by its number and its fingerprint.
struct Meant<'a> {
    name: &'a str,
    version: &'a str,
    paragraph: u64,
    fingerprint: &'a str,
}

/// A note: on a version of a narrative whose body the store holds, on a
/// paragraph it has, with that paragraph's fingerprint; the same note
/// again, a repeat (narrative-notes, "A note names what it meant").
fn check_note(
    state: &State,
    meant: &Meant<'_>,
    words: &str,
    person: &str,
) -> Result<Verdict, String> {
    let Meant {
        name,
        version,
        paragraph,
        fingerprint,
    } = *meant;
    let (claim, found) = find(state, name, version)?;
    if claim.kind != Kind::Narrative {
        return Err(format!(
            "a note is on a narrative: {name} is {} {}",
            claim::article(claim.kind),
            claim.kind.as_str()
        ));
    }
    let body = found.proposal.body.as_deref().unwrap_or_default();
    let paragraphs = state
        .paragraphs
        .get(body)
        .ok_or_else(|| format!("the store does not hold the body of {name}@{version}"))?;
    let Some(held) = usize::try_from(paragraph - 1)
        .ok()
        .and_then(|at| paragraphs.get(at))
    else {
        return Err(format!(
            "{name}@{version} has {}: it has no paragraph {paragraph}",
            paragraphs_words(paragraphs.len())
        ));
    };
    if held != fingerprint {
        return Err(format!(
            "paragraph {paragraph} of {name}@{version} has the fingerprint {held}, not {fingerprint}"
        ));
    }
    let note = crate::narrative::note_name(name, version, paragraph, words, person);
    if let Some(written) = state.notes.get(&note) {
        return Ok(Verdict::Repeats(written.noted_in));
    }
    Ok(Verdict::Counts)
}

/// `n paragraphs`, or `1 paragraph`.
pub fn paragraphs_words(count: usize) -> String {
    if count == 1 {
        "1 paragraph".to_owned()
    } else {
        format!("{count} paragraphs")
    }
}

/// A repository's settings: an owner's act; a remote no other repository
/// has; the settings it holds already, a repeat.
fn check_repository(
    state: &State,
    actor: &Actor,
    name: &str,
    remote: &str,
    session: bool,
) -> Result<Verdict, String> {
    let owner = actor_name(actor).and_then(|name| state.persons.get(name));
    if !owner.is_some_and(|owner| owner.owner) {
        return Err("only an owner sets a repository".to_owned());
    }
    if let Some(other) = state
        .repositories
        .values()
        .find(|other| other.remote == remote && other.name != name)
    {
        return Err(format!(
            "the remote {remote} is the remote of {}",
            other.name
        ));
    }
    if let Some(set) = state.repositories.get(name) {
        if set.remote == remote && set.session == session {
            return Ok(Verdict::Repeats(set.set_in));
        }
    }
    Ok(Verdict::Counts)
}

fn check_registration(
    state: &State,
    actor: &Actor,
    person: &str,
    key: &str,
    owner: bool,
) -> Result<Verdict, String> {
    let key = registered_key(key)?;
    if state.persons.is_empty() {
        // The first owner registers themselves (check_person).
        return Ok(Verdict::Counts);
    }
    let registrar = actor_name(actor).and_then(|name| state.persons.get(name));
    if !registrar.is_some_and(|registrar| registrar.owner) {
        return Err("only an owner registers a person".to_owned());
    }
    if let Some(registered) = state.persons.get(person) {
        if registered.key.key_data() == key.key_data() && registered.owner == owner {
            return Ok(Verdict::Repeats(registered.registered_in));
        }
        return Err(format!(
            "{person} is registered already, by entry {}",
            registered.registered_in
        ));
    }
    if let Some(other) = state.person_by_key(&key) {
        return Err(format!("that key is registered to {}", other.name));
    }
    Ok(Verdict::Counts)
}

fn check_proposal(
    state: &State,
    proposal: &Proposal,
    attach: Option<&Attach>,
    rules: u32,
) -> Result<Verdict, String> {
    let name = &proposal.name;
    if let Some(held) = state.holding(name, &proposal.version) {
        // The same version under the same attachment is a repeat;
        // under another, a move, which no proposal makes (record
        // 0008, point 4).
        let version = held
            .version(&proposal.version)
            .expect("the claim holds the version");
        if same_binding(&version.repositories, &version.contexts, attach) {
            return Ok(Verdict::Repeats(version.proposed_in));
        }
        return Err(format!(
            "{name}@{} is attached to {}; a version's attachment does not move",
            proposal.version,
            crate::context::binds_words(&version.repositories, &version.contexts)
        ));
    }
    // Each context it binds is a version of a context of the store
    // (context-claims, "A claim binds a context").
    for stored in contexts(attach) {
        let (target, version) = claim::edge(stored)?;
        let (context, _) =
            find(state, target, version).map_err(|why| format!("--context {stored}: {why}"))?;
        if context.kind != Kind::Context {
            return Err(format!(
                "{target} is not a context: it is {} {}",
                claim::article(context.kind),
                context.kind.as_str()
            ));
        }
    }
    // One name space, except that a word has a term in each repository
    // (term-claims).
    if let Some(other) = state
        .named(name)
        .into_iter()
        .find(|claim| claim.kind != proposal.kind)
    {
        return Err(format!(
            "the name {name} is taken by the {} {name}",
            other.kind.as_str()
        ));
    }
    let term = (proposal.kind == Kind::Term)
        .then(|| repositories(attach).first().map(String::as_str))
        .flatten();
    let claim = state.claims.get(&key(name, term));
    match (&proposal.base, claim) {
        (Some(base), Some(claim)) if claim.version(base).is_some() => {
            // A draft has one line: a draft that a newer draft
            // superseded is not amended again (claim-versions, "A new
            // draft supersedes the draft it amends").
            if let Some(newer) = claim.newer_draft(base).filter(|_| rules >= ONE_DRAFT_LINE) {
                let newer = &newer.proposal.version;
                return Err(format!(
                    "{name}@{base} is superseded by the newer draft {name}@{newer}: amend that draft, with --amends {newer}"
                ));
            }
        }
        (Some(base), _) => {
            if let Some(other) = state
                .holding(name, base)
                .filter(|other| proposal.kind == Kind::Term && other.kind == Kind::Term)
            {
                return Err(format!(
                    "the term {name}@{base} binds {}, and a term amends a version of its own binding, not {}",
                    binding_words(other.repository.as_deref()),
                    binding_words(term)
                ));
            }
            return Err(format!("the claim {name} has no version {base}"));
        }
        (
            None,
            Some(Claim {
                standing: Some(standing),
                ..
            }),
        ) => {
            return Err(format!(
                "{name}@{standing} is accepted: amend it, with --amends {standing}"
            ))
        }
        (None, _) => {}
    }
    // Each edge to a claim names a version the store holds (claim-edges).
    let cause: Vec<String> = proposal.cause.iter().cloned().collect();
    for (flag, edges) in [
        ("source", &proposal.source),
        ("changes", &proposal.changes),
        ("includes", &proposal.includes),
        ("cause", &cause),
    ] {
        for stored in edges {
            let (target, version) = claim::edge(stored)?;
            let (found, _) =
                find(state, target, version).map_err(|why| format!("--{flag} {stored}: {why}"))?;
            if flag == "includes" && found.kind == Kind::Narrative {
                return Err(format!(
                    "a narrative includes claims of other kinds: --includes {stored} names a narrative"
                ));
            }
        }
    }
    if proposal.kind == Kind::Narrative {
        if rules >= ONE_LEVEL {
            check_level(proposal, attach)?;
        }
        check_answers(state, proposal)?;
    }
    if proposal.kind == Kind::Outline {
        check_outline(state, proposal, attach)?;
    }
    Ok(Verdict::Counts)
}

/// A narrative binds one level: the whole system, one repository or one
/// context (narratives, "A narrative binds one level").
fn check_level(proposal: &Proposal, attach: Option<&Attach>) -> Result<(), String> {
    let bound = repositories(attach).len() + contexts(attach).len();
    if bound > 1 {
        let attach = attach.expect("a narrative that binds two binds something");
        return Err(format!(
            "a narrative binds one level, the whole system, one repository or one context: {} binds {}",
            proposal.name,
            crate::context::binds_words(&attach.repositories, &attach.contexts)
        ));
    }
    Ok(())
}

/// An outline: it binds the whole system, each narrative it holds is a
/// narrative of the store, and a first version of a book's outline waits
/// until no other outline of the view `book` has a version that is not
/// rejected (views).
fn check_outline(
    state: &State,
    proposal: &Proposal,
    attach: Option<&Attach>,
) -> Result<(), String> {
    if let Some(attach) = attach {
        return Err(format!(
            "an outline binds the whole system: {} binds {}",
            proposal.name,
            crate::context::binds_words(&attach.repositories, &attach.contexts)
        ));
    }
    for stored in &proposal.narratives {
        let (target, version) = claim::edge(stored)?;
        let (found, _) =
            find(state, target, version).map_err(|why| format!("--narrative {stored}: {why}"))?;
        if found.kind != Kind::Narrative {
            return Err(format!(
                "an outline holds narratives: --narrative {stored} names {} {}",
                claim::article(found.kind),
                found.kind.as_str()
            ));
        }
    }
    if proposal.base.is_none() && proposal.view.as_deref() == Some(claim::BOOK) {
        if let Some(other) = state.claims.values().find(|other| {
            other.kind == Kind::Outline
                && other.name != proposal.name
                && other.versions.iter().any(|version| {
                    !matches!(version.status, Status::Rejected | Status::Withdrawn)
                        && version.proposal.view.as_deref() == Some(claim::BOOK)
                })
        }) {
            return Err(format!(
                "a store has one design book, and the outline {} is its outline: amend it, or reject each of its versions first",
                other.name
            ));
        }
    }
    Ok(())
}

/// The answers of a new version of a narrative: one to each open note of
/// the narrative, and none to a note that does not exist, is of another
/// narrative or is not open (narrative-notes, "A note is open until a
/// version answers it").
fn check_answers(state: &State, proposal: &Proposal) -> Result<(), String> {
    let name = &proposal.name;
    for answer in &proposal.answers {
        let Some(note) = state.notes.get(&answer.note) else {
            return Err(format!(
                "--answers {}: no note has the name {}",
                answer.note, answer.note
            ));
        };
        if &note.narrative != name {
            return Err(format!(
                "--answers {}: the note {} is on the narrative {}, not on {name}",
                answer.note, answer.note, note.narrative
            ));
        }
        if let Some((version, _)) = state.answer_of(note) {
            return Err(format!(
                "--answers {}: the note {} is not open: {name}@{} answers it",
                answer.note, answer.note, version.proposal.version
            ));
        }
    }
    let left: Vec<String> = state
        .open_notes(name)
        .into_iter()
        .filter(|note| {
            !proposal
                .answers
                .iter()
                .any(|answer| answer.note == note.name)
        })
        .map(|note| {
            format!(
                "note {} on paragraph {} of {name}@{} ({}): {:?}",
                note.name, note.paragraph, note.version, note.person, note.words
            )
        })
        .collect();
    if !left.is_empty() {
        return Err(format!(
            "a new version of {name} answers each open note, with --answers <note>=<words>; it leaves open {}",
            left.join("; ")
        ));
    }
    Ok(())
}

/// The rules an imported proposal is held to, after those every
/// proposal keeps (design 7): its record read once, its name held by no
/// claim, and a decision's number held by no other claim that came in
/// with it. A record that came in before as this version passes, for
/// `check_proposal` to answer as a repeat where its attachment is the
/// same.
fn check_imported(state: &State, proposal: &Proposal, imported: &Imported) -> Result<(), String> {
    let name = &proposal.name;
    let versions = || {
        state.claims.values().flat_map(|claim| {
            claim
                .versions
                .iter()
                .filter_map(move |version| Some((claim, version, version.imported.as_ref()?)))
        })
    };
    if let Some((claim, version, _)) =
        versions().find(|(_, _, from)| from.source() == imported.source())
    {
        if &claim.name == name && version.proposal.version == proposal.version {
            return Ok(());
        }
        let held = &version.proposal.version;
        return Err(format!(
            "{} came in as {}@{held}: boom keeps the record now and never takes its file again as the record; amend that version, with boom propose --amends {held}",
            imported.named(),
            claim.name,
        ));
    }
    if let Some(claim) = state.named(name).first().copied() {
        return Err(format!(
            "the name {name} is held by the {} {name}: name {} otherwise in its draft",
            claim.kind.as_str(),
            imported.named(),
        ));
    }
    if let (Form::Madr, Some(number)) = (imported.form, &imported.legacy) {
        if let Some((claim, _, from)) = versions()
            .find(|(_, _, from)| from.form == Form::Madr && from.legacy.as_ref() == Some(number))
        {
            return Err(format!(
                "the number {number} of {} is held by the {} {}, which came in with it from {}",
                imported.named(),
                claim.kind.as_str(),
                claim.name,
                from.named(),
            ));
        }
    }
    Ok(())
}

fn find<'a>(
    state: &'a State,
    name: &str,
    version: &str,
) -> Result<(&'a Claim, &'a Version), String> {
    let claim = match state.holding(name, version) {
        Some(claim) => claim,
        None if state.named(name).is_empty() => {
            return Err(format!("no claim has the name {name}"))
        }
        None => return Err(format!("the claim {name} has no version {version}")),
    };
    let found = claim.version(version).expect("the claim holds the version");
    Ok((claim, found))
}

fn check_accept(
    state: &State,
    name: &str,
    version: &str,
    accepter: &str,
    rules: u32,
) -> Result<Verdict, String> {
    let (claim, found) = find(state, name, version)?;
    let by = found.decided_by.as_deref().unwrap_or_default();
    match claim.status_under(found, rules) {
        Status::Proposed => {}
        // A record that came in as history and was accepted keeps its
        // file's status.
        _ if found.proposal.history.is_some() && by == accepter => {
            return Ok(Verdict::Repeats(found.decided_in.unwrap_or_default()))
        }
        _ if found.proposal.history.is_some() => {
            return Err(format!(
                "{name}@{version} came in as history and is {} already, accepted by {by}",
                found.status.as_str()
            ))
        }
        Status::Accepted if by == accepter => {
            return Ok(Verdict::Repeats(found.decided_in.unwrap_or_default()))
        }
        Status::Accepted => return Err(format!("{name}@{version} is accepted already, by {by}")),
        Status::Superseded => {
            if let Some(newer) = claim.newer_draft(version) {
                return Err(format!(
                    "{name}@{version} is superseded by the newer draft {name}@{}: accept that draft",
                    newer.proposal.version
                ));
            }
            let accepted = claim.standing.as_deref().unwrap_or_default();
            return Err(format!(
                "{name}@{version} is superseded: {name}@{accepted} is accepted"
            ));
        }
        Status::Rejected => return Err(format!("{name}@{version} is rejected, by {by}")),
        Status::Deprecated => return Err(format!("{name}@{version} is deprecated")),
        Status::Withdrawn => return Err(withdrawn_words(name, version, found)),
    }
    if found.proposer == Actor::Person(accepter.to_owned()) {
        return Err(format!(
            "{name}@{version} waits on another person: no one accepts their own proposal"
        ));
    }
    // A version that comes in as history never stands, so it is accepted
    // whatever stands (design 6). Any other is accepted on the nearest
    // version in its line of bases that a person accepted, past the
    // drafts before it (claim-versions, "An accept stands only on its
    // base").
    let base = claim.accepted_base(found);
    if found.proposal.history.is_none() && base != claim.standing.as_deref() {
        let builds_on = base.map_or("no version".to_owned(), |base| format!("{name}@{base}"));
        return Err(match &claim.standing {
            Some(standing) => format!(
                "{name}@{standing} is accepted, and {name}@{version} builds on {builds_on}: propose its words again amending {standing}"
            ),
            None => format!("{name}@{version} builds on {builds_on}, which is not accepted"),
        });
    }
    Ok(Verdict::Counts)
}

fn check_reject(
    state: &State,
    name: &str,
    version: &str,
    rejecter: &str,
    rules: u32,
) -> Result<Verdict, String> {
    let (claim, found) = find(state, name, version)?;
    let by = found.decided_by.as_deref().unwrap_or_default();
    if let Some(newer) = claim
        .newer_draft(version)
        .filter(|_| rules >= ONE_DRAFT_LINE)
    {
        return Err(format!(
            "{name}@{version} is superseded by the newer draft {name}@{}: only a proposed version is rejected",
            newer.proposal.version
        ));
    }
    match claim.status_under(found, rules) {
        Status::Proposed => Ok(Verdict::Counts),
        // A person's reject carries its reason; a record that came in as
        // history, rejected as its file says, was accepted.
        Status::Rejected if found.reason.is_some() && by == rejecter => {
            Ok(Verdict::Repeats(found.decided_in.unwrap_or_default()))
        }
        Status::Rejected if found.reason.is_some() => {
            Err(format!("{name}@{version} is rejected already, by {by}"))
        }
        Status::Withdrawn => Err(withdrawn_words(name, version, found)),
        status => Err(format!(
            "only a proposed version is rejected: {name}@{version} is {}",
            status.as_str()
        )),
    }
}

/// `name@version is withdrawn`, with its reason.
fn withdrawn_words(name: &str, version: &str, found: &Version) -> String {
    format!(
        "{name}@{version} is withdrawn by its author: {}",
        found.reason.as_deref().unwrap_or_default()
    )
}

/// A withdrawal of `name@version` by `actor`: a proposed version, which
/// its author withdraws, an agent a version that an agent proposed and a
/// person a version that this person proposed; a version withdrawn
/// already, a repeat (claim-versions, "The author withdraws a proposed
/// version").
fn check_withdraw(
    state: &State,
    name: &str,
    version: &str,
    actor: &Actor,
) -> Result<Verdict, String> {
    let (claim, found) = find(state, name, version)?;
    if let Some(newer) = claim.newer_draft(version) {
        return Err(format!(
            "{name}@{version} is superseded by the newer draft {name}@{}: only a proposed version is withdrawn",
            newer.proposal.version
        ));
    }
    let author = match (&found.proposer, actor) {
        (Actor::Agent { .. }, Actor::Agent { .. }) => true,
        (Actor::Person(proposer), Actor::Person(withdrawer)) => proposer == withdrawer,
        _ => false,
    };
    if !author {
        let by = match &found.proposer {
            Actor::Person(person) => person.clone(),
            Actor::Agent { .. } => "an agent".to_owned(),
        };
        return Err(format!(
            "only its author withdraws {name}@{version}: {by} proposed it"
        ));
    }
    match found.status {
        Status::Proposed => Ok(Verdict::Counts),
        Status::Withdrawn => Ok(Verdict::Repeats(found.decided_in.unwrap_or_default())),
        status => Err(format!(
            "only a proposed version is withdrawn: {name}@{version} is {}",
            status.as_str()
        )),
    }
}

/// An approval of a plan: it covers a claim or links an entry, each
/// version it covers is accepted and its proposal linked, and no counted
/// approval approves its plan (plan-approvals).
fn check_approval(
    state: &State,
    plan: &str,
    links: &[String],
    covers: &[String],
) -> Result<Verdict, String> {
    if covers.is_empty() && links.is_empty() {
        return Err(
            "an approval covers a claim or links an entry that the plan took in: this one does neither"
                .to_owned(),
        );
    }
    if let Some(approved) = state
        .approvals
        .values()
        .find(|approval| approval.plan == plan)
    {
        return Err(format!(
            "the plan {plan} is approved already, by entry {}",
            approved.seq
        ));
    }
    let mut not_accepted = Vec::new();
    for covered in covers {
        let (name, version) = claim::edge(covered)?;
        let (_, found) = find(state, name, version)?;
        if found.status != Status::Accepted {
            not_accepted.push(format!("{covered} is {}", found.status.as_str()));
            continue;
        }
        let linked = links.iter().any(|link| {
            state
                .acts
                .get(link)
                .is_some_and(|acted| acted.seq == found.proposed_in)
        });
        if !linked {
            return Err(format!(
                "an approval links the entry that proposed each version it covers: it does not link entry {}, which proposed {covered}",
                found.proposed_in
            ));
        }
    }
    if !not_accepted.is_empty() {
        return Err(format!(
            "an approval covers only accepted versions: {}",
            not_accepted.join(", ")
        ));
    }
    Ok(Verdict::Counts)
}

/// Applies an entry that counts, numbered `seq`, whose signed part as
/// stored is `signed`, to the state.
pub fn apply(state: &mut State, seq: u64, signed: &str, entry: &Entry) {
    let actor = actor_name(&entry.actor).map(str::to_owned);
    state
        .acts
        .entry(crate::entry::act_hash(signed))
        .or_insert_with(|| Acted {
            seq,
            act: entry.act.name().to_owned(),
        });
    match &entry.act {
        Act::Other { .. } => {}
        Act::Approved { plan, covers, .. } => {
            state.approvals.insert(
                crate::entry::act_hash(signed),
                Approval {
                    seq,
                    plan: plan.clone(),
                    covers: covers.clone(),
                },
            );
        }
        Act::Registered { person, key, owner } => {
            let key = registered_key(key).expect("a counted registration's key reads");
            state.first_owner.get_or_insert_with(|| person.clone());
            state.persons.insert(
                person.clone(),
                Person {
                    name: person.clone(),
                    key,
                    owner: *owner,
                    registered_in: seq,
                },
            );
        }
        Act::Proposed {
            claim,
            imported,
            attach,
        } => {
            let term = (claim.kind == Kind::Term)
                .then(|| repositories(attach.as_ref()).first().cloned())
                .flatten();
            let entry_claim = state
                .claims
                .entry(key(&claim.name, term.as_deref()))
                .or_insert_with(|| Claim {
                    name: claim.name.clone(),
                    kind: claim.kind,
                    repository: term.clone(),
                    standing: None,
                    versions: Vec::new(),
                });
            entry_claim.versions.push(Version {
                proposal: (**claim).clone(),
                repositories: repositories(attach.as_ref()).to_vec(),
                contexts: contexts(attach.as_ref()).to_vec(),
                proposer: entry.actor.clone(),
                proposed_in: seq,
                status: Status::Proposed,
                decided_by: None,
                decided_in: None,
                reason: None,
                imported: imported.as_deref().cloned(),
            });
            // A new draft supersedes the draft it amends: this entry is
            // the event, and the old draft's entry stays as it is
            // (claim-versions, "A new draft supersedes the draft it
            // amends").
            if let Some(base) = &claim.base {
                if let Some(draft) = entry_claim
                    .version_mut(base)
                    .filter(|draft| draft.status == Status::Proposed)
                {
                    draft.status = Status::Superseded;
                }
            }
        }
        Act::Accepted { versions } => {
            for named in versions {
                apply_accept(state, seq, actor.clone(), &named.name, &named.version);
            }
        }
        Act::Repository {
            name,
            remote,
            session,
        } => {
            state.repositories.insert(
                name.clone(),
                Repository {
                    name: name.clone(),
                    remote: remote.clone(),
                    session: *session,
                    set_by: actor.unwrap_or_default(),
                    set_in: seq,
                },
            );
        }
        Act::Rejected { versions, reason } => {
            for named in versions {
                close(state, seq, named, Status::Rejected, actor.clone(), reason);
            }
        }
        Act::Withdrawn { versions, reason } => {
            for named in versions {
                close(state, seq, named, Status::Withdrawn, actor.clone(), reason);
            }
        }
        Act::Noted {
            name,
            version,
            paragraph,
            fingerprint,
            words,
        } => {
            let person = actor.unwrap_or_default();
            let note = crate::narrative::note_name(name, version, *paragraph, words, &person);
            state.notes.insert(
                note.clone(),
                Note {
                    name: note,
                    narrative: name.clone(),
                    version: version.clone(),
                    paragraph: *paragraph,
                    fingerprint: fingerprint.clone(),
                    words: words.clone(),
                    person,
                    at: entry.at,
                    noted_in: seq,
                },
            );
        }
    }
}

/// Judges one entry of the log, as stored, and applies it where it
/// counts.
pub fn step(state: &mut State, seq: u64, signed: &str, signature: Option<&str>) -> Judgment {
    let entry = match Entry::parse(signed) {
        Ok(entry) => entry,
        Err(why) => return Judgment::of(Verdict::Refused(why)),
    };
    let verdict = judge(state, &entry, signed, Proof::of(signature));
    if verdict == Verdict::Counts {
        apply(state, seq, signed, &entry);
    }
    Judgment::of(verdict)
}

/// Replays a log's entries, `(seq, signed, signature)` in order, through
/// the judge: the state they build, and each entry's judgment. It reads
/// no blob, so a note counts for nothing: [`replay_with`] reads them.
pub fn replay<'a>(
    entries: impl IntoIterator<Item = (u64, &'a str, Option<&'a str>)>,
) -> (State, Vec<(u64, Judgment)>) {
    replay_with(entries, |_, _| {})
}

/// Replays a log's entries as [`replay`] does, calling `counted` with the
/// state and each entry that counts, after it is applied: where the store
/// reads the body of a narrative proposed, for the notes after it.
pub fn replay_with<'a>(
    entries: impl IntoIterator<Item = (u64, &'a str, Option<&'a str>)>,
    mut counted: impl FnMut(&mut State, &Entry),
) -> (State, Vec<(u64, Judgment)>) {
    let mut state = State::default();
    let judgments = entries
        .into_iter()
        .map(|(seq, signed, signature)| {
            let judgment = step(&mut state, seq, signed, signature);
            if judgment.counted {
                if let Ok(entry) = Entry::parse(signed) {
                    counted(&mut state, &entry);
                }
            }
            (seq, judgment)
        })
        .collect();
    (state, judgments)
}

/// A counted reject or withdrawal of `named`, applied to the state: its
/// status, who decided it where a person did, the entry and the reason.
fn close(
    state: &mut State,
    seq: u64,
    named: &crate::entry::Named,
    status: Status,
    by: Option<String>,
    reason: &str,
) {
    let claim = state
        .holding_mut(&named.name, &named.version)
        .expect("a counted close's claim");
    let closed = claim
        .version_mut(&named.version)
        .expect("a counted close's version");
    closed.status = status;
    closed.decided_by = by;
    closed.decided_in = Some(seq);
    closed.reason = Some(reason.to_owned());
}

/// A counted accept of `name@version`, applied to the state.
fn apply_accept(state: &mut State, seq: u64, actor: Option<String>, name: &str, version: &str) {
    let claim = state
        .holding_mut(name, version)
        .expect("a counted accept's claim");
    let accepted = claim
        .version_mut(version)
        .expect("a counted accept's version");
    if let Some(history) = &accepted.proposal.history {
        // History is never in force and supersedes no version: it
        // keeps its file's own status.
        accepted.status = match history.status {
            claim::HistoryStatus::Superseded => Status::Superseded,
            claim::HistoryStatus::Deprecated => Status::Deprecated,
            claim::HistoryStatus::Rejected => Status::Rejected,
        };
        accepted.decided_by = actor;
        accepted.decided_in = Some(seq);
        return;
    }
    if let Some(standing) = claim.standing.take() {
        if let Some(superseded) = claim.version_mut(&standing) {
            superseded.status = Status::Superseded;
        }
    }
    let accepted = claim
        .version_mut(version)
        .expect("a counted accept's version");
    accepted.status = Status::Accepted;
    accepted.decided_by = actor;
    accepted.decided_in = Some(seq);
    claim.standing = Some(version.to_owned());
}

/// A refusal of `name@version` that names it.
fn naming(name: &str, version: &str, why: String) -> String {
    let target = format!("{name}@{version}");
    if why.contains(&target) {
        why
    } else {
        format!("{target}: {why}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A state whose claims each stand, `(kind, name, repositories)`,
    /// and one whose only version is proposed.
    fn state(standing: &[(Kind, &str, &[&str])]) -> State {
        let mut state = State::default();
        let version = |kind, name: &str, repositories: &[&str], status| {
            let proposal = Proposal::new(
                kind,
                name,
                "It holds.",
                vec!["Given it, then it holds.".to_owned()],
                None,
                None,
                None,
            );
            Version {
                proposal,
                repositories: repositories.iter().map(|&name| name.to_owned()).collect(),
                contexts: Vec::new(),
                proposer: Actor::Agent {
                    provenance: "run-1".to_owned(),
                },
                proposed_in: 2,
                status,
                decided_by: (status == Status::Accepted).then_some("ada".to_owned()),
                decided_in: (status == Status::Accepted).then_some(3),
                reason: None,
                imported: None,
            }
        };
        let mut insert = |kind, name: &str, repositories: &[&str], status| {
            let version = version(kind, name, repositories, status);
            state.claims.insert(
                key(name, None),
                Claim {
                    name: name.to_owned(),
                    kind,
                    repository: None,
                    standing: (status == Status::Accepted)
                        .then_some(version.proposal.version.clone()),
                    versions: vec![version],
                },
            );
        };
        for &(kind, name, repositories) in standing {
            insert(kind, name, repositories, Status::Accepted);
        }
        insert(Kind::Invariant, "a-proposal", &[], Status::Proposed);
        state
    }

    fn names(standing: &[(&Claim, &Version)]) -> Vec<String> {
        standing
            .iter()
            .map(|(claim, _)| claim.name.clone())
            .collect()
    }

    #[test]
    fn standing_for_a_repository_gives_its_own_and_the_whole_systems() {
        let state = state(&[
            (Kind::Invariant, "in-shop", &["shop"]),
            (Kind::Invariant, "in-the-system", &[]),
            (Kind::Invariant, "in-ledger", &["ledger"]),
            (Kind::Decision, "in-both", &["ledger", "shop"]),
        ]);
        let standing = state.standing_for(Some("shop"), &[]);
        assert_eq!(names(&standing), ["in-shop", "in-the-system", "in-both"]);
        for (claim, version) in &standing {
            assert_eq!(
                claim.standing.as_deref(),
                Some(version.proposal.version.as_str())
            );
            assert_eq!(version.status, Status::Accepted);
        }
        assert_eq!(
            names(&state.standing_for(Some("ledger"), &[])),
            ["in-ledger", "in-the-system", "in-both"]
        );
        assert_eq!(
            names(&state.standing_for(Some("docs"), &[])),
            ["in-the-system"]
        );
    }

    #[test]
    fn standing_for_every_repository_gives_each_in_kind_and_name_order() {
        let state = state(&[
            (Kind::Assumption, "alpha-assumed", &["shop"]),
            (Kind::Decision, "beta-kept", &["ledger"]),
            (Kind::Decision, "alpha-kept", &[]),
            (Kind::Invariant, "gamma-holds", &["shop"]),
            (Kind::Invariant, "delta-holds", &["docs"]),
        ]);
        assert_eq!(
            names(&state.standing_for(None, &[])),
            [
                "delta-holds",
                "gamma-holds",
                "alpha-kept",
                "beta-kept",
                "alpha-assumed"
            ]
        );
    }

    #[test]
    fn standing_for_a_repository_gives_the_claims_of_the_contexts_named() {
        let mut state = state(&[
            (Kind::Invariant, "in-shop", &["shop"]),
            (Kind::Invariant, "in-the-system", &[]),
            (Kind::Invariant, "in-billing", &[]),
            (Kind::Invariant, "in-shipping", &[]),
        ]);
        for (name, context) in [
            ("in-billing", "shop-billing@aaaaaaaaaaaa"),
            ("in-shipping", "shop-shipping@aaaaaaaaaaaa"),
        ] {
            let claim = state.claims.get_mut(&key(name, None)).unwrap();
            claim.versions[0].contexts = vec![context.to_owned()];
        }
        assert_eq!(
            names(&state.standing_for(Some("shop"), &[])),
            ["in-shop", "in-the-system"]
        );
        assert_eq!(
            names(&state.standing_for(Some("shop"), &["shop-billing".to_owned()])),
            ["in-billing", "in-shop", "in-the-system"]
        );
        assert_eq!(
            names(&state.standing_for(None, &[])),
            ["in-billing", "in-shipping", "in-shop", "in-the-system"]
        );
    }

    #[test]
    fn an_edge_names_a_version_the_state_holds() {
        let state = state(&[(Kind::Invariant, "money-has-currency", &[])]);
        let held = state.named("money-has-currency")[0].versions[0]
            .proposal
            .version
            .clone();
        let entry = |source: &str| {
            let mut proposal = Proposal::new(
                Kind::Invariant,
                "amounts-in-cents",
                "Amounts are in cents.",
                Vec::new(),
                None,
                None,
                None,
            );
            proposal.source = vec![source.to_owned()];
            proposal.version = proposal.fingerprint(None);
            Entry {
                at: 1,
                rules: RULES,
                actor: Actor::Agent {
                    provenance: "run-1".to_owned(),
                },
                act: Act::Proposed {
                    claim: Box::new(proposal),
                    imported: None,
                    attach: None,
                },
            }
        };
        let judged = |source: &str| {
            let entry = entry(source);
            judge(&state, &entry, &entry.signed_part(), Proof::Unsigned)
        };
        assert_eq!(
            judged(&format!("money-has-currency@{held}")),
            Verdict::Counts
        );
        assert_eq!(
            judged("money-has-currency@000000000000"),
            Verdict::Refused(
                "--source money-has-currency@000000000000: the claim money-has-currency has no version 000000000000"
                    .to_owned()
            )
        );
        assert_eq!(
            judged("no-such-claim@000000000000"),
            Verdict::Refused(
                "--source no-such-claim@000000000000: no claim has the name no-such-claim"
                    .to_owned()
            )
        );
    }
}
