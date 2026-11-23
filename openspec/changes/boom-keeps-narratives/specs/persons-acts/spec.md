# Spec Delta

## MODIFIED Requirements

### Requirement: A person's act is signed with the person present

Except a note, an entry whose actor is a person SHALL carry an SSH signature, namespace `boom`, over the entry's signed part, by the key the store registered to that person before the entry, with the security key's user-presence flag set. boom SHALL verify it before it writes the entry and whenever it reads it.

#### Scenario: Signed without presence

- **WHEN** an accept is signed by the person's registered key with the user-presence flag unset
- **THEN** it exits 2 saying the signature does not show its person present, and nothing is appended

#### Scenario: Signed by a key not registered

- **WHEN** `boom accept` is run with a key the store does not register
- **THEN** it exits 2 saying the key is not registered, and nothing is appended

#### Scenario: A note with no signature

- **WHEN** chuck, registered, writes a note with `boom note` and no `--key`
- **THEN** the `noted` entry names chuck and carries no signature, and it counts

### Requirement: An agent proposes

An entry with no signature SHALL be an agent's, except a note of a person, naming its session or run as provenance from `--provenance`, and SHALL only propose. An unsigned proposal with no provenance SHALL be refused. A person MAY propose by signing the proposal with `--key`.

#### Scenario: An agent's proposal

- **WHEN** `boom propose ... --provenance flume:place-7` runs with no `--key`
- **THEN** the `proposed` entry names an agent with the provenance `flume:place-7` and carries no signature

#### Scenario: No provenance

- **WHEN** `boom propose` runs with neither `--key` nor `--provenance`
- **THEN** it exits 2 saying an agent's proposal names its provenance, and nothing is appended

