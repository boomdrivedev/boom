## MODIFIED Requirements

### Requirement: The commands

Boom SHALL offer `init`, `register`, `propose`, `accept`, `reject`, `approve`, `record change`, `append`, `show`, `log`, `lineage`, `verify`, `import adr`, `import rules`, `repo add`, `repo set`, `repo show`, `terms check`, `note`, `withdraw`, `page`, `hook session-start` and `hook pre-tool-use`. Each command SHALL take `--store <path>`, and each but `hook` and `page` SHALL take `--json`. Boom SHALL keep `boom --version`. Each command SHALL be a thin command line over the library's engine.

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

#### Scenario: A hook answered in the JSON of Claude Code

- **WHEN** `boom hook session-start --store <path>` runs with a `SessionStart` event on standard input, in a worktree of a registered repository with an accepted claim
- **THEN** standard output is one JSON object with `systemMessage` and `hookSpecificOutput`, and boom exits 0

#### Scenario: A note answered as JSON

- **WHEN** `boom note <name>@<version> --paragraph 1 "<words>" --store <path> --json` runs on a store that registers one person
- **THEN** standard output is one JSON object whose `format` is `boom-note/1`, with the name of the note, and boom exits 0

#### Scenario: A walk answered as JSON

- **WHEN** `boom lineage <act hash> --store <path> --json` runs on a store whose log holds that act hash
- **THEN** standard output is one JSON object whose `format` is `boom-lineage/1`, and boom exits 0

#### Scenario: A change record answered as JSON

- **WHEN** `boom show boom:contexts-own-paths --store <path> --json` runs on a store that holds that change record
- **THEN** standard output is one JSON object whose `format` is `boom-change/1`, and boom exits 0

#### Scenario: A withdrawal answered as JSON

- **WHEN** `boom withdraw <name>@<version> --reason "a duplicate" --provenance run-2 --store <path> --json` runs for a version that an agent proposed
- **THEN** standard output is one JSON object whose `format` is `boom-withdraw/1`, with the version under `withdrawn`, and boom exits 0
