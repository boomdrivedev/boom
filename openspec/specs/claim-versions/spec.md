# claim-versions Specification

## Purpose
What boom keeps of a claim: one name for life, a version for each
change, each amendment naming the version it amends, and the status of
every version read from the log (record 0005, point 2; interview:
`identity-fixed-at-proposal`, `one-name-space`, `amend-names-its-base`).

## Requirements

### Requirement: A claim and its name

A claim SHALL have one kind: `invariant`, `decision`, `assumption`, `term`, `context`, `narrative` or `outline`. The first proposal SHALL fix the kind and the name. A name SHALL be kebab case: lowercase letters and digits, in words that single hyphens join. Each version SHALL have a statement of one line. A decision and a narrative SHALL also have a body. Another kind MAY have one. A new version SHALL have no scenarios. An old version SHALL keep the scenarios that its entry holds.

#### Scenario: A claim proposed

- **WHEN** `boom propose invariant refund-within-payment "A refund never exceeds what the order was paid." --provenance run-1` runs
- **THEN** boom exits 0, and `boom show refund-within-payment --json` gives an invariant with one proposed version

#### Scenario: A name not in kebab case

- **WHEN** an agent proposes a claim with the name `Refund_Within`
- **THEN** boom exits 2 and says that a name is kebab case, and boom appends nothing

#### Scenario: No scenario

- **WHEN** an agent proposes an invariant with a statement and no other field
- **THEN** boom exits 0, and the proposal's entry holds no scenarios

#### Scenario: A statement of two lines

- **WHEN** an agent proposes a claim whose statement holds a line break
- **THEN** boom exits 2 and says that a statement is one line, and boom appends nothing

#### Scenario: History with no scenario

- **WHEN** an import proposes a superseded decision
- **THEN** boom proposes its version as history, with no scenarios

#### Scenario: A decision with no body

- **WHEN** an agent proposes a decision with no `--body`
- **THEN** boom exits 2 and says that a decision needs a body, and boom appends nothing

#### Scenario: An old version with scenarios

- **WHEN** another program appends an old proposal entry that holds a scenario, chained correctly
- **THEN** the entry counts, and `boom show` gives the version with its scenario

#### Scenario: A term with no body

- **WHEN** an agent proposes a term with a meaning and no `--body`, for one repository
- **THEN** boom exits 0

#### Scenario: A context with no body

- **WHEN** an agent proposes a context with a statement, one glob and no `--body`
- **THEN** boom exits 0

#### Scenario: A narrative with its title and body

- **WHEN** an agent proposes a narrative with a title of one line and `--body`
- **THEN** boom exits 0, and `boom show <name> --json` gives the kind `narrative`

#### Scenario: An outline with its title

- **WHEN** an agent proposes an outline with a title of one line, `--view book`, one `--narrative` and no `--body`
- **THEN** boom exits 0, and `boom show <name> --json` gives the kind `outline`

### Requirement: A version is the fingerprint of its content

Boom SHALL name a version by the first 12 hexadecimal characters of the SHA-256 of its content. The content SHALL be the kind, name, statement, scenarios, body fingerprint and base. It SHALL also hold the history of a version from history, the repository and `replaces` of a term, and the globs of a context. It SHALL hold the edges `source`, `changes`, `origin` and `includes`, and the answers of a narrative. For the same content again, boom SHALL give the same version and append nothing.

#### Scenario: The same proposal twice

- **WHEN** an agent proposes one claim twice, with the same words and body
- **THEN** both runs give the same version, and the log holds one `proposed` entry for it

#### Scenario: One word changed

- **WHEN** an agent proposes a version again with one word of its statement changed
- **THEN** boom gives a second version, with a different fingerprint

#### Scenario: The same words as history

- **WHEN** an import proposes a superseded decision, and `boom propose` proposes its kind, name, statement and body with no history
- **THEN** boom gives two versions, with two fingerprints

#### Scenario: Another old word

- **WHEN** an agent proposes the term `owner` again with the same meaning and one more word in `replaces`
- **THEN** boom gives a second version, with a different fingerprint

#### Scenario: One term in two repositories

- **WHEN** an agent proposes the term `owner` with one meaning for `boom`, and then for `sluice`
- **THEN** boom gives two versions, with two fingerprints

#### Scenario: One more edge

- **WHEN** an agent proposes a claim again with the same words and one more `--source`
- **THEN** boom gives a second version, with a different fingerprint

#### Scenario: One more glob

- **WHEN** an agent proposes a context again with the same statement and one more `--path`
- **THEN** boom gives a second version, with a different fingerprint

#### Scenario: The same words bound to a context

- **WHEN** an agent proposes a claim bound to `shop`, and the same claim in the same words is proposed to another store bound to the context `shop-billing`
- **THEN** both answers give the same version

#### Scenario: A version from before terms

- **WHEN** a store holds versions that boom proposed before the kind `term` came, and this boom builds its views again
- **THEN** each version has the fingerprint that its entry names

#### Scenario: A version from before the edges

- **WHEN** a store holds versions that boom proposed before the edges `source`, `changes` and `origin` came, and this boom builds its views again
- **THEN** each version has the fingerprint that its entry names

#### Scenario: A version from before contexts

- **WHEN** a store holds versions that boom proposed before the kind `context` came, and this boom builds its views again
- **THEN** each version has the fingerprint that its entry names

#### Scenario: One more include

- **WHEN** an agent proposes a narrative again with the same title and body and one more `--includes`
- **THEN** boom gives a second version, with a different fingerprint

#### Scenario: A version from before narratives

- **WHEN** a store holds versions that boom proposed before the kind `narrative` came, and this boom builds its views again
- **THEN** each version has the fingerprint that its entry names

### Requirement: One name space

Two claims of a store SHALL NOT have the same name, except terms of one word, each of which binds another repository or the whole system. Boom SHALL refuse a first version whose name a claim of another kind has, and SHALL name that claim. Boom SHALL refuse a first version whose claim has an accepted version, and SHALL name that version, to be amended. Several first versions MAY wait together while no version of the claim is accepted.

#### Scenario: A name taken by another kind

- **WHEN** an agent proposes a decision `billing` while an invariant `billing` exists
- **THEN** boom exits 2 and names the invariant `billing`, and boom appends nothing

#### Scenario: A name whose claim stands

- **WHEN** an agent proposes a first version of `money-has-currency` while a version of it is accepted
- **THEN** boom exits 2, names the accepted version and says to amend it, and boom appends nothing

#### Scenario: Two first versions waiting

- **WHEN** an agent proposes two first versions of one invariant, with different statements, while no version is accepted
- **THEN** boom appends both, and both wait for a person

#### Scenario: A term with the name of an invariant

- **WHEN** an agent proposes the term `billing` for `shop` while an invariant `billing` exists
- **THEN** boom exits 2 and names the invariant `billing`, and boom appends nothing

#### Scenario: A term of the whole system with the name of an invariant

- **WHEN** an agent proposes the term `billing` with `--system` while an invariant `billing` exists
- **THEN** boom exits 2 and names the invariant `billing`, and boom appends nothing

#### Scenario: A word of the whole system and of a repository

- **WHEN** the term `link` of the whole system is accepted, and an agent proposes the term `link` for `boom`
- **THEN** boom exits 0, and the store holds two terms `link`

### Requirement: An amendment names the version it amends

A version that `--amends <version>` proposes SHALL name a version of the same claim as its base. Boom SHALL refuse an amendment whose base the claim does not have, and SHALL name the claim. An amendment SHALL keep the kind of its claim.

#### Scenario: An amendment proposed

- **WHEN** `boom propose invariant money-has-currency "<new words>" --amends <accepted version> --provenance run-2` runs
- **THEN** boom proposes the new version with that version as its base, and the accepted version stays accepted

#### Scenario: A base the claim does not have

- **WHEN** an amendment names a version that its claim does not have
- **THEN** boom exits 2 and names the claim and the version it does not have, and boom appends nothing

### Requirement: An accept stands only on its base

Boom SHALL refuse to accept a version unless its base is the accepted version of its claim. For a first version, boom SHALL refuse unless the claim has no accepted version. A refusal SHALL name the accepted version and record nothing. Boom SHALL NOT merge two amendments into one text.

#### Scenario: Two amendments of one base

- **WHEN** an agent proposes two amendments of the accepted version, and a person accepts the first
- **THEN** an accept of the second exits 2 and names the first as the accepted version, and the second stays proposed

#### Scenario: The same words on the new base

- **WHEN** an agent proposes the words of the second amendment again, as an amendment of the new accepted version, and a person accepts it
- **THEN** the new version is accepted, and the second amendment stays proposed

### Requirement: Each version's status is read from the log

Boom SHALL compute the status of a version from the log. The status SHALL be one of these:

- `proposed`, until a counted accept or reject.
- `accepted`, after an accept, while no later version of its claim is accepted.
- `superseded`, after a later version of its claim is accepted.
- `rejected`, after a person rejects it.
- For an accepted version that came in as history, the status of its file.

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

### Requirement: A claim shown by its name

`boom show <name>` SHALL give the kind, the accepted version or none, and the newest version with its statement, what it binds and its body. It SHALL then give the other versions as a tree by base, each with its statement, status, base, proposer and decider. `boom show <name>@<version>` SHALL give its statement, scenarios, body and `replaces`. `boom show <number>` SHALL give the version that its entry names. It SHALL refuse an unknown name, version or entry. The JSON SHALL be `boom-show/2`.

#### Scenario: A version shown

- **WHEN** `boom show money-has-currency@<version> --json` runs for a version proposed with a body
- **THEN** it gives `boom-show/2` with the statement, the scenarios, the status and the body of the version

#### Scenario: An unknown name

- **WHEN** `boom show no-such-claim` runs
- **THEN** boom exits 2 and says that no claim has that name

#### Scenario: The newest version first

- **WHEN** a version of `money-has-currency` is accepted, an agent proposes an amendment of it, and `boom show money-has-currency` runs
- **THEN** the first line says `accepted: <accepted version>`. The statement of the amendment comes next, and the accepted version comes after `other versions:`

#### Scenario: An amendment under its base

- **WHEN** `money-has-currency@<v1>` is accepted, a person rejects its amendment `<v2>`, an agent proposes its amendment `<v3>`, and `boom show money-has-currency` runs
- **THEN** after `other versions:`, boom gives `<v1>`, and `<v2>` under it, as the amendment of `<v1>`. With `--json`, each of `versions` names its `base`

#### Scenario: A version shown by its entry

- **WHEN** entry 2 proposes `money-has-currency@<version>`, entry 3 accepts it, and `boom show 2` and `boom show 3` run
- **THEN** each gives what `boom show money-has-currency@<version>` gives

#### Scenario: An entry that names no version

- **WHEN** `boom show 1` runs, and entry 1 registers a person
- **THEN** boom exits 2 and says that entry 1 names no version

#### Scenario: A number that the log does not hold

- **WHEN** `boom show 9` runs on a log of three entries
- **THEN** boom exits 2 and says that the log has no entry 9

### Requirement: A word shown for each repository

For a word with two terms or more, `boom show <word>` SHALL give each term, one after the other: the term of the whole system first, then the term of each repository, in the order of their repositories. `boom show <word> --repository <name>` SHALL give the term of that repository only. Boom SHALL refuse `--repository` for a repository where the word has no term.

#### Scenario: A word with a term in two repositories

- **WHEN** the term `owner` binds `boom`, another term `owner` binds `sluice`, and `boom show owner --json` runs
- **THEN** it gives two terms, the term of `boom` and then the term of `sluice`

#### Scenario: One repository chosen

- **WHEN** `boom show owner --repository sluice --json` runs
- **THEN** it gives the term of `sluice` only

#### Scenario: A word of the whole system and of a repository shown

- **WHEN** the term `link` binds the whole system, another term `link` binds `boom`, and `boom show link --json` runs
- **THEN** it gives two terms, the term of the whole system with no `repository`, and then the term of `boom`

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
