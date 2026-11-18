//! The command line: its arguments, its answers as words or as one JSON
//! object, and its exit codes.

use std::ffi::OsString;
use std::path::PathBuf;

use clap::Parser;

use crate::sign::Keys;

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
struct Cli {}

/// Runs boom with `args`, the arguments after the program's name.
pub fn run<I, A>(args: I, _keys: &dyn Keys, _env: &Env) -> Outcome
where
    I: IntoIterator<Item = A>,
    A: Into<OsString>,
{
    let argv = std::iter::once(OsString::from("boom")).chain(args.into_iter().map(Into::into));
    match Cli::try_parse_from(argv) {
        Ok(Cli {}) => Outcome {
            code: 0,
            stdout: String::new(),
            stderr: String::new(),
        },
        Err(error) => {
            use clap::error::ErrorKind;
            let text = error.render().to_string();
            match error.kind() {
                ErrorKind::DisplayHelp | ErrorKind::DisplayVersion => Outcome {
                    code: 0,
                    stdout: text,
                    stderr: String::new(),
                },
                _ => Outcome {
                    code: 2,
                    stdout: String::new(),
                    stderr: text,
                },
            }
        }
    }
}
