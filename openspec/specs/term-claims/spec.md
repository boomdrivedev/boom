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

A claim of the kind `term` SHALL have its word as its name, and the meaning of the word as its statement. A term version MAY have a body and MAY have `replaces`. A term SHALL bind one repository, and only one. Two terms MAY have one word when they bind two repositories. Boom SHALL refuse a first term version that binds no repository, binds more than one, or binds the whole system. An amendment of a term SHALL bind the repository of its base.

#### Scenario: A term proposed

- **WHEN** `boom propose term owner "A person who grants roles and registers keys in a store." --repository boom --replaces keeper --provenance run-1` runs <!-- boom: allow keeper -->
- **THEN** boom exits 0, and `boom show owner --json` gives a term with one proposed version that binds `boom`

#### Scenario: A term with no repository

- **WHEN** an agent proposes a term with no `--repository`, in a folder that declares no repository
- **THEN** boom exits 2 and says that a term binds one repository, and boom appends nothing

#### Scenario: A term for two repositories

- **WHEN** an agent proposes a term with `--repository boom --repository sluice`
- **THEN** boom exits 2 and says that a term binds one repository, and boom appends nothing

#### Scenario: One word in two repositories

- **WHEN** the term `owner` is accepted for `boom`, and an agent proposes the term `owner` for `sluice` with another meaning
- **THEN** boom exits 0, and `boom show owner --json` gives two terms, one for `boom` and one for `sluice`

#### Scenario: A term with no replaces

- **WHEN** `boom propose term operator "The person who runs a host." --repository boomdrive --provenance run-1` runs
- **THEN** boom exits 0, and the entry of the proposal holds no `replaces`

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
