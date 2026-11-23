# Spec Delta

## MODIFIED Requirements

### Requirement: Every change is an entry appended

Every change to a store SHALL be one entry appended to its log: a person registered, a version proposed, accepted or rejected, or a note written. An entry SHALL name its format, `boom-entry/1`, its act, its actor and the time it was made, and SHALL never be changed or removed by boom. No command edits or deletes one.

#### Scenario: A proposal and its accept

- **WHEN** a version is proposed and a person accepts it
- **THEN** `boom log --json` gives the store's entries with a `proposed` entry and, after it, an `accepted` entry, and the `proposed` entry is as it was before the accept

#### Scenario: An entry names its format

- **WHEN** `boom log --json` runs on a store
- **THEN** every entry it gives names the format `boom-entry/1`

#### Scenario: A note is an entry

- **WHEN** a person writes a note on a narrative
- **THEN** `boom log --json` gives a `noted` entry after the `proposed` entry of the version, and the `proposed` entry is as it was before the note
