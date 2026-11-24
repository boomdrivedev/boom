## MODIFIED Requirements

### Requirement: A claim and its name

A claim SHALL have one kind: `invariant`, `decision`, `assumption`, `term`, `context`, `narrative` or `outline`. The first proposal SHALL fix the kind and the name. A name SHALL be kebab case: lowercase letters and digits, in words that single hyphens join. Each version SHALL have a statement of one line. A decision and a narrative SHALL also have a body. Another kind MAY have one. A new version SHALL have no scenarios. An old version SHALL keep the scenarios that its entry holds.

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

#### Scenario: An outline with its title

- **WHEN** an agent proposes an outline with a title of one line, `--view book`, one `--narrative` and no `--body`
- **THEN** boom exits 0, and `boom show <name> --json` gives the kind `outline`
