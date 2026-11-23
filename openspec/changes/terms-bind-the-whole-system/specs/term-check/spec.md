## MODIFIED Requirements

### Requirement: boom terms check names each old word

`boom terms check <file>...` SHALL read each file as text. For each file, it SHALL use the accepted terms that stand for the repository of that file: its own, and those of the whole system. It SHALL give each old word that a file holds. For each, it SHALL give the file, the line, the old word as written, and the term that replaces it. It SHALL append nothing. It SHALL exit 0 when it finds no old word, and 2 when it finds one.

#### Scenario: An old word found

- **WHEN** the term `owner` that replaces `keeper` is accepted for `boom`, and `boom terms check notes.txt` runs on a file of `boom` whose line 3 holds `The keeper signs.` <!-- boom: allow keeper -->
- **THEN** boom exits 2, and standard error names `notes.txt`, line 3, `keeper` and `owner` <!-- boom: allow keeper -->

#### Scenario: No old word

- **WHEN** `boom terms check notes.txt` runs on a file of `boom` that holds no old word
- **THEN** boom exits 0, and the log does not change

#### Scenario: Answered as JSON

- **WHEN** `boom terms check notes.txt --json` runs on that file with `keeper` on line 3 <!-- boom: allow keeper -->
- **THEN** standard output is one `boom-refused/1` object. Its `refused.found` holds `file` `notes.txt`, `line` 3, `word` `keeper`, `term` `owner` and `repository` `boom` <!-- boom: allow keeper -->

#### Scenario: Nothing found, as JSON

- **WHEN** `boom terms check notes.txt --json` runs on a file of `boom` that holds no old word
- **THEN** standard output is one `boom-terms-check/1` object that names `notes.txt` with its repository `boom`, and `found` is empty

#### Scenario: A term of another repository

- **WHEN** the term `owner` that replaces `keeper` is accepted for `sluice` only, and `boom terms check notes.txt` runs on a file of `boom` that holds `keeper` <!-- boom: allow keeper -->
- **THEN** boom exits 0

#### Scenario: A term of the whole system in each repository

- **WHEN** the term `link` that replaces `hop` is accepted for the whole system, and `boom terms check` runs on a file of `boom` and a file of `sluice`, each of which holds `hop`
- **THEN** boom exits 2, and standard error names `hop` and `link` in the whole system for each file

#### Scenario: A term of the whole system, as JSON

- **WHEN** `boom terms check --json` runs on that file of `sluice`
- **THEN** its `refused.found` holds `word` `hop`, `term` `link` and `repository` `null`

#### Scenario: A repository with its own term for the word

- **WHEN** the term `link` of the whole system replaces `hop`, the term `link` of `boom` replaces `tie`, both are accepted, and `boom terms check` runs on a file of `boom` that holds `hop` and `tie`
- **THEN** boom exits 2, and standard error names `tie` with the term `link` of `boom`, and does not name `hop`

#### Scenario: A word that no term replaces

- **WHEN** the term `link` of the whole system is accepted with no old word `chain`, and a file of `boom` holds `the hash chain of the log`
- **THEN** the check finds no old word
