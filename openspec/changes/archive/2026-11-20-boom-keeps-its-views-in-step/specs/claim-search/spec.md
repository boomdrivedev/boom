## MODIFIED Requirements

### Requirement: Only what stands, as the person accepted it

`boom search` SHALL search the standing versions the views hold, once they are at the log's last entry, in the words a person accepted. It SHALL never search a version proposed, rejected, replaced or history, or one an entry that fails boom's checks would make stand. It SHALL keep no index and append nothing. A body whose bytes no longer match its fingerprint SHALL stop it, exit 3, naming the body.

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

- **WHEN** a standing version's statement in the `versions` table is changed by another program to hold the word `forged`, `views_head` left as it was, and `boom verify` runs
- **THEN** `boom search forged --repository shop --json` gives no claim, and a search for a word of the accepted statement gives the version

#### Scenario: An altered body

- **WHEN** the bytes of a standing version's body in the `blobs` table are changed by another program
- **THEN** `boom search` for a word of that version's statement exits 3 naming the body's fingerprint, and the log is as it was

#### Scenario: Nothing kept

- **WHEN** `boom search refund --repository shop` runs on a store whose views are at the log's last entry
- **THEN** the store holds the same tables and the same log as before it ran

#### Scenario: A large log

- **WHEN** a store's log holds 20 000 entries, its views are at its last entry, and `boom search refund --repository shop --json` runs
- **THEN** boom reads one entry of the log, the last, and answers
