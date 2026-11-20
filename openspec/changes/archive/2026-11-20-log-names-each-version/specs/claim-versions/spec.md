## MODIFIED Requirements

### Requirement: A claim shown by its name

`boom show <name>` SHALL give the kind, the accepted version or none, and the newest version with its statement, what it binds and its body. Then it SHALL give each other version, newest first, with its statement, status, base, proposer and decider. `boom show <name>@<version>` SHALL give the statement, the scenarios and the body of that version. `boom show <number>` SHALL give the version that its entry names. Boom SHALL refuse an unknown name, version or entry. Its JSON SHALL be `boom-show/2`.

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
