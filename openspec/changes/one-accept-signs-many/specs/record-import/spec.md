## MODIFIED Requirements

### Requirement: Accepted together

The answer of an import SHALL give the versions to accept together. These SHALL be all the versions that it proposed, except those whose file has the status `proposed`. The answer SHALL give the command that accepts them: `boom accept <name>@<version>... --key <key>`. When a person accepts them, each version SHALL be `accepted`. A version that came in as history SHALL get the status of its file.

#### Scenario: An import accepted together

- **WHEN** ada runs, with the key of ada, the accept that an import gives for `0001-record-architecture-decisions.md`, for `0002-amounts-as-floats.md` superseded by `0003-amounts-in-cents.md`, and for `0003-amounts-in-cents.md`
- **THEN** `record-architecture-decisions` and `amounts-in-cents` are `accepted`, `amounts-as-floats` is `superseded`, and the log gets one `accepted` entry that ada signed and that names the three versions

#### Scenario: A record left proposed waits

- **WHEN** the folder of an import holds `0004-one-database.md` with `status: proposed`, and the accept of the import runs
- **THEN** that version is not in the versions to accept together, and it stays proposed
