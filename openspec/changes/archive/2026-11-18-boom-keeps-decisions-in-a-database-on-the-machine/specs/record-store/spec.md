# Spec Delta

## Purpose

Where boom keeps its records on a machine: one database file holding the
log, the long text by fingerprint and the views of the current records,
the views always computed from the log (record 0005; core.1, core.2).

## ADDED Requirements

### Requirement: One store file on the machine

boom SHALL keep a store in one SQLite file: the file `--store` names, or else `boom.sqlite` under `$XDG_DATA_HOME/boom/`, or under `~/.local/share/boom/` where `XDG_DATA_HOME` is unset or empty. `boom init` SHALL make the file and any folder above it; every other command SHALL use a store that exists and make none.

#### Scenario: The default store

- **WHEN** `boom init` runs with `XDG_DATA_HOME` set to a folder and no `--store`
- **THEN** the store is the file `boom/boom.sqlite` under that folder, made with its folder

#### Scenario: A store named outright

- **WHEN** `boom init --store <path>` runs, then `boom log --store <path>`
- **THEN** both use the file at `<path>`, and nothing is made under `XDG_DATA_HOME`

#### Scenario: No store yet

- **WHEN** `boom propose` runs where no store exists at the path it resolves
- **THEN** it exits 3 naming the path and saying `boom init` makes one, and no file is made

### Requirement: Views are computed from the log

The store's tables of current records and of persons SHALL be views computed from the log: each append SHALL update them in the transaction that appends the entry, and boom SHALL rebuild them from the log, before it answers, whenever they are missing or were built at an entry other than the log's last. No answer SHALL change when the views are deleted.

#### Scenario: Views deleted

- **WHEN** a store holds a proposed and an accepted version, its view tables are dropped, and `boom show <name> --json` runs
- **THEN** the answer is the one given before the tables were dropped, and the tables are there again

#### Scenario: Views behind the log

- **WHEN** an entry is appended to the log by another program and the views are left as they were
- **THEN** the next command rebuilds the views from the log before it answers, and its answer counts that entry as the log's checks judge it

#### Scenario: One transaction

- **WHEN** writing a proposal's version to the views fails
- **THEN** the command exits 3, and the proposal's entry is not in the log either

### Requirement: Many readers beside one writer

boom SHALL let any number of commands read a store while one appends, and SHALL make a second writer wait for the first rather than fail, so two appends begun at once both land, one after the other, the chain whole.

#### Scenario: Two proposals at once

- **WHEN** two processes each propose a different claim to one store at the same moment
- **THEN** both exit 0, the log holds both entries one after the other, and the chain verifies

#### Scenario: A read during a write

- **WHEN** another connection holds the store's write with an entry appended and not yet committed, and `boom show <name>` runs
- **THEN** it answers without waiting for that write, from the log as it stood before the entry

### Requirement: Long text stored once by its fingerprint

boom SHALL store the body of a version as a blob named by the SHA-256 of its bytes, written once however many versions name it; the version SHALL name the blob's fingerprint, and a blob whose bytes do not hash to its name SHALL be reported as unreadable, never shown.

#### Scenario: One body, two versions

- **WHEN** two versions of a claim are proposed with the same body file
- **THEN** the store holds one blob for that body, and both versions name its fingerprint

#### Scenario: A body read back

- **WHEN** `boom show <name>@<version>` runs for a version proposed with a body
- **THEN** the body's text is given, exactly the bytes proposed

#### Scenario: A blob altered

- **WHEN** a blob's bytes are changed in the file and `boom show <name>@<version>` asks for the version that names it
- **THEN** it exits 3 naming the fingerprint whose bytes no longer match, and shows no body
