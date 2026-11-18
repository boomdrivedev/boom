//! `boom`, the claims ledger: a thin command line over [`boom::cli::run`].

use std::io::Write;
use std::process::ExitCode;

fn main() -> ExitCode {
    let outcome = boom::cli::run(
        std::env::args_os().skip(1),
        &boom::sign::SshKeygen::default(),
        &boom::cli::Env::from_process(),
    );
    // A closed pipe is the reader's choice; the exit code still says what happened.
    let _ = std::io::stdout().write_all(outcome.stdout.as_bytes());
    let _ = std::io::stderr().write_all(outcome.stderr.as_bytes());
    ExitCode::from(outcome.code)
}
