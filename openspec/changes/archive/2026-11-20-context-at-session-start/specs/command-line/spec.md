## MODIFIED Requirements

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
