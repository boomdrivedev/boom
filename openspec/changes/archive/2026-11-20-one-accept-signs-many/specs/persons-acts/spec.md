## MODIFIED Requirements

### Requirement: Several versions accepted together

`boom accept` SHALL take several `<name>@<version>` targets with one `--key`. It SHALL write one `accepted` entry that names each version that waits, in the order given. Boom SHALL judge that entry whole before the key signs it, and the key SHALL sign it one time. A refusal of any version SHALL name it, and nothing is signed or appended. Boom SHALL refuse an accept that names one claim two times. Its answer SHALL list each version under `accepted`.

#### Scenario: Three accepted together

- **WHEN** ada runs `boom accept <a>@<version> <b>@<version> <c>@<version> --key <ada's key> --json` for three versions an agent proposed
- **THEN** it exits 0 and lists the three under `accepted`, and each is accepted. The key of ada signs one time, and the log gets one `accepted` entry that names the three

#### Scenario: One of them refused

- **WHEN** one of the three is a version ada proposed with ada's key
- **THEN** it exits 2 naming that version, ada's key signs nothing, and the log is as it was

#### Scenario: Accepted again

- **WHEN** ada runs that accept again
- **THEN** it exits 0 and lists the three under `accepted`, the key signs nothing, and the log does not change

#### Scenario: One claim two times

- **WHEN** ada runs `boom accept` with two versions of `money-has-currency`
- **THEN** it exits 2 and says that it names `money-has-currency` two times, and the log does not change

#### Scenario: An entry with one bad version

- **WHEN** another program appends an entry, signed by ada, that accepts three proposed versions and a version that the store does not hold
- **THEN** the entry counts for nothing, and each of the three stays proposed

#### Scenario: The views and a replay agree

- **WHEN** one entry accepts three versions, and `boom verify` runs
- **THEN** `boom show` gives each version as accepted by that entry, before and after `boom verify`
