## ADDED Requirements

### Requirement: A new draft supersedes the draft it amends

When a version that amends a proposed version counts, boom SHALL give the proposed version the status `superseded`. Boom SHALL refuse an accept or a reject of a draft that a newer draft superseded, SHALL name the newer draft, and SHALL record nothing. Boom SHALL refuse an amendment of such a draft and SHALL name the newer draft. A reject of the newer draft SHALL leave the old draft superseded.

#### Scenario: A draft amended

- **WHEN** `<v1>` is accepted, an agent proposes `<v2>` amending `<v1>`, then `<v3>` amending `<v2>`, and `boom show <name> --json` runs
- **THEN** it gives `<v2>` as `superseded`, `<v3>` as `proposed` and the newest, and `<v1>` as the accepted version

#### Scenario: The newest draft accepted

- **WHEN** `<v1>` is accepted, `<v2>` amends `<v1>`, `<v3>` amends `<v2>`, and a person accepts `<v3>`
- **THEN** boom exits 0, and `<v3>` is `accepted`, and `<v1>` and `<v2>` are `superseded`

#### Scenario: A first draft amended

- **WHEN** no version of a claim is accepted, an agent proposes `<v1>`, then `<v2>` amending `<v1>`, and a person accepts `<v2>`
- **THEN** boom exits 0, `<v2>` is `accepted`, and `<v1>` is `superseded`

#### Scenario: An old draft accepted

- **WHEN** `<v3>` amends the draft `<v2>`, and a person accepts `<v2>`
- **THEN** boom exits 2 and names `<v3>` as the draft that superseded it, and boom appends nothing

#### Scenario: An old draft rejected

- **WHEN** `<v3>` amends the draft `<v2>`, and a person rejects `<v2>`
- **THEN** boom exits 2 and names `<v3>`, and boom appends nothing

#### Scenario: An old draft amended

- **WHEN** `<v3>` amends the draft `<v2>`, and an agent proposes `<v4>` amending `<v2>`
- **THEN** boom exits 2 and names `<v3>` as the draft to amend, and boom appends nothing

#### Scenario: The newer draft rejected

- **WHEN** `<v3>` amends the draft `<v2>`, and a person rejects `<v3>`
- **THEN** `<v3>` is `rejected`, and `<v2>` stays `superseded`

#### Scenario: The notes on an old draft

- **WHEN** a note on the narrative draft `<v2>` is open, and an agent proposes `<v3>` amending `<v2>` with `--answers <note>=<words>`
- **THEN** boom exits 0, `<v2>` is `superseded`, and `boom show <name>` gives the note as answered by `<v3>`

#### Scenario: A log from before this rule

- **WHEN** a store's views were built before this rule, its log holds a draft that a later draft amends, and a command reads the store
- **THEN** boom builds the views again, and gives the old draft as `superseded`

## MODIFIED Requirements

### Requirement: An accept stands only on its base

The base that an accept checks SHALL be the nearest version in the line of bases of the version that a person accepted, or none. Boom SHALL refuse to accept a version unless that base is the accepted version of its claim, or both are none. A refusal SHALL name the accepted version and record nothing. Boom SHALL NOT merge two amendments into one text.

#### Scenario: Two amendments of one base

- **WHEN** an agent proposes two amendments of the accepted version, and a person accepts the first
- **THEN** an accept of the second exits 2 and names the first as the accepted version, and the second stays proposed

#### Scenario: The same words on the new base

- **WHEN** an agent proposes the words of the second amendment again, as an amendment of the new accepted version, and a person accepts it
- **THEN** the new version is accepted, and the second amendment stays proposed

#### Scenario: An amendment of a rejected version

- **WHEN** `<v1>` is accepted, a person rejects `<v2>` that amends `<v1>`, an agent proposes `<v3>` amending `<v2>`, and a person accepts `<v3>`
- **THEN** boom exits 0, `<v3>` is `accepted`, and `<v2>` stays `rejected`

#### Scenario: A draft of a draft on an old base

- **WHEN** `<v2>` amends the draft `<v1>`, a person accepts another first version `<w1>`, and then accepts `<v2>`
- **THEN** boom exits 2 and names `<w1>` as the accepted version, and `<v2>` stays proposed

### Requirement: Each version's status is read from the log

Boom SHALL compute the status of a version from the log, as one of these:

- `proposed`, until it is accepted, rejected or superseded.
- `accepted`, after an accept, while no later version of its claim is accepted.
- `superseded`, after a later version of its claim is accepted, or, while proposed, after a new draft amends it.
- `rejected`, after a person rejects it.
- For accepted history, the status of its file.

No act makes a version `deprecated` yet. The log SHALL keep each entry.

#### Scenario: Proposed, standing, replaced

- **WHEN** a person accepts a version, then an agent proposes an amendment of it, and a person accepts the amendment
- **THEN** `boom show <name> --json` gives the first version as `superseded`, the amendment as `accepted`, and the amendment as the accepted version of the claim

#### Scenario: Rejected

- **WHEN** a person rejects a proposed version, with a reason
- **THEN** `boom show <name> --json` gives it as `rejected`, with the person who rejected it and the reason

#### Scenario: History accepted

- **WHEN** a person accepts a version that came in as history from a file whose status is superseded
- **THEN** `boom show <name> --json` gives that version as `superseded`, and gives the claim no accepted version
