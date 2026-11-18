# Spec Delta

## Purpose

What boom keeps of a claim: one name for life, a version for each
change, each amendment naming the version it amends, and the status of
every version read from the log (record 0005, point 2; interview:
`identity-fixed-at-proposal`, `one-name-space`, `amend-names-its-base`).

## ADDED Requirements

### Requirement: A claim and its name

A claim SHALL be of one kind, `invariant`, `decision` or `assumption`, fixed with its name at its first proposal. Its name SHALL be kebab case, lowercase letters and digits in words joined by single hyphens. Each version SHALL hold a one-line statement, at least one scenario and an optional body.

#### Scenario: A claim proposed

- **WHEN** `boom propose invariant refund-within-payment "A refund never exceeds what the order was paid." --scenario "Given an order paid 40.00, when a refund of 45.00 is asked, then it is refused." --provenance run-1` runs
- **THEN** it exits 0, and `boom show refund-within-payment --json` gives an invariant with that one version, proposed

#### Scenario: A name not in kebab case

- **WHEN** a claim is proposed under the name `Refund_Within`
- **THEN** it exits 2 saying a name is kebab case, and nothing is appended

#### Scenario: No scenario

- **WHEN** a claim is proposed with no `--scenario`
- **THEN** it exits 2 saying a claim needs a scenario, and nothing is appended

#### Scenario: A statement of two lines

- **WHEN** a claim is proposed with a statement holding a line break
- **THEN** it exits 2 saying a statement is one line, and nothing is appended

### Requirement: A version is the fingerprint of its content

A version SHALL be named by the first 12 hexadecimal characters of the SHA-256 of its content: its kind, name, statement, scenarios, body's fingerprint and the version it amends. The same content proposed again SHALL be the same version and add nothing.

#### Scenario: The same proposal twice

- **WHEN** one claim is proposed twice with the same words, scenarios and body
- **THEN** both runs give the same version, and the log holds one `proposed` entry for it

#### Scenario: One word changed

- **WHEN** a version is proposed again with one word of its statement changed
- **THEN** it is a second version, under another fingerprint

### Requirement: One name space

No two claims of a store SHALL share a name. A first version under a name a claim of another kind holds SHALL be refused naming that claim; one under a name whose claim has a standing version SHALL be refused naming that version, to be amended. Several first versions MAY wait side by side while none stands.

#### Scenario: A name taken by another kind

- **WHEN** a decision `billing` is proposed where an invariant `billing` exists
- **THEN** it exits 2 naming the invariant `billing`, and nothing is appended

#### Scenario: A name whose claim stands

- **WHEN** a first version of `money-has-currency` is proposed while a version of it stands
- **THEN** it exits 2 naming the standing version and saying to amend it, and nothing is appended

#### Scenario: Two first versions waiting

- **WHEN** two first versions of one invariant, with different statements, are proposed while none stands
- **THEN** both are appended and both wait on a person

### Requirement: An amendment names the version it amends

A version proposed with `--amends <version>` SHALL name a version of the same claim as its base, and one naming a version the claim does not have SHALL be refused naming the claim. An amendment SHALL keep its claim's kind.

#### Scenario: An amendment proposed

- **WHEN** `boom propose invariant money-has-currency "<new words>" --scenario "<text>" --amends <standing version> --provenance run-2` runs
- **THEN** the new version is proposed with that version as its base, and the standing version still stands

#### Scenario: A base the claim does not have

- **WHEN** an amendment names a version no version of its claim has
- **THEN** it exits 2 naming the claim and the version it lacks, and nothing is appended

### Requirement: An accept stands only on its base

Accepting a version SHALL be refused, naming the standing version and recording nothing, unless the version it amends is the one that stands, or, for a first version, no version of the claim stands. boom SHALL never merge two amendments into one text.

#### Scenario: Two amendments of one base

- **WHEN** two amendments of the standing version are proposed and a person accepts the first
- **THEN** accepting the second exits 2 naming the first as the standing version, and the second stays proposed

#### Scenario: The same words on the new base

- **WHEN** the second amendment's words are proposed again amending the new standing version, and a person accepts it
- **THEN** it stands as a new version, and the second amendment stays proposed

### Requirement: Each version's status is read from the log

A version's status SHALL be computed from the log: `proposed` until a counted accept or reject; `standing` once accepted while no later-accepted version of its claim exists; `replaced` once a later version of its claim is accepted; `rejected` once rejected. A version SHALL keep its status's history in the log and never lose an entry.

#### Scenario: Proposed, standing, replaced

- **WHEN** a version is accepted, then an amendment of it is proposed and accepted
- **THEN** `boom show <name> --json` gives the first version as replaced, the amendment as standing, and the claim's standing version as the amendment

#### Scenario: Rejected

- **WHEN** a proposed version is rejected with a reason
- **THEN** `boom show <name> --json` gives it as rejected, with the person who rejected it and the reason

### Requirement: A claim shown by its name

`boom show <name>` SHALL give the claim's kind, its standing version or none, and each of its versions in the order proposed with its status, base, proposer and the person who accepted or rejected it. `boom show <name>@<version>` SHALL give that version's statement, scenarios and body. An unknown name or version SHALL be refused.

#### Scenario: A version shown

- **WHEN** `boom show money-has-currency@<version> --json` runs for a version proposed with a body
- **THEN** it gives the version's statement, scenarios, status and body

#### Scenario: An unknown name

- **WHEN** `boom show no-such-claim` runs
- **THEN** it exits 2 saying no claim has that name
