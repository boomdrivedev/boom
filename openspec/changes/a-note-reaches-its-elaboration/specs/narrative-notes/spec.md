## ADDED Requirements

### Requirement: A note reaches the elaboration of its version

The elaboration of a note SHALL be the elaboration that the version it is on names in its origin, or none where that version names none. Boom SHALL read it from the version and SHALL store it in no entry. `boom show <name>` and `boom show <name>@<version>` SHALL give each note with its elaboration: its repository, its name and its reference, or none.

#### Scenario: A note on a version of an elaboration

- **WHEN** a version of a narrative names `--origin elaboration:boom/the-design-book`, a person writes a note on it, and `boom show <name> --json` runs
- **THEN** the note has `elaboration` with `repository` `boom`, `name` `the-design-book` and `reference` `elaboration:boom/the-design-book`, and the words give `elaboration: elaboration:boom/the-design-book` under the note

#### Scenario: A note on a version of no elaboration

- **WHEN** a version of a narrative names no elaboration, a person writes a note on it, and `boom show <name> --json` runs
- **THEN** the note has `elaboration` null

#### Scenario: Each note keeps the elaboration of its version

- **WHEN** a note is on `<v1>`, which names `elaboration:boom/one`, and a later version `<v2>`, which names `elaboration:boom/two`, has a note of its own
- **THEN** `boom show <name> --json` gives the first note with `boom/one` and the second with `boom/two`

### Requirement: A query gives the open notes of an elaboration

`boom query --elaboration <repository>/<name>` SHALL give each open note on a version whose origin names that elaboration, in the order written. Each SHALL have its narrative, its version, its paragraph, its person, its words and its elaboration. It SHALL give no answered note, and no note of another elaboration. Boom SHALL refuse `--elaboration` with `--repository`, `--path` or `--if-declared`. Boom SHALL refuse an elaboration in another form. It SHALL append nothing.

#### Scenario: The open notes of an elaboration

- **WHEN** two narratives name `elaboration:boom/the-design-book`, one has an open note and one a note that a later version answers, a third narrative of `elaboration:boom/other` has an open note, and `boom query --elaboration boom/the-design-book --json` runs
- **THEN** boom exits 0 and gives `notes` with the one open note of the first narrative, with its narrative, version, paragraph, person, words and `elaboration`, and gives `claims` empty

#### Scenario: An elaboration with no open note

- **WHEN** `boom query --elaboration boom/nothing-yet` runs, and no version names that elaboration
- **THEN** boom exits 0 and gives `notes` empty

#### Scenario: An elaboration and a repository

- **WHEN** `boom query --elaboration boom/the-design-book --repository boom` runs
- **THEN** boom exits 2 and says that `boom query` takes `--elaboration` or `--repository`, and not both

#### Scenario: An elaboration out of its form

- **WHEN** `boom query --elaboration the-design-book` runs
- **THEN** boom exits 2 and says that an elaboration is `<repository>/<name>`
