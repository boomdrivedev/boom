## MODIFIED Requirements

### Requirement: The words searched

`boom search <words>...` SHALL give each claim whose accepted version holds each word searched. Boom SHALL find a word in any of these fields: the name, the statement, the scenarios of an old version, the body, and the legacy number or id and the section of an imported record. A word SHALL match its other forms, its case and its accents. Boom SHALL search a body that is not UTF-8 text as a body with no words.

#### Scenario: Found by its statement

- **WHEN** an invariant `money-has-currency` is accepted for `shop` with the statement "Every amount names its currency."
- **THEN** `boom search currency --repository shop --json` gives it

#### Scenario: Found by its name

- **WHEN** that invariant is accepted, and no other field of it holds the word `money`
- **THEN** `boom search money --repository shop --json` gives it

#### Scenario: Found by its scenarios

- **WHEN** an old version that binds `shop` is accepted, its only scenario holds the word `ledger`, and no other field of it holds that word
- **THEN** `boom search ledger --repository shop --json` gives it

#### Scenario: Found in its body

- **WHEN** a decision is accepted for `shop`, and only its body holds the word `rounding`
- **THEN** `boom search rounding --repository shop --json` gives it, and its `excerpt` holds `rounding`

#### Scenario: Found by its legacy id

- **WHEN** a person accepts an import of a rules page for `shop` that holds `core.1` and `core.2`, and the words of neither rule hold `core`
- **THEN** `boom search core.2 --repository shop --json` gives the invariant whose legacy id is `core.2`, and not the one whose legacy id is `core.1`

#### Scenario: Found by its section

- **WHEN** a person accepts that import, both rules are under `### The ledger`, and the other words of neither rule hold `ledger`
- **THEN** `boom search ledger --repository shop --json` gives both invariants

#### Scenario: Every word is found

- **WHEN** one claim accepted for `shop` holds `amount` and `currency`, and another holds `amount` only
- **THEN** `boom search amount currency --repository shop --json` gives the first and not the second

#### Scenario: Another form of a word

- **WHEN** the invariant `money-has-currency` is accepted for `shop`
- **THEN** `boom search AMOUNTS Currencies --repository shop --json` gives it

#### Scenario: A word written with accents

- **WHEN** a decision is accepted for `shop` with the statement "A naive date names its zone."
- **THEN** `boom search naïve --repository shop --json` gives it

### Requirement: Only what stands, as the person accepted it

`boom search` SHALL search the accepted versions that the views hold, after the views are at the last entry of the log. It SHALL NOT search a version that is proposed, rejected, superseded or deprecated, or that only an entry that fails the checks of boom makes accepted. It SHALL read no other version. It SHALL keep no index and append nothing. When the bytes of a body do not match its fingerprint, boom SHALL exit 3 and name the body.

#### Scenario: A proposal not given

- **WHEN** an agent proposes an invariant that holds `refund` for `shop`, and no person accepts it
- **THEN** `boom search refund --repository shop --json` does not give it

#### Scenario: An amendment waiting

- **WHEN** a decision with "Amounts are kept in integer cents." is accepted for `shop`, and an agent proposes an amendment of it, "Amounts are kept in integer mills.", that no person accepts
- **THEN** `boom search mills --repository shop --json` gives no claim, and `boom search cents --repository shop --json` gives the accepted version

#### Scenario: A version that came in as history

- **WHEN** a person accepts together an import for `shop` of `0002-amounts-as-floats.md`, superseded by `0003-amounts-in-cents.md`, and of `0003-amounts-in-cents.md`, whose words do not hold `floats`
- **THEN** `boom search floats --repository shop --json` gives no claim

#### Scenario: A statement edited in the views

- **WHEN** another program changes the statement of an accepted version in the `versions` table to hold the word `forged`, and does not change `views_head`, and then `boom verify` runs
- **THEN** `boom search forged --repository shop --json` gives no claim, and a search for a word of the accepted statement gives the version

#### Scenario: An altered body

- **WHEN** another program changes the bytes of the body of an accepted version in the `blobs` table
- **THEN** `boom search` for a word of the statement of that version exits 3 and names the fingerprint of the body, and the log does not change

#### Scenario: Nothing kept

- **WHEN** `boom search refund --repository shop` runs on a store whose views are at the last entry of the log
- **THEN** the store holds the same tables and the same log as before the run

#### Scenario: A large log

- **WHEN** the log of a store holds 20 000 entries, its views are at its last entry, and `boom search refund --repository shop --json` runs
- **THEN** boom reads one entry of the log, the last, and answers

### Requirement: Answered in one JSON object, or in words an agent reads

With `--json`, `boom search` SHALL answer `boom-search/2` with `words`, `repository` (`null` for all), `head`, `total` and `claims`. Each claim SHALL hold its kind, name, version, statement, scenarios, body fingerprint, `excerpt`, `legacy`, `repositories` and the person who accepted it. In words, `boom search` SHALL write to standard output only: a first line with the number of matches, the words, the scope and the entry, then each claim. The same log SHALL give the same answer.

#### Scenario: Answered as JSON

- **WHEN** `boom search amount --repository shop --json` runs where the invariant `money-has-currency` is accepted for `shop`
- **THEN** boom prints one object with `format` `boom-search/2`, `words` `["amount"]`, `repository` `shop`, the last entry of the log as `head`, `total` 1, and `claims` with the invariant. The invariant has its version and statement, `scenarios` `[]`, `body` `null`, `excerpt` `null`, `legacy` `null`, `repositories` `["shop"]`, and the person who accepted it and the number of that entry

#### Scenario: Answered in words

- **WHEN** `boom search rounding --repository shop` runs where a decision whose body holds `rounding` is accepted for `shop`
- **THEN** boom exits 0. The first line gives the one match, `rounding`, `shop` and the number of the last entry of the log. A line gives the kind of the decision, `<name>@<version>`, what it binds and its statement. A line below gives its excerpt, and another gives `boom show <name>@<version>`. Standard error is empty

#### Scenario: Nothing matches

- **WHEN** `boom search parsnip --repository shop --json` runs where no claim accepted for `shop` holds `parsnip`
- **THEN** boom exits 0 with `total` 0 and `claims` empty

#### Scenario: The same log, the same answer

- **WHEN** `boom search amount --repository shop --json` runs two times, and the views of the store are dropped between the two runs
- **THEN** both runs print the same bytes, and the log does not change
