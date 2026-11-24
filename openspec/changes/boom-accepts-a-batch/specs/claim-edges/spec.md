## ADDED Requirements

### Requirement: A version names its cause

`boom propose --cause <claim>` SHALL store the edge `cause`: the version of another claim of the store that forced the new version, resolved as `--source` is. A version SHALL have one cause at most, and the cause SHALL be part of its content. Boom SHALL refuse a cause that the store does not hold, or that names the version's own claim. `boom show <name>@<version>` SHALL give the cause with its statement.

#### Scenario: A revision with its cause

- **WHEN** the store holds `names-no-plan`, and `boom propose invariant money-has-currency "<statement>" --amends <v1> --cause names-no-plan --provenance run-1` runs
- **THEN** boom exits 0, the entry of the proposal holds the edge `cause` to `names-no-plan@<version>`, and `boom show money-has-currency@<version>` gives `cause: names-no-plan@<version>` with its statement

#### Scenario: A cause that the store does not hold

- **WHEN** an agent proposes a claim with `--cause no-such-claim`
- **THEN** boom exits 2 and names `no-such-claim`, and boom appends nothing

#### Scenario: The cause is part of the content

- **WHEN** two versions differ only by their cause
- **THEN** their versions differ
