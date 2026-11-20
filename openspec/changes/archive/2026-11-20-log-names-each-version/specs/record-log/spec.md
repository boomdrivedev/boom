## ADDED Requirements

### Requirement: Each line of the log names its version

`boom log` SHALL give, on each line that proposes, accepts or rejects a version, the name of the claim and the version as `<name>@<version>`, after the act. A line that registers a person SHALL name no version. With `--json`, each entry SHALL give `target`: that `<name>@<version>`, or `null`.

#### Scenario: A proposal and its accept

- **WHEN** an agent proposes `money-has-currency`, ada accepts it, and `boom log` runs
- **THEN** line 2 starts with `2 proposed money-has-currency@<version> by an agent`, and line 3 is `3 accepted money-has-currency@<version> by ada`

#### Scenario: A registration

- **WHEN** `boom log --json` runs on that store
- **THEN** entry 1 gives `target` `null`, and entries 2 and 3 give `target` `money-has-currency@<version>`
