## MODIFIED Requirements

### Requirement: Each line of the log names its version

`boom log` SHALL give, on each line that proposes, accepts or rejects, each version that the act names as `<name>@<version>`, after the act. A line that registers a person SHALL name no version. With `--json`, each entry SHALL give `targets`, the list of those versions, and `target`: the one version, or `null`. `boom show <number>` SHALL refuse an entry that accepts several versions, and list them.

#### Scenario: A proposal and its accept

- **WHEN** an agent proposes `money-has-currency`, ada accepts it, and `boom log` runs
- **THEN** line 2 starts with `2 proposed money-has-currency@<version> by an agent`, and line 3 is `3 accepted money-has-currency@<version> by ada`

#### Scenario: A registration

- **WHEN** `boom log --json` runs on that store
- **THEN** entry 1 gives `target` `null`, and entries 2 and 3 give `target` `money-has-currency@<version>`

#### Scenario: One accept of three versions

- **WHEN** one entry accepts `<a>@<version>`, `<b>@<version>` and `<c>@<version>`, and `boom log` runs
- **THEN** its line gives `accepted`, then the three versions, separated by commas, then `by ada`. With `--json`, the entry gives `targets` with the three, and `target` `null`

#### Scenario: Shown by its number

- **WHEN** `boom show <number>` runs for that entry
- **THEN** boom exits 2, says that the entry accepts 3 versions, and lists them
