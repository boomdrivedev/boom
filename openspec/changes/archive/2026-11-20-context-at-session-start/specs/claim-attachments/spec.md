## MODIFIED Requirements

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

### Requirement: An amendment keeps its base's attachment

A proposal that amends a version, and names neither `--repository` nor `--system`, SHALL be attached to what its base is attached to. The repository of the session SHALL NOT change that. A proposal that names either SHALL be attached as it names.

#### Scenario: Amended with nothing named

- **WHEN** a version attached to `shop` stands and an agent proposes an amendment of it naming no repository, in a worktree of `ledger`
- **THEN** the amendment is attached to `shop`

#### Scenario: Amended for another repository

- **WHEN** a version attached to `shop` stands and an amendment of it names `--repository ledger`
- **THEN** the amendment is attached to `ledger` alone
