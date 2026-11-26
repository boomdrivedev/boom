# record-log Specification

## Purpose
The log every change to a store is written to: entries appended and
never edited, numbered in order, chained so that an entry changed or
removed is found, and each held to boom's checks whenever it is read
(record 0005, points 2, 4 and 6; core.1).

## Requirements

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

### Requirement: boom numbers the entries in order

On its own, boom SHALL give each entry the next number of its store's log, from 1, in the order the entries were appended, and a status SHALL depend on that order and never on an entry's time.

#### Scenario: Numbered from one

- **WHEN** a store is made and two claims are proposed
- **THEN** the log's entries are numbered 1, 2 and 3: the first owner's registration, then each proposal in the order it was appended

#### Scenario: A clock set back

- **WHEN** a version is accepted by an entry whose time is earlier than its proposal's
- **THEN** the version stands, read from the order of the entries

### Requirement: The log is chained

Each entry SHALL carry the SHA-256 of the entry before it, the first entry the hash of nothing. Each command SHALL check the chain from the last entry the views were built at: that entry SHALL still have the hash the views' head names, and each entry after it SHALL follow the one before it. `boom verify` SHALL check the whole chain.

#### Scenario: The chain given

- **WHEN** `boom log --json` runs on a store of three entries
- **THEN** each entry gives its hash, and each after the first gives as its previous hash the hash of the one before it

#### Scenario: An entry appended by hand that does not follow

- **WHEN** another program appends an entry whose previous hash is not the hash of the log's last entry, and `boom query --repository shop` runs
- **THEN** it exits 3 naming that entry as altered, and the views are as they were

### Requirement: A changed or removed entry is found

Where the chain does not hold, boom SHALL report the store as altered, naming the first entry whose chain fails, and the command SHALL exit 3 writing nothing and answering nothing else. Each command SHALL find a change to the views' head entry or to an entry after it. `boom verify` SHALL find a change to any entry.

#### Scenario: The head entry's words changed

- **WHEN** the text of the last entry of a three-entry store is changed in the file, its views at that entry, and `boom show <name>` runs
- **THEN** it exits 3 naming entry 3 as altered

#### Scenario: An entry's words changed

- **WHEN** the text of entry 2 of a three-entry store is changed in the file, its views at entry 3
- **THEN** `boom show <name>` answers from the views, and `boom verify` exits 3 naming entry 2 as altered

#### Scenario: An entry removed

- **WHEN** entry 2 of a three-entry store is deleted from the file, and `boom verify` runs
- **THEN** it exits 3 naming the entry whose previous hash no longer matches, and the log is as it was

### Requirement: An entry that fails boom's checks counts for nothing

boom SHALL hold every entry it reads to the checks it holds a write to: its form, its signature and its actor's right to the act. An entry that fails SHALL count for nothing in any status, and `boom log` SHALL show it as not counted, with why.

#### Scenario: An accept written by hand

- **WHEN** an agent appends to the log, chained correctly, an `accepted` entry with no signature for a proposed version
- **THEN** `boom show` gives the version as proposed, and `boom log --json` gives that entry as not counted, saying a person's act needs their signature

#### Scenario: An accept signed by a key no one registered

- **WHEN** an `accepted` entry signed by a key the store does not register is appended to the log by hand
- **THEN** it counts for nothing, and `boom log --json` says its key is not registered

### Requirement: Each line of the log names its version

`boom log` SHALL give, on each line that proposes, accepts or rejects, each version that the act names as `<name>@<version>`, after the act. A line that registers a person SHALL name no version. With `--json`, each entry SHALL give `targets`, the list of those versions, and `target`: the one version, or `null`. `boom show <number>` SHALL refuse an entry that accepts several versions, and list them.

#### Scenario: A proposal and its accept

- **WHEN** an agent proposes `money-has-currency`, ada accepts it, and `boom log` runs
- **THEN** line 2 starts with `2 proposed money-has-currency@<version> by an agent`, and line 3 is `3 accepted money-has-currency@<version> by ada`

#### Scenario: A registration

- **WHEN** `boom log --json` runs on that store
- **THEN** entry 1 gives `target` `null`, and entries 2 and 3 give `target` `money-has-currency@<version>`

#### Scenario: One accept of three versions

- **WHEN** one entry accepts `<a>@<version>`, `<b>@<version>` and `<c>@<version>`, and `boom log` runs
- **THEN** its line gives `accepted`, then the three versions, separated by commas, then `by ada`. With `--json`, the entry gives `targets` with the three, and `target` `null`

#### Scenario: Shown by its number

- **WHEN** `boom show <number>` runs for that entry
- **THEN** boom exits 2, says that the entry accepts 3 versions, and lists them

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
