# claim-versions Specification

## Purpose
What boom keeps of a claim: one name for life, a version for each
change, each amendment naming the version it amends, and the status of
every version read from the log (record 0005, point 2; interview:
`identity-fixed-at-proposal`, `one-name-space`, `amend-names-its-base`).

## Requirements

### Requirement: A claim and its name

A claim SHALL have one kind: `invariant`, `decision`, `assumption` or `term`. The first proposal SHALL fix the kind and the name. A name SHALL be kebab case: lowercase letters and digits, in words that single hyphens join. Each version SHALL have a statement of one line. A decision version SHALL also have a body. Another version MAY have a body. A new version SHALL have no scenarios. An old version SHALL keep the scenarios that its entry holds.

#### Scenario: A claim proposed

- **WHEN** `boom propose invariant refund-within-payment "A refund never exceeds what the order was paid." --provenance run-1` runs
- **THEN** boom exits 0, and `boom show refund-within-payment --json` gives an invariant with one proposed version

#### Scenario: A name not in kebab case

- **WHEN** an agent proposes a claim with the name `Refund_Within`
- **THEN** boom exits 2 and says that a name is kebab case, and boom appends nothing

#### Scenario: No scenario

- **WHEN** an agent proposes an invariant with a statement and no other field
- **THEN** boom exits 0, and the proposal's entry holds no scenarios

#### Scenario: A statement of two lines

- **WHEN** an agent proposes a claim whose statement holds a line break
- **THEN** boom exits 2 and says that a statement is one line, and boom appends nothing

#### Scenario: History with no scenario

- **WHEN** an import proposes a superseded decision
- **THEN** boom proposes its version as history, with no scenarios

#### Scenario: A decision with no body

- **WHEN** an agent proposes a decision with no `--body`
- **THEN** boom exits 2 and says that a decision needs a body, and boom appends nothing

#### Scenario: An old version with scenarios

- **WHEN** another program appends an old proposal entry that holds a scenario, chained correctly
- **THEN** the entry counts, and `boom show` gives the version with its scenario

#### Scenario: A term with no body

- **WHEN** an agent proposes a term with a meaning and no `--body`, for one repository
- **THEN** boom exits 0

### Requirement: A version is the fingerprint of its content

Boom SHALL name a version by the first 12 hexadecimal characters of the SHA-256 of its content. The content SHALL be the kind, the name, the statement, the scenarios, the body's fingerprint and the version that it amends. For a version that comes in as history, the content SHALL also include its history. For a term version, it SHALL also include its repository and any `replaces`. When a proposal gives the same content again, boom SHALL give the same version and append nothing.

#### Scenario: The same proposal twice

- **WHEN** an agent proposes one claim twice, with the same words and body
- **THEN** both runs give the same version, and the log holds one `proposed` entry for it

#### Scenario: One word changed

- **WHEN** an agent proposes a version again with one word of its statement changed
- **THEN** boom gives a second version, with a different fingerprint

#### Scenario: The same words as history

- **WHEN** an import proposes a superseded decision, and `boom propose` proposes its kind, name, statement and body with no history
- **THEN** boom gives two versions, with two fingerprints

#### Scenario: Another old word

- **WHEN** an agent proposes the term `owner` again with the same meaning and one more word in `replaces`
- **THEN** boom gives a second version, with a different fingerprint

#### Scenario: One term in two repositories

- **WHEN** an agent proposes the term `owner` with one meaning for `boom`, and then for `sluice`
- **THEN** boom gives two versions, with two fingerprints

#### Scenario: A version from before terms

- **WHEN** a store holds versions that boom proposed before the kind `term` came, and this boom builds its views again
- **THEN** each version has the fingerprint that its entry names

### Requirement: One name space

Two claims of a store SHALL NOT have the same name, except two terms that bind two repositories. Boom SHALL refuse a first version whose name a claim of another kind has, and SHALL name that claim. Boom SHALL refuse a first version whose claim has an accepted version, and SHALL name that version, to be amended. Several first versions MAY wait together while no version of the claim is accepted.

#### Scenario: A name taken by another kind

- **WHEN** an agent proposes a decision `billing` while an invariant `billing` exists
- **THEN** boom exits 2 and names the invariant `billing`, and boom appends nothing

#### Scenario: A name whose claim stands

- **WHEN** an agent proposes a first version of `money-has-currency` while a version of it is accepted
- **THEN** boom exits 2, names the accepted version and says to amend it, and boom appends nothing

#### Scenario: Two first versions waiting

- **WHEN** an agent proposes two first versions of one invariant, with different statements, while no version is accepted
- **THEN** boom appends both, and both wait for a person

#### Scenario: A term with the name of an invariant

- **WHEN** an agent proposes the term `billing` for `shop` while an invariant `billing` exists
- **THEN** boom exits 2 and names the invariant `billing`, and boom appends nothing

### Requirement: An amendment names the version it amends

A version that `--amends <version>` proposes SHALL name a version of the same claim as its base. Boom SHALL refuse an amendment whose base the claim does not have, and SHALL name the claim. An amendment SHALL keep the kind of its claim.

#### Scenario: An amendment proposed

- **WHEN** `boom propose invariant money-has-currency "<new words>" --amends <accepted version> --provenance run-2` runs
- **THEN** boom proposes the new version with that version as its base, and the accepted version stays accepted

#### Scenario: A base the claim does not have

- **WHEN** an amendment names a version that its claim does not have
- **THEN** boom exits 2 and names the claim and the version it does not have, and boom appends nothing

### Requirement: An accept stands only on its base

Boom SHALL refuse to accept a version unless its base is the accepted version of its claim. For a first version, boom SHALL refuse unless the claim has no accepted version. A refusal SHALL name the accepted version and record nothing. Boom SHALL NOT merge two amendments into one text.

#### Scenario: Two amendments of one base

- **WHEN** an agent proposes two amendments of the accepted version, and a person accepts the first
- **THEN** an accept of the second exits 2 and names the first as the accepted version, and the second stays proposed

#### Scenario: The same words on the new base

- **WHEN** an agent proposes the words of the second amendment again, as an amendment of the new accepted version, and a person accepts it
- **THEN** the new version is accepted, and the second amendment stays proposed

### Requirement: Each version's status is read from the log

Boom SHALL compute the status of a version from the log. The status SHALL be one of these:

- `proposed`, until a counted accept or reject.
- `accepted`, after an accept, while no later version of its claim is accepted.
- `superseded`, after a later version of its claim is accepted.
- `rejected`, after a person rejects it.
- For an accepted version that came in as history, the status of its file.

No act makes a version `deprecated` yet. The log SHALL keep each entry.

#### Scenario: Proposed, standing, replaced

- **WHEN** a person accepts a version, then an agent proposes an amendment of it, and a person accepts the amendment
- **THEN** `boom show <name> --json` gives the first version as `superseded`, the amendment as `accepted`, and the amendment as the accepted version of the claim

#### Scenario: Rejected

- **WHEN** a person rejects a proposed version, with a reason
- **THEN** `boom show <name> --json` gives it as `rejected`, with the person who rejected it and the reason

#### Scenario: History accepted

- **WHEN** a person accepts a version that came in as history from a file whose status is superseded
- **THEN** `boom show <name> --json` gives that version as `superseded`, and gives the claim no accepted version

### Requirement: A claim shown by its name

`boom show <name>` SHALL give the kind, the accepted version or none, and the newest version with its statement, what it binds and its body. It SHALL then give each other version, newest first, with its statement, status, base, proposer and decider. `boom show <name>@<version>` SHALL give its statement, scenarios, body and `replaces`. `boom show <number>` SHALL give the version that its entry names. It SHALL refuse an unknown name, version or entry. The JSON SHALL be `boom-show/2`.

#### Scenario: A version shown

- **WHEN** `boom show money-has-currency@<version> --json` runs for a version proposed with a body
- **THEN** it gives `boom-show/2` with the statement, the scenarios, the status and the body of the version

#### Scenario: An unknown name

- **WHEN** `boom show no-such-claim` runs
- **THEN** boom exits 2 and says that no claim has that name

#### Scenario: The newest version first

- **WHEN** a version of `money-has-currency` is accepted, an agent proposes an amendment of it, and `boom show money-has-currency` runs
- **THEN** the first line says `accepted: <accepted version>`. The statement of the amendment comes next, and the accepted version comes after `other versions:`

#### Scenario: A version shown by its entry

- **WHEN** entry 2 proposes `money-has-currency@<version>`, entry 3 accepts it, and `boom show 2` and `boom show 3` run
- **THEN** each gives what `boom show money-has-currency@<version>` gives

#### Scenario: An entry that names no version

- **WHEN** `boom show 1` runs, and entry 1 registers a person
- **THEN** boom exits 2 and says that entry 1 names no version

#### Scenario: A number that the log does not hold

- **WHEN** `boom show 9` runs on a log of three entries
- **THEN** boom exits 2 and says that the log has no entry 9

### Requirement: A word shown for each repository

For a word with a term in two or more repositories, `boom show <word>` SHALL give each term, one after the other, in the order of their repositories. `boom show <word> --repository <name>` SHALL give the term of that repository only. Boom SHALL refuse `--repository` for a repository where the word has no term.

#### Scenario: A word with a term in two repositories

- **WHEN** the term `owner` binds `boom`, another term `owner` binds `sluice`, and `boom show owner --json` runs
- **THEN** it gives two terms, the term of `boom` and then the term of `sluice`

#### Scenario: One repository chosen

- **WHEN** `boom show owner --repository sluice --json` runs
- **THEN** it gives the term of `sluice` only
