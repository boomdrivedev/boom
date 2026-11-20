## MODIFIED Requirements

### Requirement: The commands

boom SHALL offer `init`, `register`, `propose`, `accept`, `reject`, `show`, `log`, `verify`, `import adr` and `import rules`, each taking `--store <path>` and `--json`, and SHALL keep `boom --version`. Each command SHALL be the library's engine called by a thin command line, so a caller gets from the command what the engine does.

#### Scenario: The version

- **WHEN** `boom --version` runs
- **THEN** it prints `boom` and the crate's version, and exits 0

#### Scenario: The log of a new store

- **WHEN** `boom init --person ada --key <ada's key> --store <path>` runs, then `boom log --store <path>`
- **THEN** the second prints one line for the first owner's registration and exits 0

#### Scenario: An import answered as JSON

- **WHEN** `boom import adr <dir> --repository shop --dry-run --json` runs
- **THEN** standard output is one JSON object whose `format` is `boom-import/1`, and it exits 0

#### Scenario: Verify answered as JSON

- **WHEN** `boom verify --store <path> --json` runs on a new store
- **THEN** standard output is one JSON object whose `format` is `boom-verify/1`, naming the log's last entry and no view built again, and it exits 0
