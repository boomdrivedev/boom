# Spec Delta

## MODIFIED Requirements

### Requirement: A claim and its name

A claim SHALL have one kind: `invariant`, `decision`, `assumption`, `term`, `context` or `narrative`. The first proposal SHALL fix the kind and the name. A name SHALL be kebab case: lowercase letters and digits, in words that single hyphens join. Each version SHALL have a statement of one line. A decision version and a narrative version SHALL also have a body. Another version MAY have a body. A new version SHALL have no scenarios. An old version SHALL keep the scenarios that its entry holds.

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

#### Scenario: A context with no body

- **WHEN** an agent proposes a context with a statement, one glob and no `--body`
- **THEN** boom exits 0

#### Scenario: A narrative with its title and body

- **WHEN** an agent proposes a narrative with a title of one line and `--body`
- **THEN** boom exits 0, and `boom show <name> --json` gives the kind `narrative`

### Requirement: A version is the fingerprint of its content

Boom SHALL name a version by the first 12 hexadecimal characters of the SHA-256 of its content. The content SHALL be the kind, name, statement, scenarios, body fingerprint and base. It SHALL also hold the history of a version from history, the repository and `replaces` of a term, and the globs of a context. It SHALL hold the edges `source`, `changes`, `origin` and `includes`, and the answers of a narrative. For the same content again, boom SHALL give the same version and append nothing.

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

#### Scenario: One more glob

- **WHEN** an agent proposes a context again with the same statement and one more `--path`
- **THEN** boom gives a second version, with a different fingerprint

#### Scenario: The same words bound to a context

- **WHEN** an agent proposes a claim bound to `shop`, and the same claim in the same words is proposed to another store bound to the context `shop-billing`
- **THEN** both answers give the same version

#### Scenario: A version from before terms

- **WHEN** a store holds versions that boom proposed before the kind `term` came, and this boom builds its views again
- **THEN** each version has the fingerprint that its entry names

#### Scenario: A version from before the edges

- **WHEN** a store holds versions that boom proposed before the edges `source`, `changes` and `origin` came, and this boom builds its views again
- **THEN** each version has the fingerprint that its entry names

#### Scenario: A version from before contexts

- **WHEN** a store holds versions that boom proposed before the kind `context` came, and this boom builds its views again
- **THEN** each version has the fingerprint that its entry names

#### Scenario: One more include

- **WHEN** an agent proposes a narrative again with the same title and body and one more `--includes`
- **THEN** boom gives a second version, with a different fingerprint

#### Scenario: A version from before narratives

- **WHEN** a store holds versions that boom proposed before the kind `narrative` came, and this boom builds its views again
- **THEN** each version has the fingerprint that its entry names
