## MODIFIED Requirements

### Requirement: The repository is named or declared

Boom SHALL find the repository of a session from the first of these that gives one:
- the name that `--repository` gives
- the name that `BOOM_REPOSITORY` gives
- the repository whose registered remote matches a remote of the git repository that holds the working folder

Boom SHALL NOT read `.config/boom.toml`. Boom SHALL refuse a name from `BOOM_REPOSITORY` that the store does not register, and two repositories that match. Where it finds none, it SHALL refuse and say how to name one.

#### Scenario: Declared by the repository

- **WHEN** `boom query` runs in a worktree whose git remote `origin` is `git@github.com:boomdrivedev/sluice.git`, and the store registers `sluice` with that remote
- **THEN** it answers for `sluice`

#### Scenario: Named outright

- **WHEN** `boom query --repository ledger` runs in that worktree
- **THEN** it answers for `ledger`

#### Scenario: Named by the launcher

- **WHEN** `boom query` runs in that worktree with `BOOM_REPOSITORY=flume`
- **THEN** it answers for `flume`

#### Scenario: Neither named nor declared

- **WHEN** `boom query` runs with no `--repository` and no `BOOM_REPOSITORY`, in a repository whose remotes match no registered remote
- **THEN** it exits 2 and says to name the repository with `--repository` or to register its remote with `boom repo add`, and the log is as it was

#### Scenario: A declaration above the repository

- **WHEN** a `.config/boom.toml` that holds `repository = "shop"` lies in the repository, and its remotes match no registered remote
- **THEN** `boom query` reads no repository from the file, and exits 2

#### Scenario: Outside a repository

- **WHEN** `boom query` runs with no `--repository` and no `BOOM_REPOSITORY`, in a folder with no `.git` in it or above it
- **THEN** it exits 2

#### Scenario: A declaration it cannot read

- **WHEN** `boom query` runs with `BOOM_REPOSITORY=nowhere`, and the store registers no `nowhere`
- **THEN** it exits 2 and names `nowhere`

#### Scenario: Two repositories match

- **WHEN** a repository has the remotes of two registered repositories
- **THEN** `boom query` exits 2 and names both

### Requirement: Nothing where nothing is declared, when asked

With `--if-declared`, `boom query` SHALL answer as without the flag when it finds the repository of the session and its delivery at session start is on. In three cases, it SHALL make no store, write nothing to standard error, and exit 0. The cases are no store at its path, no repository found, and delivery off. In words, it SHALL print nothing. With `--json`, it SHALL print `boom-query/2` with `repository` and `head` null and `claims` empty. It SHALL refuse `--repository`.

#### Scenario: Declared

- **WHEN** `boom query --if-declared` runs in a worktree of `sluice`, whose delivery at session start is on, in words and with `--json`
- **THEN** each run prints the same bytes as `boom query` there, and exits 0

#### Scenario: Nothing declared

- **WHEN** `boom query --if-declared` runs in a repository whose remotes match no registered remote, or with no store at the path that it would open
- **THEN** boom exits 0, and standard output and standard error are empty. Boom makes no store

#### Scenario: Nothing declared, outside a repository

- **WHEN** `boom query --if-declared` runs in a folder with no `.git` in it or above it
- **THEN** boom exits 0, and both streams are empty

#### Scenario: Delivery off

- **WHEN** `boom query --if-declared` runs in a worktree of `sluice`, whose delivery at session start is off
- **THEN** boom exits 0, and both streams are empty

#### Scenario: Nothing declared, answered as JSON

- **WHEN** `boom query --if-declared --json` runs in a repository whose remotes match no registered remote
- **THEN** boom prints one object with `format` `boom-query/2`, `repository` null, `head` null and `claims` empty, and exits 0

#### Scenario: A declaration it cannot read

- **WHEN** `boom query --if-declared` runs with `BOOM_REPOSITORY=nowhere`, and the store registers no `nowhere`
- **THEN** boom exits 2 and names `nowhere`, and the log does not change

#### Scenario: Named as well

- **WHEN** `boom query --if-declared --repository shop` runs
- **THEN** boom exits 2 and says that the two flags cannot be given together

### Requirement: Answered in one JSON object, or in words an agent reads

With `--json`, `boom query` SHALL answer `boom-query/2` with `repository`, `head` and `claims`. `head` SHALL hold the `number` and `hash` of the last entry of the log. `claims` SHALL hold the invariants, then the decisions, then the assumptions, each kind by name. In words, `boom query` SHALL write to standard output only: a first line that names the repository, the number of claims and the entry, then each claim. It SHALL append nothing. The same log SHALL give the same answer.

#### Scenario: Answered as JSON

- **WHEN** `boom query --repository shop --json` runs where an invariant and a decision are accepted for `shop`
- **THEN** boom prints one object with `format` `boom-query/2`, `repository` `shop`, the last entry of the log as `head`, and `claims` with the invariant before the decision

#### Scenario: Answered in words

- **WHEN** `boom query --repository shop` runs where 2 claims are accepted for `shop`
- **THEN** boom exits 0. The first line is `Boom: 2 claims for shop, at entry <number>`, with the number of the last entry of the log. A line gives the kind, `<name>@<version>` and statement of each claim. Standard error is empty

#### Scenario: The same log, the same answer

- **WHEN** `boom query --repository shop --json` runs two times, and the views are dropped between the two runs
- **THEN** both runs print the same bytes, and the log does not change
