## MODIFIED Requirements

### Requirement: A version is the fingerprint of its content

Boom SHALL name a version by the first 12 hexadecimal characters of the SHA-256 of its content. The content SHALL be the kind, the name, the statement, the scenarios, the body's fingerprint and the version that it amends. It SHALL also include the history of a version that comes in as history, the repository and any `replaces` of a term, and any edge `source`, `changes` and `origin`. When a proposal gives the same content again, boom SHALL give the same version and append nothing.

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

#### Scenario: One more edge

- **WHEN** an agent proposes a claim again with the same words and one more `--source`
- **THEN** boom gives a second version, with a different fingerprint

#### Scenario: A version from before terms

- **WHEN** a store holds versions that boom proposed before the kind `term` came, and this boom builds its views again
- **THEN** each version has the fingerprint that its entry names

#### Scenario: A version from before the edges

- **WHEN** a store holds versions that boom proposed before the edges `source`, `changes` and `origin` came, and this boom builds its views again
- **THEN** each version has the fingerprint that its entry names

### Requirement: A claim shown by its name

`boom show <name>` SHALL give the kind, the accepted version or none, and the newest version with its statement, what it binds and its body. It SHALL then give the other versions as a tree by base, each with its statement, status, base, proposer and decider. `boom show <name>@<version>` SHALL give its statement, scenarios, body and `replaces`. `boom show <number>` SHALL give the version that its entry names. It SHALL refuse an unknown name, version or entry. The JSON SHALL be `boom-show/2`.

#### Scenario: A version shown

- **WHEN** `boom show money-has-currency@<version> --json` runs for a version proposed with a body
- **THEN** it gives `boom-show/2` with the statement, the scenarios, the status and the body of the version

#### Scenario: An unknown name

- **WHEN** `boom show no-such-claim` runs
- **THEN** boom exits 2 and says that no claim has that name

#### Scenario: The newest version first

- **WHEN** a version of `money-has-currency` is accepted, an agent proposes an amendment of it, and `boom show money-has-currency` runs
- **THEN** the first line says `accepted: <accepted version>`. The statement of the amendment comes next, and the accepted version comes after `other versions:`

#### Scenario: An amendment under its base

- **WHEN** `money-has-currency@<v1>` is accepted, a person rejects its amendment `<v2>`, an agent proposes its amendment `<v3>`, and `boom show money-has-currency` runs
- **THEN** after `other versions:`, boom gives `<v1>`, and `<v2>` under it, as the amendment of `<v1>`. With `--json`, each of `versions` names its `base`

#### Scenario: A version shown by its entry

- **WHEN** entry 2 proposes `money-has-currency@<version>`, entry 3 accepts it, and `boom show 2` and `boom show 3` run
- **THEN** each gives what `boom show money-has-currency@<version>` gives

#### Scenario: An entry that names no version

- **WHEN** `boom show 1` runs, and entry 1 registers a person
- **THEN** boom exits 2 and says that entry 1 names no version

#### Scenario: A number that the log does not hold

- **WHEN** `boom show 9` runs on a log of three entries
- **THEN** boom exits 2 and says that the log has no entry 9
