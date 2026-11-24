## ADDED Requirements

### Requirement: The plan words

The plan words SHALL be `wave`, `phase`, `milestone`, `sprint`, `epic`, `user story`, `roadmap`, `MVP`, `stretch goal`, `nice to have`, `stopgap`, `interim`, `for now`, `happy path`, `sad path`, `unhappy path`, `TODO`, `TBD`, `WIP` and `FIXME`. A plan word SHALL match as an old word of a term matches (`term-check`).

#### Scenario: Another form

- **WHEN** an agent proposes a version whose body holds `Phases` and no other plan word
- **THEN** boom warns of `phase`

#### Scenario: A plan word inside a longer word

- **WHEN** an agent proposes a version whose statement holds `microwave` and no other plan word
- **THEN** standard error holds no warning

### Requirement: A proposal warns of plan words

`boom propose` SHALL read the statement and the body of a version for the plan words. For each match, boom SHALL write a warning on standard error that names the word and where it is. With `--json`, the field `plan_words` SHALL give each match. The warning SHALL NOT refuse, and SHALL NOT change what boom appends or how it exits.

#### Scenario: A plan word in the statement

- **WHEN** an agent proposes an invariant whose statement is "In the first wave, boom keeps each entry."
- **THEN** boom exits 0 and appends the version, and standard error names `wave` in the statement

#### Scenario: A plan word in the body

- **WHEN** an agent proposes a decision whose body holds "the happy path" on its line 12
- **THEN** boom exits 0 and appends the version, and standard error names `happy path` in line 12 of the body

#### Scenario: The warnings as JSON

- **WHEN** an agent proposes with `--json` a version whose statement holds `MVP` and whose body holds `Phases` on line 3
- **THEN** the object of `boom-propose/2` gives `plan_words` with `MVP` in the statement and `phase` in line 3 of the body

#### Scenario: No plan word

- **WHEN** an agent proposes with `--json` a version whose statement and body hold no plan word
- **THEN** standard error holds no warning, and `plan_words` is empty

#### Scenario: The same version again

- **WHEN** an agent proposes a version with a plan word, and then proposes the same content again
- **THEN** both runs warn, and the log holds one `proposed` entry for the version

#### Scenario: The MCP tool warns too

- **WHEN** an agent's program calls the MCP tool `propose` with a statement that holds `roadmap`
- **THEN** the result is not an error, and its `boom-propose/2` object gives `roadmap` in `plan_words`
