//! An entry of the log: its signed part and its place in the chain
//! (design 4).
//!
//! The signed part is one JSON object, written once from a struct whose
//! fields are in a fixed order and stored exactly as written, so nothing
//! depends on writing JSON the same way twice. It holds no number, no
//! chain and no signature, so a person's signature holds wherever the
//! entry is ordered (record 0006, point 4).

use serde::{Deserialize, Serialize};

use crate::claim::Proposal;

/// The format every entry names.
pub const FORMAT: &str = "boom-entry/1";

/// The `prev` of a log's first entry: the SHA-256 of no bytes.
pub const GENESIS: &str = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";

/// Who made an entry: a person, whose entry is signed, or an agent,
/// whose entry is not and names where it came from.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase", deny_unknown_fields)]
pub enum Actor {
    /// A person the store registers, by name.
    Person(String),
    /// An agent, by its session or run.
    Agent {
        /// The session or run it names.
        provenance: String,
    },
}

/// The form of the file a record came in from.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Form {
    /// A MADR decision record.
    Madr,
    /// A rules page in the family's form.
    Rules,
}

impl Form {
    /// The form as an entry writes it.
    pub fn as_str(self) -> &'static str {
        match self {
            Form::Madr => "madr",
            Form::Rules => "rules",
        }
    }
}

/// Where an imported version came from: the repository, the file and
/// what the file said of it (record-import; design 5). It is where a
/// version came from, not what it says, and no part of its fingerprint.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Imported {
    /// The repository the import names.
    pub repository: String,
    /// The file's name, `0003-amounts-in-cents.md` or `core.md`.
    pub from: String,
    /// The file's form.
    pub form: Form,
    /// The record's number, `0003`, or the rule's id, `core.1`; none
    /// for an unnumbered record.
    pub legacy: Option<String>,
    /// The file's status as written, or none.
    pub status: Option<String>,
    /// The file's date, or none.
    pub date: Option<String>,
    /// The file's decision-makers.
    pub deciders: Vec<String>,
    /// A rule's `###` heading, or none.
    pub section: Option<String>,
    /// Whether the scenarios were drafted at import.
    pub drafted: bool,
}

impl Imported {
    /// The record's source: its repository, its file and its number or
    /// id (design 7).
    pub fn source(&self) -> (&str, &str, Option<&str>) {
        (&self.repository, &self.from, self.legacy.as_deref())
    }
}

/// What an entry does.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Act {
    /// A person registered, with their key, an owner or not.
    Registered {
        /// Their name.
        person: String,
        /// Their public key, an OpenSSH public key line.
        key: String,
        /// Whether they are an owner.
        owner: bool,
    },
    /// A version of a claim proposed.
    Proposed {
        /// The claim's version.
        claim: Proposal,
        /// Where it came in from, for a version an import proposes.
        imported: Option<Imported>,
    },
    /// A version accepted.
    Accepted {
        /// The claim's name.
        name: String,
        /// The version.
        version: String,
    },
    /// A version rejected, with why.
    Rejected {
        /// The claim's name.
        name: String,
        /// The version.
        version: String,
        /// Why.
        reason: String,
    },
}

impl Act {
    /// The act's name as an entry writes it.
    pub fn name(&self) -> &'static str {
        match self {
            Act::Registered { .. } => "registered",
            Act::Proposed { .. } => "proposed",
            Act::Accepted { .. } => "accepted",
            Act::Rejected { .. } => "rejected",
        }
    }
}

/// An entry: its time, its actor and its act.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    /// When it was made, in Unix seconds, as its maker's clock read.
    pub at: u64,
    /// Who made it.
    pub actor: Actor,
    /// What it does.
    pub act: Act,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RegisteredPart {
    format: String,
    act: String,
    at: u64,
    actor: Actor,
    person: String,
    key: String,
    owner: bool,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProposedPart {
    format: String,
    act: String,
    at: u64,
    actor: Actor,
    claim: Proposal,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    imported: Option<Imported>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct AcceptedPart {
    format: String,
    act: String,
    at: u64,
    actor: Actor,
    name: String,
    version: String,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RejectedPart {
    format: String,
    act: String,
    at: u64,
    actor: Actor,
    name: String,
    version: String,
    reason: String,
}

impl Entry {
    /// The entry's signed part: one JSON object, its fields `format`,
    /// `act`, `at`, `actor` and then the act's own, in that order; a
    /// proposal's `imported` after its `claim`, and only where present.
    pub fn signed_part(&self) -> String {
        let format = FORMAT.to_owned();
        let act = self.act.name().to_owned();
        let at = self.at;
        let actor = self.actor.clone();
        let written = match &self.act {
            Act::Registered { person, key, owner } => serde_json::to_string(&RegisteredPart {
                format,
                act,
                at,
                actor,
                person: person.clone(),
                key: key.clone(),
                owner: *owner,
            }),
            Act::Proposed { claim, imported } => serde_json::to_string(&ProposedPart {
                format,
                act,
                at,
                actor,
                claim: claim.clone(),
                imported: imported.clone(),
            }),
            Act::Accepted { name, version } => serde_json::to_string(&AcceptedPart {
                format,
                act,
                at,
                actor,
                name: name.clone(),
                version: version.clone(),
            }),
            Act::Rejected {
                name,
                version,
                reason,
            } => serde_json::to_string(&RejectedPart {
                format,
                act,
                at,
                actor,
                name: name.clone(),
                version: version.clone(),
                reason: reason.clone(),
            }),
        };
        written.expect("an entry's fields are all JSON")
    }

    /// Reads an entry back from its signed part as stored, answering
    /// what is wrong with its form where it is not one.
    pub fn parse(signed: &str) -> Result<Entry, String> {
        let value: serde_json::Value = serde_json::from_str(signed)
            .map_err(|error| format!("its signed part is not JSON: {error}"))?;
        let field = |name: &str| value.get(name).and_then(serde_json::Value::as_str);
        match field("format") {
            Some(FORMAT) => {}
            Some(other) => return Err(format!("its format is {other}, not {FORMAT}")),
            None => return Err(format!("it names no format, {FORMAT}")),
        }
        let form = |error: serde_json::Error| format!("its form: {error}");
        match field("act") {
            Some("registered") => {
                let part: RegisteredPart = serde_json::from_str(signed).map_err(form)?;
                Ok(Entry {
                    at: part.at,
                    actor: part.actor,
                    act: Act::Registered {
                        person: part.person,
                        key: part.key,
                        owner: part.owner,
                    },
                })
            }
            Some("proposed") => {
                let part: ProposedPart = serde_json::from_str(signed).map_err(form)?;
                Ok(Entry {
                    at: part.at,
                    actor: part.actor,
                    act: Act::Proposed {
                        claim: part.claim,
                        imported: part.imported,
                    },
                })
            }
            Some("accepted") => {
                let part: AcceptedPart = serde_json::from_str(signed).map_err(form)?;
                Ok(Entry {
                    at: part.at,
                    actor: part.actor,
                    act: Act::Accepted {
                        name: part.name,
                        version: part.version,
                    },
                })
            }
            Some("rejected") => {
                let part: RejectedPart = serde_json::from_str(signed).map_err(form)?;
                Ok(Entry {
                    at: part.at,
                    actor: part.actor,
                    act: Act::Rejected {
                        name: part.name,
                        version: part.version,
                        reason: part.reason,
                    },
                })
            }
            Some(other) => Err(format!(
                "its act is {other}, none of registered, proposed, accepted and rejected"
            )),
            None => Err("it names no act".to_owned()),
        }
    }
}

/// An entry's hash: the SHA-256 of its `prev`, a newline, its signed
/// part, a newline and its armored signature or nothing.
pub fn hash(prev: &str, signed: &str, signature: Option<&str>) -> String {
    let mut bytes = Vec::with_capacity(prev.len() + signed.len() + 2);
    bytes.extend_from_slice(prev.as_bytes());
    bytes.push(b'\n');
    bytes.extend_from_slice(signed.as_bytes());
    bytes.push(b'\n');
    bytes.extend_from_slice(signature.unwrap_or("").as_bytes());
    crate::sha256_hex(&bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::claim::Kind;

    fn registration() -> Entry {
        Entry {
            at: 1_763_424_000,
            actor: Actor::Person("ada".to_owned()),
            act: Act::Registered {
                person: "ada".to_owned(),
                key: "sk-ssh-ed25519@openssh.com AAAA".to_owned(),
                owner: true,
            },
        }
    }

    fn proposal() -> Entry {
        Entry {
            at: 1_763_424_100,
            actor: Actor::Agent {
                provenance: "run-1".to_owned(),
            },
            act: Act::Proposed {
                claim: Proposal::new(
                    Kind::Invariant,
                    "refund-within-payment",
                    "A refund never exceeds what the order was paid.",
                    vec!["Given an order paid 40.00, a refund of 45.00 is refused.".to_owned()],
                    None,
                    None,
                    None,
                ),
                imported: None,
            },
        }
    }

    fn imported_proposal() -> Entry {
        let mut entry = proposal();
        if let Act::Proposed { imported, .. } = &mut entry.act {
            *imported = Some(Imported {
                repository: "shop".to_owned(),
                from: "0003-amounts-in-cents.md".to_owned(),
                form: Form::Madr,
                legacy: Some("0003".to_owned()),
                status: Some("accepted".to_owned()),
                date: Some("2026-01-04".to_owned()),
                deciders: vec!["ada".to_owned(), "bo".to_owned()],
                section: None,
                drafted: true,
            });
        }
        entry
    }

    fn keys(signed: &str) -> Vec<String> {
        // The keys of the object in the order the text writes them.
        let mut keys = Vec::new();
        let mut depth = 0;
        let mut chars = signed.char_indices().peekable();
        while let Some((at, c)) = chars.next() {
            match c {
                '{' | '[' => depth += 1,
                '}' | ']' => depth -= 1,
                '"' => {
                    let start = at + 1;
                    let mut end = start;
                    for (i, c) in chars.by_ref() {
                        if c == '"' {
                            end = i;
                            break;
                        }
                    }
                    if depth == 1 && chars.peek().map(|(_, c)| *c) == Some(':') {
                        keys.push(signed[start..end].to_owned());
                    }
                }
                _ => {}
            }
        }
        keys
    }

    #[test]
    fn an_entrys_hash_chains_the_one_before() {
        let first = registration().signed_part();
        let second = proposal().signed_part();
        let first_hash = hash(GENESIS, &first, Some("SIGNATURE"));
        let second_hash = hash(&first_hash, &second, None);
        assert_eq!(
            second_hash,
            crate::sha256_hex(format!("{first_hash}\n{second}\n").as_bytes())
        );
        assert_eq!(
            first_hash,
            crate::sha256_hex(format!("{GENESIS}\n{first}\nSIGNATURE").as_bytes())
        );
        let other_prev = hash(GENESIS, &first, Some("ANOTHER"));
        assert_ne!(hash(&other_prev, &second, None), second_hash);
    }

    #[test]
    fn the_first_entry_chains_the_hash_of_nothing() {
        assert_eq!(GENESIS, crate::sha256_hex(b""));
    }

    #[test]
    fn the_signed_part_holds_no_number_chain_or_signature() {
        let registered = registration().signed_part();
        assert_eq!(
            keys(&registered),
            ["format", "act", "at", "actor", "person", "key", "owner"]
        );
        let proposed = proposal().signed_part();
        assert_eq!(keys(&proposed), ["format", "act", "at", "actor", "claim"]);
        for signed in [&registered, &proposed] {
            for absent in ["seq", "number", "prev", "hash", "signature"] {
                assert!(!keys(signed).iter().any(|key| key == absent), "{signed}");
            }
            assert!(signed.starts_with(r#"{"format":"boom-entry/1","act":"#));
        }
        assert_eq!(Entry::parse(&registered), Ok(registration()));
        assert_eq!(Entry::parse(&proposed), Ok(proposal()));
    }

    #[test]
    fn an_imported_proposal_keeps_where_it_came_from() {
        let entry = imported_proposal();
        let signed = entry.signed_part();
        assert_eq!(
            keys(&signed),
            ["format", "act", "at", "actor", "claim", "imported"]
        );
        assert!(
            signed.contains(r#""imported":{"repository":"shop","from":"0003-amounts-in-cents.md","form":"madr","legacy":"0003","status":"accepted","date":"2026-01-04","deciders":["ada","bo"],"section":null,"drafted":true}"#),
            "{signed}"
        );
        assert_eq!(Entry::parse(&signed), Ok(entry.clone()));
        assert_eq!(Entry::parse(&signed).unwrap().signed_part(), signed);
        let Act::Proposed {
            claim,
            imported: Some(imported),
        } = &entry.act
        else {
            panic!("an imported proposal");
        };
        assert_eq!(
            imported.source(),
            ("shop", "0003-amounts-in-cents.md", Some("0003"))
        );
        let Act::Proposed { claim: plain, .. } = &proposal().act else {
            panic!("a proposal");
        };
        assert_eq!(claim.version, plain.version);
    }

    #[test]
    fn a_proposal_with_nothing_imported_writes_no_imported_field() {
        let signed = proposal().signed_part();
        assert_eq!(keys(&signed), ["format", "act", "at", "actor", "claim"]);
        assert!(!signed.contains("imported"), "{signed}");
        assert_eq!(Entry::parse(&signed), Ok(proposal()));
    }
}
