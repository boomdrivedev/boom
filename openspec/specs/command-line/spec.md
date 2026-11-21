# command-line Specification

## Purpose
How every boom command is called and answers: its arguments, its one
JSON object, its exit codes, no prompt, and a repeated write that adds
nothing, so an agent or another tool can call boom and read its answer
(interview: `one-engine-every-command-json`, `exit-codes`,
`json-versioned-additive`, `no-prompts-repeats-add-nothing`).

## Requirements

### Requirement: The commands

Boom SHALL offer `init`, `register`, `propose`, `accept`, `reject`, `show`, `log`, `verify`, `import adr`, `import rules`, `repo add`, `repo set` and `repo show`. Each command SHALL take `--store <path>` and `--json`. Boom SHALL keep `boom --version`. Each command SHALL be a thin command line that calls the engine of the library. Thus a caller gets from the command what the engine does.

#### Scenario: The version

- **WHEN** `boom --version` runs
- **THEN** boom prints `boom` and the version of the crate, and exits 0

#### Scenario: The log of a new store

- **WHEN** `boom init --person ada --key <ada's key> --store <path>` runs, and then `boom log --store <path>` runs
- **THEN** the second command prints one line for the registration of the first owner, and exits 0

#### Scenario: An import answered as JSON

- **WHEN** `boom import adr <dir> --repository shop --dry-run --json` runs
- **THEN** standard output is one JSON object whose `format` is `boom-import/2`, and boom exits 0

#### Scenario: Verify answered as JSON

- **WHEN** `boom verify --store <path> --json` runs on a new store
- **THEN** standard output is one JSON object whose `format` is `boom-verify/1`. It names the last entry of the log and no view built again. Boom exits 0

#### Scenario: The repositories answered as JSON

- **WHEN** `boom repo show --store <path> --json` runs on a store that registers one repository
- **THEN** standard output is one JSON object whose `format` is `boom-repo/1`, and boom exits 0

### Requirement: Exit codes

Each command SHALL exit 0 when it did what the caller asked. It SHALL exit 2 when it refuses. Examples are the accept of an agent, a name that a claim has, an amendment whose base is no longer accepted, an unknown name, and arguments that boom cannot read. It SHALL exit 3 when it cannot run. Examples are no store, an altered store, a blob that boom cannot read, and a signature that the program of the key did not make. On exit 2 and exit 3, boom SHALL write nothing.

#### Scenario: A refusal writes nothing

- **WHEN** boom refuses the accept of an agent
- **THEN** boom exits 2, and the log of the store does not change

#### Scenario: Could not run

- **WHEN** `boom accept` runs with a key whose signing program cannot run or exits with a code other than 0
- **THEN** boom exits 3 and says that the program of the key did not sign, and the log of the store does not change

#### Scenario: Arguments it cannot read

- **WHEN** `boom propose` runs with a kind that is not `invariant`, `decision` or `assumption`
- **THEN** boom exits 2 and names the kinds that it takes

### Requirement: One JSON object with its format

With `--json`, a command SHALL print one JSON object on standard output, and each word for a person on standard error. The object SHALL name its format, `"format": "boom-<command>/<major>"`. The major version SHALL be 2 for `propose`, `accept`, `reject`, `show`, `query`, `search` and `import`, and 1 for the other commands. A refusal SHALL be `boom-refused/1` with `refused.reason`. A command that cannot run SHALL give `boom-not-run/1` with `not_run.reason`.

#### Scenario: A proposal answered as JSON

- **WHEN** `boom propose ... --json` runs
- **THEN** standard output is one JSON object whose `format` is `boom-propose/2`, and it names the claim, its version and its status

#### Scenario: A refusal answered as JSON

- **WHEN** `boom accept <name>@<version> --json` runs with no `--key`
- **THEN** standard output is one JSON object whose `format` is `boom-refused/1`, with `refused.reason`, and boom exits 2

#### Scenario: Not run, answered as JSON

- **WHEN** `boom show <name> --json` runs on a store whose chain fails at the head of its views
- **THEN** standard output is one JSON object whose `format` is `boom-not-run/1`, with `not_run.reason` that names the altered entry, and boom exits 3

### Requirement: No prompts

Every command SHALL run from its arguments, its environment and the store, and SHALL never read an answer from its terminal. A security key's touch is the key's prompt, not boom's.

#### Scenario: No terminal

- **WHEN** `boom propose` runs with its standard input closed
- **THEN** it completes and exits 0

### Requirement: A repeated write adds nothing

When a write runs again with the same arguments, boom SHALL add nothing to the log. Boom SHALL give the same answer as the first time. This applies to the same version proposed, the same version accepted or rejected again by the same person, and the same person registered with the same key.

#### Scenario: An accept repeated

- **WHEN** ada accepts a version, and then runs the same accept again
- **THEN** the second run exits 0 and gives the version as `accepted`, and the log holds one `accepted` entry for it

#### Scenario: A registration repeated

- **WHEN** an owner registers `bo` with the key of bo two times
- **THEN** the second run exits 0, and the log holds one `registered` entry for `bo`
