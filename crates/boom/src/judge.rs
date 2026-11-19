//! The rules that decide whether an entry counts, and the state a
//! replay of the log builds (design 6).
//!
//! One judge, asked before a write signs, again inside the write's
//! transaction, and for every entry whenever the views are rebuilt, so
//! an entry no write would have made counts for nothing
//! (`validated-on-read`). Its rules, in order: the entry's form; for a
//! person's entry, a signature that verifies by the key registered to
//! that person before it, with presence (for the store's first entry,
//! the key it registers); for an agent's, an act that is `proposed` and
//! a provenance; then the act's own rules.

use std::collections::BTreeMap;

use ssh_key::PublicKey;

use crate::claim::{self, Kind, Proposal};
use crate::entry::{Act, Actor, Attach, Entry, Form, Imported};
use crate::{repository, sign};

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

/// A version's status, read from the log (claim-versions).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// Waiting on a person.
    Proposed,
    /// Accepted, and no later version of its claim is.
    Standing,
    /// Accepted, and a later version of its claim is.
    Replaced,
    /// Rejected.
    Rejected,
    /// Accepted as history: it never stands and replaces no version
    /// (design 6).
    History,
}

impl Status {
    /// The status as an answer names it.
    pub fn as_str(self) -> &'static str {
        match self {
            Status::Proposed => "proposed",
            Status::Standing => "standing",
            Status::Replaced => "replaced",
            Status::Rejected => "rejected",
            Status::History => "history",
        }
    }
}

/// A version of a claim and what the log says of it.
#[derive(Debug, Clone)]
pub struct Version {
    /// The version as proposed.
    pub proposal: Proposal,
    /// What it binds: the repositories its proposal names, or none for
    /// the whole system.
    pub repositories: Vec<String>,
    /// Who proposed it.
    pub proposer: Actor,
    /// The entry that proposed it.
    pub proposed_in: u64,
    /// Its status.
    pub status: Status,
    /// The person who accepted or rejected it.
    pub decided_by: Option<String>,
    /// The entry that accepted or rejected it.
    pub decided_in: Option<u64>,
    /// Why it was rejected.
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
    /// The version that stands, where one does.
    pub standing: Option<String>,
    /// Its versions, in the order proposed.
    pub versions: Vec<Version>,
}

impl Claim {
    /// The version named `version`.
    pub fn version(&self, version: &str) -> Option<&Version> {
        self.versions
            .iter()
            .find(|candidate| candidate.proposal.version == version)
    }

    fn version_mut(&mut self, version: &str) -> Option<&mut Version> {
        self.versions
            .iter_mut()
            .find(|candidate| candidate.proposal.version == version)
    }
}

/// What the counted entries of a log, replayed in order, build.
#[derive(Debug, Clone, Default)]
pub struct State {
    /// The persons registered, by name.
    pub persons: BTreeMap<String, Person>,
    /// The claims, by name.
    pub claims: BTreeMap<String, Claim>,
    /// The store's first owner, the person its first counted entry
    /// registers.
    pub first_owner: Option<String>,
}

impl State {
    /// The person `key` is registered to.
    pub fn person_by_key(&self, key: &PublicKey) -> Option<&Person> {
        self.persons
            .values()
            .find(|person| person.key.key_data() == key.key_data())
    }

    /// Each claim that stands with its standing version: for
    /// `Some(repository)`, those whose standing version binds that
    /// repository or the whole system; for `None`, every one. In the
    /// order invariants, decisions, assumptions, each kind by name
    /// (claim-search; design 4).
    pub fn standing_for(&self, repository: Option<&str>) -> Vec<(&Claim, &Version)> {
        let mut standing = Vec::new();
        for kind in [Kind::Invariant, Kind::Decision, Kind::Assumption] {
            for claim in self.claims.values().filter(|claim| claim.kind == kind) {
                let Some(version) = claim
                    .standing
                    .as_deref()
                    .and_then(|version| claim.version(version))
                else {
                    continue;
                };
                let binds = &version.repositories;
                let bound = match repository {
                    None => true,
                    Some(repository) => {
                        binds.is_empty() || binds.iter().any(|name| name == repository)
                    }
                };
                if bound {
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
    check_form(&entry.act)?;
    match &entry.actor {
        Actor::Person(name) => check_person(state, entry, name, signed, proof)?,
        Actor::Agent { provenance } => check_agent(&entry.act, provenance, proof)?,
    }
    check_act(state, entry)
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
            claim.check()?;
            check_attach(attach.as_ref())
        }
        Act::Accepted { name, .. } => claim::check_name(name),
        Act::Rejected { name, reason, .. } => {
            claim::check_name(name)?;
            if reason.trim().is_empty() {
                Err("a reject gives its reason".to_owned())
            } else {
                Ok(())
            }
        }
    }
}

/// An attachment's form: a repository at least, each kebab case, in
/// ascending order and none twice (claim-attachments; design 2).
fn check_attach(attach: Option<&Attach>) -> Result<(), String> {
    let Some(attach) = attach else {
        return Ok(());
    };
    if attach.repositories.is_empty() {
        return Err(
            "an attach names a repository at least: one of the whole system holds no attach"
                .to_owned(),
        );
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

/// The repositories an attachment names, none for the whole system.
pub fn repositories(attach: Option<&Attach>) -> &[String] {
    attach.map_or(&[], |attach| attach.repositories.as_slice())
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
    match act {
        Act::Proposed { .. } => {}
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
    }
    if provenance.trim().is_empty() {
        return Err("an agent's proposal names its provenance, with --provenance".to_owned());
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
            check_proposal(state, claim, attach.as_ref())
        }
        Act::Accepted { name, version } => {
            let accepter = actor_name(&entry.actor).unwrap_or_default();
            check_accept(state, name, version, accepter)
        }
        Act::Rejected { name, version, .. } => {
            let rejecter = actor_name(&entry.actor).unwrap_or_default();
            check_reject(state, name, version, rejecter)
        }
    }
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
) -> Result<Verdict, String> {
    let name = &proposal.name;
    let claim = state.claims.get(name);
    if let Some(claim) = claim {
        if let Some(version) = claim.version(&proposal.version) {
            // The same version under the same attachment is a repeat;
            // under another, a move, which no proposal makes (record
            // 0008, point 4).
            if version.repositories == repositories(attach) {
                return Ok(Verdict::Repeats(version.proposed_in));
            }
            return Err(format!(
                "{name}@{} is attached to {}; a version's attachment does not move",
                proposal.version,
                repository::words(&version.repositories)
            ));
        }
        if claim.kind != proposal.kind {
            return Err(format!(
                "the name {name} is taken by the {} {name}",
                claim.kind.as_str()
            ));
        }
    }
    match (&proposal.base, claim) {
        (Some(base), Some(claim)) if claim.version(base).is_some() => {}
        (Some(base), _) => return Err(format!("the claim {name} has no version {base}")),
        (
            None,
            Some(Claim {
                standing: Some(standing),
                ..
            }),
        ) => {
            return Err(format!(
                "{name}@{standing} stands: amend it, with --amends {standing}"
            ))
        }
        (None, _) => {}
    }
    Ok(Verdict::Counts)
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
    if let Some(claim) = state.claims.get(name) {
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
    let claim = state
        .claims
        .get(name)
        .ok_or_else(|| format!("no claim has the name {name}"))?;
    let found = claim
        .version(version)
        .ok_or_else(|| format!("the claim {name} has no version {version}"))?;
    Ok((claim, found))
}

fn check_accept(
    state: &State,
    name: &str,
    version: &str,
    accepter: &str,
) -> Result<Verdict, String> {
    let (claim, found) = find(state, name, version)?;
    let by = found.decided_by.as_deref().unwrap_or_default();
    match found.status {
        Status::Standing if by == accepter => {
            return Ok(Verdict::Repeats(found.decided_in.unwrap_or_default()))
        }
        Status::Standing => {
            return Err(format!("{name}@{version} stands already, accepted by {by}"))
        }
        Status::Replaced => {
            let standing = claim.standing.as_deref().unwrap_or_default();
            return Err(format!(
                "{name}@{version} is replaced: {name}@{standing} stands"
            ));
        }
        Status::Rejected => return Err(format!("{name}@{version} is rejected, by {by}")),
        Status::History if by == accepter => {
            return Ok(Verdict::Repeats(found.decided_in.unwrap_or_default()))
        }
        Status::History => {
            return Err(format!(
                "{name}@{version} is history already, accepted by {by}"
            ))
        }
        Status::Proposed => {}
    }
    if found.proposer == Actor::Person(accepter.to_owned()) {
        return Err(format!(
            "{name}@{version} waits on another person: no one accepts their own proposal"
        ));
    }
    // A version that comes in as history never stands, so it is accepted
    // whatever stands (design 6).
    if found.proposal.history.is_none() && found.proposal.base != claim.standing {
        let amends = found
            .proposal
            .base
            .as_deref()
            .map_or("no version".to_owned(), |base| format!("{name}@{base}"));
        return Err(match &claim.standing {
            Some(standing) => format!(
                "{name}@{standing} stands, and {name}@{version} amends {amends}: propose its words again amending {standing}"
            ),
            None => format!("{name}@{version} amends {amends}, which does not stand"),
        });
    }
    Ok(Verdict::Counts)
}

fn check_reject(
    state: &State,
    name: &str,
    version: &str,
    rejecter: &str,
) -> Result<Verdict, String> {
    let (_, found) = find(state, name, version)?;
    let by = found.decided_by.as_deref().unwrap_or_default();
    match found.status {
        Status::Proposed => Ok(Verdict::Counts),
        Status::Rejected if by == rejecter => {
            Ok(Verdict::Repeats(found.decided_in.unwrap_or_default()))
        }
        Status::Rejected => Err(format!("{name}@{version} is rejected already, by {by}")),
        status => Err(format!(
            "only a proposed version is rejected: {name}@{version} is {}",
            status.as_str()
        )),
    }
}

/// Applies an entry that counts, numbered `seq`, to the state.
pub fn apply(state: &mut State, seq: u64, entry: &Entry) {
    let actor = actor_name(&entry.actor).map(str::to_owned);
    match &entry.act {
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
            let entry_claim = state
                .claims
                .entry(claim.name.clone())
                .or_insert_with(|| Claim {
                    name: claim.name.clone(),
                    kind: claim.kind,
                    standing: None,
                    versions: Vec::new(),
                });
            entry_claim.versions.push(Version {
                proposal: claim.clone(),
                repositories: repositories(attach.as_ref()).to_vec(),
                proposer: entry.actor.clone(),
                proposed_in: seq,
                status: Status::Proposed,
                decided_by: None,
                decided_in: None,
                reason: None,
                imported: imported.as_deref().cloned(),
            });
        }
        Act::Accepted { name, version } => {
            let claim = state
                .claims
                .get_mut(name)
                .expect("a counted accept's claim");
            let accepted = claim
                .version_mut(version)
                .expect("a counted accept's version");
            if accepted.proposal.history.is_some() {
                // History never stands and replaces no version.
                accepted.status = Status::History;
                accepted.decided_by = actor;
                accepted.decided_in = Some(seq);
                return;
            }
            if let Some(standing) = claim.standing.take() {
                if let Some(replaced) = claim.version_mut(&standing) {
                    replaced.status = Status::Replaced;
                }
            }
            let accepted = claim
                .version_mut(version)
                .expect("a counted accept's version");
            accepted.status = Status::Standing;
            accepted.decided_by = actor;
            accepted.decided_in = Some(seq);
            claim.standing = Some(version.clone());
        }
        Act::Rejected {
            name,
            version,
            reason,
        } => {
            let claim = state
                .claims
                .get_mut(name)
                .expect("a counted reject's claim");
            let rejected = claim
                .version_mut(version)
                .expect("a counted reject's version");
            rejected.status = Status::Rejected;
            rejected.decided_by = actor;
            rejected.decided_in = Some(seq);
            rejected.reason = Some(reason.clone());
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
        apply(state, seq, &entry);
    }
    Judgment::of(verdict)
}

/// Replays a log's entries, `(seq, signed, signature)` in order, through
/// the judge: the state they build, and each entry's judgment.
pub fn replay<'a>(
    entries: impl IntoIterator<Item = (u64, &'a str, Option<&'a str>)>,
) -> (State, Vec<(u64, Judgment)>) {
    let mut state = State::default();
    let judgments = entries
        .into_iter()
        .map(|(seq, signed, signature)| (seq, step(&mut state, seq, signed, signature)))
        .collect();
    (state, judgments)
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
                proposer: Actor::Agent {
                    provenance: "run-1".to_owned(),
                },
                proposed_in: 2,
                status,
                decided_by: (status == Status::Standing).then_some("ada".to_owned()),
                decided_in: (status == Status::Standing).then_some(3),
                reason: None,
                imported: None,
            }
        };
        let mut insert = |kind, name: &str, repositories: &[&str], status| {
            let version = version(kind, name, repositories, status);
            state.claims.insert(
                name.to_owned(),
                Claim {
                    name: name.to_owned(),
                    kind,
                    standing: (status == Status::Standing)
                        .then_some(version.proposal.version.clone()),
                    versions: vec![version],
                },
            );
        };
        for &(kind, name, repositories) in standing {
            insert(kind, name, repositories, Status::Standing);
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
        let standing = state.standing_for(Some("shop"));
        assert_eq!(names(&standing), ["in-shop", "in-the-system", "in-both"]);
        for (claim, version) in &standing {
            assert_eq!(
                claim.standing.as_deref(),
                Some(version.proposal.version.as_str())
            );
            assert_eq!(version.status, Status::Standing);
        }
        assert_eq!(
            names(&state.standing_for(Some("ledger"))),
            ["in-ledger", "in-the-system", "in-both"]
        );
        assert_eq!(names(&state.standing_for(Some("docs"))), ["in-the-system"]);
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
            names(&state.standing_for(None)),
            [
                "delta-holds",
                "gamma-holds",
                "alpha-kept",
                "beta-kept",
                "alpha-assumed"
            ]
        );
    }
}
