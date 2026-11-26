## ADDED Requirements

### Requirement: Each entry has an act hash

The act hash of an entry SHALL be the SHA-256 of its signed part, 64 lowercase hexadecimal characters. It SHALL leave out the entry's number, the hash of the entry before it and the signature. No entry SHALL store its own act hash. `boom log --json` SHALL give each entry's `act_hash`, and `links`, the act hashes that the entry links, or an empty list.

#### Scenario: The act hash of an entry

- **WHEN** `boom log --json` runs on a store of three entries
- **THEN** each entry gives `act_hash`, the SHA-256 of its signed part, and no two entries give the same `act_hash`

#### Scenario: The act hash leaves out the chain

- **WHEN** two stores hold the same signed part at different numbers, after different entries
- **THEN** `boom log --json` gives the same `act_hash` for both, and a different `hash`

#### Scenario: An entry of rule set 0

- **WHEN** `boom log --json` runs on a store whose entries name no rule set
- **THEN** each entry gives the SHA-256 of its signed part as stored as its `act_hash`

### Requirement: A link names an entry that the log holds

Boom SHALL refuse an entry whose link names an act hash that no counted entry before it in the log has. Each link SHALL be 64 lowercase hexadecimal characters, and an entry SHALL name each link once. On a replay, an entry that breaks this SHALL count for nothing, and `boom log` SHALL show it as not counted, with the link that it names.

#### Scenario: A link to an entry that the log does not hold

- **WHEN** `boom append sluice.landed --link <64 hexadecimal characters that no entry has> --provenance run-1` runs
- **THEN** boom exits 2, names the link, and appends nothing

#### Scenario: A short link

- **WHEN** `boom append sluice.landed --link <12 hexadecimal characters> --provenance run-1` runs
- **THEN** boom exits 2 and says that a link is the 64 characters of an act hash, and boom appends nothing

#### Scenario: A link given two times

- **WHEN** `boom append sluice.landed --link <h> --link <h> --provenance run-1` runs, and `<h>` is the act hash of an entry of the log
- **THEN** boom exits 0, and the entry names `<h>` once

#### Scenario: A link to a later entry, appended by hand

- **WHEN** another program appends, chained correctly, an entry whose link names the act hash of an entry after it, and `boom verify` runs
- **THEN** boom counts the entry for nothing, and `boom log --json` gives it as not counted, naming the link
