## MODIFIED Requirements

### Requirement: Every claim that stands for the repository

`boom query` SHALL give each claim whose accepted version binds the repository or the whole system. It SHALL give each claim one time, with its kind, name, version, statement, scenarios, body fingerprint and `repositories`. It SHALL give the person who accepted the version and the number of that entry. It SHALL NOT give a version that is proposed, rejected, superseded or deprecated. It SHALL NOT give a claim whose accepted version binds only other repositories.

#### Scenario: What stands for a repository

- **WHEN** a store holds an accepted invariant that binds `shop`, an accepted decision of the whole system, an accepted assumption that binds `ledger`, and a proposed invariant that binds `shop`
- **THEN** `boom query --repository shop --json` gives the invariant and the decision. It does not give the assumption or the proposed invariant

#### Scenario: An amendment waiting

- **WHEN** a version that binds `shop` is accepted, and an agent proposes an amendment of it that no person accepts
- **THEN** `boom query --repository shop` gives the words of the accepted version, not the words of the amendment

#### Scenario: An amendment accepted

- **WHEN** a person then accepts that amendment
- **THEN** `boom query --repository shop` gives the version and words of the amendment, and not the version that it superseded

#### Scenario: A rejected version

- **WHEN** a person rejects the only version of a claim that binds `shop`
- **THEN** `boom query --repository shop` does not give that claim

#### Scenario: A version that came in as history

- **WHEN** a person accepts together an import for `shop` of `0002-amounts-as-floats.md`, superseded by `0003-amounts-in-cents.md`, and of `0003-amounts-in-cents.md`
- **THEN** `boom query --repository shop --json` gives `amounts-in-cents`. It does not give `amounts-as-floats`, whose version is `superseded`

#### Scenario: Nothing stands

- **WHEN** `boom query --repository shop --json` runs on a store where no claim is accepted for `shop`
- **THEN** boom exits 0, and `claims` is empty

### Requirement: As the person accepted it

`boom query` SHALL answer from the views after the views are at the last entry of the log. It SHALL read the accepted version of each claim, and no other version. An entry that fails the checks of boom SHALL NOT change the answer. When a program edits a view, `boom verify` SHALL find the change and build the view again. After that, `boom query` SHALL give what the person accepted.

#### Scenario: A statement edited in the views

- **WHEN** another program changes the statement of an accepted version in the `versions` table, and does not change `views_head`, and then `boom verify` runs
- **THEN** `boom query` gives the statement that the person accepted

#### Scenario: A status edited in the views

- **WHEN** another program marks a proposed version that binds `shop` as accepted in the `versions` and `claims` tables, and does not change `views_head`, and then `boom verify` runs
- **THEN** `boom query --repository shop` does not give that version

#### Scenario: An accept written by hand

- **WHEN** another program appends an `accepted` entry with no signature, for a proposed version that binds `shop`, chained correctly
- **THEN** `boom query --repository shop` does not give that version

#### Scenario: The views and a full replay agree

- **WHEN** the views of a store stay in step through 200 proposals, accepts and rejects, and `boom query --repository shop --json` runs, then runs again after the views are dropped
- **THEN** both runs print the same bytes

### Requirement: Answered in one JSON object, or in words an agent reads

With `--json`, `boom query` SHALL answer `boom-query/2` with `repository`, `head` and `claims`. `head` SHALL hold the `number` and `hash` of the last entry of the log. `claims` SHALL hold the invariants, then the decisions, then the assumptions, each kind by name. In words, `boom query` SHALL write to standard output only: a first line that names the repository and the entry, then each claim. It SHALL append nothing. The same log SHALL give the same answer.

#### Scenario: Answered as JSON

- **WHEN** `boom query --repository shop --json` runs where an invariant and a decision are accepted for `shop`
- **THEN** boom prints one object with `format` `boom-query/2`, `repository` `shop`, the last entry of the log as `head`, and `claims` with the invariant before the decision

#### Scenario: Answered in words

- **WHEN** `boom query --repository shop` runs where a claim is accepted for `shop`
- **THEN** boom exits 0. The first line names `shop` and the number of the last entry of the log. A line gives the kind, `<name>@<version>` and statement of the claim. Standard error is empty

#### Scenario: The same log, the same answer

- **WHEN** `boom query --repository shop --json` runs two times, and the views are dropped between the two runs
- **THEN** both runs print the same bytes, and the log does not change

### Requirement: Nothing where nothing is declared, when asked

With `--if-declared`, `boom query` SHALL answer as it does without the flag when the working folder declares a repository. When no folder declares one, it SHALL open no store, write nothing to standard error and exit 0. In words, it SHALL print nothing. With `--json`, it SHALL print `boom-query/2` with `repository` and `head` null and `claims` empty. Boom SHALL refuse a declaration that it cannot read, as it does without the flag. Boom SHALL refuse `--if-declared` with `--repository`.

#### Scenario: Declared

- **WHEN** `boom query --if-declared` runs in a repository whose `.config/boom.toml` holds `repository = "shop"`, in words and with `--json`
- **THEN** each run prints the same bytes as `boom query` there, and exits 0

#### Scenario: Nothing declared

- **WHEN** `boom query --if-declared` runs in a repository that holds no `.config/boom.toml`, with no store at the path that it would open
- **THEN** boom exits 0, standard output and standard error are empty, and boom makes no store

#### Scenario: Nothing declared, outside a repository

- **WHEN** `boom query --if-declared` runs in a folder that holds `.config/boom.toml`, with no `.git` in it or above it
- **THEN** boom exits 0, and both streams are empty

#### Scenario: Nothing declared, answered as JSON

- **WHEN** `boom query --if-declared --json` runs in a repository that holds no `.config/boom.toml`
- **THEN** boom prints one object with `format` `boom-query/2`, `repository` null, `head` null and `claims` empty, and exits 0

#### Scenario: A declaration it cannot read

- **WHEN** `boom query --if-declared` runs in a repository whose `.config/boom.toml` holds `repository = "Shop"`
- **THEN** boom exits 2 and names the file, and the log does not change

#### Scenario: Named as well

- **WHEN** `boom query --if-declared --repository shop` runs
- **THEN** boom exits 2 and says that the two flags cannot be given together
