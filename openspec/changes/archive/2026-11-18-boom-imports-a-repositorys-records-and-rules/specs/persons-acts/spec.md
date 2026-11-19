## ADDED Requirements

### Requirement: Several versions accepted together

`boom accept` SHALL take several `<name>@<version>` targets with one `--key`, judge every one before the key signs any, sign each as an `accepted` entry of its own, and append them all in one transaction or none. A refusal of any SHALL name it, with nothing signed or appended. Its answer SHALL list, under `accepted`, each version as an accept of it alone gives it.

#### Scenario: Three accepted together

- **WHEN** ada runs `boom accept <a>@<version> <b>@<version> <c>@<version> --key <ada's key> --json` for three versions an agent proposed
- **THEN** it exits 0 listing the three under `accepted`, each stands, and the log gains three `accepted` entries, each signed by ada

#### Scenario: One of them refused

- **WHEN** one of the three is a version ada proposed with ada's key
- **THEN** it exits 2 naming that version, ada's key signs nothing, and the log is as it was
