## MODIFIED Requirements

### Requirement: A claim and its name

A claim SHALL be of one kind, `invariant`, `decision` or `assumption`, fixed with its name at its first proposal. Its name SHALL be kebab case, lowercase letters and digits in words joined by single hyphens. Each version SHALL hold a one-line statement, at least one scenario and an optional body; a version that comes in as history (record-import) MAY hold no scenario.

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

#### Scenario: History with no scenario

- **WHEN** a superseded decision is imported with no scenario drafted for it
- **THEN** its version is proposed with no scenario, as history

### Requirement: A version is the fingerprint of its content

A version SHALL be named by the first 12 hexadecimal characters of the SHA-256 of its content: its kind, name, statement, scenarios, body's fingerprint and the version it amends, and, for a version that comes in as history, its history. The same content proposed again SHALL be the same version and add nothing.

#### Scenario: The same proposal twice

- **WHEN** one claim is proposed twice with the same words, scenarios and body
- **THEN** both runs give the same version, and the log holds one `proposed` entry for it

#### Scenario: One word changed

- **WHEN** a version is proposed again with one word of its statement changed
- **THEN** it is a second version, under another fingerprint

#### Scenario: The same words as history

- **WHEN** a superseded decision is imported with a drafted scenario, and `boom propose` proposes its kind, name, statement, scenario and body with no history
- **THEN** the two are two versions, under two fingerprints

### Requirement: Each version's status is read from the log

A version's status SHALL be computed from the log: `proposed` until a counted accept or reject; `standing` once accepted while no later-accepted version of its claim exists; `replaced` once a later version of its claim is accepted; `rejected` once rejected; and `history` once accepted where it came in as history, which never stands or replaces. A version SHALL keep its status's history in the log and never lose an entry.

#### Scenario: Proposed, standing, replaced

- **WHEN** a version is accepted, then an amendment of it is proposed and accepted
- **THEN** `boom show <name> --json` gives the first version as replaced, the amendment as standing, and the claim's standing version as the amendment

#### Scenario: Rejected

- **WHEN** a proposed version is rejected with a reason
- **THEN** `boom show <name> --json` gives it as rejected, with the person who rejected it and the reason

#### Scenario: History accepted

- **WHEN** a person accepts a version that came in as history
- **THEN** `boom show <name> --json` gives that version as history and the claim with no standing version
