## MODIFIED Requirements

### Requirement: Views are computed from the log

The store's views SHALL be computed from the log and kept in step with it. Each append SHALL update the rows its entry touches, in the transaction that appends it. A command SHALL bring the views to the log's last entry before it answers: it applies each entry after the views' head, in order. Where the views are missing or of another form, it SHALL build them again from the whole log. No answer SHALL change when the views are deleted.

#### Scenario: Views deleted

- **WHEN** a store holds a proposed and an accepted version, its view tables are dropped, and `boom show <name> --json` runs
- **THEN** the answer is the one given before the tables were dropped, and the tables are there again

#### Scenario: Views behind the log

- **WHEN** an entry is appended to the log by another program and the views are left as they were
- **THEN** the next command applies that entry to the views before it answers, and its answer counts that entry as the log's checks judge it

#### Scenario: One transaction

- **WHEN** writing a proposal's version to the views fails
- **THEN** the command exits 3, and the proposal's entry is not in the log either

#### Scenario: Views of an older form

- **WHEN** a store's views were written in an earlier form and `boom query --repository shop --json` runs
- **THEN** the views are built again from the whole log, and the answer is the one a full replay gives

## ADDED Requirements

### Requirement: A command answers from the views

A command SHALL answer from the views, once they are at the log's last entry. It SHALL check that the views' head names the log's last entry and its hash, and SHALL NOT read the entries before the head. A write SHALL judge its entry against the state the views hold.

#### Scenario: A large log

- **WHEN** a store's log holds 20 000 entries, its views are at its last entry, and `boom query --repository shop --json` runs
- **THEN** boom reads one entry of the log, the last, and answers

#### Scenario: A large log, three entries behind

- **WHEN** another program appends three entries to that log and `boom query --repository shop --json` runs
- **THEN** boom reads the views' head entry and the three entries after it, and no other entry of the log

#### Scenario: A write on a large log

- **WHEN** `boom propose` runs on that log with its views at its last entry
- **THEN** boom reads the log's last entry, and no other, before it appends, and reads as many as it does on a log of ten entries

### Requirement: boom verify checks the log and rebuilds the views

`boom verify` SHALL check the whole chain and replay the whole log. It SHALL compare each view with the replay, build again any view that differs, and name each one it built again. Where the chain does not hold, it SHALL report the store as altered, naming the first entry whose chain fails, exit 3 and write nothing.

#### Scenario: Nothing differs

- **WHEN** `boom verify --json` runs on a store whose log and views agree
- **THEN** it exits 0, says the chain holds, and names no view built again

#### Scenario: A view edited by hand

- **WHEN** a standing version's statement in the `versions` table is changed by another program, and `boom verify --json` runs
- **THEN** it exits 0, names `versions` as built again, and `boom query` then gives the statement the person accepted

#### Scenario: An older entry changed

- **WHEN** the text of entry 2 of a three-entry store is changed in the file, and `boom verify` runs
- **THEN** it exits 3 naming entry 2 as altered, and the store is as it was
