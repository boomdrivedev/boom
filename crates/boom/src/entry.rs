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
    /// Whether an agent drafted its scenarios at import: true only of an
    /// old entry, since a new version has no scenarios.
    pub drafted: bool,
}

impl Imported {
    /// The record's source: its repository, its file and its number or
    /// id (design 7).
    pub fn source(&self) -> (&str, &str, Option<&str>) {
        (&self.repository, &self.from, self.legacy.as_deref())
    }

    /// The record as a refusal names it: an ADR by its file, a rule by
    /// its id and its page, each of its repository.
    pub fn named(&self) -> String {
        match (self.form, &self.legacy) {
            (Form::Rules, Some(id)) => format!("{id} of {} of {}", self.from, self.repository),
            _ => format!("{} of {}", self.from, self.repository),
        }
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
        /// The claim's version, boxed so a proposal with its edges is no
        /// larger than the other acts by much.
        claim: Box<Proposal>,
        /// Where it came in from, for a version an import proposes,
        /// boxed so a proposal is no larger than the other acts by it.
        imported: Option<Box<Imported>>,
        /// What it binds: the repositories and the contexts it names,
        /// or, where it is `None`, the whole system.
        attach: Option<Attach>,
    },
    /// One or more versions accepted, by one act of a person that one
    /// signature covers (record-log, "One accept names each version").
    Accepted {
        /// The versions, in the order the person named them, each claim
        /// once.
        versions: Vec<Named>,
    },
    /// A repository's settings, set by an owner (repository-settings):
    /// its name, its remote in one form, and whether a session gets its
    /// context when it starts.
    Repository {
        /// The repository's name, kebab case.
        name: String,
        /// Its remote, in the one form `repository::remote_form` gives.
        remote: String,
        /// Whether a session in it gets its context at session start.
        session: bool,
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
    /// A person's note on one paragraph of a version of a narrative: an
    /// entry with the person's name and no signature (narrative-notes).
    Noted {
        /// The narrative's name.
        name: String,
        /// The version.
        version: String,
        /// The paragraph's number, from 1.
        paragraph: u64,
        /// The paragraph's fingerprint, 12 hexadecimal characters.
        fingerprint: String,
        /// The note's words.
        words: String,
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
            Act::Repository { .. } => "repository",
            Act::Noted { .. } => "noted",
        }
    }

    /// The versions the act names, each as its claim's name and its
    /// version: a proposal's or a reject's one, an accept's each, and
    /// none for a registration.
    pub fn targets(&self) -> Vec<(&str, &str)> {
        match self {
            Act::Registered { .. } | Act::Repository { .. } => Vec::new(),
            Act::Proposed { claim, .. } => vec![(&claim.name, &claim.version)],
            Act::Accepted { versions } => versions
                .iter()
                .map(|named| (named.name.as_str(), named.version.as_str()))
                .collect(),
            Act::Rejected { name, version, .. } | Act::Noted { name, version, .. } => {
                vec![(name, version)]
            }
        }
    }

    /// An accept of the one version `name@version`.
    pub fn accepted(name: &str, version: &str) -> Act {
        Act::Accepted {
            versions: vec![Named {
                name: name.to_owned(),
                version: version.to_owned(),
            }],
        }
    }
}

/// A version named by its claim's name and its fingerprint.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Named {
    /// The claim's name.
    pub name: String,
    /// The version.
    pub version: String,
}

/// What a proposed version binds, written after its claim and no part
/// of its fingerprint (claim-attachments; design 1): its repositories and
/// its contexts, each field written only where it names one, so an entry
/// written before contexts reads and writes as it did.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Attach {
    /// The repositories, each kebab case, sorted and each once.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub repositories: Vec<String>,
    /// The contexts, each `<name>@<version>` of the version of the
    /// context that the binding meant, sorted by name and each name once
    /// (context-claims).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub contexts: Vec<String>,
}

/// An entry: its time, the rule set it was written under, its actor and
/// its act.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    /// When it was made, in Unix seconds, as its maker's clock read.
    pub at: u64,
    /// The rule set it was written under, [`crate::judge::RULES`] for an
    /// entry boom writes now, and 0 for one written before entries named
    /// their rule set, whose signed part holds no `rules` (record-log,
    /// "An entry is judged by the rules it was written under").
    pub rules: u32,
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
    #[serde(default, skip_serializing_if = "is_zero")]
    rules: u32,
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
    #[serde(default, skip_serializing_if = "is_zero")]
    rules: u32,
    actor: Actor,
    claim: Box<Proposal>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    imported: Option<Box<Imported>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    attach: Option<Attach>,
}

/// An accept of one version keeps the form it always had, `name` and
/// `version`; an accept of several names them in `versions`, so an entry
/// written before several could be accepted at once reads as it did.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct AcceptedPart {
    format: String,
    act: String,
    at: u64,
    #[serde(default, skip_serializing_if = "is_zero")]
    rules: u32,
    actor: Actor,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    versions: Option<Vec<Named>>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RepositoryPart {
    format: String,
    act: String,
    at: u64,
    #[serde(default, skip_serializing_if = "is_zero")]
    rules: u32,
    actor: Actor,
    name: String,
    remote: String,
    session: bool,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct NotedPart {
    format: String,
    act: String,
    at: u64,
    #[serde(default, skip_serializing_if = "is_zero")]
    rules: u32,
    actor: Actor,
    name: String,
    version: String,
    paragraph: u64,
    fingerprint: String,
    words: String,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RejectedPart {
    format: String,
    act: String,
    at: u64,
    #[serde(default, skip_serializing_if = "is_zero")]
    rules: u32,
    actor: Actor,
    name: String,
    version: String,
    reason: String,
}

impl Entry {
    /// The entry's signed part: one JSON object, its fields `format`,
    /// `act`, `at`, `rules` where it is not 0, `actor` and then the act's
    /// own, in that order, so a signature covers the rule set; a
    /// proposal's `imported` after its `claim` and its `attach` after
    /// that, each only where present (design 7).
    pub fn signed_part(&self) -> String {
        let format = FORMAT.to_owned();
        let act = self.act.name().to_owned();
        let at = self.at;
        let rules = self.rules;
        let actor = self.actor.clone();
        let written = match &self.act {
            Act::Registered { person, key, owner } => serde_json::to_string(&RegisteredPart {
                format,
                act,
                at,
                rules,
                actor,
                person: person.clone(),
                key: key.clone(),
                owner: *owner,
            }),
            Act::Proposed {
                claim,
                imported,
                attach,
            } => serde_json::to_string(&ProposedPart {
                format,
                act,
                at,
                rules,
                actor,
                claim: claim.clone(),
                imported: imported.clone(),
                attach: attach.clone(),
            }),
            Act::Accepted { versions } => serde_json::to_string(&match versions.as_slice() {
                [one] => AcceptedPart {
                    format,
                    act,
                    at,
                    rules,
                    actor,
                    name: Some(one.name.clone()),
                    version: Some(one.version.clone()),
                    versions: None,
                },
                several => AcceptedPart {
                    format,
                    act,
                    at,
                    rules,
                    actor,
                    name: None,
                    version: None,
                    versions: Some(several.to_vec()),
                },
            }),
            Act::Repository {
                name,
                remote,
                session,
            } => serde_json::to_string(&RepositoryPart {
                format,
                act,
                at,
                rules,
                actor,
                name: name.clone(),
                remote: remote.clone(),
                session: *session,
            }),
            Act::Rejected {
                name,
                version,
                reason,
            } => serde_json::to_string(&RejectedPart {
                format,
                act,
                at,
                rules,
                actor,
                name: name.clone(),
                version: version.clone(),
                reason: reason.clone(),
            }),
            Act::Noted {
                name,
                version,
                paragraph,
                fingerprint,
                words,
            } => serde_json::to_string(&NotedPart {
                format,
                act,
                at,
                rules,
                actor,
                name: name.clone(),
                version: version.clone(),
                paragraph: *paragraph,
                fingerprint: fingerprint.clone(),
                words: words.clone(),
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
                    rules: part.rules,
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
                    rules: part.rules,
                    actor: part.actor,
                    act: Act::Proposed {
                        claim: part.claim,
                        imported: part.imported,
                        attach: part.attach,
                    },
                })
            }
            Some("accepted") => {
                let part: AcceptedPart = serde_json::from_str(signed).map_err(form)?;
                let versions =
                    match (part.name, part.version, part.versions) {
                        (Some(name), Some(version), None) => vec![Named { name, version }],
                        (None, None, Some(several)) if several.len() > 1 => several,
                        (None, None, Some(_)) => return Err(
                            "its form: an accept of one version names it by `name` and `version`"
                                .to_owned(),
                        ),
                        _ => {
                            return Err(
                                "its form: an accept names `name` and `version`, or `versions`"
                                    .to_owned(),
                            )
                        }
                    };
                Ok(Entry {
                    at: part.at,
                    rules: part.rules,
                    actor: part.actor,
                    act: Act::Accepted { versions },
                })
            }
            Some("repository") => {
                let part: RepositoryPart = serde_json::from_str(signed).map_err(form)?;
                Ok(Entry {
                    at: part.at,
                    rules: part.rules,
                    actor: part.actor,
                    act: Act::Repository {
                        name: part.name,
                        remote: part.remote,
                        session: part.session,
                    },
                })
            }
            Some("rejected") => {
                let part: RejectedPart = serde_json::from_str(signed).map_err(form)?;
                Ok(Entry {
                    at: part.at,
                    rules: part.rules,
                    actor: part.actor,
                    act: Act::Rejected {
                        name: part.name,
                        version: part.version,
                        reason: part.reason,
                    },
                })
            }
            Some("noted") => {
                let part: NotedPart = serde_json::from_str(signed).map_err(form)?;
                Ok(Entry {
                    at: part.at,
                    rules: part.rules,
                    actor: part.actor,
                    act: Act::Noted {
                        name: part.name,
                        version: part.version,
                        paragraph: part.paragraph,
                        fingerprint: part.fingerprint,
                        words: part.words,
                    },
                })
            }
            Some(other) => Err(format!(
                "its act is {other}, none of registered, proposed, accepted, rejected, repository and noted"
            )),
            None => Err("it names no act".to_owned()),
        }
    }
}

/// Whether a rule set is 0, which a signed part leaves out.
#[allow(clippy::trivially_copy_pass_by_ref)]
fn is_zero(rules: &u32) -> bool {
    *rules == 0
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
            rules: crate::judge::RULES,
            actor: Actor::Person("ada".to_owned()),
            act: Act::Registered {
                person: "ada".to_owned(),
                key: "sk-ssh-ed25519@openssh.com AAAA".to_owned(),
                owner: true,
            },
        }
    }

    fn proposal() -> Entry {
        attached(None)
    }

    fn attached(attach: Option<Attach>) -> Entry {
        Entry {
            at: 1_763_424_100,
            rules: crate::judge::RULES,
            actor: Actor::Agent {
                provenance: "run-1".to_owned(),
            },
            act: Act::Proposed {
                claim: Box::new(Proposal::new(
                    Kind::Invariant,
                    "refund-within-payment",
                    "A refund never exceeds what the order was paid.",
                    vec!["Given an order paid 40.00, a refund of 45.00 is refused.".to_owned()],
                    None,
                    None,
                    None,
                )),
                imported: None,
                attach,
            },
        }
    }

    fn imported_proposal() -> Entry {
        let mut entry = proposal();
        if let Act::Proposed { imported, .. } = &mut entry.act {
            *imported = Some(Box::new(Imported {
                repository: "shop".to_owned(),
                from: "0003-amounts-in-cents.md".to_owned(),
                form: Form::Madr,
                legacy: Some("0003".to_owned()),
                status: Some("accepted".to_owned()),
                date: Some("2026-01-04".to_owned()),
                deciders: vec!["ada".to_owned(), "bo".to_owned()],
                section: None,
                drafted: true,
            }));
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
            ["format", "act", "at", "rules", "actor", "person", "key", "owner"]
        );
        let proposed = proposal().signed_part();
        assert_eq!(
            keys(&proposed),
            ["format", "act", "at", "rules", "actor", "claim"]
        );
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
    fn an_entry_of_rule_set_0_names_no_rule_set() {
        let mut old = proposal();
        old.rules = 0;
        let signed = old.signed_part();
        assert_eq!(keys(&signed), ["format", "act", "at", "actor", "claim"]);
        assert_eq!(Entry::parse(&signed), Ok(old.clone()));
        assert_eq!(Entry::parse(&signed).unwrap().signed_part(), signed);
        let now = proposal().signed_part();
        assert!(
            now.contains(&format!(r#""rules":{},"actor""#, crate::judge::RULES)),
            "{now}"
        );
        assert_eq!(Entry::parse(&now).unwrap().rules, crate::judge::RULES);
    }

    #[test]
    fn an_attachment_follows_the_claim_in_the_signed_part() {
        let attach = Attach {
            repositories: vec!["ledger".to_owned(), "shop".to_owned()],
            contexts: Vec::new(),
        };
        let entry = attached(Some(attach));
        let signed = entry.signed_part();
        assert_eq!(
            keys(&signed),
            ["format", "act", "at", "rules", "actor", "claim", "attach"]
        );
        assert!(
            signed.ends_with(r#","attach":{"repositories":["ledger","shop"]}}"#),
            "{signed}"
        );
        assert_eq!(Entry::parse(&signed), Ok(entry));
        let unknown = signed.replace(r#"{"repositories":"#, r#"{"paths":[],"repositories":"#);
        let refused = Entry::parse(&unknown).unwrap_err();
        assert!(refused.contains("unknown field `paths`"), "{refused}");
    }

    #[test]
    fn a_binding_to_a_context_follows_the_repositories() {
        let both = attached(Some(Attach {
            repositories: vec!["flume".to_owned()],
            contexts: vec!["sluice-overview@aaaaaaaaaaaa".to_owned()],
        }));
        let signed = both.signed_part();
        assert!(
            signed.ends_with(
                r#","attach":{"repositories":["flume"],"contexts":["sluice-overview@aaaaaaaaaaaa"]}}"#
            ),
            "{signed}"
        );
        assert_eq!(Entry::parse(&signed), Ok(both));
        let only = attached(Some(Attach {
            repositories: Vec::new(),
            contexts: vec!["sluice-overview@aaaaaaaaaaaa".to_owned()],
        }));
        let signed = only.signed_part();
        assert!(
            signed.ends_with(r#","attach":{"contexts":["sluice-overview@aaaaaaaaaaaa"]}}"#),
            "{signed}"
        );
        assert_eq!(Entry::parse(&signed), Ok(only));
    }

    #[test]
    fn a_proposal_with_no_attachment_is_written_as_before() {
        // As before rule sets: an entry of rule set 0.
        let mut entry = proposal();
        entry.rules = 0;
        let Act::Proposed { claim, .. } = &entry.act else {
            unreachable!("a proposal");
        };
        let before = format!(
            r#"{{"format":"boom-entry/1","act":"proposed","at":1763424100,"actor":{{"agent":{{"provenance":"run-1"}}}},"claim":{}}}"#,
            serde_json::to_string(claim).unwrap()
        );
        assert_eq!(entry.signed_part().as_bytes(), before.as_bytes());
        assert!(!entry.signed_part().contains("attach"));
        assert_eq!(Entry::parse(&before), Ok(entry));
    }

    #[test]
    fn an_imported_proposal_keeps_its_origin() {
        let entry = imported_proposal();
        let signed = entry.signed_part();
        assert_eq!(
            keys(&signed),
            ["format", "act", "at", "rules", "actor", "claim", "imported"]
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
            ..
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
    fn an_imported_proposal_is_attached_after_imported() {
        let mut entry = imported_proposal();
        if let Act::Proposed { attach, .. } = &mut entry.act {
            *attach = Some(Attach {
                repositories: vec!["shop".to_owned()],
                contexts: Vec::new(),
            });
        }
        let signed = entry.signed_part();
        assert_eq!(
            keys(&signed),
            ["format", "act", "at", "rules", "actor", "claim", "imported", "attach"]
        );
        assert!(
            signed.ends_with(r#","attach":{"repositories":["shop"]}}"#),
            "{signed}"
        );
        assert_eq!(Entry::parse(&signed), Ok(entry.clone()));
        assert_eq!(Entry::parse(&signed).unwrap().signed_part(), signed);
    }

    #[test]
    fn a_proposal_with_nothing_imported_writes_no_imported_field() {
        let signed = proposal().signed_part();
        assert_eq!(
            keys(&signed),
            ["format", "act", "at", "rules", "actor", "claim"]
        );
        assert!(!signed.contains("imported"), "{signed}");
        assert_eq!(Entry::parse(&signed), Ok(proposal()));
    }
}
