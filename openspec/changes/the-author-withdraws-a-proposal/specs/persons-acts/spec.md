## MODIFIED Requirements

### Requirement: A person's act is signed with the person present

Except a note or a withdrawal, an entry whose actor is a person SHALL carry an SSH signature, namespace `boom`, over the entry's signed part. The key SHALL be the key the store registered to that person before the entry, with the security key's user-presence flag set. boom SHALL verify it before it writes the entry and whenever it reads it.

#### Scenario: Signed without presence

- **WHEN** an accept is signed by the person's registered key with the user-presence flag unset
- **THEN** it exits 2 saying the signature does not show its person present, and nothing is appended

#### Scenario: Signed by a key not registered

- **WHEN** `boom accept` is run with a key the store does not register
- **THEN** it exits 2 saying the key is not registered, and nothing is appended

#### Scenario: A note with no signature

- **WHEN** chuck, registered, writes a note with `boom note` and no `--key`
- **THEN** the `noted` entry names chuck and carries no signature, and it counts

#### Scenario: A withdrawal with no signature

- **WHEN** chuck, registered, withdraws a version that chuck proposed, with `boom withdraw --person chuck` and no `--key`
- **THEN** the `withdrawn` entry names chuck and carries no signature, and it counts

### Requirement: An agent proposes

Except a note or a withdrawal of a person, an entry with no signature SHALL be an agent's. It SHALL name its session or run as provenance from `--provenance`, and SHALL only propose or withdraw. An unsigned proposal or withdrawal of an agent with no provenance SHALL be refused. A person MAY propose by signing the proposal with `--key`.

#### Scenario: An agent's proposal

- **WHEN** `boom propose ... --provenance flume:place-7` runs with no `--key`
- **THEN** the `proposed` entry names an agent with the provenance `flume:place-7` and carries no signature

#### Scenario: No provenance

- **WHEN** `boom propose` runs with neither `--key` nor `--provenance`
- **THEN** it exits 2 saying an agent's proposal names its provenance, and nothing is appended

#### Scenario: An agent's withdrawal

- **WHEN** `boom withdraw <name>@<version> --reason "a duplicate" --provenance flume:place-7` runs for a version that an agent proposed
- **THEN** the `withdrawn` entry names an agent with the provenance `flume:place-7` and carries no signature

#### Scenario: A withdrawal with no author

- **WHEN** `boom withdraw <name>@<version> --reason "a duplicate"` runs with neither `--provenance` nor `--person`
- **THEN** it exits 2 saying a withdrawal names its author, and nothing is appended

## ADDED Requirements

### Requirement: Several versions rejected together

`boom reject` SHALL take several `<name>@<version>` targets with one `--reason` and one `--key`. It SHALL write one `rejected` entry that names each version, in the order given, with the reason. Boom SHALL judge that entry whole before the key signs it, and the key SHALL sign it one time. A refusal of any version SHALL name it, and nothing is signed or appended. Its answer SHALL be `boom-reject/3` and list each version under `rejected`.

#### Scenario: Three rejected together

- **WHEN** ada runs `boom reject <a>@<version> <b>@<version> <c>@<version> --reason "stray drafts" --key <ada's key> --json` for three versions an agent proposed
- **THEN** it exits 0 and lists the three under `rejected`, and each is `rejected` with the reason. The key of ada signs one time, and the log gets one `rejected` entry that names the three

#### Scenario: One of them refused

- **WHEN** one of the three is accepted
- **THEN** it exits 2 naming that version, ada's key signs nothing, and the log is as it was

#### Scenario: Rejected again

- **WHEN** ada runs that reject again
- **THEN** it exits 0 and lists the three under `rejected`, the key signs nothing, and the log does not change

#### Scenario: Two versions of one claim

- **WHEN** two versions of one claim wait, neither amends the other, and ada rejects both together
- **THEN** it exits 0, and both are `rejected`

#### Scenario: One version keeps its form

- **WHEN** ada rejects one version
- **THEN** its `rejected` entry names it by `name` and `version`, as a reject did before this change
