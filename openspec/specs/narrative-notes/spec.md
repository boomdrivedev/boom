# narrative-notes Specification

## Purpose
A note of a person on one paragraph of a version of a narrative: an
entry of the log that names the person and has no signature, open
until a later version of the narrative answers it. The note is the
signal that goes back to the author of the narrative
(`events-are-appended-never-changed`, `status-read-from-events`,
`a-link-remembers-the-version-it-meant`).

## Requirements

### Requirement: A person writes a note on a paragraph

`boom note <name>@<version> --paragraph <n> "<words>"` SHALL append one entry with the act `noted`. The entry SHALL hold the name of the narrative, the version, the paragraph number, the fingerprint of the paragraph and the words. Its actor SHALL be the person, by name, with no signature. Boom SHALL take the person from `--person <name>`. With no `--person`, boom SHALL take the one person that the store registers.

#### Scenario: A note written

- **WHEN** the store registers chuck only, and `boom note boom-end-to-end@<version> --paragraph 3 "Say what a drive is."` runs
- **THEN** boom exits 0, and the log gets one `noted` entry whose actor is `chuck`, with no signature, for paragraph 3 of that version

#### Scenario: Two persons and no --person

- **WHEN** the store registers ada and bo, and `boom note` runs with no `--person`
- **THEN** boom exits 2 and says that `--person` names the person, and boom appends nothing

### Requirement: A note names what it meant

Boom SHALL refuse a note, and append nothing, if one of these is true:

- The person is not registered.
- The version is not a version of a narrative.
- The version has no paragraph with that number.
- The words are empty.
- The command gives `--key` or `--provenance`.

#### Scenario: A person not registered

- **WHEN** `boom note <name>@<version> --paragraph 1 "<words>" --person cy` runs, and the store does not register cy
- **THEN** boom exits 2 and says that cy is not registered, and boom appends nothing

#### Scenario: A note on an invariant

- **WHEN** `boom note status-read-from-events@<version> --paragraph 1 "<words>"` runs
- **THEN** boom exits 2 and says that a note is on a narrative, and boom appends nothing

#### Scenario: A paragraph the version does not have

- **WHEN** `boom note <name>@<version> --paragraph 40 "<words>"` runs on a version with 12 paragraphs
- **THEN** boom exits 2 and says that the version has 12 paragraphs, and boom appends nothing

#### Scenario: An agent's note

- **WHEN** `boom note <name>@<version> --paragraph 1 "<words>" --provenance run-1` runs
- **THEN** boom exits 2 and says that only a person writes a note, and boom appends nothing

### Requirement: A note's name

The name of a note SHALL be the first 12 hexadecimal characters of a SHA-256. The SHA-256 SHALL be of the narrative's name, the version, the paragraph number, the words and the person. For the same note again, boom SHALL give the same name and append nothing.

#### Scenario: The same note twice

- **WHEN** chuck writes the same words on the same paragraph of the same version two times
- **THEN** both runs give the same name, and the log holds one `noted` entry

### Requirement: Only a note counts with no signature

The judge SHALL count an entry of a person with no signature only if its act is `noted`. The store SHALL register the person before that entry. An entry of an agent with the act `noted` SHALL count for nothing.

#### Scenario: An unsigned accept

- **WHEN** another program appends an `accepted` entry of chuck with no signature, chained correctly
- **THEN** the entry counts for nothing, and the version stays proposed

#### Scenario: A note from an agent

- **WHEN** another program appends a `noted` entry whose actor is an agent, chained correctly
- **THEN** the entry counts for nothing, and `boom show` gives no note

### Requirement: A note is open until a version answers it

A note SHALL be open until a version of its narrative answers it and no person rejects that version. `--answers <note>=<words>` SHALL answer a note in a new version. The answers SHALL be part of the content of the version. Boom SHALL refuse a new version of a narrative that leaves a note of that narrative open, and SHALL name each open note. Boom SHALL refuse an answer to a note that does not exist. Boom SHALL refuse an answer to a note of another narrative or to a note that is not open.

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
