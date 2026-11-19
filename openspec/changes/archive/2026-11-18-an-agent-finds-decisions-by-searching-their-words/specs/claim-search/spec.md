# Spec Delta

## Purpose

How an agent finds the decisions, rules and records about a subject by
searching their words, in the repository it works in or across every
repository: only the claims that stand, in the words a person accepted,
replayed from the store's log for each answer with no index kept, and
ranked so the few that bear on the subject come first (record 0005;
record 0008; record 0009; core.1, core.2).

## ADDED Requirements

### Requirement: The words searched

`boom search <words>...` SHALL give each claim whose standing version holds every word searched, each found in any of its name, statement, scenarios, body, or an imported record's legacy number or id and its section. A word SHALL match its other forms, its case and its accents aside. A body that is not UTF-8 text SHALL be searched as holding no word.

#### Scenario: Found by its statement

- **WHEN** an invariant `money-has-currency` stands for `shop` with the statement "Every amount names its currency."
- **THEN** `boom search currency --repository shop --json` gives it

#### Scenario: Found by its name

- **WHEN** that invariant stands, and no other field of it holds the word `money`
- **THEN** `boom search money --repository shop --json` gives it

#### Scenario: Found by its scenarios

- **WHEN** a decision stands for `shop` whose only scenario holds the word `ledger`, and no other field of it does
- **THEN** `boom search ledger --repository shop --json` gives it

#### Scenario: Found in its body

- **WHEN** a decision stands for `shop` whose body alone holds the word `rounding`
- **THEN** `boom search rounding --repository shop --json` gives it, its `excerpt` holding `rounding`

#### Scenario: Found by its legacy id

- **WHEN** an import of a rules page for `shop` holding `core.1` and `core.2` is accepted, and neither rule's words hold `core`
- **THEN** `boom search core.2 --repository shop --json` gives the invariant whose legacy id is `core.2`, and not the one whose legacy id is `core.1`

#### Scenario: Found by its section

- **WHEN** that import is accepted, both rules under `### The ledger`, and neither rule's other words hold `ledger`
- **THEN** `boom search ledger --repository shop --json` gives both invariants

#### Scenario: Every word is found

- **WHEN** one claim standing for `shop` holds `amount` and `currency`, and another holds `amount` alone
- **THEN** `boom search amount currency --repository shop --json` gives the first and not the second

#### Scenario: Another form of a word

- **WHEN** the invariant `money-has-currency` stands for `shop`
- **THEN** `boom search AMOUNTS Currencies --repository shop --json` gives it

#### Scenario: A word written with accents

- **WHEN** a decision stands for `shop` with the statement "A naive date names its zone."
- **THEN** `boom search naïve --repository shop --json` gives it

### Requirement: A word is always a word

`boom search` SHALL search for each word as written and never read one as an operator of a query language. A word holding punctuation SHALL be found where its parts stand together in that order. A word with no letter or digit SHALL be dropped, and a search with no word left SHALL be refused, writing nothing.

#### Scenario: Words, not a query language

- **WHEN** the invariant `money-has-currency` stands for `shop`
- **THEN** `boom search 'currency*' '"amount' 'OR' --repository shop --json` exits 0 and gives no claim, no claim holding the word `or`, and `boom search 'currency*' '"amount' --repository shop --json` gives that invariant

#### Scenario: No word to search

- **WHEN** `boom search '***' --repository shop` runs
- **THEN** it exits 2 saying a search needs a word holding a letter or a digit, and the log is as it was

### Requirement: The repository it works in, or every repository

`boom search` SHALL search the claims that stand for the repository `--repository` names, or else the one `.config/boom.toml` declares as `boom query` reads it, each standing version attached to that repository or the whole system; with `--all`, every claim that stands, whatever it binds. With none of these it SHALL refuse, saying how to name one, and SHALL refuse `--all` beside `--repository`.

#### Scenario: In the repository it works in

- **WHEN** invariants holding `refund` stand, one attached to `shop`, one to the whole system and one to `ledger`
- **THEN** `boom search refund --repository shop --json` gives the first two and not the third

#### Scenario: Declared by the repository

- **WHEN** `boom search refund` runs in a folder within a repository whose `.config/boom.toml` holds `repository = "shop"`
- **THEN** it searches for `shop`

#### Scenario: Across every repository

- **WHEN** `boom search refund --all --json` runs on that store
- **THEN** it gives all three invariants, each with its `repositories`, and `repository` is `null`

#### Scenario: Neither named, declared nor all

- **WHEN** `boom search refund` runs with no `--repository` and no `--all` where no `.config/boom.toml` lies between the working folder and the folder holding `.git`
- **THEN** it exits 2 saying to name the repository with `--repository` or in `.config/boom.toml`, or to search every repository with `--all`, and the log is as it was

#### Scenario: A repository and every repository

- **WHEN** `boom search refund --repository shop --all` runs
- **THEN** it exits 2, and the log is as it was

### Requirement: Only what stands, as the person accepted it

`boom search` SHALL search the standing versions the judge's replay of the store's whole log gives for that answer, in the words a person accepted, never a view table, a version proposed, rejected, replaced or history, or an entry that fails boom's checks. It SHALL keep no index and append nothing. A body whose bytes no longer match its fingerprint SHALL stop it, exit 3, naming the body.

#### Scenario: A proposal not given

- **WHEN** an invariant holding `refund` is proposed for `shop` and not accepted
- **THEN** `boom search refund --repository shop --json` does not give it

#### Scenario: An amendment waiting

- **WHEN** a decision stands for `shop` with "Amounts are kept in integer cents." and an amendment of it, "Amounts are kept in integer mills.", is proposed and not accepted
- **THEN** `boom search mills --repository shop --json` gives no claim, and `boom search cents --repository shop --json` gives the standing version

#### Scenario: A version that came in as history

- **WHEN** an import for `shop` of `0002-amounts-as-floats.md`, superseded by `0003-amounts-in-cents.md`, and of `0003-amounts-in-cents.md`, whose words do not hold `floats`, is accepted together
- **THEN** `boom search floats --repository shop --json` gives no claim

#### Scenario: A statement edited in the views

- **WHEN** a standing version's statement in the `versions` table is changed by another program to hold the word `forged`, `views_head` left as it was
- **THEN** `boom search forged --repository shop --json` gives no claim, and a search for a word of the accepted statement gives the version

#### Scenario: An altered body

- **WHEN** the bytes of a standing version's body in the `blobs` table are changed by another program
- **THEN** `boom search` for a word of that version's statement exits 3 naming the body's fingerprint, and the log is as it was

#### Scenario: Nothing kept

- **WHEN** `boom search refund --repository shop` runs
- **THEN** the store holds the same tables and the same log as before it ran

### Requirement: Ranked, the best first

`boom search` SHALL order the claims it gives by how well their words match, a word found in a name or a legacy id counting above one in a statement or a section, one in a statement above one in a scenario, and one in a scenario above one in a body; claims that match equally by kind, invariants, decisions, then assumptions, then by name. It SHALL give the first ten, or the number from 1 that `--limit` names, and how many matched in all.

#### Scenario: A name above a body

- **WHEN** an invariant `refund-never-exceeds-payment` stands for `shop` with the statement "A refund never exceeds the payment.", and a decision `orders-are-paid-once` stands for `shop` whose body alone holds `refund`
- **THEN** `boom search refund --repository shop --json` gives the invariant first and the decision second

#### Scenario: Equal matches by kind and name

- **WHEN** the decisions `beta-kept`, "Beta is kept.", and `alpha-kept`, "Alpha is kept.", and the invariant `gamma-kept`, "Gamma is kept.", stand for `shop`, each with the same scenario and no body
- **THEN** `boom search kept --repository shop --json` gives `gamma-kept`, `alpha-kept`, then `beta-kept`

#### Scenario: Limited

- **WHEN** three claims holding `kept` stand for `shop`
- **THEN** `boom search kept --repository shop --limit 2 --json` gives the first two of them with `total` 3

#### Scenario: A limit of none

- **WHEN** `boom search kept --repository shop --limit 0` runs
- **THEN** it exits 2, and the log is as it was

### Requirement: Answered in one JSON object, or in words an agent reads

With `--json`, `boom search` SHALL answer `boom-search/1` with `words`, `repository`, `null` for every repository, `head`, `total` and `claims`, each with its kind, name, version, statement, scenarios, body fingerprint, `excerpt`, `legacy`, `repositories` and who accepted it. In words, it SHALL write to standard output alone a line naming the match count, words, scope and entry, then each claim. The same log SHALL give the same answer.

#### Scenario: Answered as JSON

- **WHEN** `boom search amount --repository shop --json` runs where the invariant `money-has-currency` stands for `shop`
- **THEN** it prints one object with `format` `boom-search/1`, `words` `["amount"]`, `repository` `shop`, the log's last entry as `head`, `total` 1, and `claims` holding the invariant with its version, statement, scenarios, `body` `null`, `excerpt` `null`, `legacy` `null`, `repositories` `["shop"]`, and the person who accepted it and that entry's number

#### Scenario: Answered in words

- **WHEN** `boom search rounding --repository shop` runs where a decision whose body holds `rounding` stands for `shop`
- **THEN** it exits 0, its first line names the one match, `rounding`, `shop` and the log's last entry's number, a line gives the decision's kind, `<name>@<version>`, what it binds and its statement, a line beneath gives its excerpt and another `boom show <name>@<version>`, and standard error is empty

#### Scenario: Nothing matches

- **WHEN** `boom search parsnip --repository shop --json` runs where no claim standing for `shop` holds `parsnip`
- **THEN** it exits 0 with `total` 0 and `claims` empty

#### Scenario: The same log, the same answer

- **WHEN** `boom search amount --repository shop --json` runs twice, the store's views dropped between the two
- **THEN** both print the same bytes, and the log is as it was before the first
