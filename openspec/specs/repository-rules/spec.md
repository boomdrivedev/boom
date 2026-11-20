# repository-rules Specification

## Purpose
What an agent working in a repository gets from boom: every claim that
stands for that repository, in the words a person accepted, computed
from the store's log each time it is asked and written nowhere in the
repository, so no one edits by hand the rules an agent reads (record
0005; record 0008; core.1, core.2).

## Requirements

### Requirement: The repository is named or declared

`boom query` SHALL answer for the repository `--repository <name>` names, or else for the one `.config/boom.toml` declares, read from the nearest folder holding it, from the working folder up to the nearest folder holding `.git`, and outside a repository from none. The file SHALL hold one key, `repository`, a kebab-case name. With neither, or a file that holds anything else, it SHALL refuse, naming the file or saying how to name a repository.

#### Scenario: Declared by the repository

- **WHEN** `boom query` runs in a folder within a repository whose `.config/boom.toml` holds `repository = "shop"`
- **THEN** it answers for `shop`

#### Scenario: Named outright

- **WHEN** `boom query --repository ledger` runs in that repository
- **THEN** it answers for `ledger`

#### Scenario: Neither named nor declared

- **WHEN** `boom query` runs with no `--repository` where no `.config/boom.toml` lies between the working folder and the folder holding `.git`
- **THEN** it exits 2 saying to name the repository with `--repository` or in `.config/boom.toml`, and the log is as it was

#### Scenario: A declaration above the repository

- **WHEN** a `.config/boom.toml` lies only in a folder above the one holding `.git`
- **THEN** `boom query` with no `--repository` reads none, and exits 2

#### Scenario: Outside a repository

- **WHEN** `boom query` runs with no `--repository` in a folder holding `.config/boom.toml`, with no `.git` in it or above it
- **THEN** it reads no declaration, and exits 2

#### Scenario: A declaration it cannot read

- **WHEN** the repository's `.config/boom.toml` holds `repository = "Shop"`, or a key besides `repository`
- **THEN** `boom query` exits 2 naming the file and what it cannot read

### Requirement: Every claim that stands for the repository

`boom query` SHALL give every claim whose standing version is attached to the repository or to the whole system, each once with its kind, name, version, statement, scenarios, body fingerprint, `repositories`, and the person who accepted it and that entry's number. It SHALL give no version proposed, rejected, replaced or history, and no claim whose standing version is attached only to other repositories.

#### Scenario: What stands for a repository

- **WHEN** a store holds a standing invariant attached to `shop`, a standing decision of the whole system, a standing assumption attached to `ledger`, and a proposed invariant attached to `shop`
- **THEN** `boom query --repository shop --json` gives the invariant and the decision, and neither the assumption nor the proposed invariant

#### Scenario: An amendment waiting

- **WHEN** a version attached to `shop` stands and an amendment of it is proposed and not accepted
- **THEN** `boom query --repository shop` gives the standing version's words and not the amendment's

#### Scenario: An amendment accepted

- **WHEN** that amendment is then accepted
- **THEN** `boom query --repository shop` gives the amendment's version and words, and not the version it replaced

#### Scenario: A rejected version

- **WHEN** the only version of a claim attached to `shop` is rejected
- **THEN** `boom query --repository shop` does not give that claim

#### Scenario: A version that came in as history

- **WHEN** an import for `shop` of `0002-amounts-as-floats.md`, superseded by `0003-amounts-in-cents.md`, and of `0003-amounts-in-cents.md` is accepted together
- **THEN** `boom query --repository shop --json` gives `amounts-in-cents` and not `amounts-as-floats`, whose version is history

#### Scenario: Nothing stands

- **WHEN** `boom query --repository shop --json` runs on a store where nothing stands for `shop`
- **THEN** it exits 0 with `claims` empty

### Requirement: As the person accepted it

`boom query` SHALL take its answer from the views, once they are at the log's last entry, as the judge built them from the log. An entry that fails boom's checks SHALL change nothing an agent gets. A view edited by hand SHALL be found and built again by `boom verify`, after which `boom query` gives what the person accepted.

#### Scenario: A statement edited in the views

- **WHEN** a standing version's statement in the `versions` table is changed by another program, `views_head` left as it was, and `boom verify` runs
- **THEN** `boom query` gives the statement the person accepted

#### Scenario: A status edited in the views

- **WHEN** a proposed version attached to `shop` is marked standing in the `versions` and `claims` tables by another program, `views_head` left as it was, and `boom verify` runs
- **THEN** `boom query --repository shop` does not give it

#### Scenario: An accept written by hand

- **WHEN** an `accepted` entry with no signature, for a proposed version attached to `shop`, is appended to the log by another program, chained correctly
- **THEN** `boom query --repository shop` does not give that version

#### Scenario: The views and a full replay agree

- **WHEN** a store's views are kept in step through 200 proposals, accepts and rejects, and `boom query --repository shop --json` runs, then runs again after the views are dropped
- **THEN** both print the same bytes

### Requirement: Answered in one JSON object, or in words an agent reads

With `--json`, `boom query` SHALL answer `boom-query/1` with `repository`, `head`, the log's last entry's `number` and `hash`, and `claims`: invariants, then decisions, then assumptions, each kind by name. In words, it SHALL write to standard output alone a line naming the repository and the entry, then each claim, so its output can be an agent's context as it is. It SHALL append nothing, and the same log SHALL give the same answer.

#### Scenario: Answered as JSON

- **WHEN** `boom query --repository shop --json` runs where an invariant and a decision stand for `shop`
- **THEN** it prints one object with `format` `boom-query/1`, `repository` `shop`, the log's last entry as `head`, and `claims` holding the invariant before the decision

#### Scenario: Answered in words

- **WHEN** `boom query --repository shop` runs where a claim stands for `shop`
- **THEN** it exits 0, its first line names `shop` and the log's last entry's number, a line gives the claim's kind, `<name>@<version>` and statement with its scenarios beneath, and standard error is empty

#### Scenario: The same log, the same answer

- **WHEN** `boom query --repository shop --json` runs twice, its views dropped between the two
- **THEN** both print the same bytes, and the log is as it was before the first

### Requirement: Nothing where nothing is declared, when asked

With `--if-declared`, `boom query` SHALL answer as it does without it where the working folder declares a repository. Where none is declared it SHALL open no store, write nothing to standard error and exit 0, printing nothing in words, and with `--json` `boom-query/1` with `repository` and `head` null and `claims` empty. A declaration it cannot read SHALL be refused as without it, and `--if-declared` with `--repository` refused.

#### Scenario: Declared

- **WHEN** `boom query --if-declared` runs in a repository whose `.config/boom.toml` holds `repository = "shop"`, in words and with `--json`
- **THEN** each prints the same bytes as `boom query` there, and exits 0

#### Scenario: Nothing declared

- **WHEN** `boom query --if-declared` runs in a repository holding no `.config/boom.toml`, with no store at the path it would open
- **THEN** it exits 0, standard output and standard error are empty, and no store is made

#### Scenario: Nothing declared, outside a repository

- **WHEN** `boom query --if-declared` runs in a folder holding `.config/boom.toml` with no `.git` in it or above it
- **THEN** it exits 0 and both streams are empty

#### Scenario: Nothing declared, answered as JSON

- **WHEN** `boom query --if-declared --json` runs in a repository holding no `.config/boom.toml`
- **THEN** it prints one object with `format` `boom-query/1`, `repository` null, `head` null and `claims` empty, and exits 0

#### Scenario: A declaration it cannot read

- **WHEN** `boom query --if-declared` runs in a repository whose `.config/boom.toml` holds `repository = "Shop"`
- **THEN** it exits 2 naming the file, and the log is as it was

#### Scenario: Named as well

- **WHEN** `boom query --if-declared --repository shop` runs
- **THEN** it exits 2, saying the two cannot be given together
