# term-claims Specification

## Purpose
What a term is: a word and its meaning in one repository. A term is a
claim that a person accepts, with the old words that it replaces. The
old words stay in the log. Boom gives them to the person who accepts.
Boom never gives them to an agent that loads its words
(`a-term-binds-one-repository`, `status-read-from-events` and
`views-computed-from-the-ledger`).

## Requirements

### Requirement: A term names a word and its meaning

A claim of the kind `term` SHALL have its word as its name, and the meaning of the word as its statement. A term version MAY have a body and MAY have `replaces`. A term SHALL bind one repository or the whole system. Boom SHALL refuse a first term version that binds two repositories or more, or that names no binding in a folder of no repository. An amendment of a term SHALL bind what its base binds.

#### Scenario: A term proposed

- **WHEN** `boom propose term owner "A person who grants roles and registers keys in a store." --repository boom --replaces keeper --provenance run-1` runs <!-- boom: allow keeper -->
- **THEN** boom exits 0, and `boom show owner --json` gives a term with one proposed version that binds `boom`

#### Scenario: A term of the whole system

- **WHEN** `boom propose term link "One hop of a lineage." --system --replaces hop --provenance run-1` runs
- **THEN** boom exits 0, and `boom show link --json` gives a term with no `repository`, whose proposed version binds no repository and no context

#### Scenario: A term with no repository

- **WHEN** an agent proposes a term with no `--repository` and no `--system`, in a folder that declares no repository
- **THEN** boom exits 2 and says that a term binds one repository or the whole system, and boom appends nothing

#### Scenario: A term for two repositories

- **WHEN** an agent proposes a term with `--repository boom --repository sluice`
- **THEN** boom exits 2 and says that a term binds one repository or the whole system, and boom appends nothing

#### Scenario: One word in two repositories

- **WHEN** the term `owner` is accepted for `boom`, and an agent proposes the term `owner` for `sluice` with another meaning
- **THEN** boom exits 0, and `boom show owner --json` gives two terms, one for `boom` and one for `sluice`

#### Scenario: A term with no replaces

- **WHEN** `boom propose term operator "The person who runs a host." --repository boomdrive --provenance run-1` runs
- **THEN** boom exits 0, and the entry of the proposal holds no `replaces`

#### Scenario: An amendment of a term of the whole system

- **WHEN** the term `link` of the whole system is proposed, and an agent proposes the term `link` with `--amends <its version>` and no binding
- **THEN** boom exits 0, and the new version binds the whole system

#### Scenario: An amendment across bindings

- **WHEN** the term `link` of the whole system is proposed, and an agent proposes the term `link` with `--amends <its version>` and `--repository boom`
- **THEN** boom exits 2 and says that a term amends a version of its own binding, and boom appends nothing

### Requirement: What a term replaces

Each word in `replaces` SHALL be one line, and SHALL NOT be empty. It SHALL NOT be the word of its term. Boom SHALL keep each word of `replaces` one time, in lowercase, in the order of the alphabet. Boom SHALL refuse `replaces` on a claim of another kind. A word in `replaces` MAY be two or more words, such as `drive log`.

#### Scenario: Replaces on an invariant

- **WHEN** an agent proposes an invariant with `--replaces keeper` <!-- boom: allow keeper -->
- **THEN** boom exits 2 and says that only a term replaces words, and boom appends nothing

#### Scenario: A term that replaces its own word

- **WHEN** an agent proposes the term `owner` with `--replaces Owner`
- **THEN** boom exits 2 and says that a term does not replace its own word, and boom appends nothing

#### Scenario: The same words in another order

- **WHEN** an agent proposes the term `owner` with `--replaces keeper --replaces Warden`, and then with `--replaces warden --replaces keeper` <!-- boom: allow keeper -->
- **THEN** both runs give the same version, and its `replaces` is `["keeper", "warden"]` <!-- boom: allow keeper -->

### Requirement: The old words stay in the log

Boom SHALL keep `replaces` in the entry of the proposal, and SHALL give it in `boom show` and `boom log`. `boom query` and `boom search` SHALL NOT give `replaces`. `boom search` SHALL NOT find a claim by a word of `replaces`. Boom SHALL write no list of old words into a repository.

#### Scenario: Shown to the person who accepts

- **WHEN** `boom show owner@<version> --json` runs for a term version that replaces `keeper` <!-- boom: allow keeper -->
- **THEN** it gives `replaces` `["keeper"]` <!-- boom: allow keeper -->

#### Scenario: Not given to an agent

- **WHEN** a person accepts that term for `boom`, and `boom query --repository boom --json` runs
- **THEN** the answer gives the term `owner` with its meaning, and no byte of the answer is `keeper` <!-- boom: allow keeper -->

#### Scenario: Not searched

- **WHEN** a person accepts that term for `boom`, and no other field of an accepted claim for `boom` holds `keeper` <!-- boom: allow keeper -->
- **THEN** `boom search keeper --repository boom --json` gives no claim <!-- boom: allow keeper -->

### Requirement: The term of a repository stands over the term of the whole system

Where one word has a standing term of the whole system and a standing term of a repository, boom SHALL give the term of the repository for that repository, and SHALL NOT give the term of the whole system for it. `boom query`, `boom search` with `--repository`, the hooks and `boom terms check` SHALL follow this. Each other repository SHALL get the term of the whole system. `boom search --all` SHALL give both terms.

#### Scenario: The term of the whole system in a repository

- **WHEN** the term `link` of the whole system is accepted, and `boom query --repository sluice --json` runs
- **THEN** the answer gives the term `link` with its meaning

#### Scenario: A repository with its own term

- **WHEN** the term `link` of the whole system and the term `link` of `boom`, with another meaning, are accepted, and `boom query --repository boom --json` runs
- **THEN** the answer gives one term `link`, with the meaning of the term of `boom`

#### Scenario: Searched in a repository and in all

- **WHEN** the term `link` of the whole system and the term `link` of `boom` are accepted, and `boom search link --repository boom --json` and `boom search link --all --json` run
- **THEN** the search of `boom` gives the term of `boom` only, and the search of all gives both terms
