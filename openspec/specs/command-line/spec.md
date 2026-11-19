# command-line Specification

## Purpose
How every boom command is called and answers: its arguments, its one
JSON object, its exit codes, no prompt, and a repeated write that adds
nothing, so an agent or another tool can call boom and read its answer
(interview: `one-engine-every-command-json`, `exit-codes`,
`json-versioned-additive`, `no-prompts-repeats-add-nothing`).

## Requirements

### Requirement: The commands

boom SHALL offer `init`, `register`, `propose`, `accept`, `reject`, `show`, `log`, `import adr` and `import rules`, each taking `--store <path>` and `--json`, and SHALL keep `boom --version`. Each command SHALL be the library's engine called by a thin command line, so a caller gets from the command what the engine does.

#### Scenario: The version

- **WHEN** `boom --version` runs
- **THEN** it prints `boom` and the crate's version, and exits 0

#### Scenario: The log of a new store

- **WHEN** `boom init --person ada --key <ada's key> --store <path>` runs, then `boom log --store <path>`
- **THEN** the second prints one line for the first owner's registration and exits 0

#### Scenario: An import answered as JSON

- **WHEN** `boom import adr <dir> --repository shop --dry-run --json` runs
- **THEN** standard output is one JSON object whose `format` is `boom-import/1`, and it exits 0

### Requirement: Exit codes

Every command SHALL exit 0 when it did what was asked; 2 when it refuses, as for an agent's accept, a name taken, an amendment on a base no longer standing, an unknown name or arguments it cannot read; and 3 when it could not run, as for no store, an altered store, an unreadable blob or a signature the key's program did not make. On 2 and 3 nothing SHALL be written.

#### Scenario: A refusal writes nothing

- **WHEN** an agent's accept is refused
- **THEN** boom exits 2 and the store's log is as it was

#### Scenario: Could not run

- **WHEN** `boom accept` runs with a key whose signing program cannot be run or exits non-zero
- **THEN** boom exits 3 saying the key's program did not sign, and the store's log is as it was

#### Scenario: Arguments it cannot read

- **WHEN** `boom propose` runs with a kind that is none of `invariant`, `decision` and `assumption`
- **THEN** it exits 2 naming the kinds it takes

### Requirement: One JSON object with its format

With `--json`, a command SHALL print exactly one JSON object on standard output, naming its format and major version as `"format": "boom-<command>/1"`, and every word for a person on standard error. A refusal SHALL be `{"format": "boom-refused/1", "refused": {"reason": ...}}` and a command that could not run `{"format": "boom-not-run/1", "not_run": {"reason": ...}}`.

#### Scenario: A proposal answered as JSON

- **WHEN** `boom propose ... --json` runs
- **THEN** standard output is one JSON object whose `format` is `boom-propose/1`, naming the claim, its version and its status

#### Scenario: A refusal answered as JSON

- **WHEN** `boom accept <name>@<version> --json` runs with no `--key`
- **THEN** standard output is one JSON object whose `format` is `boom-refused/1` with `refused.reason`, and it exits 2

#### Scenario: Not run, answered as JSON

- **WHEN** `boom show <name> --json` runs on a store whose chain fails
- **THEN** standard output is one JSON object whose `format` is `boom-not-run/1` with `not_run.reason` naming the altered entry, and it exits 3

### Requirement: No prompts

Every command SHALL run from its arguments, its environment and the store, and SHALL never read an answer from its terminal. A security key's touch is the key's prompt, not boom's.

#### Scenario: No terminal

- **WHEN** `boom propose` runs with its standard input closed
- **THEN** it completes and exits 0

### Requirement: A repeated write adds nothing

A write run again with the same arguments SHALL add nothing to the log and SHALL answer as the first did: the same version proposed, the same version accepted or rejected again by the same person, the same person registered with the same key.

#### Scenario: An accept repeated

- **WHEN** ada accepts a version and then runs the same accept again
- **THEN** the second exits 0 giving the version as standing, and the log holds one `accepted` entry for it

#### Scenario: A registration repeated

- **WHEN** an owner registers `bo` with bo's key twice
- **THEN** the second exits 0, and the log holds one `registered` entry for `bo`
