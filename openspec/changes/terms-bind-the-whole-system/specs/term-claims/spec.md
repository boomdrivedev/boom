## MODIFIED Requirements

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

## ADDED Requirements

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
