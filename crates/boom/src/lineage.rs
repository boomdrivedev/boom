//! The walk of the links: from an entry back to each entry it reaches
//! through its links, and forward to each entry whose links reach it
//! (lineage, "boom lineage walks the links").
//!
//! The walk reads the views `acts` and `links`, kept in step with the
//! log, and each entry it gives from the log itself (core.2).

use std::collections::{BTreeSet, VecDeque};

use rusqlite::{Connection, OptionalExtension};
use serde_json::{json, Value};

use crate::entry::{Act, Actor, Entry};
use crate::{store, Failure};

/// The fewest characters of an act hash that name an entry.
const PREFIX: usize = 12;

/// The format of the walk's answer as JSON.
pub const FORMAT: &str = "boom-lineage/1";

/// The walk from `target`: its entry, then each entry back and forward,
/// each once, the nearest first and then by number. Answers the JSON
/// object and the words.
pub fn walk(conn: &Connection, target: &str) -> Result<(Value, String), Failure> {
    let seq = resolve(conn, target)?;
    let start = shown(conn, seq)?;
    let back = reached(conn, seq, Way::Back)?;
    let forward = reached(conn, seq, Way::Forward)?;
    let mut words = format!("{}\n", start.words);
    for (heading, found) in [("back", &back), ("forward", &forward)] {
        if found.is_empty() {
            words.push_str(&format!("{heading}: none\n"));
            continue;
        }
        words.push_str(&format!("{heading}:\n"));
        for (hops, entry) in found {
            words.push_str(&format!(
                "  {} ({hops} {})\n",
                entry.words.lines().next().unwrap_or_default(),
                if *hops == 1 { "hop" } else { "hops" }
            ));
        }
    }
    let listed = |found: &[(u64, Shown)]| -> Vec<Value> {
        found
            .iter()
            .map(|(hops, entry)| {
                let mut json = entry.json.clone();
                json["hops"] = json!(hops);
                json
            })
            .collect()
    };
    Ok((
        json!({
            "format": FORMAT,
            "target": target,
            "entry": start.json,
            "back": listed(&back),
            "forward": listed(&forward),
        }),
        words,
    ))
}

/// The entry `target` names: an act hash of 12 characters or more that
/// starts one act hash alone, `<repository>:<name>` for the newest
/// version of a change record, or `<repository>:<name>@<version>`
/// (lineage, "The target of a walk").
fn resolve(conn: &Connection, target: &str) -> Result<u64, Failure> {
    if let Some((record, version)) = target.split_once(':') {
        let (name, version) = match version.split_once('@') {
            Some((name, version)) => (name, Some(version)),
            None => (version, None),
        };
        let found = store::change_record(conn, record, name)?.ok_or_else(|| {
            Failure::refused(format!("no change record has the name {record}:{name}"))
        })?;
        let chosen = match version {
            Some(version) => found
                .versions
                .iter()
                .find(|found| found.version == version)
                .ok_or_else(|| {
                    Failure::refused(format!("{record}:{name} has no version {version}"))
                })?,
            None => found.newest().expect("a change record has a version"),
        };
        return Ok(chosen.seq);
    }
    let hexadecimal = target
        .chars()
        .all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c));
    if target.len() < PREFIX || target.len() > 64 || !hexadecimal {
        return Err(Failure::refused(format!(
            "boom lineage takes an act hash of {PREFIX} characters or more, <repository>:<name> or <repository>:<name>@<version>: {target} is none of them"
        )));
    }
    let mut statement = conn.prepare(
        "SELECT act_hash, seq FROM acts WHERE substr(act_hash, 1, ?2) = ?1 ORDER BY counted DESC, seq",
    )?;
    let found = statement
        .query_map(rusqlite::params![target, target.len() as i64], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    let hashes: BTreeSet<&str> = found.iter().map(|(hash, _)| hash.as_str()).collect();
    match (hashes.len(), found.first()) {
        (0, _) | (_, None) => Err(Failure::refused(format!(
            "no entry has an act hash that starts with {target}"
        ))),
        (1, Some((_, seq))) => Ok(u64::try_from(*seq).unwrap_or(0)),
        _ => {
            let mut numbers: Vec<i64> = found.iter().map(|(_, seq)| *seq).collect();
            numbers.sort_unstable();
            Err(Failure::refused(format!(
                "{target} starts the act hashes of several entries, {}: give more of the act hash",
                numbers
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(", ")
            )))
        }
    }
}

/// Which way a walk goes.
#[derive(Clone, Copy)]
enum Way {
    /// To the entries an entry links.
    Back,
    /// To the entries that link an entry.
    Forward,
}

/// The entries one hop from the entry `seq`, each by its number.
fn next(conn: &Connection, seq: u64, way: Way) -> Result<Vec<u64>, Failure> {
    let sql = match way {
        Way::Back => {
            "SELECT acts.seq FROM links JOIN acts ON acts.act_hash = links.target \
             WHERE links.seq = ?1 AND acts.counted = 1 \
             AND acts.seq = (SELECT MIN(first.seq) FROM acts AS first \
                             WHERE first.act_hash = links.target AND first.counted = 1) \
             ORDER BY acts.seq"
        }
        Way::Forward => {
            "SELECT DISTINCT links.seq FROM links JOIN acts ON acts.act_hash = links.target \
             WHERE acts.seq = ?1 ORDER BY links.seq"
        }
    };
    let mut statement = conn.prepare_cached(sql)?;
    let found = statement
        .query_map([i64::try_from(seq).unwrap_or(i64::MAX)], |row| {
            row.get::<_, i64>(0)
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(found
        .into_iter()
        .map(|seq| u64::try_from(seq).unwrap_or(0))
        .collect())
}

/// Each entry the walk from `seq` reaches `way`, each once with its
/// number of hops, the nearest first and then by number.
fn reached(conn: &Connection, seq: u64, way: Way) -> Result<Vec<(u64, Shown)>, Failure> {
    let mut seen = BTreeSet::from([seq]);
    let mut queue = VecDeque::from([(seq, 0_u64)]);
    let mut found: Vec<(u64, u64)> = Vec::new();
    while let Some((at, hops)) = queue.pop_front() {
        for reached in next(conn, at, way)? {
            if seen.insert(reached) {
                found.push((hops + 1, reached));
                queue.push_back((reached, hops + 1));
            }
        }
    }
    found.sort_unstable();
    found
        .into_iter()
        .map(|(hops, seq)| Ok((hops, shown(conn, seq)?)))
        .collect()
}

/// An entry as the walk gives it.
struct Shown {
    json: Value,
    words: String,
}

/// The entry `seq`: its number, act, act hash, actor and links, and for
/// an approval or a version of a change record the claims it covers,
/// each marked where its accepted version moved on.
fn shown(conn: &Connection, seq: u64) -> Result<Shown, Failure> {
    let row = store::row_at(conn, seq)?
        .ok_or_else(|| Failure::not_run(format!("the log holds no entry {seq}")))?;
    let act_hash = crate::entry::act_hash(&row.signed);
    let parsed = Entry::parse(&row.signed)
        .map_err(|why| Failure::not_run(format!("boom cannot read entry {seq}: {why}")))?;
    let (what, covers) = match &parsed.act {
        Act::Approved { plan, covers, .. } => (
            format!(" plan {}", plan.get(..12).unwrap_or(plan)),
            covers.clone(),
        ),
        Act::Recorded { change, .. } => (
            format!(" {}@{}", change.record(), change.version),
            change.covers.clone(),
        ),
        act => {
            let targets: Vec<String> = act
                .targets()
                .iter()
                .map(|(name, version)| format!("{name}@{version}"))
                .collect();
            (
                if targets.is_empty() {
                    String::new()
                } else {
                    format!(" {}", targets.join(", "))
                },
                Vec::new(),
            )
        }
    };
    let actor = match &parsed.actor {
        Actor::Person(name) => name.clone(),
        Actor::Agent { provenance } => format!("an agent ({provenance})"),
    };
    let mut words = format!(
        "{seq} {}{what} by {actor}, act hash {act_hash}\n",
        parsed.act.name()
    );
    let mut covered = Vec::with_capacity(covers.len());
    for claim_version in &covers {
        let (claim, version) = claim_version.split_once('@').unwrap_or((claim_version, ""));
        let accepted = accepted_version(conn, claim)?;
        let out_of_date = accepted
            .as_deref()
            .is_some_and(|accepted| accepted != version);
        if out_of_date {
            words.push_str(&format!(
                "  covers {claim_version}, out of date: {claim}@{} is accepted\n",
                accepted.as_deref().unwrap_or_default()
            ));
        } else {
            words.push_str(&format!("  covers {claim_version}\n"));
        }
        covered.push(json!({
            "claim": claim,
            "version": version,
            "out_of_date": out_of_date,
            "accepted": accepted,
        }));
    }
    Ok(Shown {
        json: json!({
            "number": seq,
            "act": parsed.act.name(),
            "act_hash": act_hash,
            "actor": parsed.actor,
            "links": parsed.act.links(),
            "covers": covered,
        }),
        words,
    })
}

/// The accepted version of the claim `name`, as the views hold it.
fn accepted_version(conn: &Connection, name: &str) -> Result<Option<String>, Failure> {
    Ok(conn
        .query_row(
            "SELECT standing FROM claims WHERE name = ?1 AND standing IS NOT NULL ORDER BY term LIMIT 1",
            [name],
            |row| row.get::<_, String>(0),
        )
        .optional()?)
}
