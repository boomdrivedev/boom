# persons-acts Specification

## Purpose
Who may do what to a store's claims, and how boom tells a person's act
from an agent's: an agent proposes, and only a person, signing with a
security key an owner registered, accepts or rejects (interview:
`agents-propose`, `accept-by-role-never-agent-never-author`,
`reject-is-a-persons`, `persons-acts-signed-with-presence`,
`self-registered-first-owner`; record 0006).

## Requirements

### Requirement: The first owner registers their own key

`boom init --person <name> --key <key>` SHALL make the store with one entry registering `<name>` as an owner with the public key beside `<key>`, the entry signed by that key. No one else SHALL verify that first owner; boom proves a person's acts were signed by their registered key, never who the person is.

#### Scenario: A store made

- **WHEN** `boom init --person ada --key <ada's key>` runs
- **THEN** the store's log holds one entry, `registered`, naming `ada` as an owner with ada's public key, signed by it

#### Scenario: Made twice with the same key

- **WHEN** `boom init --person ada --key <ada's key>` runs again on that store
- **THEN** it exits 0 answering as the first did, and nothing is appended

#### Scenario: Made again with another key

- **WHEN** `boom init --person bo --key <bo's key>` runs on a store that exists
- **THEN** it exits 2 saying the store exists and naming its first owner, and nothing is appended

### Requirement: Only a security key is registered

boom SHALL register only a key that signs with its person present, a security key of type `sk-ssh-ed25519@openssh.com` or `sk-ecdsa-sha2-nistp256@openssh.com`, and SHALL refuse any other, naming its type.

#### Scenario: A plain key file

- **WHEN** `boom init` or `boom register` is given an `ssh-ed25519` key
- **THEN** it exits 2 naming the type `ssh-ed25519` and the types boom registers, and nothing is appended

### Requirement: Owners register persons

`boom register <person> <public-key-file> [--owner] --key <key>` SHALL append an entry registering the person, an owner where `--owner` is given, signed by an owner's registered key. A register not signed by an owner, or naming a person or a key the store already registers to another, SHALL be refused.

#### Scenario: A second person registered

- **WHEN** ada, an owner, runs `boom register bo <bo's public key> --key <ada's key>`
- **THEN** the log gains a `registered` entry naming `bo` with bo's key, signed by ada

#### Scenario: Registered by someone not an owner

- **WHEN** bo, registered and not an owner, runs `boom register cy <cy's public key> --key <bo's key>`
- **THEN** it exits 2 saying only an owner registers a person, and nothing is appended

#### Scenario: A key registered to another

- **WHEN** an owner registers `cy` with the key already registered to `bo`
- **THEN** it exits 2 naming `bo`, and nothing is appended

### Requirement: A person's act is signed with the person present

An entry whose actor is a person SHALL carry an SSH signature, namespace `boom`, over the entry's signed part, by the key the store registered to that person before the entry, with the security key's user-presence flag set. boom SHALL verify it before it writes the entry and whenever it reads it.

#### Scenario: Signed without presence

- **WHEN** an accept is signed by the person's registered key with the user-presence flag unset
- **THEN** it exits 2 saying the signature does not show its person present, and nothing is appended

#### Scenario: Signed by a key not registered

- **WHEN** `boom accept` is run with a key the store does not register
- **THEN** it exits 2 saying the key is not registered, and nothing is appended

### Requirement: A signature covers the act, not its place in the log

An entry's signed part SHALL be the entry without its number, its chain and its signature, so the signature holds wherever the entry is ordered. `boom log --json` SHALL give each signed entry's signed part and signature, so a reader can verify it with `ssh-keygen -Y verify` and no boom.

#### Scenario: Verified with no boom

- **WHEN** `boom log --json` gives an `accepted` entry
- **THEN** its signature verifies, by an SSH signature verifier that is not boom's, over its signed part in the namespace `boom` with the public key the store registered to its person

### Requirement: An agent proposes

An entry with no signature SHALL be an agent's, naming its session or run as provenance from `--provenance`, and SHALL only propose. An unsigned proposal with no provenance SHALL be refused. A person MAY propose by signing the proposal with `--key`.

#### Scenario: An agent's proposal

- **WHEN** `boom propose ... --provenance flume:place-7` runs with no `--key`
- **THEN** the `proposed` entry names an agent with the provenance `flume:place-7` and carries no signature

#### Scenario: No provenance

- **WHEN** `boom propose` runs with neither `--key` nor `--provenance`
- **THEN** it exits 2 saying an agent's proposal names its provenance, and nothing is appended

### Requirement: Only a person accepts or rejects

`boom accept <name>@<version> --key <key>` and `boom reject <name>@<version> --reason <text> --key <key>` SHALL each append a person's signed entry. An accept or a reject with no `--key` SHALL be refused saying only a person accepts or rejects, and one found in the log unsigned SHALL count for nothing. A reject SHALL carry its reason.

#### Scenario: An agent's accept

- **WHEN** `boom accept refund-within-payment@<version>` runs with no `--key`
- **THEN** it exits 2 saying only a person accepts, and the version stays proposed

#### Scenario: A person accepts an agent's proposal

- **WHEN** ada accepts, with ada's key, a version an agent proposed
- **THEN** the version stands, and its `accepted` entry names ada

#### Scenario: A reject with no reason

- **WHEN** `boom reject <name>@<version> --key <ada's key>` runs with no `--reason`
- **THEN** it exits 2 saying a reject gives its reason, and nothing is appended

### Requirement: No one accepts their own proposal

Accepting a version SHALL be refused, recording nothing, where the person accepting is the person whose signed entry proposed it; that version waits on another registered person.

#### Scenario: Ada accepts what Ada proposed

- **WHEN** ada proposes a version with ada's key and then accepts it with the same key
- **THEN** the accept exits 2 saying the version waits on another person, and it stays proposed

#### Scenario: Bo accepts Ada's

- **WHEN** bo, registered, accepts with bo's key the version ada proposed
- **THEN** it stands, and its `accepted` entry names bo

### Requirement: Several versions accepted together

`boom accept` SHALL take several `<name>@<version>` targets with one `--key`, judge every one before the key signs any, sign each as an `accepted` entry of its own, and append them all in one transaction or none. A refusal of any SHALL name it, with nothing signed or appended. Its answer SHALL list, under `accepted`, each version as an accept of it alone gives it.

#### Scenario: Three accepted together

- **WHEN** ada runs `boom accept <a>@<version> <b>@<version> <c>@<version> --key <ada's key> --json` for three versions an agent proposed
- **THEN** it exits 0 listing the three under `accepted`, each stands, and the log gains three `accepted` entries, each signed by ada

#### Scenario: One of them refused

- **WHEN** one of the three is a version ada proposed with ada's key
- **THEN** it exits 2 naming that version, ada's key signs nothing, and the log is as it was
