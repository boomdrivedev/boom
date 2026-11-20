## MODIFIED Requirements

### Requirement: The log is chained

Each entry SHALL carry the SHA-256 of the entry before it, the first entry the hash of nothing. Each command SHALL check the chain from the last entry the views were built at: that entry SHALL still have the hash the views' head names, and each entry after it SHALL follow the one before it. `boom verify` SHALL check the whole chain.

#### Scenario: The chain given

- **WHEN** `boom log --json` runs on a store of three entries
- **THEN** each entry gives its hash, and each after the first gives as its previous hash the hash of the one before it

#### Scenario: An entry appended by hand that does not follow

- **WHEN** another program appends an entry whose previous hash is not the hash of the log's last entry, and `boom query --repository shop` runs
- **THEN** it exits 3 naming that entry as altered, and the views are as they were

### Requirement: A changed or removed entry is found

Where the chain does not hold, boom SHALL report the store as altered, naming the first entry whose chain fails, and the command SHALL exit 3 writing nothing and answering nothing else. Each command SHALL find a change to the views' head entry or to an entry after it. `boom verify` SHALL find a change to any entry.

#### Scenario: The head entry's words changed

- **WHEN** the text of the last entry of a three-entry store is changed in the file, its views at that entry, and `boom show <name>` runs
- **THEN** it exits 3 naming entry 3 as altered

#### Scenario: An entry's words changed

- **WHEN** the text of entry 2 of a three-entry store is changed in the file, its views at entry 3
- **THEN** `boom show <name>` answers from the views, and `boom verify` exits 3 naming entry 2 as altered

#### Scenario: An entry removed

- **WHEN** entry 2 of a three-entry store is deleted from the file, and `boom verify` runs
- **THEN** it exits 3 naming the entry whose previous hash no longer matches, and the log is as it was
