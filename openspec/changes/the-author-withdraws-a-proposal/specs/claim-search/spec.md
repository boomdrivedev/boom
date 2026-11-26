## ADDED Requirements

### Requirement: The versions that wait

`boom search --proposed [<words>...]` SHALL search the proposed versions in place of the accepted ones, in the same repository, the repository of the session, or every repository with `--all`. With words, it SHALL rank the matches as a search does. With no word, it SHALL give each version that waits, the oldest first. Each version SHALL hold `status`, `proposer`, `proposed_in`, `proposed_at` and `age_days`, its age in whole days by the clock of the machine. The answer SHALL hold `proposed` true.

#### Scenario: Each version that waits

- **WHEN** an agent proposed two versions for `shop`, a person accepted one more, and an agent withdrew one more, and `boom search --proposed --repository shop --json` runs
- **THEN** boom exits 0 with `total` 2 and `proposed` true, the older version first, each with `status` `proposed`, its proposer and the entry that proposed it

#### Scenario: The age of a version

- **WHEN** an agent proposed a version three days before the clock of the machine
- **THEN** `boom search --proposed --all --json` gives it with `age_days` 3, and in words with "3 days"

#### Scenario: Words among the versions that wait

- **WHEN** two versions wait for `shop`, and only one holds `refund`
- **THEN** `boom search --proposed refund --repository shop --json` gives that one, and not the other

#### Scenario: An old draft does not wait

- **WHEN** `<v3>` amends the draft `<v2>`
- **THEN** `boom search --proposed --all --json` gives `<v3>` and not `<v2>`

#### Scenario: No word with no `--proposed`

- **WHEN** `boom search --repository shop` runs with no word
- **THEN** it exits 2, and the log is as it was
