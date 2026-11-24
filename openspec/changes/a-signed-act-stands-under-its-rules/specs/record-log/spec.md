## ADDED Requirements

### Requirement: An entry names its rule set

Each entry that boom writes SHALL name its rule set, `rules`, in its signed part, so that a signature covers it. An entry that names no rule set SHALL be of rule set 0. Boom SHALL refuse an entry of a rule set that it does not hold. `boom log --json` SHALL give each entry's `rules`.

#### Scenario: A new entry names its rule set

- **WHEN** a store is made and an agent proposes a claim
- **THEN** `boom log --json` gives `rules` 1 for each entry, and each signed part holds `"rules":1` before `actor`

#### Scenario: A rule set that this boom does not hold

- **WHEN** an entry of rule set 2 is appended to the log
- **THEN** it counts for nothing, and `boom log --json` says that a newer boom judges it

### Requirement: An entry is judged by the rules it was written under

Boom SHALL judge an entry by the rules of its rule set, and SHALL NOT hold it to a rule that a later rule set brought. Rule set 1 SHALL bring two rules: "a narrative binds one level", and "a draft that a newer draft superseded is not accepted, rejected or amended". Under rule set 0, such a draft SHALL be judged as a draft that waits.

#### Scenario: An accept signed before a rule

- **WHEN** a log holds an agent's proposal of a narrative that binds `boom` and `sluice`, and a person's signed accept of it, each of rule set 0, and `boom verify` runs
- **THEN** both entries count, and the narrative is `accepted`

#### Scenario: A proposal written under the rule

- **WHEN** an agent's proposal of a narrative that binds `boom` and `sluice`, of rule set 1, is appended to the log
- **THEN** it counts for nothing, and `boom log --json` says that a narrative binds one level

#### Scenario: A draft accepted before a newer draft superseded it

- **WHEN** `<v2>` amends the draft `<v1>`, and the log holds a person's signed accept of `<v1>` of rule set 0
- **THEN** the accept counts, `<v1>` is `accepted`, and `<v2>` is `proposed`

#### Scenario: A draft rejected before a newer draft superseded it

- **WHEN** `<v2>` amends the draft `<v1>`, and the log holds a person's signed reject of `<v1>` of rule set 0
- **THEN** the reject counts, and `<v1>` is `rejected`

#### Scenario: A superseded draft accepted or rejected under the rule

- **WHEN** `<v2>` amends the draft `<v1>`, and the log holds a person's signed accept of `<v1>` and a signed reject of `<v1>`, each of rule set 1
- **THEN** each counts for nothing and names `<v2>`, and `<v1>` stays `superseded`

## MODIFIED Requirements

### Requirement: An entry that fails boom's checks counts for nothing

boom SHALL hold every entry it reads to the checks of the rule set that the entry names, as it holds a write to them: its form, its signature and its actor's right to the act. An entry that fails SHALL count for nothing in any status, and `boom log` SHALL show it as not counted, with why.

#### Scenario: An accept written by hand

- **WHEN** an agent appends to the log, chained correctly, an `accepted` entry with no signature for a proposed version
- **THEN** `boom show` gives the version as proposed, and `boom log --json` gives that entry as not counted, saying a person's act needs their signature

#### Scenario: An accept signed by a key no one registered

- **WHEN** an `accepted` entry signed by a key the store does not register is appended to the log by hand
- **THEN** it counts for nothing, and `boom log --json` says its key is not registered
