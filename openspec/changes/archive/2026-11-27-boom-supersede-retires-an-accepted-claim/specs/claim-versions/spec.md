## ADDED Requirements

### Requirement: boom supersede proposes a version that supersedes an accepted claim

`boom supersede <name> <new-name> [<statement>]` SHALL propose one version that supersedes the accepted version of `<name>`: an agent's with `--provenance`, or a person's with `--key`. The version SHALL wait as each proposal waits, and `<name>` SHALL stay accepted until a person accepts the version. Boom SHALL refuse a `<name>` with no accepted version, and a `<new-name>` that is `<name>`, and SHALL append nothing. Boom SHALL answer `boom-supersede/1`.

#### Scenario: A proposal that waits

- **WHEN** `billing-owns-invoices@<b1>` is accepted and binds `shop`, and `boom supersede billing-owns-invoices invoice-writers "Only billing and refunds write invoices." --reason "Refunds issue their own credit notes." --provenance run-2` runs
- **THEN** boom exits 0, the log gets one `proposed` entry, `invoice-writers@<w1>` is `proposed`, `billing-owns-invoices@<b1>` stays `accepted`, and `boom query --repository shop` gives `billing-owns-invoices` and not `invoice-writers`

#### Scenario: A supersession answered as JSON

- **WHEN** the same command runs with `--json`
- **THEN** standard output is one JSON object whose `format` is `boom-supersede/1`. It names the claim `invoice-writers`, its version and its status `proposed`, `"supersedes": "billing-owns-invoices"`, the version `<b1>` that it supersedes, the reason, and the command that a person runs to accept it

#### Scenario: A claim with no accepted version

- **WHEN** `cash-only` has one proposed version and none accepted, and an agent runs `boom supersede cash-only bank-transfers "We take bank transfers." --provenance run-2`
- **THEN** boom exits 2 and says that `cash-only` has no accepted version to supersede, and boom appends nothing

#### Scenario: A claim that supersedes itself

- **WHEN** an agent runs `boom supersede billing-owns-invoices billing-owns-invoices --provenance run-2`
- **THEN** boom exits 2 and says that a claim does not supersede itself, and to amend it with `boom propose --amends`, and boom appends nothing

### Requirement: A supersession by a new claim

Where `<new-name>` names no accepted claim, `boom supersede` SHALL propose a first version of `<new-name>`, with `<statement>` and the body of `--body`. Its kind SHALL be `--kind`, or else the kind of `<name>`. It SHALL bind what the accepted version of `<name>` binds. Boom SHALL refuse it with no `<statement>`, and SHALL append nothing.

#### Scenario: A new claim replaces an invariant

- **WHEN** `billing-owns-invoices@<b1>`, an invariant, is accepted and binds `shop`, and an agent runs `boom supersede billing-owns-invoices invoice-writers "Only billing and refunds write invoices." --provenance run-2 --json`
- **THEN** boom exits 0, and the answer gives `invoice-writers` as an invariant with no `base`, that binds `shop` and supersedes `billing-owns-invoices@<b1>`

#### Scenario: A new claim of another kind

- **WHEN** an agent runs `boom supersede billing-owns-invoices invoice-writers "Refunds write their own credit notes." --kind decision --body <file> --provenance run-2`
- **THEN** boom exits 0, and `invoice-writers` is a decision with the body of `<file>`, that supersedes `billing-owns-invoices@<b1>`

#### Scenario: A new claim with no statement

- **WHEN** an agent runs `boom supersede billing-owns-invoices invoice-writers --provenance run-2`, and no claim has the name `invoice-writers`
- **THEN** boom exits 2 and says that a new claim gives its statement, and boom appends nothing

### Requirement: A supersession by an accepted claim of the same kind

Where `<new-name>` is an accepted claim of the same kind as `<name>`, `boom supersede` SHALL propose a version of `<new-name>` that amends its accepted version. The version SHALL copy the statement, the body, the sources and the binding of that version unchanged. `<statement>` or `--body`, where given, SHALL replace the one that it copies. Boom SHALL refuse it where `<new-name>` is of another kind, or `--kind` names another kind, and SHALL append nothing.

#### Scenario: A duplicate retired by the claim to keep

- **WHEN** the invariants `tests-as-units@<k1>`, with a body and two sources, and `tests-as-units-too@<d1>` are accepted and bind the whole system, and an agent runs `boom supersede tests-as-units-too tests-as-units --reason "a duplicate" --provenance run-2 --json`
- **THEN** boom exits 0. The answer gives `tests-as-units@<k2>`, a version that amends `<k1>`, with the statement, the body fingerprint and the two sources of `<k1>`, that binds the whole system and supersedes `tests-as-units-too@<d1>`

#### Scenario: New words for the claim to keep

- **WHEN** the same two claims are accepted, and an agent runs `boom supersede tests-as-units-too tests-as-units "Unit tests prove a tool's logic." --provenance run-2 --json`
- **THEN** boom exits 0, and the answer gives a version that amends `<k1>` with the new statement and the body and the sources of `<k1>`

#### Scenario: An accepted claim of another kind

- **WHEN** `tests-as-units-too` is an accepted invariant, `amounts-in-cents` is an accepted decision, and an agent runs `boom supersede tests-as-units-too amounts-in-cents --provenance run-2`
- **THEN** boom exits 2 and says that `amounts-in-cents` is a decision and `tests-as-units-too` an invariant, and boom appends nothing

### Requirement: A version names the accepted version that it supersedes

A version that supersedes SHALL store the edge `supersedes`, `<name>@<version>` of another claim, and MAY store a reason of one line. Both SHALL be part of its content. Boom SHALL count the version only where it and the claim that it names are each an invariant, a decision or an assumption, it holds no history, and the version that it names is the accepted version of its claim.

#### Scenario: The reason is part of the content

- **WHEN** an agent runs one supersession two times with one reason, and a third time with another reason
- **THEN** the first two runs give one version, and the log holds one `proposed` entry for it, and the third run gives a second version, with a different fingerprint

#### Scenario: A term superseded

- **WHEN** `owner` is an accepted term, and an agent runs `boom supersede owner registrar "A person who grants roles." --provenance run-2`
- **THEN** boom exits 2 and says that only an invariant, a decision or an assumption is superseded, and boom appends nothing

#### Scenario: A reason of two lines

- **WHEN** an agent runs `boom supersede` with a `--reason` that holds a line break
- **THEN** boom exits 2 and says that a reason is one line, and boom appends nothing

#### Scenario: A version no longer accepted, from another program

- **WHEN** `billing-owns-invoices@<b2>` amends `<b1>` and is accepted, and another program appends, chained correctly, a proposal of an invariant that supersedes `billing-owns-invoices@<b1>`
- **THEN** the entry counts for nothing, and its judgment names `billing-owns-invoices@<b2>` as the accepted version

### Requirement: The accept of a supersession takes the claim it names out of force

When the accept of a version that supersedes counts, the version that it names SHALL take the status `superseded`, and its claim SHALL have no accepted version. Boom SHALL refuse the accept, and record nothing, where that version is no longer the accepted version of its claim, or where the accept also names a version of that claim or another version that supersedes it. The log SHALL keep each entry.

#### Scenario: A new claim accepted

- **WHEN** `invoice-writers@<w1>` supersedes `billing-owns-invoices@<b1>`, and a person accepts `invoice-writers@<w1>`
- **THEN** boom exits 0. `invoice-writers@<w1>` is `accepted`, `billing-owns-invoices@<b1>` is `superseded`, and `billing-owns-invoices` has no accepted version. `boom query --repository shop` gives `invoice-writers` and not `billing-owns-invoices`, and `boom log` keeps each entry

#### Scenario: The duplicate leaves force

- **WHEN** `tests-as-units@<k2>` amends `<k1>` and supersedes `tests-as-units-too@<d1>`, and a person accepts `tests-as-units@<k2>`
- **THEN** boom exits 0. `<k2>` is `accepted`, and `<k1>` and `tests-as-units-too@<d1>` are `superseded`. `boom query` gives `tests-as-units@<k2>` and not `tests-as-units-too`, and `boom search` for the words of `<d1>` does not give `tests-as-units-too`

#### Scenario: The claim named moved on

- **WHEN** `invoice-writers@<w1>` supersedes `billing-owns-invoices@<b1>`, then a person accepts `billing-owns-invoices@<b2>` that amends `<b1>`, and then accepts `invoice-writers@<w1>`
- **THEN** boom exits 2 and names `billing-owns-invoices@<b2>` as the accepted version, the key signs nothing, and `invoice-writers@<w1>` stays `proposed`

#### Scenario: One accept names both claims

- **WHEN** `invoice-writers@<w1>` supersedes `billing-owns-invoices@<b1>`, `billing-owns-invoices@<b2>` amends `<b1>`, and a person accepts both in one command
- **THEN** boom exits 2 and names both versions, the key signs nothing, and each stays `proposed`

### Requirement: The person reads what a supersession takes out of force

Before the key signs, `boom accept` SHALL list, beside each version that supersedes, the version that it supersedes, and its answer SHALL name that version under `supersedes`. `boom show <name>@<version>` SHALL give the edge `supersedes` and the reason of a version. `boom show <name>` SHALL give each version that supersedes the claim, with its status.

#### Scenario: The accept lists the claim it supersedes

- **WHEN** `invoice-writers@<w1>` supersedes `billing-owns-invoices@<b1>`, and a person runs `boom accept invoice-writers@<w1> --json`
- **THEN** before the key signs, boom tells the person that the entry takes `invoice-writers@<w1>`, which supersedes `billing-owns-invoices@<b1>`. The answer names `billing-owns-invoices@<b1>` under `supersedes`

#### Scenario: A superseded claim shown

- **WHEN** a person accepted `invoice-writers@<w1>`, and `boom show billing-owns-invoices --json` runs
- **THEN** it gives no accepted version, `<b1>` as `superseded`, and `invoice-writers@<w1>` as `accepted` under `superseded_by`

#### Scenario: A version that supersedes shown

- **WHEN** `boom show invoice-writers@<w1> --json` runs
- **THEN** it gives `supersedes` as `billing-owns-invoices@<b1>`, with the reason of the supersession
