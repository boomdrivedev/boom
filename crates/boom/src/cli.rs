//! The command line: its arguments, its answers as words or as one JSON
//! object, and its exit codes (command-line; design 9).
//!
//! An answer is printed as words on standard output, or, with `--json`,
//! as one JSON object naming its format on standard output with every
//! word on standard error. A command exits 0 for an answer, 2 for a
//! refusal and 3 where it could not run, and on 2 and 3 writes nothing.
//! No command reads its terminal.

use std::collections::BTreeSet;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

use clap::{CommandFactory, Parser, Subcommand};
use serde_json::{json, Value};

use crate::claim::{History, Kind, Origin, Proposal};
use crate::entry::{Act, Actor, Attach, Entry, Form, Imported, Named};
use crate::import::{self, Read, Record};
use crate::judge::{self, Proof, State, Verdict};
use crate::sign::{self, Keys, Signer};
use crate::store::{self, Append, Scope, Store, Written};
use crate::{batch, context, repository, Failure};

/// What a command reads from its process: the folders a store's path
/// is resolved under, the working folder a repository is declared from,
/// and the clock an entry's time is read from.
#[derive(Debug, Clone, Default)]
pub struct Env {
    /// `XDG_DATA_HOME`, where it is set.
    pub xdg_data_home: Option<PathBuf>,
    /// `XDG_STATE_HOME`, where it is set: the records of the sessions are
    /// under it (session-context).
    pub xdg_state_home: Option<PathBuf>,
    /// `HOME`, where it is set.
    pub home: Option<PathBuf>,
    /// The working folder, where it can be read.
    pub cwd: Option<PathBuf>,
    /// `BOOM_REPOSITORY`, the repository a launcher names for the
    /// session, where it is set.
    pub boom_repository: Option<String>,
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
            xdg_state_home: var("XDG_STATE_HOME").map(PathBuf::from),
            home: var("HOME").map(PathBuf::from),
            cwd: std::env::current_dir().ok(),
            boom_repository: var(repository::LAUNCHER).and_then(|value| value.into_string().ok()),
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
    /// Standard output as bytes, in place of `stdout`: an artifact whose
    /// bytes are not text, which `boom show --artifact` writes as they
    /// are (claim-edges).
    pub bytes: Option<Vec<u8>>,
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
#[allow(
    clippy::large_enum_variant,
    reason = "a process parses one command one time, so the size of the largest does not matter"
)]
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
        /// no repository and no context, an amendment binds what its base
        /// binds, and a first version the repository of the session, else
        /// the whole system.
        #[arg(long = "repository", value_name = "NAME", conflicts_with = "system")]
        repositories: Vec<String>,
        /// A context the version binds, `<name>` or `<name>@<version>` of a
        /// context of the store; any number of times. A name stores the
        /// accepted version of the context, or its newest where none is
        /// accepted.
        #[arg(long = "context", value_name = "CONTEXT", conflicts_with = "system")]
        contexts: Vec<String>,
        /// Bind the version to the whole system, every repository.
        #[arg(long)]
        system: bool,
        /// An old word that a term replaces; any number of times. Only a
        /// term replaces words.
        #[arg(long = "replaces", value_name = "WORD")]
        replaces: Vec<String>,
        /// A claim this version rests on, `<name>` or `<name>@<version>`
        /// of a claim of the store; any number of times. A name stores
        /// its accepted version, or its newest where none is accepted.
        #[arg(long = "source", value_name = "CLAIM")]
        source: Vec<String>,
        /// For a decision, a claim it changes, `<name>` or
        /// `<name>@<version>` of a claim of the store; any number of
        /// times.
        #[arg(long = "changes", value_name = "CLAIM")]
        changes: Vec<String>,
        /// Where this version came from: a file, which boom keeps by the
        /// fingerprint of its bytes, a file in a commit,
        /// `git:<repository>@<commit>:<path>`, or the elaboration that made
        /// it, `elaboration:<repository>/<name>`, one at most; any number of
        /// times.
        #[arg(long = "origin", value_name = "ORIGIN")]
        origin: Vec<String>,
        /// For a context, a glob it owns, `<repository>:<pattern>`, the
        /// pattern relative to the root of the repository; any number of
        /// times. Only a context owns globs.
        #[arg(long = "path", value_name = "GLOB")]
        paths: Vec<String>,
        /// For a narrative, a claim it includes, `<name>` or
        /// `<name>@<version>` of a claim of the store; any number of times.
        /// A name stores its accepted version, or its newest where none is
        /// accepted.
        #[arg(long = "includes", value_name = "CLAIM")]
        includes: Vec<String>,
        /// For a narrative, the answer of this version to an open note,
        /// `<note>=<words>`, in one line; any number of times. A new
        /// version answers each open note of its narrative.
        #[arg(long = "answers", value_name = "NOTE=WORDS")]
        answers: Vec<String>,
        /// For an outline, the view it computes: `book`, the design book.
        #[arg(long, value_name = "VIEW")]
        view: Option<String>,
        /// For an outline, a narrative it holds, `<name>` or
        /// `<name>@<version>` of a narrative of the store; any number of
        /// times, in the view's order. A name stores its accepted version,
        /// or its newest where none is accepted.
        #[arg(long = "narrative", value_name = "NARRATIVE")]
        narratives: Vec<String>,
        /// The claim whose change forced this version, `<name>` or
        /// `<name>@<version>` of another claim of the store; one at most.
        /// A name stores its accepted version, or its newest where none is
        /// accepted. `boom accept --cause` takes each version that names
        /// it.
        #[arg(long, value_name = "CLAIM")]
        cause: Option<String>,
    },
    /// Write a person's note on one paragraph of a version of a
    /// narrative: an entry with the person's name and no signature.
    Note {
        /// The version, `<name>@<version>`.
        target: String,
        /// The paragraph's number, as `boom show <name>@<version>` gives it.
        #[arg(long, value_name = "N")]
        paragraph: u64,
        /// The note's words.
        words: String,
        /// The person who writes it; by default the one person the store
        /// registers.
        #[arg(long, value_name = "NAME")]
        person: Option<String>,
        /// Refused: a note carries the person's name and no signature.
        #[arg(long, hide = true)]
        key: Option<PathBuf>,
        /// Refused: only a person writes a note.
        #[arg(long, hide = true, value_name = "TEXT")]
        provenance: Option<String>,
    },
    /// Serve the page of a narrative on 127.0.0.1, where a person reads it
    /// with the claims it includes and writes a note on a paragraph, and
    /// open it in the browser; with no name, the design book.
    Page {
        /// The narrative, `<name>` for its newest version or
        /// `<name>@<version>`.
        target: Option<String>,
        /// The view of an outline, `<name>` of an outline of the store, in
        /// place of a narrative.
        #[arg(long, value_name = "OUTLINE", conflicts_with = "target")]
        view: Option<String>,
        /// The port; by default a free one.
        #[arg(long, value_name = "PORT", default_value_t = 0)]
        port: u16,
        /// Print the URL and open no browser.
        #[arg(long)]
        no_open: bool,
        /// Stop after this many minutes with no request.
        #[arg(
            long,
            value_name = "MINUTES",
            default_value_t = 60,
            value_parser = clap::value_parser!(u64).range(1..)
        )]
        idle: u64,
        /// The person who writes the notes of the page; by default the one
        /// person the store registers.
        #[arg(long, value_name = "NAME")]
        person: Option<String>,
    },
    /// Accept one or more versions, as a person, in one entry that their
    /// security key signs one time: every version, or none. A narrative
    /// takes each proposed version it includes; `--cause` takes a cause
    /// and each version that names it; `--waiting` lists what waits on a
    /// person, and with `--key` takes each of it.
    Accept {
        /// The versions, each `<name>@<version>`.
        #[arg(
            value_name = "TARGET",
            required_unless_present_any = ["cause", "waiting"],
            conflicts_with = "waiting"
        )]
        targets: Vec<String>,
        /// A cause, `<name>@<version>`: accept it, where it is proposed,
        /// with each version whose edge `cause` names it.
        #[arg(long, value_name = "CAUSE", conflicts_with = "waiting")]
        cause: Option<String>,
        /// List each batch that waits on a person; with `--key`, accept
        /// each batch that Boom does not refuse, in one entry.
        #[arg(long)]
        waiting: bool,
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
    /// Show a claim and its versions, or one version, `<name>@<version>`,
    /// or write the bytes of an artifact a version came from.
    Show {
        /// The claim's name, `<name>@<version>`, or the number of an
        /// entry of the log, for the version that the entry names.
        target: Option<String>,
        /// For a word with a term in several repositories, the term of
        /// this repository.
        #[arg(long, value_name = "NAME")]
        repository: Option<String>,
        /// Write the bytes of the artifact boom keeps under this
        /// fingerprint, which an `--origin` named, in place of a target.
        #[arg(long, value_name = "FINGERPRINT")]
        artifact: Option<String>,
    },
    /// Show the log's entries in order, each with whether it counts.
    Log,
    /// Check the log's whole chain, replay the whole log, and build the
    /// views again from it, naming each view that differed.
    Verify,
    /// Give every claim that stands for a repository, as the person
    /// accepted it: words an agent's harness can put in its context as
    /// they are.
    Query {
        /// The repository; by default the repository of the session:
        /// the one `BOOM_REPOSITORY` names, else the registered one whose
        /// remote matches the working folder's git remote.
        #[arg(long, value_name = "NAME")]
        repository: Option<String>,
        /// Answer nothing, and exit 0, where boom finds no repository of
        /// the session or its delivery at session start is off, as a
        /// harness's hook in every session asks.
        #[arg(long, conflicts_with = "repository")]
        if_declared: bool,
        /// A file, `<repository>:<path>`, for the claims that bind it: the
        /// whole system's, its repository's, and those of each context
        /// whose accepted globs match it; any number of times, each of one
        /// repository.
        #[arg(long = "path", value_name = "PATH")]
        paths: Vec<String>,
        /// An elaboration, `<repository>/<name>`, for the open notes on
        /// the versions that came from it, in place of the claims.
        #[arg(long, value_name = "ELABORATION")]
        elaboration: Option<String>,
    },
    /// Find the claims that stand whose words hold every word searched,
    /// in a repository or every repository, the best matches first,
    /// with no index kept.
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
    /// Register a repository, change its settings, or show them: the
    /// settings an owner signs in the log.
    Repo {
        #[command(subcommand)]
        what: RepoCommand,
    },
    /// Check files, or the text an agent writes, against the accepted
    /// terms of their repositories.
    Terms {
        #[command(subcommand)]
        what: TermsCommand,
    },
    /// Serve boom's commands to an agent's program as tools, over the
    /// Model Context Protocol on standard input and output; none of them
    /// is a person's act.
    Mcp,
    /// Answer one event of Claude Code, read as JSON on standard input,
    /// with the JSON of Claude Code's hook output, or nothing: the claims
    /// that an agent's session gets, in layers.
    Hook {
        #[command(subcommand)]
        what: HookCommand,
    },
}

#[derive(Subcommand, Debug)]
enum HookCommand {
    /// Answer `SessionStart`: the rules of the repository of the session
    /// in full, its other claims by name, and a menu.
    SessionStart,
    /// Answer `PreToolUse` of `Write`, `Edit` or `MultiEdit`: stop a write
    /// that holds an old word, and stop the first edit of a file in a
    /// context to give the rules of the context.
    PreToolUse,
}

#[derive(Subcommand, Debug)]
enum TermsCommand {
    /// Name each old word of an accepted term in each file, with its line
    /// and the term that replaces it; exit 2 where one is found. It reads
    /// the store and writes nothing.
    Check {
        /// The files.
        #[arg(value_name = "FILE")]
        files: Vec<PathBuf>,
        /// The repository of every file; by default the repository of
        /// each file, as `boom query` finds the repository of a session
        /// from the folder of the file.
        #[arg(long, value_name = "NAME")]
        repository: Option<String>,
        /// Skip a file where boom finds no repository, and check nothing
        /// where there is no store, as a hook in every session asks.
        #[arg(long)]
        if_declared: bool,
        /// Read one Claude Code `PreToolUse` event from standard input and
        /// check the text that `Write`, `Edit` or `MultiEdit` writes, in
        /// place of files; say nothing where no old word is found.
        #[arg(long, conflicts_with = "files")]
        hook: bool,
    },
}

/// On or off.
#[derive(clap::ValueEnum, Clone, Copy, Debug)]
enum OnOff {
    /// On.
    On,
    /// Off.
    Off,
}

#[derive(Subcommand, Debug)]
enum RepoCommand {
    /// Register a repository and its remote, as an owner, signed with
    /// their security key; a session there gets its context when it
    /// starts, unless `--session off`.
    Add {
        /// The repository's name, kebab case.
        name: String,
        /// Its remote, as git names it.
        #[arg(long, value_name = "URL")]
        remote: String,
        /// Whether a session in it gets its context when it starts.
        #[arg(long, value_enum, default_value = "on")]
        session: OnOff,
        /// The owner's security key, a private key file.
        #[arg(long, value_name = "KEY")]
        key: Option<PathBuf>,
    },
    /// Change a registered repository's remote or its delivery at session
    /// start, as an owner, signed with their security key.
    Set {
        /// The repository's name.
        name: String,
        /// Its new remote.
        #[arg(long, value_name = "URL")]
        remote: Option<String>,
        /// Whether a session in it gets its context when it starts.
        #[arg(long, value_enum)]
        session: Option<OnOff>,
        /// The owner's security key, a private key file.
        #[arg(long, value_name = "KEY")]
        key: Option<PathBuf>,
    },
    /// Show the repositories the store registers, or one.
    Show {
        /// The repository's name; every one where none is given.
        name: Option<String>,
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
    /// The drafts file, `boom-import-drafts/2`: each rule's name, and any
    /// record's other name, by its source.
    #[arg(long, value_name = "FILE")]
    drafts: Option<PathBuf>,
    /// The agent's session or run.
    #[arg(long, value_name = "TEXT")]
    provenance: Option<String>,
    /// Read and answer, writing nothing.
    #[arg(long)]
    dry_run: bool,
}

/// The command line's own definition, clap's reading of its commands
/// and arguments, which `boom mcp`'s tools are read from (agent-tools;
/// design 3).
pub(crate) fn definition() -> clap::Command {
    Cli::command()
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
    run_with_input(args, keys, env, &mut std::io::stdin())
}

/// Runs boom as [`run`] does, with `input` as its standard input, which
/// only `boom terms check --hook` and `boom hook` read.
pub fn run_with_input<I, A>(
    args: I,
    keys: &dyn Keys,
    env: &Env,
    input: &mut dyn std::io::Read,
) -> Outcome
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
                    bytes: None,
                    code: 0,
                    stdout: text,
                    stderr: String::new(),
                },
                _ => failed(json, Failure::Refused(text.trim_end().to_owned())),
            };
        }
    };
    if let Command::Mcp = cli.command {
        return mcp(cli.json, cli.store.as_deref(), env);
    }
    if let Command::Page {
        target,
        view,
        port,
        no_open,
        idle,
        person,
    } = &cli.command
    {
        if cli.json {
            return failed(
                false,
                Failure::refused("boom page takes no --json: it serves a page until it stops"),
            );
        }
        let path = store::resolve(
            cli.store.as_deref(),
            env.xdg_data_home.as_deref(),
            env.home.as_deref(),
        );
        let options = crate::page::Options {
            target: target.clone(),
            view: view.clone(),
            port: *port,
            open: !*no_open,
            idle: std::time::Duration::from_secs(idle.saturating_mul(60)),
            person: person.clone(),
        };
        return match path.and_then(|path| crate::page::serve(&path, &options, env)) {
            Ok(()) => Outcome {
                bytes: None,
                code: 0,
                stdout: String::new(),
                stderr: String::new(),
            },
            Err(failure) => failed(false, failure),
        };
    }
    if let Command::Hook { what } = &cli.command {
        let path = store::resolve(
            cli.store.as_deref(),
            env.xdg_data_home.as_deref(),
            env.home.as_deref(),
        );
        return hook(cli.json, what, path, env, input);
    }
    let path = store::resolve(
        cli.store.as_deref(),
        env.xdg_data_home.as_deref(),
        env.home.as_deref(),
    );
    if let Command::Terms {
        what:
            TermsCommand::Check {
                files,
                repository,
                if_declared,
                hook,
            },
    } = cli.command
    {
        let check = TermsCheck {
            files,
            named: repository,
            if_declared,
            hook,
        };
        return terms_check(cli.json, path, &check, env, input);
    }
    if let Command::Show {
        target: None,
        repository: None,
        artifact: Some(fingerprint),
    } = &cli.command
    {
        return artifact(cli.json, path, fingerprint);
    }
    let answer = execute(cli.command, path, keys, env);
    match answer {
        Ok(answer) if cli.json => Outcome {
            bytes: None,
            code: 0,
            stdout: format!("{}\n", answer.json),
            stderr: answer.words,
        },
        Ok(answer) => Outcome {
            bytes: None,
            code: 0,
            stdout: answer.words,
            stderr: String::new(),
        },
        Err(failure) => failed(cli.json, failure),
    }
}

/// `boom mcp` (agent-tools; design 2): refused with `--json`, else
/// serving the process's standard input and output until its input ends.
fn mcp(json: bool, store: Option<&Path>, env: &Env) -> Outcome {
    if json {
        return failed(
            true,
            Failure::refused(
                "boom mcp answers in the protocol's messages, one a line, never one JSON object",
            ),
        );
    }
    crate::mcp::serve(
        std::io::stdin().lock(),
        std::io::stdout().lock(),
        store,
        env,
    );
    Outcome {
        bytes: None,
        code: 0,
        stdout: String::new(),
        stderr: String::new(),
    }
}

/// `boom hook <event>` (session-context): refused with `--json`, else the
/// answer to the event on standard input as one JSON object, or nothing.
fn hook(
    json: bool,
    what: &HookCommand,
    path: Result<PathBuf, Failure>,
    env: &Env,
    input: &mut dyn std::io::Read,
) -> Outcome {
    if json {
        return failed(
            false,
            Failure::refused(
                "boom hook takes no --json: a hook always answers in the JSON of Claude Code",
            ),
        );
    }
    let mut event = String::new();
    let answered = input
        .read_to_string(&mut event)
        .map_err(|error| Failure::refused(format!("the hook's event cannot be read: {error}")))
        .and_then(|_| {
            serde_json::from_str::<Value>(&event)
                .map_err(|error| Failure::refused(format!("the hook's event is not JSON: {error}")))
        })
        .and_then(|event| match what {
            HookCommand::SessionStart => crate::hook::session_start(path, env, &event),
            HookCommand::PreToolUse => crate::hook::pre_tool_use(path, env, &event),
        });
    match answered {
        Ok(answer) => Outcome {
            bytes: None,
            code: 0,
            stdout: answer.map_or_else(String::new, |answer| format!("{answer}\n")),
            stderr: String::new(),
        },
        Err(failure) => failed(false, failure),
    }
}

/// `boom show --artifact <fingerprint>` (claim-edges, "boom show writes
/// an artifact"): the bytes of the artifact as they are; with `--json`,
/// one object with its name and its bytes, as `text` where they are
/// UTF-8 and as `base64` where they are not.
fn artifact(json: bool, path: Result<PathBuf, Failure>, fingerprint: &str) -> Outcome {
    let read = path.and_then(|path| {
        let mut store = Store::open(&path)?;
        store.read(|conn| {
            let Some(name) = store::artifact_name(conn, fingerprint)? else {
                return Err(Failure::refused(format!(
                    "no artifact has the fingerprint {fingerprint}: boom keeps the files that an --origin names"
                )));
            };
            Ok((name, store::blob(conn, fingerprint)?))
        })
    });
    let (name, bytes) = match read {
        Ok(read) => read,
        Err(failure) => return failed(json, failure),
    };
    let size = bytes.len();
    let words = format!("artifact {name}, sha256 {fingerprint}, {size} bytes\n");
    let text = String::from_utf8(bytes);
    if json {
        let (text, base64) = match &text {
            Ok(text) => (Some(text.clone()), None),
            Err(raw) => (None, Some(base64(raw.as_bytes()))),
        };
        let object = json!({
            "format": "boom-show/2",
            "artifact": fingerprint,
            "name": name,
            "size": size,
            "text": text,
            "base64": base64,
        });
        return Outcome {
            code: 0,
            stdout: format!("{object}\n"),
            stderr: words,
            bytes: None,
        };
    }
    match text {
        Ok(text) => Outcome {
            code: 0,
            stdout: text,
            stderr: String::new(),
            bytes: None,
        },
        Err(raw) => Outcome {
            code: 0,
            stdout: String::new(),
            stderr: String::new(),
            bytes: Some(raw.into_bytes()),
        },
    }
}

/// `bytes` in standard Base64, with padding.
fn base64(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut written = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let joined = chunk.iter().enumerate().fold(0u32, |joined, (at, byte)| {
            joined | u32::from(*byte) << (16 - 8 * at)
        });
        for at in 0..4 {
            if at <= chunk.len() {
                written.push(char::from(
                    ALPHABET[(joined >> (18 - 6 * at)) as usize & 63],
                ));
            } else {
                written.push('=');
            }
        }
    }
    written
}

/// A failure's outcome: its words on standard error and, with `json`,
/// its `boom-refused/1` or `boom-not-run/1` object on standard output.
pub(crate) fn failed(json: bool, failure: Failure) -> Outcome {
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
        bytes: None,
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
            amends,
            body,
            provenance,
            key,
            repositories,
            contexts,
            system,
            replaces,
            source,
            changes,
            origin,
            paths,
            includes,
            answers,
            view,
            narratives,
            cause,
        } => propose(
            &path?,
            Proposed {
                kind,
                name,
                statement,
                amends,
                body,
                provenance,
                repositories,
                contexts,
                system,
                replaces,
                source,
                changes,
                origin,
                paths,
                includes,
                answers,
                view,
                narratives,
                cause,
            },
            key.as_deref(),
            keys,
            env,
        ),
        Command::Note {
            target,
            paragraph,
            words,
            person,
            key,
            provenance,
        } => {
            if key.is_some() {
                return Err(Failure::refused(
                    "a note carries the person's name and no signature: boom note takes no --key",
                ));
            }
            if provenance.is_some() {
                return Err(Failure::refused(
                    "only a person writes a note: boom note takes no --provenance",
                ));
            }
            note(&path?, &target, paragraph, &words, person.as_deref(), env)
        }
        Command::Page { .. } => unreachable!("boom page serves before a command answers"),
        Command::Accept {
            targets,
            cause,
            waiting,
            key,
        } => {
            if waiting {
                accept_waiting(&path?, key.as_deref(), keys, env)
            } else {
                accept(
                    &path?,
                    &targets,
                    cause.as_deref(),
                    key.as_deref(),
                    keys,
                    env,
                )
            }
        }
        Command::Reject {
            target,
            reason,
            key,
        } => reject(&path?, &target, reason, key.as_deref(), keys, env),
        Command::Show {
            target,
            repository,
            artifact,
        } => match (target, artifact) {
            (Some(target), None) => show(&path?, &target, repository.as_deref()),
            (Some(_), Some(_)) => Err(Failure::refused(
                "boom show takes a target or an artifact, and not both",
            )),
            (None, Some(_)) => Err(Failure::refused(
                "boom show --artifact takes no --repository: an artifact is named by its fingerprint alone",
            )),
            (None, None) => Err(Failure::refused(
                "boom show takes a target, <name>, <name>@<version> or the number of an entry, or --artifact <fingerprint>",
            )),
        },
        Command::Log => log(&path?),
        Command::Verify => verify(&path?),
        Command::Query {
            repository,
            if_declared,
            paths,
            elaboration: Some(elaboration),
        } => elaboration_notes(path, &elaboration, repository.is_some(), if_declared, &paths),
        Command::Query {
            repository,
            if_declared,
            paths,
            elaboration: None,
        } => query(path, repository.as_deref(), if_declared, &paths, env),
        Command::Search {
            words,
            repository,
            all,
            limit,
        } => search(&path?, &words, repository.as_deref(), all, limit, env),
        Command::Mcp => unreachable!("boom mcp is served before a store's path is resolved"),
        Command::Hook { .. } => unreachable!("boom hook answers before a store is opened"),
        Command::Terms { .. } => {
            unreachable!("boom terms check is run before a command answers")
        }
        Command::Repo { what } => match what {
            RepoCommand::Add {
                name,
                remote,
                session,
                key,
            } => repo_add(
                &path?,
                &name,
                &remote,
                matches!(session, OnOff::On),
                key.as_deref(),
                keys,
                env,
            ),
            RepoCommand::Set {
                name,
                remote,
                session,
                key,
            } => repo_set(
                &path?,
                &name,
                remote.as_deref(),
                session.map(|session| matches!(session, OnOff::On)),
                key.as_deref(),
                keys,
                env,
            ),
            RepoCommand::Show { name } => repo_show(&path?, name.as_deref()),
        },
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
    scope: &Scope,
    signer: Option<&dyn Signer>,
    blobs: &[&[u8]],
    build: impl FnOnce(&State) -> Result<Entry, Failure>,
) -> Result<(Written, State), Failure> {
    let state = store.state_for(scope)?;
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
    store.append(&signed, signature.as_deref(), blobs)
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
        let state = Store::open(path)?.state_for(&Scope::default())?;
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
        rules: crate::judge::RULES,
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
    let (written, state) = store.append(&signed, Some(&signature), &[])?;
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
    let (written, state) = write(
        &mut store,
        &Scope::default(),
        Some(signer.as_ref()),
        &[],
        |state| {
            Ok(Entry {
                at: env.clock.now(),
                rules: crate::judge::RULES,
                actor: Actor::Person(person_of(state, signer.as_ref(), key)?),
                act: Act::Registered {
                    person: person.to_owned(),
                    key: sign::key_line(&registered),
                    owner,
                },
            })
        },
    )?;
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
    amends: Option<String>,
    body: Option<PathBuf>,
    provenance: Option<String>,
    repositories: Vec<String>,
    contexts: Vec<String>,
    system: bool,
    replaces: Vec<String>,
    source: Vec<String>,
    changes: Vec<String>,
    origin: Vec<String>,
    paths: Vec<String>,
    includes: Vec<String>,
    answers: Vec<String>,
    view: Option<String>,
    narratives: Vec<String>,
    cause: Option<String>,
}

impl Proposed {
    /// Whether it names what it binds: a repository, a context or the
    /// whole system.
    fn names_its_binding(&self) -> bool {
        !self.repositories.is_empty() || !self.contexts.is_empty() || self.system
    }
}

/// The edge to a claim that `--<flag> <given>` names, as an entry stores
/// it, `<name>@<version>`: for a name, its claim's accepted version, or
/// its newest where none is accepted (claim-edges, "An edge to a claim
/// names the version that it meant").
fn resolve_edge(state: &State, flag: &str, given: &str) -> Result<String, Failure> {
    let (name, version) = match given.split_once('@') {
        Some((name, version)) => (name, Some(version)),
        None => (given, None),
    };
    if crate::claim::check_name(name).is_err() || version.is_some_and(str::is_empty) {
        return Err(Failure::refused(format!(
            "an edge names a claim of the store, <name> or <name>@<version>: --{flag} {given} does not"
        )));
    }
    let claims = state.named(name);
    if claims.is_empty() {
        return Err(Failure::refused(format!(
            "no claim has the name {name}: --{flag} {given} names a claim of the store"
        )));
    }
    if let Some(version) = version {
        return match state.holding(name, version) {
            Some(_) => Ok(format!("{name}@{version}")),
            None => Err(Failure::refused(format!(
                "the claim {name} has no version {version}: --{flag} {given}"
            ))),
        };
    }
    let [claim] = claims.as_slice() else {
        return Err(Failure::refused(format!(
            "the word {name} has a term in several repositories: name the version, --{flag} {name}@<version>"
        )));
    };
    let meant = claim
        .standing
        .clone()
        .or_else(|| {
            claim
                .versions
                .last()
                .map(|version| version.proposal.version.clone())
        })
        .ok_or_else(|| Failure::not_run(format!("the claim {name} has no version")))?;
    Ok(format!("{name}@{meant}"))
}

/// Each edge of `given` resolved, each one time, in the order given.
fn resolve_edges(state: &State, flag: &str, given: &[String]) -> Result<Vec<String>, Failure> {
    let resolved = given
        .iter()
        .map(|edge| resolve_edge(state, flag, edge))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(crate::claim::once(&resolved))
}

/// The origins `--origin` names, each one time, in the order given,
/// with the bytes of each file boom keeps (claim-edges, "The origin of a
/// claim").
fn origins(given: &[String]) -> Result<(Vec<Origin>, Vec<Vec<u8>>), Failure> {
    let mut kept = Vec::new();
    let mut files = Vec::new();
    for origin in given {
        if origin.starts_with(crate::claim::GIT) {
            kept.push(Origin::git(origin).map_err(Failure::Refused)?);
            continue;
        }
        if origin.starts_with(crate::claim::ELABORATION) {
            kept.push(Origin::elaboration(origin).map_err(Failure::Refused)?);
            continue;
        }
        let file = Path::new(origin);
        let bytes = std::fs::read(file).map_err(|error| {
            Failure::refused(format!(
                "the origin {origin} cannot be read: {error}; an origin is a file, git:<repository>@<commit>:<path> or elaboration:<repository>/<name>"
            ))
        })?;
        let name = file
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| Failure::refused(format!("the origin {origin} names no file")))?;
        let origin = Origin::File {
            name: name.to_owned(),
            fingerprint: crate::sha256_hex(&bytes),
        };
        origin.check().map_err(Failure::Refused)?;
        kept.push(origin);
        files.push(bytes);
    }
    Ok((crate::claim::once(&kept), files))
}

/// The binding that `--context <given>` names, as an entry stores it,
/// `<name>@<version>`: for a name, the accepted version of the context, or
/// its newest where none is accepted (context-claims, "A claim binds a
/// context").
fn resolve_context(state: &State, given: &str) -> Result<String, Failure> {
    let (name, version) = match given.split_once('@') {
        Some((name, version)) => (name, Some(version)),
        None => (given, None),
    };
    if crate::claim::check_name(name).is_err() || version.is_some_and(str::is_empty) {
        return Err(Failure::refused(format!(
            "--context names a context of the store, <name> or <name>@<version>: {given} does not"
        )));
    }
    let Some(claim) = state.named(name).first().copied() else {
        return Err(Failure::refused(format!(
            "no claim has the name {name}: --context {given} names a context of the store"
        )));
    };
    if claim.kind != Kind::Context {
        return Err(Failure::refused(format!(
            "{name} is not a context: it is {} {}, and --context names a context",
            crate::claim::article(claim.kind),
            claim.kind.as_str()
        )));
    }
    let meant = match version {
        Some(version) if claim.version(version).is_some() => version.to_owned(),
        Some(version) => {
            return Err(Failure::refused(format!(
                "the context {name} has no version {version}: --context {given}"
            )))
        }
        None => claim
            .standing
            .clone()
            .or_else(|| {
                claim
                    .versions
                    .last()
                    .map(|version| version.proposal.version.clone())
            })
            .ok_or_else(|| Failure::not_run(format!("the context {name} has no version")))?,
    };
    Ok(format!("{name}@{meant}"))
}

/// The bindings that `given` names, sorted by name, each name one time.
fn resolve_contexts(state: &State, given: &[String]) -> Result<Vec<String>, Failure> {
    let mut resolved = given
        .iter()
        .map(|context| resolve_context(state, context))
        .collect::<Result<Vec<_>, _>>()?;
    resolved.sort();
    resolved.dedup();
    let names = context::names(&resolved);
    if let Some(pair) = names.windows(2).find(|pair| pair[0] == pair[1]) {
        return Err(Failure::refused(format!(
            "a version binds each context one time: --context names two versions of {}",
            pair[0]
        )));
    }
    Ok(resolved)
}

/// What a proposal binds (claim-attachments): for a context, the
/// repositories of its globs; else the repositories and the contexts it
/// names; else, with `--system`, the whole system; else, for an
/// amendment, the repositories of its base and the contexts of its base
/// by name, each at its accepted version now, else its newest; else the
/// repository `declared`; else the whole system.
fn attachment(
    proposed: &Proposed,
    declared: Option<&str>,
    state: &State,
) -> Result<Option<Attach>, Failure> {
    let made = |repositories: Vec<String>, contexts: Vec<String>| {
        (!repositories.is_empty() || !contexts.is_empty()).then_some(Attach {
            repositories,
            contexts,
        })
    };
    if proposed.kind == Kind::Context {
        return Ok(made(context::repositories(&proposed.paths), Vec::new()));
    }
    // An outline binds the whole system, wherever it is proposed (views).
    if proposed.kind == Kind::Outline {
        return Ok(None);
    }
    if !proposed.repositories.is_empty() || !proposed.contexts.is_empty() {
        let mut repositories = proposed.repositories.clone();
        repositories.sort();
        repositories.dedup();
        return Ok(made(
            repositories,
            resolve_contexts(state, &proposed.contexts)?,
        ));
    }
    if proposed.system {
        return Ok(None);
    }
    if let Some(base) = &proposed.amends {
        let Some(base) = state
            .holding(&proposed.name, base)
            .and_then(|claim| claim.version(base))
        else {
            // The judge refuses a base the claim does not have.
            return Ok(None);
        };
        let names: Vec<String> = context::names(&base.contexts)
            .into_iter()
            .map(str::to_owned)
            .collect();
        return Ok(made(
            base.repositories.clone(),
            resolve_contexts(state, &names)?,
        ));
    }
    Ok(declared.and_then(|name| made(vec![name.to_owned()], Vec::new())))
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
                if proposed.kind == Kind::Narrative {
                    return Err(Failure::refused(format!(
                        "the body of a narrative is UTF-8 text: {} is not",
                        file.display()
                    )));
                }
                return Err(Failure::refused(format!(
                    "a body is text: {} is not UTF-8",
                    file.display()
                )));
            }
            Some(bytes)
        }
        None => None,
    };
    match proposed.kind {
        Kind::Context if proposed.names_its_binding() => {
            return Err(Failure::refused(
                "a context binds the repositories of its globs: boom propose context takes no --repository, --system or --context",
            ))
        }
        Kind::Outline if !proposed.repositories.is_empty() || !proposed.contexts.is_empty() => {
            return Err(Failure::refused(
                "an outline binds the whole system: boom propose outline takes no --repository or --context",
            ))
        }
        Kind::Term if !proposed.contexts.is_empty() => {
            return Err(Failure::refused(
                "a term binds one repository or the whole system, and never a context: boom propose term takes no --context",
            ))
        }
        Kind::Context => {}
        kind if !proposed.paths.is_empty() => {
            return Err(Failure::refused(format!(
                "only a context owns paths: {} is proposed as {} {}, with --path",
                proposed.name,
                crate::claim::article(kind),
                kind.as_str()
            )))
        }
        _ => {}
    }
    let declared = if !proposed.names_its_binding()
        && proposed.amends.is_none()
        && proposed.kind != Kind::Context
        && proposed.kind != Kind::Outline
    {
        session_repository(path, None, env)?
    } else {
        None
    };
    // A term binds the whole system only where --system says so: in a
    // folder of no repository, a first term that names no binding is
    // refused (term-claims).
    if proposed.kind == Kind::Term
        && !proposed.names_its_binding()
        && proposed.amends.is_none()
        && declared.is_none()
    {
        return Err(Failure::refused(format!(
            "a term binds one repository or the whole system: {} names neither, and its folder is of no repository; name it with --repository or --system",
            proposed.name
        )));
    }
    if !proposed.changes.is_empty() && proposed.kind != Kind::Decision {
        return Err(Failure::refused(format!(
            "only a decision changes claims: {} is proposed as {} {}, with --changes",
            proposed.name,
            crate::claim::article(proposed.kind),
            proposed.kind.as_str()
        )));
    }
    if proposed.kind != Kind::Narrative {
        for (flag, given, what) in [
            ("includes", &proposed.includes, "includes claims"),
            ("answers", &proposed.answers, "answers notes"),
        ] {
            if !given.is_empty() {
                return Err(Failure::refused(format!(
                    "only a narrative {what}: {} is proposed as {} {}, with --{flag}",
                    proposed.name,
                    crate::claim::article(proposed.kind),
                    proposed.kind.as_str()
                )));
            }
        }
    }
    if proposed.kind == Kind::Outline {
        let mut held: Vec<&str> = Vec::with_capacity(proposed.narratives.len());
        for given in &proposed.narratives {
            let name = given
                .split_once('@')
                .map_or(given.as_str(), |(name, _)| name);
            if held.contains(&name) {
                return Err(Failure::refused(format!(
                    "the outline holds {name} two times: --narrative names each narrative one time"
                )));
            }
            held.push(name);
        }
    } else {
        for (flag, given, what) in [
            (
                "narrative",
                !proposed.narratives.is_empty(),
                "holds narratives",
            ),
            ("view", proposed.view.is_some(), "names a view"),
        ] {
            if given {
                return Err(Failure::refused(format!(
                    "only an outline {what}: {} is proposed as {} {}, with --{flag}",
                    proposed.name,
                    crate::claim::article(proposed.kind),
                    proposed.kind.as_str()
                )));
            }
        }
    }
    let answers = proposed
        .answers
        .iter()
        .map(|given| crate::claim::Answer::parse(given).map_err(Failure::Refused))
        .collect::<Result<Vec<_>, _>>()?;
    let (origin, files) = origins(&proposed.origin)?;
    let signer = key.map(|key| keys.signer(key)).transpose()?;
    // The claims the edges and the contexts name are read with the
    // claim, for the judge to find the versions they mean (claim-edges;
    // context-claims). An amendment that names nothing binds the contexts
    // of its base, which are read first.
    let mut scope = Scope::names(
        std::iter::once(proposed.name.as_str())
            .chain(
                proposed
                    .source
                    .iter()
                    .chain(&proposed.changes)
                    .chain(&proposed.contexts)
                    .chain(&proposed.includes)
                    .chain(&proposed.narratives)
                    .chain(&proposed.cause)
                    .map(|edge| edge.split_once('@').map_or(edge.as_str(), |(name, _)| name)),
            )
            .map(str::to_owned),
    );
    // The judge reads each outline for the one book of a store (views).
    scope.outlines = proposed.kind == Kind::Outline;
    scope
        .notes
        .extend(answers.iter().map(|answer| answer.note.clone()));
    if let (Some(base), false) = (&proposed.amends, proposed.names_its_binding()) {
        let bound = store.read(|conn| store::contexts_of(conn, &proposed.name, base))?;
        scope
            .names
            .extend(context::names(&bound).into_iter().map(str::to_owned));
    }
    let mut blobs: Vec<&[u8]> = body.as_deref().into_iter().collect();
    blobs.extend(files.iter().map(Vec::as_slice));
    let body_fingerprint = body.as_deref().map(crate::sha256_hex);
    // The version is made with what it binds: a term's version is the
    // fingerprint of its repository too (term-claims).
    let version_without_edges = |attach: Option<&Attach>| {
        if proposed.kind == Kind::Context {
            return Proposal::context(
                &proposed.name,
                &proposed.statement,
                body_fingerprint.clone(),
                proposed.amends.clone(),
                &proposed.paths,
            );
        }
        if proposed.kind == Kind::Term {
            return Proposal::term(
                &proposed.name,
                &proposed.statement,
                body_fingerprint.clone(),
                proposed.amends.clone(),
                judge::repositories(attach).first().map(String::as_str),
                &proposed.replaces,
            );
        }
        // Another kind with old words is refused by the judge, which
        // reads them from the version.
        let mut proposal = Proposal::new(
            proposed.kind,
            &proposed.name,
            &proposed.statement,
            Vec::new(),
            body_fingerprint.clone(),
            proposed.amends.clone(),
            None,
        );
        proposal.replaces = crate::claim::replaces(&proposed.replaces);
        proposal.view.clone_from(&proposed.view);
        proposal.version = proposal.fingerprint(None);
        proposal
    };
    // The edges are part of its content, so a person signs them
    // (claim-edges, "The edges are part of the content").
    let version_of = |attach: Option<&Attach>, edges: [Vec<String>; 4], cause: Option<String>| {
        let mut proposal = version_without_edges(attach);
        let [source, changes, includes, narratives] = edges;
        if source.is_empty()
            && changes.is_empty()
            && origin.is_empty()
            && includes.is_empty()
            && answers.is_empty()
            && narratives.is_empty()
            && cause.is_none()
        {
            return proposal;
        }
        proposal.cause = cause;
        proposal.source = source;
        proposal.changes = changes;
        proposal.origin = origin.clone();
        proposal.includes = includes;
        proposal.answers = answers.clone();
        proposal.narratives = narratives;
        let term = (proposed.kind == Kind::Term)
            .then(|| judge::repositories(attach).first().map(String::as_str))
            .flatten();
        proposal.version = proposal.fingerprint(term);
        proposal
    };
    let mut made = None;
    let (written, state) = write(&mut store, &scope, signer.as_deref(), &blobs, |state| {
        let actor = match (&signer, key) {
            (Some(signer), Some(key)) => Actor::Person(person_of(state, signer.as_ref(), key)?),
            _ => Actor::Agent {
                provenance: proposed.provenance.clone().unwrap_or_default(),
            },
        };
        let attach = attachment(&proposed, declared.as_deref(), state)?;
        let source = resolve_edges(state, "source", &proposed.source)?;
        let changes = resolve_edges(state, "changes", &proposed.changes)?;
        let includes = resolve_edges(state, "includes", &proposed.includes)?;
        // An outline's narratives keep the order given, each one time
        // (views).
        let narratives = proposed
            .narratives
            .iter()
            .map(|given| resolve_edge(state, "narrative", given))
            .collect::<Result<Vec<_>, _>>()?;
        let cause = proposed
            .cause
            .as_deref()
            .map(|given| resolve_edge(state, "cause", given))
            .transpose()?;
        let proposal = version_of(
            attach.as_ref(),
            [source, changes, includes, narratives],
            cause,
        );
        made = Some(proposal.clone());
        Ok(Entry {
            at: env.clock.now(),
            rules: crate::judge::RULES,
            actor,
            act: Act::Proposed {
                claim: Box::new(proposal),
                imported: None,
                attach,
            },
        })
    })?;
    let proposal = made.expect("a proposal written was made");
    let claim = state
        .holding(&proposal.name, &proposal.version)
        .expect("a proposal written is in the state");
    let version = claim
        .version(&proposal.version)
        .expect("a proposal written is in the state");
    let status = version.status.as_str();
    // What waits on a person is said: the command a person runs to accept
    // the version, while it is proposed (agent-tools; design 7).
    let accept = (status == "proposed").then(|| {
        format!(
            "boom accept {}@{} --key <key>",
            proposal.name, proposal.version
        )
    });
    let mut words = format!(
        "{} {}@{} is {status}, by entry {}\nit binds {}\n",
        claim.kind.as_str(),
        proposal.name,
        proposal.version,
        written.seq(),
        context::binds_words(&version.repositories, &version.contexts)
    );
    for glob in &proposal.paths {
        words.push_str(&format!("path: {glob}\n"));
    }
    words.push_str(&edges_words(
        &proposal.source,
        &proposal.changes,
        &proposal.origin,
    ));
    for edge in &proposal.includes {
        words.push_str(&format!("includes: {edge}\n"));
    }
    if let Some(view) = &proposal.view {
        words.push_str(&format!("view: {view}\n"));
    }
    for edge in &proposal.narratives {
        words.push_str(&format!("narrative: {edge}\n"));
    }
    for answer in &proposal.answers {
        words.push_str(&format!(
            "answers: note {}: {}\n",
            answer.note, answer.words
        ));
    }
    if let Some(cause) = &proposal.cause {
        words.push_str(&format!("cause: {cause}\n"));
    }
    if let Some(accept) = &accept {
        words.push_str(&format!("a person accepts it with: {accept}\n"));
    }
    Ok(Answer {
        json: json!({
            "format": "boom-propose/2",
            "claim": proposal.name,
            "kind": claim.kind.as_str(),
            "version": proposal.version,
            "base": proposal.base,
            "status": status,
            "entry": written.seq(),
            "repositories": version.repositories,
            "contexts": version.contexts,
            "paths": proposal.paths,
            "source": proposal.source,
            "changes": proposal.changes,
            "origin": proposal.origin.iter().map(origin_json).collect::<Vec<_>>(),
            "includes": proposal.includes,
            "answers": proposal.answers,
            "view": proposal.view,
            "narratives": proposal.narratives,
            "cause": proposal.cause,
            "accept": accept,
        }),
        words,
    })
}

/// `boom note`: a person's note on one paragraph of a version of a
/// narrative, with the person's name and no signature (narrative-notes).
fn note(
    path: &Path,
    target: &str,
    paragraph: u64,
    words: &str,
    person: Option<&str>,
    env: &Env,
) -> Result<Answer, Failure> {
    let (name, version) = version_target("note", target)?;
    let mut store = Store::open(path)?;
    let written =
        crate::page::write_note(&mut store, name, version, paragraph, words, person, env)?;
    let mut lines = format!(
        "note {} on paragraph {paragraph} of {name}@{version}, by {}, by entry {}\n",
        written.note, written.person, written.entry
    );
    lines.push_str("It stays open until a version of the narrative answers it.\n");
    Ok(Answer {
        json: json!({
            "format": "boom-note/1",
            "note": written.note,
            "narrative": name,
            "version": version,
            "paragraph": paragraph,
            "fingerprint": written.fingerprint,
            "person": written.person,
            "words": words,
            "entry": written.entry,
        }),
        words: lines,
    })
}

/// The edges of a version as `boom propose` names them, one line each.
fn edges_words(source: &[String], changes: &[String], origin: &[Origin]) -> String {
    let mut words = String::new();
    for edge in source {
        words.push_str(&format!("source: {edge}\n"));
    }
    for edge in changes {
        words.push_str(&format!("changes: {edge}\n"));
    }
    for origin in origin {
        words.push_str(&format!("origin: {}\n", origin.words()));
    }
    words
}

/// An origin as JSON: a file's `name` and `fingerprint`, a commit's
/// `repository`, `commit` and `path`, or an elaboration's `repository`
/// and `elaboration`, with its `reference` as a person writes it.
fn origin_json(origin: &Origin) -> Value {
    match origin {
        Origin::File { name, fingerprint } => json!({
            "file": name,
            "fingerprint": fingerprint,
        }),
        Origin::Git {
            repository,
            commit,
            path,
        } => json!({
            "repository": repository,
            "commit": commit,
            "path": path,
            "reference": origin.words(),
        }),
        Origin::Elaboration { repository, name } => json!({
            "repository": repository,
            "elaboration": name,
            "reference": origin.words(),
        }),
    }
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
    cause: Option<&str>,
    key: Option<&Path>,
    keys: &dyn Keys,
    env: &Env,
) -> Result<Answer, Failure> {
    let versions = targets
        .iter()
        .map(|target| version_target("accept", target))
        .collect::<Result<Vec<_>, _>>()?;
    let cause = cause
        .map(|cause| version_target("accept --cause", cause))
        .transpose()?;
    let Some(key) = key else {
        return Err(Failure::refused(
            "only a person accepts: boom accept takes the person's --key",
        ));
    };
    let mut store = Store::open(path)?;
    let taken = store.read(|conn| accept_takes(conn, &versions, cause))?;
    let (accepted, words) = accept_versions(&mut store, &taken, key, keys, env)?;
    Ok(accepted_answer(accepted, words, serde_json::Map::new()))
}

/// The versions that an accept of `targets` and of `cause` takes, in the
/// order of its entry, each one time: each target, after a proposed
/// narrative each proposed version it includes, and the cause with each
/// version that names it (persons-acts, "The accept of a narrative takes
/// the claims that it includes" and "The accept of a cause takes its
/// revision"). A closed include or cause refuses it.
fn accept_takes(
    conn: &rusqlite::Connection,
    targets: &[(&str, &str)],
    cause: Option<(&str, &str)>,
) -> Result<Vec<(String, String)>, Failure> {
    let naming = match cause {
        Some((name, version)) => store::naming_cause(conn, name, version)?,
        None => Vec::new(),
    };
    let mut names: BTreeSet<String> = targets
        .iter()
        .chain(&cause)
        .map(|(name, _)| (*name).to_owned())
        .chain(naming.iter().map(|(name, _)| name.clone()))
        .collect();
    let first = store::load_state_for(conn, &Scope::names(names.iter().cloned()))?;
    for &(name, version) in targets {
        if let Some(found) = first
            .holding(name, version)
            .and_then(|c| c.version(version))
        {
            names.extend(
                found
                    .proposal
                    .includes
                    .iter()
                    .filter_map(|edge| edge.split_once('@'))
                    .map(|(name, _)| name.to_owned()),
            );
        }
    }
    let state = store::load_state_for(conn, &Scope::names(names))?;
    let find = |name: &str, version: &str| {
        state
            .holding(name, version)
            .and_then(|claim| claim.version(version))
    };
    let mut entry: Vec<(String, String)> = Vec::new();
    let mut push = |name: &str, version: &str| {
        let pair = (name.to_owned(), version.to_owned());
        if !entry.contains(&pair) {
            entry.push(pair);
        }
    };
    let mut closed = Vec::new();
    for &(name, version) in targets {
        push(name, version);
        let Some(narrative) = find(name, version).filter(|v| v.proposal.kind == Kind::Narrative)
        else {
            continue;
        };
        let takes = batch::of_narrative(&state, name, narrative);
        if narrative.status == judge::Status::Proposed {
            closed.extend(takes.closed);
            for named in &takes.versions {
                push(&named.name, &named.version);
            }
        } else {
            // A narrative accepted already lists again the versions that
            // its accept took, which its entry accepted with it.
            for named in takes.accepted.iter().filter(|named| {
                find(&named.name, &named.version)
                    .is_some_and(|v| v.decided_in.is_some() && v.decided_in == narrative.decided_in)
            }) {
                push(&named.name, &named.version);
            }
        }
    }
    if let Some((name, version)) = cause {
        let Some(found) = find(name, version) else {
            return Err(Failure::refused(format!(
                "boom accept --cause names a version of the store: {name}@{version} is none"
            )));
        };
        let takes = batch::of_cause(&state, name, found);
        if takes.versions.len() + takes.accepted.len() + takes.closed.len() < 2 {
            return Err(Failure::refused(format!(
                "no version names {name}@{version} as its cause"
            )));
        }
        closed.extend(takes.closed);
        for named in takes.versions.iter().chain(&takes.accepted) {
            push(&named.name, &named.version);
        }
    }
    if !closed.is_empty() {
        return Err(Failure::refused(format!(
            "boom refuses the accept: {}; its author proposes a new version that names the current one",
            batch::closed_words(&closed)
        )));
    }
    Ok(entry)
}

/// Accepts `versions` as the person whose key is `key`: each version
/// that waits goes into one entry, judged whole, which the key signs one
/// time once Boom has told the person what it takes. A version this
/// person accepted already is left out of it. Gives each version as the
/// answer lists it, and its words.
fn accept_versions(
    store: &mut Store,
    versions: &[(String, String)],
    key: &Path,
    keys: &dyn Keys,
    env: &Env,
) -> Result<(Vec<Value>, String), Failure> {
    let signer = keys.signer(key)?;
    let scope = Scope::names(versions.iter().map(|(name, _)| name.as_str()));
    let state = store.state_for(&scope)?;
    let person = person_of(&state, signer.as_ref(), key)?;
    let now = env.clock.now();
    let one = |name: &str, version: &str| Entry {
        at: now,
        rules: crate::judge::RULES,
        actor: Actor::Person(person.clone()),
        act: Act::accepted(name, version),
    };
    let mut waiting = Vec::new();
    for (name, version) in versions {
        if waiting.iter().any(|named: &Named| &named.name == name) {
            return Err(Failure::refused(format!(
                "boom accept names each claim one time: it names {name} twice"
            )));
        }
        let entry = one(name, version);
        let signed = entry.signed_part();
        match judge::judge(
            &state,
            &entry,
            &signed,
            Proof::ToBeSigned(signer.public_key()),
        ) {
            Verdict::Counts => waiting.push(Named {
                name: name.clone(),
                version: version.clone(),
            }),
            Verdict::Refused(why) => return Err(Failure::Refused(naming(name, version, why))),
            Verdict::Repeats(_) => {}
        }
    }
    let state = if waiting.is_empty() {
        store.state_for(&scope)?
    } else {
        let mut told = format!(
            "This accept takes {} in one entry:\n",
            plural(waiting.len(), "version", "versions")
        );
        for named in &waiting {
            let kind = state
                .holding(&named.name, &named.version)
                .map_or("claim", |claim| claim.kind.as_str());
            told.push_str(&format!("  {kind} {}@{}\n", named.name, named.version));
        }
        let entry = Entry {
            at: now,
            rules: crate::judge::RULES,
            actor: Actor::Person(person.clone()),
            act: Act::Accepted { versions: waiting },
        };
        let signed = entry.signed_part();
        if let Verdict::Refused(why) = judge::judge(
            &state,
            &entry,
            &signed,
            Proof::ToBeSigned(signer.public_key()),
        ) {
            return Err(Failure::Refused(why));
        }
        signer.tell(&told);
        let signature = signer.sign(signed.as_bytes())?;
        store
            .append_all(&[Append {
                signed: &signed,
                signature: Some(&signature),
                blobs: &[],
            }])?
            .1
    };
    let mut accepted = Vec::with_capacity(versions.len());
    let mut words = String::new();
    for (name, version) in versions {
        let claim = state
            .holding(name, version)
            .expect("an accept written is in the state");
        let found = claim
            .version(version)
            .expect("an accept written is in the state");
        let by = found.decided_by.clone().unwrap_or_default();
        let seq = found.decided_in.unwrap_or_default();
        accepted.push(json!({
            "claim": name,
            "version": version,
            "kind": claim.kind.as_str(),
            "status": found.status.as_str(),
            "superseded": found.proposal.base,
            "by": by,
            "entry": seq,
        }));
        words.push_str(&format!(
            "{name}@{version} is {}, accepted by {by}, by entry {seq}\n",
            found.status.as_str(),
        ));
    }
    Ok((accepted, words))
}

/// `n` and its noun, of one or of many.
fn plural(n: usize, one: &str, many: &str) -> String {
    format!("{n} {}", if n == 1 { one } else { many })
}

/// The answer of an accept: `accepted`, each version, and where it is
/// one, its fields at the top as before; with `more` besides.
fn accepted_answer(
    accepted: Vec<Value>,
    words: String,
    more: serde_json::Map<String, Value>,
) -> Answer {
    let mut object = serde_json::Map::new();
    object.insert("format".into(), json!("boom-accept/2"));
    if let [one] = accepted.as_slice() {
        if let Some(fields) = one.as_object() {
            object.extend(fields.clone());
        }
    }
    object.insert("accepted".into(), Value::Array(accepted));
    object.extend(more);
    Answer {
        json: Value::Object(object),
        words,
    }
}

/// A batch as `boom accept --waiting` gives it.
fn batch_json(batch: &batch::Batch) -> Value {
    let named = |list: &[batch::Named]| {
        list.iter()
            .map(|named| {
                json!({
                    "claim": named.name,
                    "version": named.version,
                    "kind": named.kind.as_str(),
                    "status": named.status.as_str(),
                })
            })
            .collect::<Vec<_>>()
    };
    json!({
        "shape": batch.shape.as_str(),
        "claim": batch.head.name,
        "version": batch.head.version,
        "kind": batch.head.kind.as_str(),
        "title": batch.title,
        "counts": batch.counts(),
        "proposer": batch.proposer,
        "repositories": batch.repositories,
        "command": batch.command(),
        "versions": named(&batch.takes.versions),
        "accepted": named(&batch.takes.accepted),
        "closed": named(&batch.takes.closed),
    })
}

/// A batch in words: its shape and head, its title, its counts, and its
/// command or what refuses it.
fn batch_words(batch: &batch::Batch) -> String {
    let mut words = format!(
        "{} {}: {}\n  {}\n",
        batch.shape.as_str(),
        batch.head.target(),
        batch.title,
        batch.counts()
    );
    match batch.command() {
        Some(command) => words.push_str(&format!("  {command}\n")),
        None => words.push_str(&format!(
            "  refused: {}; its author proposes a new version\n",
            batch::closed_words(&batch.takes.closed)
        )),
    }
    words
}

/// `boom accept --waiting`: each batch that waits on a person, and with
/// `key`, each batch that Boom does not refuse, accepted in one entry
/// (persons-acts, "boom accept --waiting").
fn accept_waiting(
    path: &Path,
    key: Option<&Path>,
    keys: &dyn Keys,
    env: &Env,
) -> Result<Answer, Failure> {
    let mut store = Store::open(path)?;
    let batches = store.read(batch::waiting)?;
    let mut versions: Vec<(String, String)> = Vec::new();
    for batch in batches.iter().filter(|batch| batch.command().is_some()) {
        for named in &batch.takes.versions {
            let pair = (named.name.clone(), named.version.clone());
            if !versions.contains(&pair) {
                versions.push(pair);
            }
        }
    }
    let listed: Vec<Value> = batches.iter().map(batch_json).collect();
    let Some(key) = key else {
        let mut words = if batches.is_empty() {
            "Nothing waits on a person.\n".to_owned()
        } else {
            format!(
                "{} wait on a person, {}; boom accept --waiting --key <key> accepts each one that Boom does not refuse, in one sign.\n",
                plural(batches.len(), "batch", "batches"),
                plural(versions.len(), "version", "versions"),
            )
        };
        for batch in &batches {
            words.push_str(&batch_words(batch));
        }
        return Ok(Answer {
            json: json!({
                "format": "boom-waiting/1",
                "batches": listed,
                "versions": versions.len(),
            }),
            words,
        });
    };
    let left_out: Vec<&batch::Batch> = batches
        .iter()
        .filter(|batch| batch.command().is_none())
        .collect();
    let (accepted, mut words) = if versions.is_empty() {
        (
            Vec::new(),
            "Nothing waits on a person that Boom does not refuse.\n".to_owned(),
        )
    } else {
        let mut store = Store::open(path)?;
        accept_versions(&mut store, &versions, key, keys, env)?
    };
    for batch in &left_out {
        words.push_str(&format!(
            "left out: {} {}: {}\n",
            batch.shape.as_str(),
            batch.head.target(),
            batch::closed_words(&batch.takes.closed)
        ));
    }
    let mut more = serde_json::Map::new();
    more.insert(
        "left_out".into(),
        Value::Array(left_out.iter().map(|batch| batch_json(batch)).collect()),
    );
    Ok(accepted_answer(accepted, words, more))
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
    let (written, state) = write(
        &mut store,
        &Scope::names([name]),
        Some(signer.as_ref()),
        &[],
        |state| {
            Ok(Entry {
                at: env.clock.now(),
                rules: crate::judge::RULES,
                actor: Actor::Person(person_of(state, signer.as_ref(), key)?),
                act: Act::Rejected {
                    name: name.to_owned(),
                    version: version.to_owned(),
                    reason,
                },
            })
        },
    )?;
    let rejected = state
        .holding(name, version)
        .and_then(|claim| claim.version(version))
        .expect("a reject written is in the state");
    let by = rejected.decided_by.clone().unwrap_or_default();
    let why = rejected.reason.clone().unwrap_or_default();
    Ok(Answer {
        json: json!({
            "format": "boom-reject/2",
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
    // A person's reject carries its reason; every other decision is an
    // accept, a record that came in as history among them.
    let rejected = view.reason.is_some();
    let accepted = view.decided_by.is_some() && !rejected;
    let mut object = serde_json::Map::new();
    object.insert("version".into(), json!(view.version));
    object.insert("status".into(), json!(view.status));
    object.insert("base".into(), json!(view.base));
    object.insert("repositories".into(), json!(view.repositories));
    object.insert("contexts".into(), json!(view.contexts));
    object.insert("paths".into(), json!(view.paths));
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
        let act = if view.reason.is_some() {
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

fn show(path: &Path, target: &str, repository: Option<&str>) -> Result<Answer, Failure> {
    let mut store = Store::open(path)?;
    store.read(|conn| {
        let named;
        let (name, version) = if !target.is_empty() && target.bytes().all(|b| b.is_ascii_digit()) {
            named = entry_target(conn, target)?;
            (named.0.as_str(), Some(named.1.as_str()))
        } else {
            match target.split_once('@') {
                Some((name, version)) => (name, Some(version)),
                None => (target, None),
            }
        };
        let mut claims = store::claim_views(conn, name)?;
        if claims.is_empty() {
            return Err(Failure::refused(format!("no claim has the name {name}")));
        }
        if let Some(repository) = repository {
            claims.retain(|claim| claim.repository.as_deref() == Some(repository));
            if claims.is_empty() {
                return Err(Failure::refused(format!(
                    "{name} has no term for the repository {repository}"
                )));
            }
        }
        let Some(version) = version else {
            // A word with a term in several repositories: each term, in
            // the order of their repositories (term-claims).
            if let [claim] = claims.as_slice() {
                let (object, words) = claim_shown(conn, claim)?;
                let mut json = serde_json::Map::new();
                json.insert("format".into(), json!("boom-show/2"));
                json.extend(object);
                return Ok(Answer {
                    json: Value::Object(json),
                    words,
                });
            }
            let mut terms = Vec::with_capacity(claims.len());
            let mut words = String::new();
            for claim in &claims {
                let (object, shown) = claim_shown(conn, claim)?;
                terms.push(Value::Object(object));
                if !words.is_empty() {
                    words.push('\n');
                }
                words.push_str(&shown);
            }
            return Ok(Answer {
                json: json!({
                    "format": "boom-show/2",
                    "claim": name,
                    "kind": claims[0].kind,
                    "terms": terms,
                }),
                words,
            });
        };
        let (claim, view) = claims
            .iter()
            .find_map(|claim| {
                claim
                    .versions
                    .iter()
                    .find(|view| view.version == version)
                    .map(|view| (claim, view))
            })
            .ok_or_else(|| {
                Failure::refused(format!("the claim {name} has no version {version}"))
            })?;
        let body = body_text(conn, view.body.as_deref())?;
        let (imported, history) = proposed_with(conn, view.proposed_in)?;
        let mut object = serde_json::Map::new();
        object.insert("format".into(), json!("boom-show/2"));
        object.insert("claim".into(), json!(claim.name));
        object.insert("kind".into(), json!(claim.kind));
        if let Some(repository) = &claim.repository {
            object.insert("repository".into(), json!(repository));
        }
        object.extend(version_json(view));
        object.insert("statement".into(), json!(view.statement));
        object.insert("scenarios".into(), json!(view.scenarios));
        object.insert("replaces".into(), json!(view.replaces));
        object.insert("body_fingerprint".into(), json!(view.body));
        object.insert("body".into(), json!(body));
        object.insert("imported".into(), json!(imported));
        object.insert("history".into(), json!(history));
        let narrative = claim.kind == Kind::Narrative.as_str();
        let (edges, edge_words) = edges_shown(conn, view, &claim.kind)?;
        object.extend(edges);
        let view_words = match &view.view {
            Some(shown) => {
                object.insert("view".into(), json!(shown));
                format!("view: {shown}\n")
            }
            None => String::new(),
        };
        let mut narrative_words = String::new();
        if narrative {
            let state = store::load_state_for(conn, &Scope::names([name]))?;
            let paragraphs = crate::narrative::paragraphs(body.as_deref().unwrap_or_default());
            object.insert(
                "paragraphs".into(),
                Value::Array(
                    paragraphs
                        .iter()
                        .map(|paragraph| {
                            json!({
                                "number": paragraph.number,
                                "fingerprint": paragraph.fingerprint,
                                "kind": paragraph.kind,
                            })
                        })
                        .collect(),
                ),
            );
            let (answers, answer_words) = answers_shown(view);
            object.insert("answers".into(), answers);
            let (notes, note_words) = notes_shown(&state, name, Some(version));
            object.insert("notes".into(), notes);
            narrative_words = format!(
                "paragraphs: {}\n{answer_words}{note_words}",
                paragraphs.len()
            );
        }
        let (bindings, binding_words) = bindings_shown(conn, view)?;
        object.insert("bindings".into(), bindings);
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
        if !view.replaces.is_empty() {
            // The old words are given to the person who accepts the term
            // (term-claims, "The old words stay in the log").
            words.push_str(&format!("replaces: {}\n", view.replaces.join(", ")));
        }
        words.push_str(&format!(
            "binds: {}\n",
            context::binds_words(&view.repositories, &view.contexts)
        ));
        words.push_str(&binding_words);
        for glob in &view.paths {
            words.push_str(&format!("path: {glob}\n"));
        }
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
        words.push_str(&view_words);
        words.push_str(&edge_words);
        words.push_str(&narrative_words);
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

/// The edges of a version as `boom show <name>@<version>` gives them, in
/// their order (claim-edges, "boom show gives the edges of a version"):
/// for an edge to a claim, the version it meant and its statement,
/// marked out of date where another version of that claim is accepted
/// now; for a change, `Before:` and `Now:`; for an origin, its file and
/// fingerprint, or its repository, commit and path.
fn edges_shown(
    conn: &rusqlite::Connection,
    view: &store::VersionView,
    kind: &str,
) -> Result<(serde_json::Map<String, Value>, String), Failure> {
    let mut object = serde_json::Map::new();
    let mut words = String::new();
    let mut kinds = vec![("source", &view.source), ("changes", &view.changes)];
    if kind == Kind::Narrative.as_str() {
        // The claims a narrative includes, each at the version it meant
        // (narratives, "boom show gives a narrative").
        kinds.push(("includes", &view.includes));
    }
    if kind == Kind::Outline.as_str() {
        // The narratives an outline holds, in its order, each at the
        // version it meant (views, "boom show gives an outline").
        kinds.push(("narratives", &view.narratives));
    }
    // The version of the claim that forced it (claim-edges, "A version
    // names its cause").
    let cause: Vec<String> = view.cause.iter().cloned().collect();
    if !cause.is_empty() {
        kinds.push(("cause", &cause));
    }
    for (flag, edges) in kinds {
        let mut shown = Vec::with_capacity(edges.len());
        for stored in edges.iter() {
            let (name, version) = stored.split_once('@').unwrap_or((stored, ""));
            let (statement, accepted) = store::meant(conn, name, version)?.ok_or_else(|| {
                Failure::not_run(format!(
                    "the views hold no version {stored}, which an edge names"
                ))
            })?;
            let out_of_date = accepted
                .as_deref()
                .is_some_and(|accepted| accepted != version);
            let mut edge = json!({
                "claim": name,
                "version": version,
                "statement": statement,
                "accepted": accepted,
                "out_of_date": out_of_date,
            });
            words.push_str(&format!("{flag}: {stored}"));
            if let (true, Some(accepted)) = (out_of_date, &accepted) {
                words.push_str(&format!(", out of date: {name}@{accepted} is accepted"));
            }
            words.push('\n');
            if flag == "changes" {
                edge["before"] = json!(statement);
                edge["now"] = json!(view.statement);
                words.push_str(&format!(
                    "  Before: {statement}\n  Now: {}\n",
                    view.statement
                ));
            } else {
                words.push_str(&format!("  {statement}\n"));
            }
            shown.push(edge);
        }
        object.insert(flag.into(), Value::Array(shown));
    }
    for origin in &view.origin {
        words.push_str(&format!("origin: {}\n", origin.words()));
    }
    object.insert(
        "origin".into(),
        Value::Array(view.origin.iter().map(origin_json).collect()),
    );
    Ok((object, words))
}

/// The contexts a version binds, as `boom show <name>@<version>` gives
/// them (context-claims, "boom show gives a context and a binding"): each
/// with the version that the binding meant, marked out of date where
/// another version of that context is accepted now.
fn bindings_shown(
    conn: &rusqlite::Connection,
    view: &store::VersionView,
) -> Result<(Value, String), Failure> {
    let mut shown = Vec::with_capacity(view.contexts.len());
    let mut words = String::new();
    for stored in &view.contexts {
        let (name, version) = stored.split_once('@').unwrap_or((stored, ""));
        let (_, accepted) = store::meant(conn, name, version)?.ok_or_else(|| {
            Failure::not_run(format!(
                "the views hold no version {stored}, which a binding names"
            ))
        })?;
        let out_of_date = accepted
            .as_deref()
            .is_some_and(|accepted| accepted != version);
        words.push_str(&format!("context: {stored}"));
        if let (true, Some(accepted)) = (out_of_date, &accepted) {
            words.push_str(&format!(", out of date: {name}@{accepted} is accepted"));
        }
        words.push('\n');
        shown.push(json!({
            "context": name,
            "version": version,
            "accepted": accepted,
            "out_of_date": out_of_date,
        }));
    }
    Ok((Value::Array(shown), words))
}

/// The claims whose accepted version binds the context `claim`, as
/// `boom show <context>` gives them under `bound:`, each with the version
/// of the context that its binding meant.
fn bound_shown(
    conn: &rusqlite::Connection,
    claim: &store::ClaimView,
) -> Result<(Value, String), Failure> {
    let bound = store::bound_to(conn, &claim.name)?;
    let mut words = String::new();
    if !bound.is_empty() {
        words.push_str("\nbound:\n");
    }
    for found in &bound {
        words.push_str(&format!(
            "  {}@{}, on {}@{}",
            found.name, found.version, claim.name, found.meant
        ));
        if let Some(accepted) = claim.standing.as_deref().filter(|a| *a != found.meant) {
            words.push_str(&format!(
                ", out of date: {}@{accepted} is accepted",
                claim.name
            ));
        }
        words.push('\n');
    }
    let listed = bound
        .iter()
        .map(|found| {
            json!({
                "claim": found.name,
                "version": found.version,
                "meant": found.meant,
            })
        })
        .collect();
    Ok((Value::Array(listed), words))
}

/// The other versions of a claim as a tree by base, each amendment under
/// the version it amends and the versions of one base newest first, one
/// line pair each, indented by its depth (claim-versions, "A claim shown
/// by its name").
fn tree_words(others: &[&store::VersionView]) -> String {
    fn under(others: &[&store::VersionView], base: Option<&str>, depth: usize, words: &mut String) {
        let roots = |view: &&&store::VersionView| match base {
            None => view
                .base
                .as_deref()
                .is_none_or(|base| !others.iter().any(|other| other.version == base)),
            Some(base) => view.base.as_deref() == Some(base),
        };
        for view in others.iter().filter(roots) {
            let indent = "  ".repeat(depth + 1);
            words.push_str(&format!(
                "{indent}{}\n{indent}  {}\n{indent}  binds: {}\n",
                version_words(view),
                view.statement,
                context::binds_words(&view.repositories, &view.contexts)
            ));
            under(others, Some(&view.version), depth + 1, words);
        }
    }
    let mut words = String::new();
    under(others, None, 0, &mut words);
    words
}

/// The claims that rest on `claim`, as `boom show <name>` gives them
/// (claim-edges, "boom show gives the claims that rest on a claim"): each
/// version that counts for its claim and names a version of `claim` as a
/// source, then each that changes one.
fn resting_shown(
    conn: &rusqlite::Connection,
    claim: &store::ClaimView,
) -> Result<(serde_json::Map<String, Value>, String), Failure> {
    let resting: Vec<store::Resting> = store::resting_on(conn, &claim.name)?
        .into_iter()
        .filter(|resting| {
            claim
                .versions
                .iter()
                .any(|view| view.version == resting.meant)
        })
        .collect();
    let mut object = serde_json::Map::new();
    let mut words = String::new();
    for (edge, field, heading) in [
        ("source", "source_of", "source of"),
        ("changes", "changed_by", "changed by"),
    ] {
        let found: Vec<&store::Resting> = resting.iter().filter(|r| r.edge == edge).collect();
        if !found.is_empty() {
            words.push_str(&format!("\n{heading}:\n"));
        }
        for resting in &found {
            words.push_str(&format!(
                "  {}@{}, on {}@{}\n",
                resting.name, resting.version, claim.name, resting.meant
            ));
        }
        object.insert(
            field.into(),
            Value::Array(
                found
                    .iter()
                    .map(|resting| {
                        json!({
                            "claim": resting.name,
                            "version": resting.version,
                            "meant": resting.meant,
                        })
                    })
                    .collect(),
            ),
        );
    }
    Ok((object, words))
}

/// A claim as `boom show <name>` gives it: its kind, its accepted
/// version, the newest version in full, then the others as a tree by
/// base, then the claims that rest on it (claim-versions, "A claim shown
/// by its name"; claim-edges).
fn claim_shown(
    conn: &rusqlite::Connection,
    claim: &store::ClaimView,
) -> Result<(serde_json::Map<String, Value>, String), Failure> {
    // The newest version in full, then the others by base, newest first
    // (claim-versions, "A claim shown by its name").
    let newest = claim
        .versions
        .last()
        .ok_or_else(|| Failure::not_run(format!("the claim {} has no version", claim.name)))?;
    let newest_body = body_text(conn, newest.body.as_deref())?;
    let others: Vec<_> = claim.versions.iter().rev().skip(1).collect();
    let mut words = format!(
        "{} {}, {}\n\nnewest: {}\n{}\nbinds: {}\n",
        claim.kind,
        claim.name,
        claim
            .standing
            .as_deref()
            .map_or("no version is accepted".to_owned(), |standing| format!(
                "accepted: {standing}"
            )),
        version_words(newest),
        newest.statement,
        context::binds_words(&newest.repositories, &newest.contexts)
    );
    for glob in &newest.paths {
        words.push_str(&format!("path: {glob}\n"));
    }
    if !newest.replaces.is_empty() {
        words.push_str(&format!("replaces: {}\n", newest.replaces.join(", ")));
    }
    let narrative = claim.kind == Kind::Narrative.as_str();
    if let Some(view) = &newest.view {
        words.push_str(&format!("view: {view}\n"));
    }
    let (newest_edges, newest_edge_words) = edges_shown(conn, newest, &claim.kind)?;
    words.push_str(&newest_edge_words);
    let notes = if narrative {
        let state = store::load_state_for(conn, &Scope::names([claim.name.as_str()]))?;
        let (notes, note_words) = notes_shown(&state, &claim.name, None);
        words.push_str(&note_words);
        Some(notes)
    } else {
        None
    };
    if let Some(body) = &newest_body {
        words.push_str(&format!("\n{}", body));
        if !body.ends_with('\n') {
            words.push('\n');
        }
    }
    if !others.is_empty() {
        words.push_str("\nother versions:\n");
        words.push_str(&tree_words(&others));
    }
    let (resting, resting_words) = resting_shown(conn, claim)?;
    words.push_str(&resting_words);
    let bound = if claim.kind == Kind::Context.as_str() {
        let (bound, bound_words) = bound_shown(conn, claim)?;
        words.push_str(&bound_words);
        Some(bound)
    } else {
        None
    };
    // An outline's view as Boom computes it from the log, and the
    // narratives of the book that it does not hold (views, "boom show
    // gives an outline").
    let book = if claim.kind == Kind::Outline.as_str() {
        let state = store::view_state(conn)?;
        let book = crate::view::book(&state, Some(&claim.name));
        words.push('\n');
        words.push_str(&book.words());
        let unheld: Vec<&str> = book
            .unheld()
            .iter()
            .map(|entry| entry.narrative.as_str())
            .collect();
        Some((json!(book), json!(unheld)))
    } else {
        None
    };
    let mut newest_json = version_json(newest);
    newest_json.insert("statement".into(), json!(newest.statement));
    newest_json.insert("replaces".into(), json!(newest.replaces));
    newest_json.insert("body".into(), json!(newest_body));
    newest_json.extend(newest_edges);
    let versions: Vec<Value> = claim
        .versions
        .iter()
        .map(|view| Value::Object(version_json(view)))
        .collect();
    let mut object = serde_json::Map::new();
    object.insert("claim".into(), json!(claim.name));
    object.insert("kind".into(), json!(claim.kind));
    if let Some(repository) = &claim.repository {
        object.insert("repository".into(), json!(repository));
    }
    object.insert("accepted".into(), json!(claim.standing));
    let held = newest_json.get("narratives").cloned();
    object.insert("newest".into(), Value::Object(newest_json));
    object.insert("versions".into(), Value::Array(versions));
    object.extend(resting);
    if let Some(bound) = bound {
        object.insert("bound".into(), bound);
    }
    if let Some(notes) = notes {
        object.insert("notes".into(), notes);
    }
    if let Some((book, unheld)) = book {
        object.insert("view".into(), json!(newest.view));
        object.insert("narratives".into(), held.unwrap_or_default());
        object.insert("book".into(), book);
        object.insert("unheld".into(), unheld);
    }
    Ok((object, words))
}

/// The answers of a version of a narrative to the notes before it, as
/// `boom show <name>@<version>` gives them.
fn answers_shown(view: &store::VersionView) -> (Value, String) {
    let mut words = String::new();
    for answer in &view.answers {
        words.push_str(&format!(
            "answers: note {}: {}\n",
            answer.note, answer.words
        ));
    }
    (json!(view.answers), words)
}

/// The notes on the narrative `name`, or on its version `version` alone,
/// in the order written: each with its paragraph, its person, its words
/// and its elaboration, and `open` or the version that answers it with
/// its answer (narrative-notes).
fn notes_shown(state: &State, name: &str, version: Option<&str>) -> (Value, String) {
    let mut shown = Vec::new();
    let mut words = String::new();
    for note in state.notes_of(name) {
        if version.is_some_and(|version| version != note.version) {
            continue;
        }
        let (json, note_words) = note_shown(state, note);
        shown.push(json);
        words.push_str(&note_words);
    }
    (Value::Array(shown), words)
}

/// One note as `boom show` and `boom query --elaboration` give it: its
/// narrative, its version, its paragraph, its person, its words and its
/// elaboration, and `open` or the version that answers it with its
/// answer (narrative-notes).
fn note_shown(state: &State, note: &crate::judge::Note) -> (Value, String) {
    let name = &note.narrative;
    let answer = state.answer_of(note);
    let elaboration = state.elaboration_of(note);
    let mut words = format!(
        "note {} on paragraph {} of {name}@{}, by {}: {}\n",
        note.name, note.paragraph, note.version, note.person, note.words
    );
    if let Some(elaboration) = elaboration {
        words.push_str(&format!("  elaboration: {}\n", elaboration.words()));
    }
    match answer {
        Some((by, answer)) => words.push_str(&format!(
            "  answered by {name}@{}: {answer}\n",
            by.proposal.version
        )),
        None => words.push_str("  open\n"),
    }
    let json = json!({
        "note": note.name,
        "narrative": name,
        "version": note.version,
        "paragraph": note.paragraph,
        "fingerprint": note.fingerprint,
        "person": note.person,
        "words": note.words,
        "at": note.at,
        "entry": note.noted_in,
        "elaboration": elaboration.map(elaboration_json),
        "status": if answer.is_some() { "answered" } else { "open" },
        "answer": answer.map(|(by, answer)| json!({
            "version": by.proposal.version,
            "words": answer,
        })),
    });
    (json, words)
}

/// A note's elaboration as JSON: its `repository`, its `name` and its
/// `reference` (narrative-notes).
fn elaboration_json(origin: &Origin) -> Value {
    match origin {
        Origin::Elaboration { repository, name } => json!({
            "repository": repository,
            "name": name,
            "reference": origin.words(),
        }),
        other => origin_json(other),
    }
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

/// The text of the body whose fingerprint is `fingerprint`, or none.
fn body_text(
    conn: &rusqlite::Connection,
    fingerprint: Option<&str>,
) -> Result<Option<String>, Failure> {
    fingerprint
        .map(|fingerprint| {
            String::from_utf8(store::blob(conn, fingerprint)?)
                .map_err(|_| Failure::not_run(format!("the blob {fingerprint} is not text")))
        })
        .transpose()
}

/// The claim's name and the version that the entry numbered `number`
/// names (record-log, "Each line of the log names its version"): refused
/// where the log has no such entry, where the entry names no version, or
/// where it names several, which it lists.
fn entry_target(conn: &rusqlite::Connection, number: &str) -> Result<(String, String), Failure> {
    let seq: u64 = number
        .parse()
        .map_err(|_| Failure::refused(format!("the log has no entry {number}")))?;
    let row = store::row_at(conn, seq)?
        .ok_or_else(|| Failure::refused(format!("the log has no entry {number}")))?;
    let entry = Entry::parse(&row.signed)
        .map_err(|why| Failure::refused(format!("boom cannot read entry {number}: {why}")))?;
    match entry.act.targets().as_slice() {
        [] => Err(Failure::refused(format!(
            "entry {number} names no version: it is a {} entry",
            entry.act.name()
        ))),
        [(name, version)] => Ok(((*name).to_owned(), (*version).to_owned())),
        several => Err(Failure::refused(format!(
            "entry {number} accepts {} versions; show one of them by <name>@<version>: {}",
            several.len(),
            several
                .iter()
                .map(|(name, version)| format!("{name}@{version}"))
                .collect::<Vec<_>>()
                .join(", ")
        ))),
    }
}

fn log(path: &Path) -> Result<Answer, Failure> {
    let mut store = Store::open(path)?;
    let path = path.to_owned();
    store.read(|conn| {
        let mut entries = Vec::new();
        let mut words = String::new();
        for found in store::log_entries(conn, &path)? {
            let row = &found.row;
            let signed: Option<Value> = serde_json::from_str(&row.signed).ok();
            let field = |name: &str| {
                signed
                    .as_ref()
                    .and_then(|signed| signed.get(name))
                    .cloned()
                    .unwrap_or(Value::Null)
            };
            let parsed = Entry::parse(&row.signed);
            let targets: Vec<String> = parsed
                .as_ref()
                .map(|entry| {
                    entry
                        .act
                        .targets()
                        .iter()
                        .map(|(name, version)| format!("{name}@{version}"))
                        .collect()
                })
                .unwrap_or_default();
            let target = match targets.as_slice() {
                [one] => Some(one.clone()),
                _ => None,
            };
            entries.push(json!({
                "number": row.seq,
                "target": target,
                "targets": targets,
                "hash": row.hash,
                "prev": row.prev,
                "format": field("format"),
                "act": field("act"),
                "actor": field("actor"),
                "at": field("at"),
                "rules": parsed.as_ref().ok().map(|entry| entry.rules),
                "signed": row.signed,
                "signature": row.signature,
                "counted": found.counted,
                "reason": found.reason,
            }));
            let act = field("act");
            let actor = serde_json::from_value::<Actor>(field("actor"))
                .map_or("no one boom reads".to_owned(), |actor| actor_words(&actor));
            words.push_str(&format!(
                "{} {}{} by {}{}\n",
                row.seq,
                act.as_str().unwrap_or("(unreadable)"),
                if targets.is_empty() {
                    String::new()
                } else {
                    format!(" {}", targets.join(", "))
                },
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

fn verify(path: &Path) -> Result<Answer, Failure> {
    let verified = Store::open(path)?.verify()?;
    let rebuilt = if verified.rebuilt.is_empty() {
        "No view differed from the replay.".to_owned()
    } else {
        format!(
            "Built again, as they differed from the replay: {}.",
            verified.rebuilt.join(", ")
        )
    };
    Ok(Answer {
        json: json!({
            "format": "boom-verify/1",
            "head": {"number": verified.seq, "hash": verified.hash},
            "chain": "holds",
            "rebuilt": verified.rebuilt,
        }),
        words: format!(
            "The chain holds, to entry {} of the store's log.\n{rebuilt}\n",
            verified.seq
        ),
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
    object.insert("contexts".into(), json!(standing.contexts));
    object.insert(
        "accepted".into(),
        json!({"by": standing.decided_by, "entry": standing.decided_in}),
    );
    object
}

/// The repository of the session (repository-rules): the one `named`
/// names, else the one `BOOM_REPOSITORY` names, else the registered one
/// whose remote matches a remote of the working folder's repository.
fn session_repository(
    path: &Path,
    named: Option<&str>,
    env: &Env,
) -> Result<Option<String>, Failure> {
    if named.is_some() {
        return repository::find(named, None, None, &[]);
    }
    let mut store = Store::open(path)?;
    let registered = store.read(store::repositories)?;
    repository::find(
        None,
        env.boom_repository.as_deref(),
        env.cwd.as_deref(),
        &registered,
    )
}

/// `boom repo add`: an owner registers a repository and its remote
/// (repository-settings).
fn repo_add(
    path: &Path,
    name: &str,
    remote: &str,
    session: bool,
    key: Option<&Path>,
    keys: &dyn Keys,
    env: &Env,
) -> Result<Answer, Failure> {
    repository::check_name(name).map_err(Failure::Refused)?;
    let remote = remote_form(remote)?;
    let mut store = Store::open(path)?;
    let registered = store.read(store::repositories)?;
    if registered.iter().any(|found| found.name == name) {
        return Err(Failure::refused(format!(
            "{name} is registered already: change it with boom repo set"
        )));
    }
    repo_write(&mut store, path, name, &remote, session, key, keys, env)
}

/// `boom repo set`: an owner changes a registered repository's remote or
/// its delivery at session start.
fn repo_set(
    path: &Path,
    name: &str,
    remote: Option<&str>,
    session: Option<bool>,
    key: Option<&Path>,
    keys: &dyn Keys,
    env: &Env,
) -> Result<Answer, Failure> {
    if remote.is_none() && session.is_none() {
        return Err(Failure::refused(
            "boom repo set changes --remote, --session, or both",
        ));
    }
    let mut store = Store::open(path)?;
    let registered = store.read(store::repositories)?;
    let Some(found) = registered.into_iter().find(|found| found.name == name) else {
        return Err(Failure::refused(format!(
            "this store registers no repository {name}: register it with boom repo add"
        )));
    };
    let remote = match remote {
        Some(remote) => remote_form(remote)?,
        None => found.remote,
    };
    let session = session.unwrap_or(found.session);
    repo_write(&mut store, path, name, &remote, session, key, keys, env)
}

/// A remote in its one form, or a refusal that names it.
fn remote_form(remote: &str) -> Result<String, Failure> {
    repository::remote_form(remote).ok_or_else(|| {
        Failure::refused(format!(
            "a remote is an https://, ssh:// or git@<host>: address of a repository: {remote} is not"
        ))
    })
}

/// Writes a `repository` entry an owner signs, and answers as
/// `boom repo show` for it.
#[allow(clippy::too_many_arguments)]
fn repo_write(
    store: &mut Store,
    path: &Path,
    name: &str,
    remote: &str,
    session: bool,
    key: Option<&Path>,
    keys: &dyn Keys,
    env: &Env,
) -> Result<Answer, Failure> {
    let Some(key) = key else {
        return Err(Failure::refused(
            "only an owner sets a repository: boom repo takes the owner's --key",
        ));
    };
    let signer = keys.signer(key)?;
    write(
        store,
        &Scope::default(),
        Some(signer.as_ref()),
        &[],
        |state| {
            Ok(Entry {
                at: env.clock.now(),
                rules: crate::judge::RULES,
                actor: Actor::Person(person_of(state, signer.as_ref(), key)?),
                act: Act::Repository {
                    name: name.to_owned(),
                    remote: remote.to_owned(),
                    session,
                },
            })
        },
    )?;
    repo_show(path, Some(name))
}

/// `boom repo show`: the repositories the store registers, or one, with
/// their settings and the entry and owner that set them.
fn repo_show(path: &Path, name: Option<&str>) -> Result<Answer, Failure> {
    let mut store = Store::open(path)?;
    let mut registered = store.read(store::repositories)?;
    if let Some(name) = name {
        registered.retain(|found| found.name == name);
        if registered.is_empty() {
            return Err(Failure::refused(format!(
                "this store registers no repository {name}"
            )));
        }
    }
    let mut words = String::new();
    let mut listed = Vec::with_capacity(registered.len());
    for found in &registered {
        let session = if found.session { "on" } else { "off" };
        words.push_str(&format!(
            "{}  {}\n  at session start: {session}\n  set by {} in entry {}\n",
            found.name, found.remote, found.set_by, found.set_in
        ));
        listed.push(json!({
            "name": found.name,
            "remote": found.remote,
            "session": found.session,
            "set_by": found.set_by,
            "set_in": found.set_in,
        }));
    }
    if registered.is_empty() {
        words.push_str("This store registers no repository: register one with boom repo add.\n");
    }
    Ok(Answer {
        json: json!({"format": "boom-repo/1", "repositories": listed}),
        words,
    })
}

/// The claims that stand for the repository `named`, or else the one the
/// working folder declares (repository-rules; design 5): taken from the
/// views, once they are at the log's last entry (record 0008, point 2, as
/// amended on 2026-11-20), and appending nothing. With `if_declared`,
/// where none is named or declared, nothing: no store opened, no words,
/// and `boom-query/1` with no repository (design 10). With `paths`, the
/// claims for those files of one repository, and the contexts each falls
/// in (context-claims, "The claims for a path").
fn query(
    path: Result<PathBuf, Failure>,
    named: Option<&str>,
    if_declared: bool,
    paths: &[String],
    env: &Env,
) -> Result<Answer, Failure> {
    let files = paths
        .iter()
        .map(|given| context::path(given).map_err(Failure::Refused))
        .collect::<Result<Vec<_>, _>>()?;
    if let Some((first, _)) = files.first() {
        if named.is_some() {
            return Err(Failure::refused(
                "a path names its repository: boom query takes --path or --repository, and not both",
            ));
        }
        if if_declared {
            return Err(Failure::refused(
                "boom query takes --path or --if-declared: the two flags cannot be given together",
            ));
        }
        if let Some((other, _)) = files.iter().find(|(name, _)| name != first) {
            return Err(Failure::refused(format!(
                "the paths of one query name one repository: they name {first} and {other}"
            )));
        }
    }
    let nothing = || Answer {
        json: json!({
            "format": "boom-query/2",
            "repository": null,
            "head": null,
            "claims": [],
        }),
        words: String::new(),
    };
    let path = path?;
    if let Some(launched) = env.boom_repository.as_deref() {
        // A launcher's name boom cannot read is refused before any store
        // is opened, so the person sees what to fix.
        repository::check_name(launched).map_err(|why| {
            Failure::refused(format!(
                "{} names {launched:?}: {why}",
                repository::LAUNCHER
            ))
        })?;
    }
    if if_declared && !path.exists() {
        // No store where it would open one: no store made, nothing said.
        return Ok(nothing());
    }
    let mut store = Store::open(&path)?;
    // One read of the views: the repositories, then the claims.
    store.read(|conn| {
        let registered = store::repositories(conn)?;
        let found = match files.first() {
            Some((name, _)) => Some((*name).to_owned()),
            None => repository::find(
                named,
                env.boom_repository.as_deref(),
                env.cwd.as_deref(),
                &registered,
            )?,
        };
        let Some(repository) = found else {
            if if_declared {
                return Ok(nothing());
            }
            return Err(Failure::refused(
                "boom query answers for a repository: name it with --repository, or register its remote with boom repo add",
            ));
        };
        let delivered = registered
            .iter()
            .find(|registered| registered.name == repository)
            .is_none_or(|registered| registered.session);
        if if_declared && !delivered {
            return Ok(nothing());
        }
        // The contexts each path falls in, and the claims of each.
        let mut fallen = Vec::with_capacity(files.len());
        let mut lines = String::new();
        let mut names: Vec<String> = Vec::new();
        for (given, (_, file)) in paths.iter().zip(&files) {
            let contexts = context::falls_in(conn, &repository, file)?;
            let words = if contexts.is_empty() {
                "no context".to_owned()
            } else {
                contexts.join(", ")
            };
            lines.push_str(&format!("path {given}: {words}\n"));
            names.extend(context::names(&contexts).into_iter().map(str::to_owned));
            fallen.push(json!({"path": given, "contexts": contexts}));
        }
        let accepted = store::accepted_for(conn, Some(&repository), &names)?;
        let (number, hash) = store::head_entry(conn)?;
        let mut claims = Vec::new();
        for (claim, standing) in &accepted {
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
        let count = match claims.len() {
            1 => "1 claim".to_owned(),
            n => format!("{n} claims"),
        };
        let mut json = json!({
            "format": "boom-query/2",
            "repository": repository,
            "head": {"number": number, "hash": hash},
            "claims": claims,
        });
        if !files.is_empty() {
            json["paths"] = Value::Array(fallen);
        }
        Ok(Answer {
            json,
            words: format!("Boom: {count} for {repository}, at entry {number}\n{lines}"),
        })
    })
}

/// The open notes on the versions that came from the elaboration
/// `given`, `<repository>/<name>`, in the order written, each with its
/// narrative and its elaboration (narrative-notes): read from the views
/// once they are at the log's last entry, appending nothing.
fn elaboration_notes(
    path: Result<PathBuf, Failure>,
    given: &str,
    repository: bool,
    if_declared: bool,
    paths: &[String],
) -> Result<Answer, Failure> {
    for (named, flag) in [
        (repository, "--repository"),
        (!paths.is_empty(), "--path"),
        (if_declared, "--if-declared"),
    ] {
        if named {
            return Err(Failure::refused(format!(
                "boom query takes --elaboration or {flag}, and not both: an elaboration's notes bind no repository"
            )));
        }
    }
    let elaboration = Origin::elaboration_named(given).map_err(Failure::Refused)?;
    let mut store = Store::open(&path?)?;
    store.read(|conn| {
        let state = store::view_state(conn)?;
        let (number, hash) = store::head_entry(conn)?;
        let mut notes: Vec<&crate::judge::Note> = state
            .notes
            .values()
            .filter(|note| state.elaboration_of(note) == Some(&elaboration))
            .filter(|note| state.answer_of(note).is_none())
            .collect();
        notes.sort_by_key(|note| note.noted_in);
        let mut shown = Vec::with_capacity(notes.len());
        let mut lines = String::new();
        for note in &notes {
            let (json, words) = note_shown(&state, note);
            shown.push(json);
            lines.push_str(&words);
        }
        let count = match notes.len() {
            1 => "1 open note".to_owned(),
            n => format!("{n} open notes"),
        };
        Ok(Answer {
            json: json!({
                "format": "boom-query/2",
                "repository": null,
                "elaboration": elaboration_json(&elaboration),
                "head": {"number": number, "hash": hash},
                "claims": [],
                "notes": shown,
            }),
            words: format!(
                "Boom: {count} of {}, at entry {number}\n{lines}",
                elaboration.words()
            ),
        })
    })
}

/// The claims that stand whose words hold every word of `given`, for
/// the repository `named`, or else the one the working folder declares,
/// or with `all` every claim (claim-search; designs 1 to 6): taken from
/// the views, once they are at the log's last entry, searched in memory,
/// keeping no index and appending nothing.
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
        return Err(Failure::refused(crate::search::NO_WORD));
    }
    let mut store = Store::open(path)?;
    // One read of the views: the repository of the session, then the
    // claims.
    store.read(|conn| {
        let repository = if all {
            None
        } else {
            let registered = store::repositories(conn)?;
            Some(
                repository::find(
                    named,
                    env.boom_repository.as_deref(),
                    env.cwd.as_deref(),
                    &registered,
                )?
                .ok_or_else(|| {
                    Failure::refused(
                        "boom search searches a repository: name it with --repository, register its remote with boom repo add, or search every repository with --all",
                    )
                })?,
            )
        };
        // A search of a repository finds the claims of its contexts too
        // (context-claims).
        let contexts = match repository.as_deref() {
            Some(repository) => store::contexts_in(conn, repository)?,
            None => Vec::new(),
        };
        let found =
            crate::search::standing(conn, repository.as_deref(), &contexts, &words)?;
        let (number, hash) = store::head_entry(conn)?;
        let total = found.len();
        let shown = usize::try_from(limit).unwrap_or(usize::MAX);
        let mut claims = Vec::new();
        let mut lines = String::new();
        for found in found.iter().take(shown) {
            let (claim, version) = (&found.claim, &found.version);
            let proposal = &version.proposal;
            let legacy = found.legacy.as_deref();
            let mut object = standing_json(claim, version);
            object.insert("excerpt".into(), json!(found.excerpt));
            object.insert("legacy".into(), json!(legacy));
            claims.push(Value::Object(object));
            let binds = context::binds_words(&version.repositories, &version.contexts);
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
                "format": "boom-search/2",
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
    let scope = Scope {
        names: read
            .records
            .iter()
            .filter_map(|record| record.name.clone())
            .collect(),
        imported: true,
        notes: std::collections::BTreeSet::new(),
        outlines: false,
    };
    let mut judged = store.state_for(&scope)?;
    let mut entries = Vec::new();
    for record in &read.records {
        let entry = Entry {
            at: env.clock.now(),
            rules: crate::judge::RULES,
            actor: Actor::Agent {
                provenance: provenance.clone(),
            },
            act: Act::Proposed {
                claim: Box::new(record.proposal()),
                imported: Some(Box::new(record.imported.clone())),
                // Imported for one repository, it stands for that one
                // alone (design 7).
                attach: Some(Attach {
                    repositories: vec![repository.clone()],
                    contexts: Vec::new(),
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
    let blobs: Vec<Vec<&[u8]>> = entries
        .iter()
        .map(|(_, body)| body.map(str::as_bytes).into_iter().collect())
        .collect();
    let appends: Vec<Append<'_>> = entries
        .iter()
        .zip(&blobs)
        .map(|((signed, _), blobs)| Append {
            signed,
            signature: None,
            blobs,
        })
        .collect();
    let state = if appends.is_empty() {
        store.state_for(&scope)?
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
                .holding(&proposal.name, &proposal.version)
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
            "format": "boom-import/2",
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

/// The arguments of `boom terms check`.
struct TermsCheck {
    files: Vec<PathBuf>,
    named: Option<String>,
    if_declared: bool,
    hook: bool,
}

/// What `boom terms check` checks: each file, or each text that a hook's
/// event writes, with its name.
struct Checked {
    file: PathBuf,
    texts: Vec<(Option<String>, String)>,
}

/// `boom terms check` (term-check): each old word of the accepted terms
/// of the repository of each file, from the store, which it reads and
/// never writes. An old word found is a refusal, exit 2, with each one in
/// `refused.found`. With `--hook`, the text of one `PreToolUse` event,
/// and no word where none is found.
fn terms_check(
    json: bool,
    path: Result<PathBuf, Failure>,
    check: &TermsCheck,
    env: &Env,
    input: &mut dyn std::io::Read,
) -> Outcome {
    let silent = Outcome {
        bytes: None,
        code: 0,
        stdout: String::new(),
        stderr: String::new(),
    };
    let checked = match terms_input(check, input) {
        Ok(Some(checked)) => checked,
        Ok(None) => return silent,
        Err(failure) => return failed(json, failure),
    };
    match terms_found(path, check, &checked, env) {
        Ok(None) if check.hook => silent,
        Ok(None) => {
            let answer = json!({
                "format": "boom-terms-check/1",
                "head": null,
                "files": [],
                "found": [],
            });
            Outcome {
                bytes: None,
                code: 0,
                stdout: if json {
                    format!("{answer}\n")
                } else {
                    String::new()
                },
                stderr: String::new(),
            }
        }
        Ok(Some(report)) if report.found.is_empty() => {
            if check.hook {
                return silent;
            }
            let files = report.files.len();
            let words = format!(
                "No old word in {} file{}, at entry {} of the store's log.\n",
                files,
                if files == 1 { "" } else { "s" },
                report.head.0
            );
            let answer = json!({
                "format": "boom-terms-check/1",
                "head": {"number": report.head.0, "hash": report.head.1},
                "files": report.files,
                "found": [],
            });
            if json {
                Outcome {
                    bytes: None,
                    code: 0,
                    stdout: format!("{answer}\n"),
                    stderr: words,
                }
            } else {
                Outcome {
                    bytes: None,
                    code: 0,
                    stdout: words,
                    stderr: String::new(),
                }
            }
        }
        Ok(Some(report)) => {
            let count = report.found.len();
            let reason = format!(
                "{count} old word{} of an accepted term: write the term in place of each",
                if count == 1 { "" } else { "s" }
            );
            let object = json!({
                "format": "boom-refused/1",
                "refused": {
                    "reason": reason,
                    "head": {"number": report.head.0, "hash": report.head.1},
                    "found": report.found,
                },
            });
            Outcome {
                bytes: None,
                code: 2,
                stdout: if json {
                    format!("{object}\n")
                } else {
                    String::new()
                },
                stderr: format!("boom: refused: {reason}\n{}", report.words),
            }
        }
        Err(failure) => failed(json, failure),
    }
}

/// The files a check names, or the text of the hook's event: none for an
/// event of a tool that writes no file's text.
fn terms_input(
    check: &TermsCheck,
    input: &mut dyn std::io::Read,
) -> Result<Option<Vec<Checked>>, Failure> {
    if !check.hook {
        return Ok(Some(
            check
                .files
                .iter()
                .map(|file| Checked {
                    file: file.clone(),
                    texts: Vec::new(),
                })
                .collect(),
        ));
    }
    let mut event = String::new();
    input
        .read_to_string(&mut event)
        .map_err(|error| Failure::refused(format!("the hook's event cannot be read: {error}")))?;
    let event: Value = serde_json::from_str(&event)
        .map_err(|error| Failure::refused(format!("the hook's event is not JSON: {error}")))?;
    Ok(crate::terms::written(&event)?.map(|(file, texts)| {
        vec![Checked {
            file,
            texts: texts
                .into_iter()
                .map(|(name, text)| (Some(name), text))
                .collect(),
        }]
    }))
}

/// What a check found, with the entry of the log it read at.
struct TermsReport {
    head: (u64, String),
    files: Vec<Value>,
    found: Vec<Value>,
    words: String,
}

/// Checks `checked`: none where `--if-declared` finds no store; else
/// each file of a repository with what it holds.
fn terms_found(
    path: Result<PathBuf, Failure>,
    check: &TermsCheck,
    checked: &[Checked],
    env: &Env,
) -> Result<Option<TermsReport>, Failure> {
    let path = path?;
    if let Some(launched) = env.boom_repository.as_deref() {
        repository::check_name(launched).map_err(|why| {
            Failure::refused(format!(
                "{} names {launched:?}: {why}",
                repository::LAUNCHER
            ))
        })?;
    }
    if let Some(named) = &check.named {
        repository::check_name(named).map_err(Failure::Refused)?;
    }
    if check.if_declared && !path.exists() {
        return Ok(None);
    }
    let read = store::read_terms(&path)?;
    let mut report = TermsReport {
        head: read.head.clone(),
        files: Vec::new(),
        found: Vec::new(),
        words: String::new(),
    };
    for one in checked {
        let shown = one.file.display().to_string();
        let found = crate::terms::repository_of(
            &one.file,
            check.named.as_deref(),
            env.boom_repository.as_deref(),
            &read.repositories,
        )?;
        let Some(repository) = found else {
            if check.if_declared {
                continue;
            }
            return Err(Failure::refused(format!(
                "boom finds no repository of {shown}: name it with --repository, or register its remote with boom repo add"
            )));
        };
        let terms = crate::terms::accepted(&read.state, &repository);
        let texts = if one.texts.is_empty() && !check.hook {
            let bytes = std::fs::read(&one.file)
                .map_err(|error| Failure::refused(format!("{shown} cannot be read: {error}")))?;
            match String::from_utf8(bytes) {
                Ok(text) => vec![(None, text)],
                Err(_) => {
                    // A file that is not text holds no word.
                    report.files.push(json!({
                        "file": shown,
                        "repository": repository,
                        "text": false,
                    }));
                    continue;
                }
            }
        } else {
            one.texts.clone()
        };
        report.files.push(json!({
            "file": shown,
            "repository": repository,
            "text": true,
        }));
        for (name, text) in &texts {
            for found in crate::terms::find(text, &terms)? {
                report.found.push(crate::terms::found_json(
                    &shown,
                    &found,
                    &terms,
                    name.as_deref(),
                ));
                report.words.push_str(&crate::terms::found_words(
                    &shown,
                    &found,
                    &terms,
                    name.as_deref(),
                ));
            }
        }
    }
    Ok(Some(report))
}
