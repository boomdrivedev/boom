## MODIFIED Requirements

### Requirement: The repository it works in, or every repository

`boom search` SHALL search the claims that stand for one repository: the one that `--repository` names, or else the repository of the session, as `boom query` finds it. It SHALL search each version attached to that repository or to the whole system. With `--all`, it SHALL search each claim that stands. With none of these, it SHALL refuse and say how to name a repository. It SHALL refuse `--all` with `--repository`.

#### Scenario: In the repository it works in

- **WHEN** invariants holding `refund` stand, one attached to `shop`, one to the whole system and one to `ledger`
- **THEN** `boom search refund --repository shop --json` gives the first two and not the third

#### Scenario: Declared by the repository

- **WHEN** `boom search refund` runs in a worktree whose remote the store registers for `shop`
- **THEN** it searches for `shop`

#### Scenario: Across every repository

- **WHEN** `boom search refund --all --json` runs on that store
- **THEN** it gives all three invariants, each with its `repositories`, and `repository` is `null`

#### Scenario: Neither named, declared nor all

- **WHEN** `boom search refund` runs with no `--repository` and no `--all`, where boom finds no repository of the session
- **THEN** it exits 2, and the log is as it was. It says to name the repository with `--repository`, to register its remote with `boom repo add`, or to search every repository with `--all`

#### Scenario: A repository and every repository

- **WHEN** `boom search refund --repository shop --all` runs
- **THEN** it exits 2, and the log is as it was
