# Spec Delta

## MODIFIED Requirements

### Requirement: An import proposes, all or nothing

An import SHALL name its repository with `--repository <name>`, and SHALL be an agent's: each record one `proposed` entry with no signature, naming `--provenance`, carrying `imported`, where the record came from, the repository and the file among it, and `attach` naming that repository, so a record accepted stands for that repository alone. It SHALL accept nothing, and where any record is refused it SHALL append none.

#### Scenario: An import proposes and accepts nothing

- **WHEN** an import of two records runs with `--provenance run-1`
- **THEN** the log gains two `proposed` entries naming the agent's provenance `run-1` and no `accepted` entry, and each version is proposed

#### Scenario: One record refused, none appended

- **WHEN** an import of three records runs with drafts naming one of them `Bad_Name`
- **THEN** it exits 2 naming `Bad_Name`, and the log is as it was

#### Scenario: No provenance

- **WHEN** an import runs with drafts and no `--provenance`
- **THEN** it exits 2 saying an agent's proposal names its provenance, and nothing is appended

#### Scenario: No repository

- **WHEN** `boom import adr <dir>` runs with no `--repository`
- **THEN** it exits 2 saying an import names its repository, and nothing is appended

#### Scenario: An imported record stands for its repository alone

- **WHEN** `0003-amounts-in-cents.md` is imported with `--repository shop` and accepted, and then `boom show amounts-in-cents --json`, `boom query --repository shop --json` and `boom query --repository billing --json` run
- **THEN** the version's `repositories` is `["shop"]`, the query for `shop` gives `amounts-in-cents`, and the query for `billing` does not
