## MODIFIED Requirements

### Requirement: Every claim that stands for the repository

`boom query` SHALL give each claim whose accepted version binds the repository or the whole system. It SHALL give each claim one time, with its kind, name, version, statement, scenarios, body fingerprint and `repositories`. It SHALL give the person who accepted the version and the number of that entry. It SHALL NOT give `replaces`. It SHALL NOT give a version that is proposed, rejected, superseded or deprecated. It SHALL NOT give a claim whose accepted version binds only other repositories.

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

#### Scenario: A term of the repository

- **WHEN** a term `refund` that replaces `chargeback` is accepted for `shop`
- **THEN** `boom query --repository shop --json` gives the term `refund` with its meaning as its statement, and no field that holds `chargeback`

### Requirement: Answered in one JSON object, or in words an agent reads

With `--json`, `boom query` SHALL answer `boom-query/2` with `repository`, `head` and `claims`. `head` SHALL hold the `number` and `hash` of the last entry of the log. `claims` SHALL hold the invariants, the decisions, the assumptions and then the terms, each kind by name. In words, `boom query` SHALL write to standard output only: a first line that names the repository, the number of claims and the entry, then each claim. It SHALL append nothing. The same log SHALL give the same answer.

#### Scenario: Answered as JSON

- **WHEN** `boom query --repository shop --json` runs where an invariant and a decision are accepted for `shop`
- **THEN** boom prints one object with `format` `boom-query/2`, `repository` `shop`, the last entry of the log as `head`, and `claims` with the invariant before the decision

#### Scenario: Answered in words

- **WHEN** `boom query --repository shop` runs where 2 claims are accepted for `shop`
- **THEN** boom exits 0. The first line is `Boom: 2 claims for shop, at entry <number>`, with the number of the last entry of the log. A line gives the kind, `<name>@<version>` and statement of each claim. Standard error is empty

#### Scenario: The same log, the same answer

- **WHEN** `boom query --repository shop --json` runs two times, and the views are dropped between the two runs
- **THEN** both runs print the same bytes, and the log does not change

#### Scenario: Terms after the other claims

- **WHEN** `boom query --repository shop` runs where an invariant and the term `refund` are accepted for `shop`
- **THEN** the line of the term comes after the line of the invariant, and gives `term`, `refund@<version>` and its meaning
