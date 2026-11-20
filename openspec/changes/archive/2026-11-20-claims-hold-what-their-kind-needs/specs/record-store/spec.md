## MODIFIED Requirements

### Requirement: A command answers from the views

A command SHALL answer from the views after the views are at the last entry of the log. It SHALL check that the head of the views names that entry and its hash. It SHALL NOT read the entries before the head. It SHALL read only the view rows that it needs:

- `boom query` and `boom search`: the accepted version of each claim.
- A write: each person, the claims that its entries name, and for an import the claims with an imported version.

A write SHALL judge its entries against those rows.

#### Scenario: A large log

- **WHEN** the log of a store holds 20 000 entries, its views are at its last entry, and `boom query --repository shop --json` runs
- **THEN** boom reads one entry of the log, the last, and answers

#### Scenario: A large log, three entries behind

- **WHEN** another program appends three entries to that log, and `boom query --repository shop --json` runs
- **THEN** boom reads the head entry of the views and the three entries after it, and no other entry of the log

#### Scenario: A write on a large log

- **WHEN** `boom propose` runs on that log, with its views at its last entry
- **THEN** boom reads the last entry of the log, and no other, before it appends. It reads as many entries as it does on a log of ten entries

#### Scenario: A query reads the accepted versions only

- **WHEN** a claim for `shop` has 20 versions, one of them accepted, and `boom query --repository shop --json` runs
- **THEN** boom reads one version of that claim from the views

#### Scenario: A write reads the claims it names

- **WHEN** a store holds two claims of 20 versions each, and an agent proposes an amendment of one of them
- **THEN** boom reads the versions of that claim from the views, and no version of the other claim
