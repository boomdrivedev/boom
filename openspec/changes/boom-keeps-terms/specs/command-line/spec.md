## MODIFIED Requirements

### Requirement: The commands

Boom SHALL offer `init`, `register`, `propose`, `accept`, `reject`, `show`, `log`, `verify`, `import adr`, `import rules`, `repo add`, `repo set`, `repo show` and `terms check`. Each command SHALL take `--store <path>` and `--json`. Boom SHALL keep `boom --version`. Each command SHALL be a thin command line that calls the engine of the library. Thus a caller gets from the command what the engine does.

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

#### Scenario: A check answered as JSON

- **WHEN** `boom terms check <file> --store <path> --repository shop --json` runs on a file that holds no old word
- **THEN** standard output is one JSON object whose `format` is `boom-terms-check/1`, and boom exits 0

### Requirement: Exit codes

Each command SHALL exit 0 when it did what the caller asked. It SHALL exit 2 when it refuses. Examples are the accept of an agent, a name that a claim has, an amendment whose base is no longer accepted, an unknown name, and arguments that boom cannot read. It SHALL exit 3 when it cannot run. Examples are no store, an altered store, a blob that boom cannot read, and a signature that the program of the key did not make. On exit 2 and exit 3, boom SHALL write nothing.

#### Scenario: A refusal writes nothing

- **WHEN** boom refuses the accept of an agent
- **THEN** boom exits 2, and the log of the store does not change

#### Scenario: Could not run

- **WHEN** `boom accept` runs with a key whose signing program cannot run or exits with a code other than 0
- **THEN** boom exits 3 and says that the program of the key did not sign, and the log of the store does not change

#### Scenario: Arguments it cannot read

- **WHEN** `boom propose` runs with a kind that is not `invariant`, `decision`, `assumption` or `term`
- **THEN** boom exits 2 and names the kinds that it takes
