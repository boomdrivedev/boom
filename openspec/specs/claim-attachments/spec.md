# claim-attachments Specification

## Purpose
What a claim's version binds: the repositories its proposal names, or
the whole system where it names none. An attachment rides with its
version, takes effect only when a person accepts that version, and is
no part of the version's content (interview:
`retire-attach-and-rules-need-a-person`, `scope-crosses-by-homes`;
record 0005, point 5; record 0008).

## Requirements

### Requirement: A proposal names what its version binds

The entry of a proposal SHALL hold its attachment after its claim: `attach`, with `repositories` in kebab case, sorted, each one time. An entry with no `attach` SHALL bind the whole system. `boom propose` SHALL attach a version to the first of these:
- the repositories that `--repository` names
- the whole system, with `--system`
- what an amended base is attached to
- the repository of the session, as `boom query` finds it
- the whole system

#### Scenario: Bound to one repository

- **WHEN** an agent runs `boom propose invariant money-has-currency "Every amount names its currency." --repository shop --provenance run-1`
- **THEN** it exits 0 giving the version attached to `shop`, and the entry's signed part holds `attach` with `repositories` `["shop"]` after its `claim`

#### Scenario: Several repositories named

- **WHEN** a proposal names `--repository shop --repository ledger --repository shop`
- **THEN** its entry's `repositories` are `["ledger", "shop"]`

#### Scenario: Bound to the whole system

- **WHEN** a proposal runs with `--system`, or with no `--repository` in a folder where boom finds no repository of the session
- **THEN** its entry holds no `attach`, and the answer gives the version as the whole system's

#### Scenario: The repository the working folder declares

- **WHEN** an agent proposes a first version with no `--repository` and no `--system`, in a worktree whose remote the store registers for `shop`
- **THEN** the version is attached to `shop`

#### Scenario: A repository name not in kebab case

- **WHEN** a proposal names `--repository Shop_Front`
- **THEN** it exits 2 saying a repository's name is kebab case, and the log is as it was

#### Scenario: Both a repository and the whole system

- **WHEN** a proposal names `--repository shop` and `--system`
- **THEN** it exits 2, and the log is as it was

### Requirement: An attachment is no part of a version

A version's fingerprint SHALL be taken from its content alone and never from its attachment, so a version proposed before attachments keeps its fingerprint. The same content proposed again with the same attachment SHALL add nothing and answer as the first did; with another attachment it SHALL be refused, naming the attachment the version holds.

#### Scenario: One fingerprint wherever it binds

- **WHEN** the same claim, in the same words, is proposed to one store attached to `shop` and to another attached to the whole system
- **THEN** both answers give the same version

#### Scenario: Proposed again for the same repository

- **WHEN** the same proposal attached to `shop` runs twice
- **THEN** the second answers as the first did, and the log holds one entry for it

#### Scenario: The same words for another repository

- **WHEN** a version attached to `shop` is proposed, then the same claim in the same words is proposed attached to `ledger`
- **THEN** the second exits 2 naming `shop` as what the version is attached to, and the log is as after the first

### Requirement: An amendment keeps its base's attachment

A proposal that amends a version, and names neither `--repository` nor `--system`, SHALL be attached to what its base is attached to. The repository of the session SHALL NOT change that. A proposal that names either SHALL be attached as it names.

#### Scenario: Amended with nothing named

- **WHEN** a version attached to `shop` stands and an agent proposes an amendment of it naming no repository, in a worktree of `ledger`
- **THEN** the amendment is attached to `shop`

#### Scenario: Amended for another repository

- **WHEN** a version attached to `shop` stands and an amendment of it names `--repository ledger`
- **THEN** the amendment is attached to `ledger` alone

### Requirement: A version shown gives what it binds

`boom show <name>` SHALL give each version's attachment, and `boom show <name>@<version>` that version's, as `repositories`, an empty list for the whole system, so the person who accepts a version reads what it will bind.

#### Scenario: A version shown

- **WHEN** `boom show money-has-currency@<version> --json` runs for a version attached to `shop`
- **THEN** it gives `repositories` `["shop"]` beside the version's statement and status

#### Scenario: A claim shown

- **WHEN** `boom show money-has-currency --json` runs for a claim whose first version binds the whole system and whose amendment is attached to `shop`
- **THEN** it gives the first version's `repositories` as `[]` and the amendment's as `["shop"]`

### Requirement: An attachment is judged at the write and on every read

boom SHALL hold an entry's attachment to these rules wherever it reads the entry: an `attach` with no repository, a name not in kebab case, names out of order or named twice, or a repeat of a version under another attachment SHALL make the entry count for nothing, and `boom log` SHALL say why.

#### Scenario: A repository named twice by hand

- **WHEN** a proposal whose `attach` names `shop` twice is appended to the log by another program, chained correctly
- **THEN** `boom log --json` gives the entry as not counted, saying why, and `boom show` has no such version
