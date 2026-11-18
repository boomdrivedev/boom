//! The command line: its arguments, its answers as words or as one JSON
//! object, and its exit codes (command-line; design 9).
//!
//! An answer is printed as words on standard output, or, with `--json`,
//! as one JSON object naming its format on standard output with every
//! word on standard error. A command exits 0 for an answer, 2 for a
//! refusal and 3 where it could not run, and on 2 and 3 writes nothing.
//! No command reads its terminal.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};
use serde_json::{json, Value};

use crate::claim::{Kind, Proposal};
use crate::entry::{Act, Actor, Entry};
use crate::judge::{self, Proof, State, Verdict};
use crate::sign::{self, Keys, Signer};
use crate::store::{self, Store, Written};
use crate::Failure;

/// What a command reads from its process: the folders a store's path
/// is resolved under, and the clock an entry's time is read from.
#[derive(Debug, Clone, Default)]
pub struct Env {
    /// `XDG_DATA_HOME`, where it is set.
    pub xdg_data_home: Option<PathBuf>,
    /// `HOME`, where it is set.
    pub home: Option<PathBuf>,
    /// The clock.
    pub clock: Clock,
}

/// The clock an entry's `at` is read from.
#[derive(Debug, Clone, Copy, Default)]
pub enum Clock {
    /// The system's, in Unix seconds.
    #[default]
    System,
    /// A time a test names, in Unix seconds.
    Fixed(u64),
}

impl Clock {
    /// Now, in Unix seconds.
    pub fn now(self) -> u64 {
        match self {
            Clock::System => std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|elapsed| elapsed.as_secs())
                .unwrap_or(0),
            Clock::Fixed(at) => at,
        }
    }
}

impl Env {
    /// The process's own.
    pub fn from_process() -> Self {
        let var = |name: &str| std::env::var_os(name).filter(|value| !value.is_empty());
        Env {
            xdg_data_home: var("XDG_DATA_HOME").map(PathBuf::from),
            home: var("HOME").map(PathBuf::from),
            clock: Clock::System,
        }
    }
}

/// What a command did: its exit code and what it printed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Outcome {
    /// 0 for an answer, 2 for a refusal, 3 for not run.
    pub code: u8,
    /// Standard output.
    pub stdout: String,
    /// Standard error.
    pub stderr: String,
}

#[derive(Parser, Debug)]
#[command(
    name = "boom",
    version,
    about = "boom keeps a store's claims in a log on the machine"
)]
struct Cli {
    /// The store's file: by default `boom/boom.sqlite` under
    /// `$XDG_DATA_HOME`, or under `~/.local/share`.
    #[arg(long, global = true, value_name = "PATH")]
    store: Option<PathBuf>,
    /// Answer as one JSON object on standard output, every word on
    /// standard error.
    #[arg(long, global = true)]
    json: bool,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Make a store, its first entry registering its first owner with
    /// their security key, signed by it.
    Init {
        /// The first owner's name.
        #[arg(long)]
        person: String,
        /// The first owner's security key; its public half is `<key>.pub`.
        #[arg(long)]
        key: PathBuf,
    },
    /// Register a person with their security key, signed by an owner.
    Register {
        /// The person's name.
        person: String,
        /// The person's public key file.
        public_key_file: PathBuf,
        /// Register them as an owner.
        #[arg(long)]
        owner: bool,
        /// The registering owner's security key.
        #[arg(long)]
        key: Option<PathBuf>,
    },
    /// Propose a version of a claim: an agent's, naming its provenance,
    /// or a person's, signed with their key.
    Propose {
        /// The claim's kind.
        kind: Kind,
        /// The claim's name, kebab case.
        name: String,
        /// The claim in one line.
        statement: String,
        /// A scenario; one at least.
        #[arg(long = "scenario", value_name = "TEXT")]
        scenarios: Vec<String>,
        /// The version this one amends.
        #[arg(long, value_name = "VERSION")]
        amends: Option<String>,
        /// A file whose text is the version's body.
        #[arg(long, value_name = "FILE")]
        body: Option<PathBuf>,
        /// The agent's session or run.
        #[arg(long, value_name = "TEXT", conflicts_with = "key")]
        provenance: Option<String>,
        /// A person's security key, to propose as that person.
        #[arg(long)]
        key: Option<PathBuf>,
    },
    /// Accept a version, as a person signing with their security key.
    Accept {
        /// The version, `<name>@<version>`.
        target: String,
        /// The accepting person's security key.
        #[arg(long)]
        key: Option<PathBuf>,
    },
    /// Reject a version, with why, as a person signing with their
    /// security key.
    Reject {
        /// The version, `<name>@<version>`.
        target: String,
        /// Why it is rejected.
        #[arg(long)]
        reason: Option<String>,
        /// The rejecting person's security key.
        #[arg(long)]
        key: Option<PathBuf>,
    },
    /// Show a claim and its versions, or one version, `<name>@<version>`.
    Show {
        /// The claim's name, or `<name>@<version>`.
        target: String,
    },
    /// Show the log's entries in order, each with whether it counts.
    Log,
}

/// An answer: one JSON object naming its format, and its words.
struct Answer {
    json: Value,
    words: String,
}

/// Runs boom with `args`, the arguments after the program's name,
/// signing with `keys` and reading `env`.
pub fn run<I, A>(args: I, keys: &dyn Keys, env: &Env) -> Outcome
where
    I: IntoIterator<Item = A>,
    A: Into<OsString>,
{
    let args: Vec<OsString> = args.into_iter().map(Into::into).collect();
    let json = args.iter().any(|arg| arg == "--json");
    let argv = std::iter::once(OsString::from("boom")).chain(args);
    let cli = match Cli::try_parse_from(argv) {
        Ok(cli) => cli,
        Err(error) => {
            use clap::error::ErrorKind;
            let text = error.render().to_string();
            return match error.kind() {
                ErrorKind::DisplayHelp | ErrorKind::DisplayVersion => Outcome {
                    code: 0,
                    stdout: text,
                    stderr: String::new(),
                },
                _ => failed(json, Failure::Refused(text.trim_end().to_owned())),
            };
        }
    };
    let path = store::resolve(
        cli.store.as_deref(),
        env.xdg_data_home.as_deref(),
        env.home.as_deref(),
    );
    let answer = path.and_then(|path| execute(cli.command, &path, keys, env));
    match answer {
        Ok(answer) if cli.json => Outcome {
            code: 0,
            stdout: format!("{}\n", answer.json),
            stderr: answer.words,
        },
        Ok(answer) => Outcome {
            code: 0,
            stdout: answer.words,
            stderr: String::new(),
        },
        Err(failure) => failed(cli.json, failure),
    }
}

fn failed(json: bool, failure: Failure) -> Outcome {
    let (code, words, object) = match &failure {
        Failure::Refused(why) => (
            2,
            format!("boom: refused: {why}\n"),
            json!({"format": "boom-refused/1", "refused": {"reason": why}}),
        ),
        Failure::NotRun(why) => (
            3,
            format!("boom: not run: {why}\n"),
            json!({"format": "boom-not-run/1", "not_run": {"reason": why}}),
        ),
    };
    Outcome {
        code,
        stdout: if json {
            format!("{object}\n")
        } else {
            String::new()
        },
        stderr: words,
    }
}

fn execute(command: Command, path: &Path, keys: &dyn Keys, env: &Env) -> Result<Answer, Failure> {
    match command {
        Command::Init { person, key } => init(path, &person, &key, keys, env),
        Command::Register {
            person,
            public_key_file,
            owner,
            key,
        } => register(
            path,
            &person,
            &public_key_file,
            owner,
            key.as_deref(),
            keys,
            env,
        ),
        Command::Propose {
            kind,
            name,
            statement,
            scenarios,
            amends,
            body,
            provenance,
            key,
        } => propose(
            path,
            Proposed {
                kind,
                name,
                statement,
                scenarios,
                amends,
                body,
                provenance,
            },
            key.as_deref(),
            keys,
            env,
        ),
        Command::Accept { target, key } => accept(path, &target, key.as_deref(), keys, env),
        Command::Reject {
            target,
            reason,
            key,
        } => reject(path, &target, reason, key.as_deref(), keys, env),
        Command::Show { target } => show(path, &target),
        Command::Log => log(path),
    }
}

/// Judges an entry against the store before anything is signed, signs
/// it where it is a person's, and appends it: a repeat answered with
/// nothing appended.
fn write(
    store: &mut Store,
    signer: Option<&dyn Signer>,
    blob: Option<&[u8]>,
    build: impl FnOnce(&State) -> Result<Entry, Failure>,
) -> Result<(Written, State), Failure> {
    let state = store.state()?;
    let entry = build(&state)?;
    let signed = entry.signed_part();
    let proof = signer.map_or(Proof::Unsigned, |signer| {
        Proof::ToBeSigned(signer.public_key())
    });
    match judge::judge(&state, &entry, &signed, proof) {
        Verdict::Counts => {}
        Verdict::Refused(why) => return Err(Failure::Refused(why)),
        Verdict::Repeats(seq) => return Ok((Written::Repeats(seq), state)),
    }
    let signature = match signer {
        Some(signer) => Some(signer.sign(signed.as_bytes())?),
        None => None,
    };
    store.append(&signed, signature.as_deref(), blob)
}

/// The person the store registers `signer`'s key to.
fn person_of(state: &State, signer: &dyn Signer, key: &Path) -> Result<String, Failure> {
    state
        .person_by_key(signer.public_key())
        .map(|person| person.name.clone())
        .ok_or_else(|| {
            Failure::refused(format!(
                "the key {} is not registered in this store",
                key.display()
            ))
        })
}

fn registration_answer(format: &str, path: &Path, state: &State, person: &str, seq: u64) -> Answer {
    let registered = &state.persons[person];
    let owner = if registered.owner { ", an owner" } else { "" };
    let words = if format == "boom-init/1" {
        format!(
            "{person} is the first owner of the store {}, registered by entry {seq}\n",
            path.display()
        )
    } else {
        format!("{person} is registered{owner}, by entry {seq}\n")
    };
    Answer {
        json: json!({
            "format": format,
            "store": path.display().to_string(),
            "person": person,
            "owner": registered.owner,
            "key": sign::key_line(&registered.key),
            "entry": seq,
        }),
        words,
    }
}

fn init(
    path: &Path,
    person: &str,
    key: &Path,
    keys: &dyn Keys,
    env: &Env,
) -> Result<Answer, Failure> {
    let signer = keys.signer(key)?;
    let public_key = signer.public_key();
    if path.exists() {
        let state = Store::open(path)?.state()?;
        return match state
            .first_owner
            .as_deref()
            .map(|name| &state.persons[name])
        {
            Some(owner)
                if owner.name == person && owner.key.key_data() == public_key.key_data() =>
            {
                Ok(registration_answer(
                    "boom-init/1",
                    path,
                    &state,
                    person,
                    owner.registered_in,
                ))
            }
            Some(owner) => Err(Failure::refused(format!(
                "the store at {} exists; its first owner is {}",
                path.display(),
                owner.name
            ))),
            None => Err(Failure::refused(format!(
                "the store at {} exists, with no first owner",
                path.display()
            ))),
        };
    }
    let entry = Entry {
        at: env.clock.now(),
        actor: Actor::Person(person.to_owned()),
        act: Act::Registered {
            person: person.to_owned(),
            key: sign::key_line(public_key),
            owner: true,
        },
    };
    let signed = entry.signed_part();
    let empty = State::default();
    if let Verdict::Refused(why) =
        judge::judge(&empty, &entry, &signed, Proof::ToBeSigned(public_key))
    {
        return Err(Failure::Refused(why));
    }
    let signature = signer.sign(signed.as_bytes())?;
    if let Verdict::Refused(why) = judge::judge(&empty, &entry, &signed, Proof::Signed(&signature))
    {
        return Err(Failure::Refused(why));
    }
    let mut store = Store::create(path)?;
    let (written, state) = store.append(&signed, Some(&signature), None)?;
    store.finish()?;
    Ok(registration_answer(
        "boom-init/1",
        path,
        &state,
        person,
        written.seq(),
    ))
}

fn register(
    path: &Path,
    person: &str,
    public_key_file: &Path,
    owner: bool,
    key: Option<&Path>,
    keys: &dyn Keys,
    env: &Env,
) -> Result<Answer, Failure> {
    let Some(key) = key else {
        return Err(Failure::refused(
            "only an owner registers a person: boom register takes the owner's --key",
        ));
    };
    let mut store = Store::open(path)?;
    let registered = sign::read_public_key(public_key_file)?;
    let signer = keys.signer(key)?;
    let (written, state) = write(&mut store, Some(signer.as_ref()), None, |state| {
        Ok(Entry {
            at: env.clock.now(),
            actor: Actor::Person(person_of(state, signer.as_ref(), key)?),
            act: Act::Registered {
                person: person.to_owned(),
                key: sign::key_line(&registered),
                owner,
            },
        })
    })?;
    Ok(registration_answer(
        "boom-register/1",
        path,
        &state,
        person,
        written.seq(),
    ))
}

/// A proposal's arguments.
struct Proposed {
    kind: Kind,
    name: String,
    statement: String,
    scenarios: Vec<String>,
    amends: Option<String>,
    body: Option<PathBuf>,
    provenance: Option<String>,
}

fn propose(
    path: &Path,
    proposed: Proposed,
    key: Option<&Path>,
    keys: &dyn Keys,
    env: &Env,
) -> Result<Answer, Failure> {
    if key.is_none()
        && proposed
            .provenance
            .as_deref()
            .is_none_or(|p| p.trim().is_empty())
    {
        return Err(Failure::refused(
            "an agent's proposal names its provenance, with --provenance; a person's is signed, with --key",
        ));
    }
    let mut store = Store::open(path)?;
    let body = match &proposed.body {
        Some(file) => {
            let bytes = std::fs::read(file).map_err(|error| {
                Failure::refused(format!(
                    "the body {} cannot be read: {error}",
                    file.display()
                ))
            })?;
            if std::str::from_utf8(&bytes).is_err() {
                return Err(Failure::refused(format!(
                    "a body is text: {} is not UTF-8",
                    file.display()
                )));
            }
            Some(bytes)
        }
        None => None,
    };
    let proposal = Proposal::new(
        proposed.kind,
        &proposed.name,
        &proposed.statement,
        proposed.scenarios,
        body.as_deref().map(crate::sha256_hex),
        proposed.amends,
        None,
    );
    let signer = key.map(|key| keys.signer(key)).transpose()?;
    let (written, state) = write(&mut store, signer.as_deref(), body.as_deref(), |state| {
        let actor = match (&signer, key) {
            (Some(signer), Some(key)) => Actor::Person(person_of(state, signer.as_ref(), key)?),
            _ => Actor::Agent {
                provenance: proposed.provenance.clone().unwrap_or_default(),
            },
        };
        Ok(Entry {
            at: env.clock.now(),
            actor,
            act: Act::Proposed {
                claim: proposal.clone(),
                imported: None,
            },
        })
    })?;
    let claim = &state.claims[&proposal.name];
    let version = claim
        .version(&proposal.version)
        .expect("a proposal written is in the state");
    let status = version.status.as_str();
    Ok(Answer {
        json: json!({
            "format": "boom-propose/1",
            "claim": proposal.name,
            "kind": claim.kind.as_str(),
            "version": proposal.version,
            "base": proposal.base,
            "status": status,
            "entry": written.seq(),
        }),
        words: format!(
            "{} {}@{} is {status}, by entry {}\n",
            claim.kind.as_str(),
            proposal.name,
            proposal.version,
            written.seq()
        ),
    })
}

/// Splits `<name>@<version>`.
fn version_target<'a>(command: &str, target: &'a str) -> Result<(&'a str, &'a str), Failure> {
    target
        .split_once('@')
        .filter(|(name, version)| !name.is_empty() && !version.is_empty())
        .ok_or_else(|| {
            Failure::refused(format!(
                "boom {command} names a version, <name>@<version>: {target} does not"
            ))
        })
}

fn accept(
    path: &Path,
    target: &str,
    key: Option<&Path>,
    keys: &dyn Keys,
    env: &Env,
) -> Result<Answer, Failure> {
    let (name, version) = version_target("accept", target)?;
    let Some(key) = key else {
        return Err(Failure::refused(
            "only a person accepts: boom accept takes the person's --key",
        ));
    };
    let mut store = Store::open(path)?;
    let signer = keys.signer(key)?;
    let (written, state) = write(&mut store, Some(signer.as_ref()), None, |state| {
        Ok(Entry {
            at: env.clock.now(),
            actor: Actor::Person(person_of(state, signer.as_ref(), key)?),
            act: Act::Accepted {
                name: name.to_owned(),
                version: version.to_owned(),
            },
        })
    })?;
    let accepted = state.claims[name]
        .version(version)
        .expect("an accept written is in the state");
    let by = accepted.decided_by.clone().unwrap_or_default();
    Ok(Answer {
        json: json!({
            "format": "boom-accept/1",
            "claim": name,
            "version": version,
            "status": accepted.status.as_str(),
            "replaced": accepted.proposal.base,
            "by": by,
            "entry": written.seq(),
        }),
        words: format!(
            "{name}@{version} is {}, accepted by {by}, by entry {}\n",
            accepted.status.as_str(),
            written.seq()
        ),
    })
}

fn reject(
    path: &Path,
    target: &str,
    reason: Option<String>,
    key: Option<&Path>,
    keys: &dyn Keys,
    env: &Env,
) -> Result<Answer, Failure> {
    let (name, version) = version_target("reject", target)?;
    let Some(key) = key else {
        return Err(Failure::refused(
            "only a person rejects: boom reject takes the person's --key",
        ));
    };
    let Some(reason) = reason.filter(|reason| !reason.trim().is_empty()) else {
        return Err(Failure::refused("a reject gives its reason, with --reason"));
    };
    let mut store = Store::open(path)?;
    let signer = keys.signer(key)?;
    let (written, state) = write(&mut store, Some(signer.as_ref()), None, |state| {
        Ok(Entry {
            at: env.clock.now(),
            actor: Actor::Person(person_of(state, signer.as_ref(), key)?),
            act: Act::Rejected {
                name: name.to_owned(),
                version: version.to_owned(),
                reason,
            },
        })
    })?;
    let rejected = state.claims[name]
        .version(version)
        .expect("a reject written is in the state");
    let by = rejected.decided_by.clone().unwrap_or_default();
    let why = rejected.reason.clone().unwrap_or_default();
    Ok(Answer {
        json: json!({
            "format": "boom-reject/1",
            "claim": name,
            "version": version,
            "status": rejected.status.as_str(),
            "by": by,
            "reason": why,
            "entry": written.seq(),
        }),
        words: format!(
            "{name}@{version} is {}, rejected by {by}: {why}, by entry {}\n",
            rejected.status.as_str(),
            written.seq()
        ),
    })
}

fn actor_words(actor: &Actor) -> String {
    match actor {
        Actor::Person(name) => name.clone(),
        Actor::Agent { provenance } => format!("an agent ({provenance})"),
    }
}

fn version_json(view: &store::VersionView) -> serde_json::Map<String, Value> {
    let accepted = matches!(view.status.as_str(), "standing" | "replaced");
    let rejected = view.status == "rejected";
    let mut object = serde_json::Map::new();
    object.insert("version".into(), json!(view.version));
    object.insert("status".into(), json!(view.status));
    object.insert("base".into(), json!(view.base));
    object.insert("proposer".into(), json!(view.proposer));
    object.insert("proposed_in".into(), json!(view.proposed_in));
    object.insert(
        "accepted_by".into(),
        json!(view.decided_by.as_ref().filter(|_| accepted)),
    );
    object.insert(
        "rejected_by".into(),
        json!(view.decided_by.as_ref().filter(|_| rejected)),
    );
    object.insert("decided_in".into(), json!(view.decided_in));
    object.insert("reason".into(), json!(view.reason));
    object
}

fn version_words(view: &store::VersionView) -> String {
    let mut words = format!("{} {}", view.version, view.status);
    if let Some(base) = &view.base {
        words.push_str(&format!(", amends {base}"));
    }
    words.push_str(&format!(
        ", proposed by {} in entry {}",
        actor_words(&view.proposer),
        view.proposed_in
    ));
    if let (Some(by), Some(seq)) = (&view.decided_by, view.decided_in) {
        let act = if view.status == "rejected" {
            "rejected"
        } else {
            "accepted"
        };
        words.push_str(&format!("; {act} by {by} in entry {seq}"));
    }
    if let Some(reason) = &view.reason {
        words.push_str(&format!(": {reason}"));
    }
    words
}

fn show(path: &Path, target: &str) -> Result<Answer, Failure> {
    let (name, version) = match target.split_once('@') {
        Some((name, version)) => (name, Some(version)),
        None => (target, None),
    };
    let mut store = Store::open(path)?;
    store.read(|conn| {
        let claim = store::claim_view(conn, name)?
            .ok_or_else(|| Failure::refused(format!("no claim has the name {name}")))?;
        let Some(version) = version else {
            let versions: Vec<Value> = claim
                .versions
                .iter()
                .map(|view| Value::Object(version_json(view)))
                .collect();
            let mut words = format!(
                "{} {}, {}\n",
                claim.kind,
                claim.name,
                claim
                    .standing
                    .as_deref()
                    .map_or("no version stands".to_owned(), |standing| format!(
                        "{standing} stands"
                    ))
            );
            for view in &claim.versions {
                words.push_str(&format!("  {}\n", version_words(view)));
            }
            return Ok(Answer {
                json: json!({
                    "format": "boom-show/1",
                    "claim": claim.name,
                    "kind": claim.kind,
                    "standing": claim.standing,
                    "versions": versions,
                }),
                words,
            });
        };
        let view = claim
            .versions
            .iter()
            .find(|view| view.version == version)
            .ok_or_else(|| {
                Failure::refused(format!("the claim {name} has no version {version}"))
            })?;
        let body = match &view.body {
            Some(fingerprint) => Some(
                String::from_utf8(store::blob(conn, fingerprint)?)
                    .map_err(|_| Failure::not_run(format!("the blob {fingerprint} is not text")))?,
            ),
            None => None,
        };
        let mut object = serde_json::Map::new();
        object.insert("format".into(), json!("boom-show/1"));
        object.insert("claim".into(), json!(claim.name));
        object.insert("kind".into(), json!(claim.kind));
        object.extend(version_json(view));
        object.insert("statement".into(), json!(view.statement));
        object.insert("scenarios".into(), json!(view.scenarios));
        object.insert("body_fingerprint".into(), json!(view.body));
        object.insert("body".into(), json!(body));
        let mut words = format!(
            "{} {}@{}\n{}\n",
            claim.kind,
            claim.name,
            version_words(view),
            view.statement
        );
        for scenario in &view.scenarios {
            words.push_str(&format!("scenario: {scenario}\n"));
        }
        if let Some(body) = &body {
            words.push('\n');
            words.push_str(body);
            if !body.ends_with('\n') {
                words.push('\n');
            }
        }
        Ok(Answer {
            json: Value::Object(object),
            words,
        })
    })
}

fn log(path: &Path) -> Result<Answer, Failure> {
    let mut store = Store::open(path)?;
    store.read(|conn| {
        let mut entries = Vec::new();
        let mut words = String::new();
        for found in store::log_entries(conn)? {
            let row = &found.row;
            let signed: Option<Value> = serde_json::from_str(&row.signed).ok();
            let field = |name: &str| {
                signed
                    .as_ref()
                    .and_then(|signed| signed.get(name))
                    .cloned()
                    .unwrap_or(Value::Null)
            };
            entries.push(json!({
                "number": row.seq,
                "hash": row.hash,
                "prev": row.prev,
                "format": field("format"),
                "act": field("act"),
                "actor": field("actor"),
                "at": field("at"),
                "signed": row.signed,
                "signature": row.signature,
                "counted": found.counted,
                "reason": found.reason,
            }));
            let act = field("act");
            let actor = serde_json::from_value::<Actor>(field("actor"))
                .map_or("no one boom reads".to_owned(), |actor| actor_words(&actor));
            words.push_str(&format!(
                "{} {} by {}{}\n",
                row.seq,
                act.as_str().unwrap_or("(unreadable)"),
                actor,
                found
                    .reason
                    .as_deref()
                    .filter(|_| !found.counted)
                    .map_or(String::new(), |why| format!(", not counted: {why}"))
            ));
        }
        Ok(Answer {
            json: json!({"format": "boom-log/1", "entries": entries}),
            words,
        })
    })
}
