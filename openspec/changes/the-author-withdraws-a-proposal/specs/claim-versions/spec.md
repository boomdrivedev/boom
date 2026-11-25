## MODIFIED Requirements

### Requirement: Each version's status is read from the log

Boom SHALL read the status of a version from the log: `proposed` until an accept, a reject or a withdrawal counts; `accepted` while no later version of its claim is accepted; `superseded` after one is; `rejected` after a person rejects it; `withdrawn` after its author withdraws it. An accepted version that came in as history SHALL take the status of its file. No act makes a version `deprecated` yet. The log SHALL keep each entry.

#### Scenario: Proposed, standing, replaced

- **WHEN** a person accepts a version, then an agent proposes an amendment of it, and a person accepts the amendment
- **THEN** `boom show <name> --json` gives the first version as `superseded`, the amendment as `accepted`, and the amendment as the accepted version of the claim

#### Scenario: Rejected

- **WHEN** a person rejects a proposed version, with a reason
- **THEN** `boom show <name> --json` gives it as `rejected`, with the person who rejected it and the reason

#### Scenario: Withdrawn

- **WHEN** an agent withdraws a version that an agent proposed, with a reason
- **THEN** `boom show <name> --json` gives it as `withdrawn`, with the entry that withdrew it and the reason, and with no person who accepted or rejected it

#### Scenario: History accepted

- **WHEN** a person accepts a version that came in as history from a file whose status is superseded
- **THEN** `boom show <name> --json` gives that version as `superseded`, and gives the claim no accepted version

## ADDED Requirements

### Requirement: The author withdraws a proposed version

`boom withdraw <name>@<version>... --reason <text>` SHALL append one `withdrawn` entry with no signature: an agent's with `--provenance`, or a person's with `--person`. Boom SHALL count it only when each version is proposed and its author withdraws it: an agent, a version that an agent proposed; a person, a version that this person proposed. Boom SHALL judge the entry whole. Boom SHALL refuse a withdrawal with no reason or with `--key`.

#### Scenario: An agent withdraws its proposal

- **WHEN** an agent proposed a version, and `boom withdraw <name>@<version> --reason "a duplicate" --provenance run-2 --json` runs
- **THEN** boom exits 0 and lists the version under `withdrawn`, the log gets one `withdrawn` entry of an agent with no signature, the version is `withdrawn`, and `boom accept --waiting` does not list it

#### Scenario: Several withdrawn together

- **WHEN** an agent proposed three versions, and `boom withdraw` names the three with one reason
- **THEN** boom exits 0, the log gets one `withdrawn` entry that names the three, and each is `withdrawn`

#### Scenario: A person withdraws their proposal

- **WHEN** ada proposed a version with ada's key, and `boom withdraw <name>@<version> --reason "no longer needed" --person ada` runs
- **THEN** boom exits 0, the entry names ada and carries no signature, and the version is `withdrawn`

#### Scenario: Another author's proposal

- **WHEN** ada proposed a version, and an agent, or bo with `--person bo`, withdraws it
- **THEN** boom exits 2 and says that only its author withdraws it, and boom appends nothing

#### Scenario: A version that is not proposed

- **WHEN** a version is accepted, and an agent withdraws it
- **THEN** boom exits 2 and says that only a proposed version is withdrawn, and boom appends nothing

#### Scenario: An old draft withdrawn

- **WHEN** `<v3>` amends the draft `<v2>`, and an agent withdraws `<v2>`
- **THEN** boom exits 2 and names `<v3>` as the draft that superseded it, and boom appends nothing

#### Scenario: One of several refused

- **WHEN** an agent withdraws two versions that an agent proposed and one version that is accepted
- **THEN** boom exits 2 and names the accepted version, and each of the two stays `proposed`

#### Scenario: No reason

- **WHEN** `boom withdraw <name>@<version> --provenance run-2` runs with no `--reason`
- **THEN** boom exits 2 and says that a withdrawal gives its reason, and boom appends nothing

#### Scenario: A key given

- **WHEN** `boom withdraw <name>@<version> --reason "a duplicate" --key <ada's key>` runs
- **THEN** boom exits 2 and says that a withdrawal carries no signature, and boom appends nothing

#### Scenario: Withdrawn again

- **WHEN** an agent withdraws a version, then runs the same withdrawal again
- **THEN** the second run exits 0, and the log does not change

#### Scenario: A withdrawal from another program

- **WHEN** another program appends a `withdrawn` entry of an agent that names a version that a person proposed, chained correctly
- **THEN** the entry counts for nothing, and the version stays `proposed`

### Requirement: A withdrawn version is closed

Boom SHALL refuse an accept, a reject or a withdrawal of a withdrawn version, and SHALL refuse the accept of a narrative that includes one. A withdrawn version SHALL answer no note. A new version MAY amend a withdrawn version, and the withdrawn version SHALL stay `withdrawn`.

#### Scenario: A withdrawn version accepted

- **WHEN** an agent withdrew a version, and a person accepts it or rejects it
- **THEN** boom exits 2 and says that the version is withdrawn, and the key signs nothing

#### Scenario: A narrative that includes a withdrawn version

- **WHEN** a proposed narrative includes a version that an agent withdrew, and a person accepts the narrative
- **THEN** boom exits 2 and names the withdrawn version, and the key signs nothing

#### Scenario: The answer withdrawn

- **WHEN** an agent withdraws the version of a narrative that answered a note
- **THEN** `boom show <name>` gives the note as open again

#### Scenario: A withdrawn version amended

- **WHEN** an agent withdrew `<v1>`, and an agent proposes `<v2>` amending `<v1>`
- **THEN** boom exits 0, `<v2>` is `proposed`, and `<v1>` stays `withdrawn`
