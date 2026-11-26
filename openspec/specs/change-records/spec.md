# change-records Specification

## Purpose
Each version of a change record that an agent records with no accept:
the change's files kept by their fingerprints, a version that is the
fingerprint of the files and the links, its approval, its unit and its
bolt, the claims that it covers, and its status, written or landed, read
from the log.

## Requirements

### Requirement: An agent records a version of a change record

`boom record change <repository>:<name>` SHALL append a `recorded` entry, an agent's with its provenance and no signature. Its `change` SHALL hold the repository, the name, the version, its base, `approval`, `unit`, `bolt`, `covers` and `files`. Its `origin`, from `--origin git:<repository>@<commit>:<path>`, SHALL be out of `change`. No person SHALL accept or reject it, and a person's entry with no signature SHALL count for nothing.

#### Scenario: A first version recorded

- **WHEN** the log holds the `approved` entry `<a>`, and an agent runs `boom record change boom:contexts-own-paths --approval <a> --unit contexts --bolt b-12 --provenance run-1` in a working folder with `openspec/changes/contexts-own-paths/`
- **THEN** boom exits 0, the log gets one `recorded` entry for `boom:contexts-own-paths` with no base, and `boom show boom:contexts-own-paths` gives it as `written`

#### Scenario: An accept of a change record

- **WHEN** ada runs `boom accept boom:contexts-own-paths --key <ada's key>`
- **THEN** boom exits 2 and says that no person accepts a change record, and boom appends nothing

#### Scenario: Recorded with no provenance

- **WHEN** `boom record change boom:contexts-own-paths --approval <a> --unit contexts --bolt b-12` runs
- **THEN** boom exits 2 and says that a change record names its provenance, and boom appends nothing

### Requirement: Boom keeps the change's files by their fingerprints

Boom SHALL read each file under `openspec/changes/<name>` of the working folder, or under `--folder <path>`, and keep each file once as a blob by the SHA-256 of its bytes. `files` SHALL hold each file's path in the folder and its SHA-256, sorted by path. Boom SHALL refuse an empty folder, a symbolic link in it, and a file of more than 1 MiB.

#### Scenario: Each file by its fingerprint

- **WHEN** the folder holds `proposal.md`, `design.md`, `tasks.md` and `specs/change-records/spec.md`, and an agent records it
- **THEN** `files` holds the four paths, sorted, each with its SHA-256, and `boom show --artifact <the SHA-256 of tasks.md>` gives the bytes of `tasks.md`

#### Scenario: An empty folder

- **WHEN** an agent records a change whose folder holds no file
- **THEN** boom exits 2 and names the folder, and boom appends nothing

### Requirement: A version is the fingerprint of the change's files and links

The version SHALL be the first 12 hexadecimal characters of the SHA-256 of the change's repository, name, base, approval, unit, bolt, covered claims and files. The origin SHALL be no part of it. The same content SHALL append nothing.

#### Scenario: A rebase keeps the version

- **WHEN** an agent records a change at `git:boom@<c1>:openspec/changes/contexts-own-paths`, then records the same files at `<c2>`
- **THEN** both give one version, and boom appends nothing the second time

#### Scenario: A file changed gives a new version

- **WHEN** an agent changes `design.md` of a recorded change, and records it with `--amends <the version>`
- **THEN** boom exits 0 and gives a new version, whose base is the version before

### Requirement: A new version amends the newest version

Boom SHALL refuse a first version with `--amends`, a later version that does not amend the newest version, and a new version of a change record that landed.

#### Scenario: A second version with no base

- **WHEN** `boom:contexts-own-paths` has a version, and an agent records a new version of it with no `--amends`
- **THEN** boom exits 2 and names the newest version to amend, and boom appends nothing

#### Scenario: A change record that landed

- **WHEN** an entry `sluice.landed` links the `recorded` entry of `boom:contexts-own-paths@<v1>`, and an agent records a version with `--amends <v1>`
- **THEN** boom exits 2 and says that a landed change never changes, and boom appends nothing

### Requirement: A version links its approval and covers what its approval covers

`--approval <act hash>` SHALL name a counted `approved` entry. `--unit <name>` and `--bolt <name>` SHALL each name one kebab name, which boom keeps as given. Each `--covers <name>[@<version>]` SHALL be a version that the approval covers. Boom SHALL refuse a version with no approval, no unit or no bolt.

#### Scenario: An approval that is not an approval

- **WHEN** `<l>` is the act hash of a `sluice.landed` entry, and an agent records a change with `--approval <l>`
- **THEN** boom exits 2 and says that `<l>` is not an `approved` entry, and boom appends nothing

#### Scenario: A claim that the approval does not cover

- **WHEN** the approval `<a>` covers `status-read-from-events@<v1>` alone, and an agent records a change with `--approval <a> --covers money-has-currency`
- **THEN** boom exits 2 and names `money-has-currency` as a claim that `<a>` does not cover, and boom appends nothing

#### Scenario: A name covers the version of the approval

- **WHEN** the approval `<a>` covers `status-read-from-events@<v1>`, `<v2>` is accepted later, and an agent records a change with `--approval <a> --covers status-read-from-events`
- **THEN** the version covers `status-read-from-events@<v1>`

### Requirement: The status of a change record

A change record SHALL be `written` from its first version, and `landed` once a counted `sluice.landed` entry links the `recorded` entry of one of its versions, at that version. Boom SHALL read no other entry of another tool for it.

#### Scenario: Landed at a version

- **WHEN** `boom:contexts-own-paths` has `<v1>` and `<v2>`, and a `sluice.landed` entry links the `recorded` entry of `<v2>`
- **THEN** `boom show boom:contexts-own-paths --json` gives `status` `landed` and `landed` `<v2>`

### Requirement: boom show gives a change record

`boom show <repository>:<name>[@<version>]` SHALL give the change record: its status, its versions by base, and for each its approval, unit, bolt, covered claims, files and origin. A covered claim whose accepted version moved on SHALL be marked out of date, naming the accepted version. With `--json`, the object's `format` SHALL be `boom-change/1`.

#### Scenario: A covered claim out of date

- **WHEN** a version covers `status-read-from-events@<v1>`, and a person accepts `<v2>`
- **THEN** `boom show boom:contexts-own-paths` marks `status-read-from-events@<v1>` out of date, naming `<v2>` as accepted

#### Scenario: A change record that the store does not hold

- **WHEN** `boom show boom:no-such-change` runs
- **THEN** boom exits 2 and says that no change record has that name

### Requirement: The log names a version of a change record

`boom log` SHALL give the line of a `recorded` entry as its number, `recorded`, `<repository>:<name>@<version>` and its actor.

#### Scenario: The line of a change record

- **WHEN** entry 8 records `boom:contexts-own-paths@<v1>` and `boom log` runs
- **THEN** line 8 starts with `8 recorded boom:contexts-own-paths@<v1> by an agent`
