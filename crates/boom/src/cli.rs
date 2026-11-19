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

use crate::claim::{History, Kind, Proposal};
use crate::entry::{Act, Actor, Attach, Entry, Form, Imported, GENESIS};
use crate::import::{self, Read, Record};
use crate::judge::{self, Proof, State, Verdict};
use crate::sign::{self, Keys, Signer};
use crate::store::{self, Append, Store, Written};
use crate::{repository, Failure};

/// What a command reads from its process: the folders a store's path
/// is resolved under, the working folder a repository is declared from,
/// and the clock an entry's time is read from.
#[derive(Debug, Clone, Default)]
pub struct Env {
    /// `XDG_DATA_HOME`, where it is set.
    pub xdg_data_home: Option<PathBuf>,
    /// `HOME`, where it is set.
    pub home: Option<PathBuf>,
    /// The working folder, where it can be read.
    pub cwd: Option<PathBuf>,
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
            cwd: std::env::current_dir().ok(),
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
        /// A repository the version binds; any number of times. Naming
        /// none, an amendment binds what its base binds, and a first
        /// version the repository the working folder declares, else the
        /// whole system.
        #[arg(long = "repository", value_name = "NAME", conflicts_with = "system")]
        repositories: Vec<String>,
        /// Bind the version to the whole system, every repository.
        #[arg(long)]
        system: bool,
    },
    /// Accept one or more versions, as a person signing each with their
    /// security key, all appended together or none.
    Accept {
        /// The versions, each `<name>@<version>`.
        #[arg(required = true, value_name = "TARGET")]
        targets: Vec<String>,
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
    /// Give every claim that stands for a repository, as the person
    /// accepted it, replayed from the log: words an agent's harness can
    /// put in its context as they are.
    Query {
        /// The repository; by default the one the working folder's
        /// `.config/boom.toml` declares.
        #[arg(long, value_name = "NAME")]
        repository: Option<String>,
        /// Answer nothing, and exit 0, where the working folder declares
        /// no repository, as a harness's hook in every session asks.
        #[arg(long, conflicts_with = "repository")]
        if_declared: bool,
    },
    /// Find the claims that stand whose words hold every word searched,
    /// in a repository or every repository, the best matches first,
    /// replayed from the log with no index kept.
    Search {
        /// The words, each searched as written; `--` before a word that
        /// begins with `-`.
        #[arg(required = true, value_name = "WORD")]
        words: Vec<String>,
        /// The repository; by default the one the working folder's
        /// `.config/boom.toml` declares.
        #[arg(long, value_name = "NAME", conflicts_with = "all")]
        repository: Option<String>,
        /// Search every repository's claims.
        #[arg(long)]
        all: bool,
        /// How many of the best matches to give.
        #[arg(
            long,
            value_name = "N",
            default_value_t = 10,
            value_parser = clap::value_parser!(u64).range(1..)
        )]
        limit: u64,
    },
    /// Import a repository's decision records or rules once, proposing
    /// each in its file's own words with what its agent drafts.
    Import {
        #[command(subcommand)]
        what: ImportCommand,
    },
}

#[derive(Subcommand, Debug)]
enum ImportCommand {
    /// Import a folder of MADR decision records.
    Adr {
        /// The folder.
        dir: PathBuf,
        #[command(flatten)]
        options: ImportOptions,
    },
    /// Import a rules page, each rule a bullet `**[<id>]**`.
    Rules {
        /// The page.
        page: PathBuf,
        #[command(flatten)]
        options: ImportOptions,
    },
}

#[derive(clap::Args, Debug)]
struct ImportOptions {
    /// The repository the records are of, kebab case.
    #[arg(long, value_name = "NAME")]
    repository: Option<String>,
    /// The drafts file, `boom-import-drafts/1`: each record's scenarios
    /// and each rule's name, by its source.
    #[arg(long, value_name = "FILE")]
    drafts: Option<PathBuf>,
    /// The agent's session or run.
    #[arg(long, value_name = "TEXT")]
    provenance: Option<String>,
    /// Read and answer, writing nothing.
    #[arg(long)]
    dry_run: bool,
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
    let answer = execute(cli.command, path, keys, env);
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

/// Runs `command` on the store at `path`, which a command resolves only
/// where it opens the store.
fn execute(
    command: Command,
    path: Result<PathBuf, Failure>,
    keys: &dyn Keys,
    env: &Env,
) -> Result<Answer, Failure> {
    match command {
        Command::Init { person, key } => init(&path?, &person, &key, keys, env),
        Command::Register {
            person,
            public_key_file,
            owner,
            key,
        } => register(
            &path?,
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
            repositories,
            system,
        } => propose(
            &path?,
            Proposed {
                kind,
                name,
                statement,
                scenarios,
                amends,
                body,
                provenance,
                repositories,
                system,
            },
            key.as_deref(),
            keys,
            env,
        ),
        Command::Accept { targets, key } => accept(&path?, &targets, key.as_deref(), keys, env),
        Command::Reject {
            target,
            reason,
            key,
        } => reject(&path?, &target, reason, key.as_deref(), keys, env),
        Command::Show { target } => show(&path?, &target),
        Command::Log => log(&path?),
        Command::Query {
            repository,
            if_declared,
        } => query(path, repository.as_deref(), if_declared, env),
        Command::Search {
            words,
            repository,
            all,
            limit,
        } => search(&path?, &words, repository.as_deref(), all, limit, env),
        Command::Import { what } => match what {
            ImportCommand::Adr { dir, options } => import(&path?, Form::Madr, &dir, options, env),
            ImportCommand::Rules { page, options } => {
                import(&path?, Form::Rules, &page, options, env)
            }
        },
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
    repositories: Vec<String>,
    system: bool,
}

/// What a proposal binds, in the order record 0008 (point 4) gives:
/// the repositories it names, sorted and each once; else, with
/// `--system`, the whole system; else, for an amendment, what its base
/// binds in `state`; else the repository `declared`; else the whole
/// system.
fn attachment(
    proposed: &Proposed,
    proposal: &Proposal,
    declared: Option<&str>,
    state: &State,
) -> Option<Attach> {
    let named = |repositories: Vec<String>| {
        Some(repositories)
            .filter(|repositories| !repositories.is_empty())
            .map(|repositories| Attach { repositories })
    };
    if !proposed.repositories.is_empty() {
        let mut repositories = proposed.repositories.clone();
        repositories.sort();
        repositories.dedup();
        return named(repositories);
    }
    if proposed.system {
        return None;
    }
    if let Some(base) = &proposal.base {
        let base = state
            .claims
            .get(&proposal.name)
            .and_then(|claim| claim.version(base));
        return base.and_then(|base| named(base.repositories.clone()));
    }
    declared.and_then(|name| named(vec![name.to_owned()]))
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
    let declared =
        if proposed.repositories.is_empty() && !proposed.system && proposed.amends.is_none() {
            repository::find(None, env.cwd.as_deref())?
        } else {
            None
        };
    let proposal = Proposal::new(
        proposed.kind,
        &proposed.name,
        &proposed.statement,
        proposed.scenarios.clone(),
        body.as_deref().map(crate::sha256_hex),
        proposed.amends.clone(),
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
                attach: attachment(&proposed, &proposal, declared.as_deref(), state),
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
            "repositories": version.repositories,
        }),
        words: format!(
            "{} {}@{} is {status}, by entry {}\nit binds {}\n",
            claim.kind.as_str(),
            proposal.name,
            proposal.version,
            written.seq(),
            repository::words(&version.repositories)
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
    targets: &[String],
    key: Option<&Path>,
    keys: &dyn Keys,
    env: &Env,
) -> Result<Answer, Failure> {
    let versions = targets
        .iter()
        .map(|target| version_target("accept", target))
        .collect::<Result<Vec<_>, _>>()?;
    let Some(key) = key else {
        return Err(Failure::refused(
            "only a person accepts: boom accept takes the person's --key",
        ));
    };
    let mut store = Store::open(path)?;
    let signer = keys.signer(key)?;
    let state = store.state()?;
    let person = person_of(&state, signer.as_ref(), key)?;
    // Every target is judged, each against the state the ones before it
    // build, before the key signs any (design 9).
    let mut judged = state;
    let mut to_sign = Vec::new();
    for &(name, version) in &versions {
        let entry = Entry {
            at: env.clock.now(),
            actor: Actor::Person(person.clone()),
            act: Act::Accepted {
                name: name.to_owned(),
                version: version.to_owned(),
            },
        };
        let signed = entry.signed_part();
        match judge::judge(
            &judged,
            &entry,
            &signed,
            Proof::ToBeSigned(signer.public_key()),
        ) {
            Verdict::Counts => {
                judge::apply(&mut judged, 0, &entry);
                to_sign.push(signed);
            }
            Verdict::Refused(why) => return Err(Failure::Refused(naming(name, version, why))),
            Verdict::Repeats(_) => {}
        }
    }
    let mut signatures = Vec::with_capacity(to_sign.len());
    for signed in &to_sign {
        signatures.push(signer.sign(signed.as_bytes())?);
    }
    let appends: Vec<Append<'_>> = to_sign
        .iter()
        .zip(&signatures)
        .map(|(signed, signature)| Append {
            signed,
            signature: Some(signature),
            blob: None,
        })
        .collect();
    let state = if appends.is_empty() {
        store.state()?
    } else {
        store.append_all(&appends)?.1
    };
    let mut accepted = Vec::with_capacity(versions.len());
    let mut words = String::new();
    for &(name, version) in &versions {
        let found = state.claims[name]
            .version(version)
            .expect("an accept written is in the state");
        let by = found.decided_by.clone().unwrap_or_default();
        let seq = found.decided_in.unwrap_or_default();
        accepted.push(json!({
            "claim": name,
            "version": version,
            "status": found.status.as_str(),
            "replaced": found.proposal.base,
            "by": by,
            "entry": seq,
        }));
        words.push_str(&format!(
            "{name}@{version} is {}, accepted by {by}, by entry {seq}\n",
            found.status.as_str(),
        ));
    }
    let mut object = serde_json::Map::new();
    object.insert("format".into(), json!("boom-accept/1"));
    if let [one] = accepted.as_slice() {
        if let Some(fields) = one.as_object() {
            object.extend(fields.clone());
        }
    }
    object.insert("accepted".into(), Value::Array(accepted));
    Ok(Answer {
        json: Value::Object(object),
        words,
    })
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
    let accepted = matches!(view.status.as_str(), "standing" | "replaced" | "history");
    let rejected = view.status == "rejected";
    let mut object = serde_json::Map::new();
    object.insert("version".into(), json!(view.version));
    object.insert("status".into(), json!(view.status));
    object.insert("base".into(), json!(view.base));
    object.insert("repositories".into(), json!(view.repositories));
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
                words.push_str(&format!(
                    "  {}\n    binds: {}\n",
                    version_words(view),
                    repository::words(&view.repositories)
                ));
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
        let (imported, history) = proposed_with(conn, view.proposed_in)?;
        let mut object = serde_json::Map::new();
        object.insert("format".into(), json!("boom-show/1"));
        object.insert("claim".into(), json!(claim.name));
        object.insert("kind".into(), json!(claim.kind));
        object.extend(version_json(view));
        object.insert("statement".into(), json!(view.statement));
        object.insert("scenarios".into(), json!(view.scenarios));
        object.insert("body_fingerprint".into(), json!(view.body));
        object.insert("body".into(), json!(body));
        object.insert("imported".into(), json!(imported));
        object.insert("history".into(), json!(history));
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
        words.push_str(&format!(
            "binds: {}\n",
            repository::words(&view.repositories)
        ));
        if let Some(history) = &history {
            words.push_str(&format!("history: {}", history.status.as_str()));
            if let Some(by) = &history.by {
                words.push_str(&format!(" by {by}"));
            }
            words.push('\n');
        }
        if let Some(imported) = &imported {
            words.push_str(&format!(
                "imported from {} of {}",
                imported.from, imported.repository
            ));
            if let Some(legacy) = &imported.legacy {
                words.push_str(&format!(", {legacy}"));
            }
            if imported.drafted {
                words.push_str(", its scenarios drafted");
            }
            words.push('\n');
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

/// What the entry that proposed a version says of where it came from
/// and its history (design 5).
fn proposed_with(
    conn: &rusqlite::Connection,
    seq: u64,
) -> Result<(Option<Imported>, Option<History>), Failure> {
    let signed = store::signed_at(conn, seq)?
        .ok_or_else(|| Failure::not_run(format!("the log holds no entry {seq}")))?;
    match Entry::parse(&signed) {
        Ok(Entry {
            act: Act::Proposed {
                claim, imported, ..
            },
            ..
        }) => Ok((imported.map(|imported| *imported), claim.history)),
        _ => Err(Failure::not_run(format!(
            "entry {seq} is not the proposal its version names"
        ))),
    }
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

/// A standing claim's fields as `boom query` gives them, in its order;
/// `boom search` gives the same and more.
fn standing_json(
    claim: &judge::Claim,
    standing: &judge::Version,
) -> serde_json::Map<String, Value> {
    let proposal = &standing.proposal;
    let mut object = serde_json::Map::new();
    object.insert("name".into(), json!(claim.name));
    object.insert("kind".into(), json!(claim.kind.as_str()));
    object.insert("version".into(), json!(proposal.version));
    object.insert("statement".into(), json!(proposal.statement));
    object.insert("scenarios".into(), json!(proposal.scenarios));
    object.insert("body".into(), json!(proposal.body));
    object.insert("repositories".into(), json!(standing.repositories));
    object.insert(
        "accepted".into(),
        json!({"by": standing.decided_by, "entry": standing.decided_in}),
    );
    object
}

/// The claims that stand for the repository `named`, or else the one the
/// working folder declares (repository-rules; design 5): taken from the
/// judge's replay of the whole log for this answer, never from the views
/// (record 0008, point 2), and appending nothing. With `if_declared`,
/// where none is named or declared, nothing: no store opened, no words,
/// and `boom-query/1` with no repository (design 10).
fn query(
    path: Result<PathBuf, Failure>,
    named: Option<&str>,
    if_declared: bool,
    env: &Env,
) -> Result<Answer, Failure> {
    let Some(repository) = repository::find(named, env.cwd.as_deref())? else {
        if if_declared {
            return Ok(Answer {
                json: json!({
                    "format": "boom-query/1",
                    "repository": null,
                    "head": null,
                    "claims": [],
                }),
                words: String::new(),
            });
        }
        return Err(Failure::refused(format!(
            "boom query answers for a repository: name it with --repository, or in {}",
            repository::DECLARATION
        )));
    };
    let mut store = Store::open(&path?)?;
    store.read(|conn| {
        let rows = store::log_rows(conn)?;
        let (state, _) = judge::replay(
            rows.iter()
                .map(|row| (row.seq, row.signed.as_str(), row.signature.as_deref())),
        );
        let (number, hash) = rows
            .last()
            .map_or((0, GENESIS), |row| (row.seq, row.hash.as_str()));
        let mut claims = Vec::new();
        let mut lines = String::new();
        for (claim, standing) in state.standing_for(Some(&repository)) {
            let proposal = &standing.proposal;
            claims.push(Value::Object(standing_json(claim, standing)));
            lines.push_str(&format!(
                "{} {}@{}: {}\n",
                claim.kind.as_str(),
                claim.name,
                proposal.version,
                proposal.statement
            ));
            for scenario in &proposal.scenarios {
                lines.push_str(&format!("  - {scenario}\n"));
            }
            if proposal.body.is_some() {
                lines.push_str(&format!(
                    "  body: boom show {}@{}\n",
                    claim.name, proposal.version
                ));
            }
        }
        let stand = match claims.len() {
            0 => "No claim stands".to_owned(),
            1 => "1 claim stands".to_owned(),
            n => format!("{n} claims stand"),
        };
        Ok(Answer {
            json: json!({
                "format": "boom-query/1",
                "repository": repository,
                "head": {"number": number, "hash": hash},
                "claims": claims,
            }),
            words: format!(
                "{stand} for {repository}, at entry {number} of the store's log.\n{lines}"
            ),
        })
    })
}

/// The claims that stand whose words hold every word of `given`, for
/// the repository `named`, or else the one the working folder declares,
/// or with `all` every claim (claim-search; designs 1 to 6): replayed
/// from the whole log for this answer, never from the views, searched
/// in memory, keeping nothing and appending nothing.
fn search(
    path: &Path,
    given: &[String],
    named: Option<&str>,
    all: bool,
    limit: u64,
    env: &Env,
) -> Result<Answer, Failure> {
    let words = crate::search::words(given);
    if words.is_empty() {
        return Err(Failure::refused(
            "boom search needs a word holding a letter or a digit",
        ));
    }
    let repository = if all {
        None
    } else {
        Some(repository::find(named, env.cwd.as_deref())?.ok_or_else(|| {
            Failure::refused(format!(
                "boom search searches a repository: name it with --repository, or in {}, or search every repository with --all",
                repository::DECLARATION
            ))
        })?)
    };
    let mut store = Store::open(path)?;
    store.read(|conn| {
        let rows = store::log_rows(conn)?;
        let (state, _) = judge::replay(
            rows.iter()
                .map(|row| (row.seq, row.signed.as_str(), row.signature.as_deref())),
        );
        let (number, hash) = rows
            .last()
            .map_or((0, GENESIS), |row| (row.seq, row.hash.as_str()));
        let standing = state.standing_for(repository.as_deref());
        // A body that is not text is searched as holding no word; one
        // whose bytes no longer match its fingerprint stops the search.
        let bodies = standing
            .iter()
            .map(|(_, version)| match &version.proposal.body {
                Some(fingerprint) => Ok(String::from_utf8(store::blob(conn, fingerprint)?).ok()),
                None => Ok(None),
            })
            .collect::<Result<Vec<_>, Failure>>()?;
        let searched: Vec<crate::search::Claim<'_>> = standing
            .iter()
            .zip(&bodies)
            .map(|((claim, version), body)| {
                let imported = version.imported.as_ref();
                crate::search::Claim {
                    name: &claim.name,
                    legacy: imported.and_then(|imported| imported.legacy.as_deref()),
                    statement: &version.proposal.statement,
                    section: imported.and_then(|imported| imported.section.as_deref()),
                    scenarios: &version.proposal.scenarios,
                    body: body.as_deref(),
                }
            })
            .collect();
        let found = crate::search::search(&searched, &words)?;
        let total = found.len();
        let shown = usize::try_from(limit).unwrap_or(usize::MAX);
        let mut claims = Vec::new();
        let mut lines = String::new();
        for found in found.iter().take(shown) {
            let (claim, version) = standing[found.place];
            let proposal = &version.proposal;
            let legacy = searched[found.place].legacy;
            let mut object = standing_json(claim, version);
            object.insert("excerpt".into(), json!(found.excerpt));
            object.insert("legacy".into(), json!(legacy));
            claims.push(Value::Object(object));
            let binds = if version.repositories.is_empty() {
                "the whole system".to_owned()
            } else {
                version.repositories.join(", ")
            };
            lines.push_str(&format!(
                "{} {}@{}, for {binds}: {}\n",
                claim.kind.as_str(),
                claim.name,
                proposal.version,
                proposal.statement
            ));
            if let Some(excerpt) = &found.excerpt {
                lines.push_str(&format!("  excerpt: {excerpt}\n"));
            }
            if proposal.body.is_some() {
                lines.push_str(&format!(
                    "  body: boom show {}@{}\n",
                    claim.name, proposal.version
                ));
            }
        }
        let matches = match total {
            0 => "No claim matches".to_owned(),
            1 => "1 claim matches".to_owned(),
            n => format!("{n} claims match"),
        };
        let scope = repository.as_deref().map_or_else(
            || "across every repository".to_owned(),
            |repository| format!("for {repository}"),
        );
        let follow = if total > shown {
            format!("; the first {shown} follow")
        } else {
            String::new()
        };
        Ok(Answer {
            json: json!({
                "format": "boom-search/1",
                "words": words,
                "repository": repository,
                "head": {"number": number, "hash": hash},
                "total": total,
                "claims": claims,
            }),
            words: format!(
                "{matches} \"{}\" {scope}, at entry {number} of the store's log{follow}.\n{lines}",
                words.join(" ")
            ),
        })
    })
}

fn import(
    path: &Path,
    form: Form,
    target: &Path,
    options: ImportOptions,
    env: &Env,
) -> Result<Answer, Failure> {
    let Some(repository) = options
        .repository
        .filter(|repository| !repository.trim().is_empty())
    else {
        return Err(Failure::refused(
            "an import names its repository, with --repository",
        ));
    };
    crate::claim::check_name(&repository)
        .map_err(|why| Failure::refused(format!("the repository's name: {why}")))?;
    let provenance = options
        .provenance
        .filter(|provenance| !provenance.trim().is_empty());
    if !options.dry_run && provenance.is_none() {
        return Err(Failure::refused(
            "an agent's proposal names its provenance, with --provenance",
        ));
    }
    let mut read = match form {
        Form::Madr => import::read_adr(target, &repository)?,
        Form::Rules => import::read_rules(target, &repository)?,
    };
    if let Some(drafts) = &options.drafts {
        import::read_drafts(drafts)?.apply(&mut read)?;
    }
    if options.dry_run {
        return Ok(import_answer(&repository, &read, None));
    }
    let needing: Vec<String> = read
        .records
        .iter()
        .filter(|record| !record.needs().is_empty())
        .map(|record| format!("{} needs {}", record.source, record.needs().join(" and ")))
        .collect();
    if !needing.is_empty() {
        return Err(Failure::refused(format!(
            "an import proposes a record once its draft gives what it needs: {}",
            needing.join("; ")
        )));
    }
    import::check_names(&read.records)?;
    let provenance = provenance.unwrap_or_default();
    let mut store = Store::open(path)?;
    let mut judged = store.state()?;
    let mut entries = Vec::new();
    for record in &read.records {
        let entry = Entry {
            at: env.clock.now(),
            actor: Actor::Agent {
                provenance: provenance.clone(),
            },
            act: Act::Proposed {
                claim: record.proposal(),
                imported: Some(Box::new(record.imported.clone())),
                // Imported for one repository, it stands for that one
                // alone (design 7).
                attach: Some(Attach {
                    repositories: vec![repository.clone()],
                }),
            },
        };
        let signed = entry.signed_part();
        match judge::judge(&judged, &entry, &signed, Proof::Unsigned) {
            Verdict::Counts => {
                judge::apply(&mut judged, 0, &entry);
                entries.push((signed, record.body.as_deref()));
            }
            Verdict::Refused(why) => {
                return Err(Failure::Refused(format!("{}: {why}", record.source)))
            }
            Verdict::Repeats(_) => {}
        }
    }
    let appends: Vec<Append<'_>> = entries
        .iter()
        .map(|(signed, body)| Append {
            signed,
            signature: None,
            blob: body.map(str::as_bytes),
        })
        .collect();
    let state = if appends.is_empty() {
        store.state()?
    } else {
        store.append_all(&appends)?.1
    };
    Ok(import_answer(&repository, &read, Some(&state)))
}

/// An import's answer: each record, as read or as proposed, the files
/// skipped, and the versions to accept together.
fn import_answer(repository: &str, read: &Read, state: Option<&State>) -> Answer {
    let mut records = Vec::with_capacity(read.records.len());
    let mut words = String::new();
    let mut to_accept = Vec::new();
    for record in &read.records {
        let proposal = record.proposal();
        let found = state.and_then(|state| {
            state
                .claims
                .get(&proposal.name)
                .and_then(|claim| claim.version(&proposal.version))
        });
        let name = record.name.as_deref();
        records.push(json!({
            "source": record.source,
            "kind": record.kind.as_str(),
            "name": name,
            "version": found.map(|_| &proposal.version),
            "status": found.map(|version| version.status.as_str()),
            "entry": found.map(|version| version.proposed_in),
            "statement": record.statement,
            "body": record.body,
            "scenarios": record.scenarios,
            "imported": record.imported,
            "history": record.history,
            "needs": record.needs(),
        }));
        words.push_str(&record_words(
            record,
            found.map(|version| {
                (
                    proposal.version.as_str(),
                    version.status.as_str(),
                    version.proposed_in,
                )
            }),
        ));
        if found.is_some() && !record.waits() {
            to_accept.push(format!("{}@{}", proposal.name, proposal.version));
        }
    }
    let skipped: Vec<Value> = read
        .skipped
        .iter()
        .map(|skipped| json!({"file": skipped.file, "why": skipped.why}))
        .collect();
    for skipped in &read.skipped {
        words.push_str(&format!("skipped {}: {}\n", skipped.file, skipped.why));
    }
    let accept = state.map(|_| {
        let command = (!to_accept.is_empty())
            .then(|| format!("boom accept {} --key <key>", to_accept.join(" ")));
        if let Some(command) = &command {
            words.push_str(&format!("accept them together: {command}\n"));
        }
        json!({"versions": to_accept, "command": command})
    });
    Answer {
        json: json!({
            "format": "boom-import/1",
            "repository": repository,
            "dry_run": state.is_none(),
            "records": records,
            "skipped": skipped,
            "accept": accept,
        }),
        words,
    }
}

fn record_words(record: &Record, proposed: Option<(&str, &str, u64)>) -> String {
    let name = record.name.as_deref().unwrap_or("(no name)");
    let mut words = format!("{} {name}", record.kind.as_str());
    if let Some((version, status, seq)) = proposed {
        words.push_str(&format!("@{version} is {status}, by entry {seq},"));
    }
    words.push_str(&format!(" from {}", record.source));
    if let Some(history) = &record.history {
        words.push_str(&format!(", history: {}", history.status.as_str()));
        if let Some(by) = &history.by {
            words.push_str(&format!(" by {by}"));
        }
    }
    let needs = record.needs();
    if !needs.is_empty() {
        words.push_str(&format!(", needs {}", needs.join(" and ")));
    }
    words.push('\n');
    words
}
