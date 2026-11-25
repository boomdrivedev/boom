## RENAMED Requirements

- FROM: `### Requirement: Only a note counts with no signature`
- TO: `### Requirement: Only a note or a withdrawal counts with no signature`

## MODIFIED Requirements

### Requirement: Only a note or a withdrawal counts with no signature

The judge SHALL count an entry of a person with no signature only if its act is `noted` or `withdrawn`. The store SHALL register the person before that entry. An entry of an agent with the act `noted` SHALL count for nothing.

#### Scenario: An unsigned accept

- **WHEN** another program appends an `accepted` entry of chuck with no signature, chained correctly
- **THEN** the entry counts for nothing, and the version stays proposed

#### Scenario: A note from an agent

- **WHEN** another program appends a `noted` entry whose actor is an agent, chained correctly
- **THEN** the entry counts for nothing, and `boom show` gives no note

#### Scenario: A signed withdrawal

- **WHEN** another program appends a `withdrawn` entry of chuck, signed by chuck's key, that names a version chuck proposed
- **THEN** the entry counts for nothing, and the version stays proposed

### Requirement: A note is open until a version answers it

A note SHALL be open until a version of its narrative answers it, while no person rejects that version and no author withdraws it. `--answers <note>=<words>` SHALL answer a note in a new version, as part of its content. Boom SHALL refuse a new version of a narrative that leaves a note of that narrative open, and SHALL name each open note. Boom SHALL refuse an answer to a note that does not exist, is of another narrative or is not open.

#### Scenario: An amendment answers the note

- **WHEN** a note on `<v1>` is open, and an agent proposes `--amends <v1> --answers <note>="Section 10 now says it."`
- **THEN** boom exits 0, and `boom show <name>` gives the note as answered by the new version, with its words

#### Scenario: An amendment that leaves a note open

- **WHEN** a note on `<v1>` is open, and an agent proposes an amendment of `<v1>` with no `--answers`
- **THEN** boom exits 2 and names the open note with its paragraph and its words, and boom appends nothing

#### Scenario: The answer rejected

- **WHEN** a person rejects the version that answered a note
- **THEN** `boom show <name>` gives the note as open again

#### Scenario: A note of another narrative

- **WHEN** an agent proposes an amendment of one narrative that answers a note of another narrative
- **THEN** boom exits 2 and names the note and its narrative, and boom appends nothing
