## MODIFIED Requirements

### Requirement: One name space

Two claims of a store SHALL NOT have the same name, except terms of one word, each of which binds another repository or the whole system. Boom SHALL refuse a first version whose name a claim of another kind has, and SHALL name that claim. Boom SHALL refuse a first version whose claim has an accepted version, and SHALL name that version, to be amended. Several first versions MAY wait together while no version of the claim is accepted.

#### Scenario: A name taken by another kind

- **WHEN** an agent proposes a decision `billing` while an invariant `billing` exists
- **THEN** boom exits 2 and names the invariant `billing`, and boom appends nothing

#### Scenario: A name whose claim stands

- **WHEN** an agent proposes a first version of `money-has-currency` while a version of it is accepted
- **THEN** boom exits 2, names the accepted version and says to amend it, and boom appends nothing

#### Scenario: Two first versions waiting

- **WHEN** an agent proposes two first versions of one invariant, with different statements, while no version is accepted
- **THEN** boom appends both, and both wait for a person

#### Scenario: A term with the name of an invariant

- **WHEN** an agent proposes the term `billing` for `shop` while an invariant `billing` exists
- **THEN** boom exits 2 and names the invariant `billing`, and boom appends nothing

#### Scenario: A term of the whole system with the name of an invariant

- **WHEN** an agent proposes the term `billing` with `--system` while an invariant `billing` exists
- **THEN** boom exits 2 and names the invariant `billing`, and boom appends nothing

#### Scenario: A word of the whole system and of a repository

- **WHEN** the term `link` of the whole system is accepted, and an agent proposes the term `link` for `boom`
- **THEN** boom exits 0, and the store holds two terms `link`

### Requirement: A word shown for each repository

For a word with two terms or more, `boom show <word>` SHALL give each term, one after the other: the term of the whole system first, then the term of each repository, in the order of their repositories. `boom show <word> --repository <name>` SHALL give the term of that repository only. Boom SHALL refuse `--repository` for a repository where the word has no term.

#### Scenario: A word with a term in two repositories

- **WHEN** the term `owner` binds `boom`, another term `owner` binds `sluice`, and `boom show owner --json` runs
- **THEN** it gives two terms, the term of `boom` and then the term of `sluice`

#### Scenario: One repository chosen

- **WHEN** `boom show owner --repository sluice --json` runs
- **THEN** it gives the term of `sluice` only

#### Scenario: A word of the whole system and of a repository shown

- **WHEN** the term `link` binds the whole system, another term `link` binds `boom`, and `boom show link --json` runs
- **THEN** it gives two terms, the term of the whole system with no `repository`, and then the term of `boom`
