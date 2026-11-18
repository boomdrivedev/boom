# Spec Delta

## Purpose

The log every change to a store is written to: entries appended and
never edited, numbered in order, chained so that an entry changed or
removed is found, and each held to boom's checks whenever it is read
(record 0005, points 2, 4 and 6; core.1).

## ADDED Requirements

### Requirement: Every change is an entry appended

Every change to a store SHALL be one entry appended to its log: a person registered, a version proposed, accepted or rejected. An entry SHALL name its format, `boom-entry/1`, its act, its actor and the time it was made, and SHALL never be changed or removed by boom; no command edits or deletes one.

#### Scenario: A proposal and its accept

- **WHEN** a version is proposed and a person accepts it
- **THEN** `boom log --json` gives the store's entries with a `proposed` entry and, after it, an `accepted` entry, and the `proposed` entry is as it was before the accept

#### Scenario: An entry names its format

- **WHEN** `boom log --json` runs on a store
- **THEN** every entry it gives names the format `boom-entry/1`

### Requirement: boom numbers the entries in order

On its own, boom SHALL give each entry the next number of its store's log, from 1, in the order the entries were appended, and a status SHALL depend on that order and never on an entry's time.

#### Scenario: Numbered from one

- **WHEN** a store is made and two claims are proposed
- **THEN** the log's entries are numbered 1, 2 and 3: the first owner's registration, then each proposal in the order it was appended

#### Scenario: A clock set back

- **WHEN** a version is accepted by an entry whose time is earlier than its proposal's
- **THEN** the version stands, read from the order of the entries

### Requirement: The log is chained

Each entry SHALL carry the SHA-256 of the entry before it, the first entry the hash of nothing, and boom SHALL check the whole chain each time it opens a store.

#### Scenario: The chain given

- **WHEN** `boom log --json` runs on a store of three entries
- **THEN** each entry gives its hash, and each after the first gives as its previous hash the hash of the one before it

### Requirement: A changed or removed entry is found

Where the chain does not hold, boom SHALL report the store as altered, naming the first entry whose chain fails, and every command SHALL exit 3 writing nothing and answering nothing else.

#### Scenario: An entry's words changed

- **WHEN** the text of entry 2 of a three-entry store is changed in the file, and `boom show <name>` runs
- **THEN** it exits 3 naming entry 2 as altered

#### Scenario: An entry removed

- **WHEN** entry 2 of a three-entry store is deleted from the file, and `boom propose` runs
- **THEN** it exits 3 naming the entry whose previous hash no longer matches, and the log is as it was

### Requirement: An entry that fails boom's checks counts for nothing

boom SHALL hold every entry it reads to the checks it holds a write to: its form, its signature and its actor's right to the act. An entry that fails SHALL count for nothing in any status, and `boom log` SHALL show it as not counted, with why.

#### Scenario: An accept written by hand

- **WHEN** an agent appends to the log, chained correctly, an `accepted` entry with no signature for a proposed version
- **THEN** `boom show` gives the version as proposed, and `boom log --json` gives that entry as not counted, saying a person's act needs their signature

#### Scenario: An accept signed by a key no one registered

- **WHEN** an `accepted` entry signed by a key the store does not register is appended to the log by hand
- **THEN** it counts for nothing, and `boom log --json` says its key is not registered
