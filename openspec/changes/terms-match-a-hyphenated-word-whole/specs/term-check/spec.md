## MODIFIED Requirements

### Requirement: The words that the check matches

An old word SHALL match the same word in another form, case or accent, as `boom search` matches a word. An old word of two or more words SHALL match those words in that order. Boom SHALL split a name in code into its words at `_`, `-`, `.` and a change from lowercase to uppercase. Boom SHALL NOT match an old word inside a longer word. An old word that holds a hyphen SHALL match only its words joined by `-`, `_` or a change of case.

#### Scenario: Another form

- **WHEN** a file of `boom` holds `Keepers` and `KEEPER` <!-- boom: allow keeper -->
- **THEN** the check names both, each as `keeper`, with `owner` <!-- boom: allow keeper -->

#### Scenario: A name in code

- **WHEN** a file of `boom` holds `keeper_id` and `KeeperStore` <!-- boom: allow keeper -->
- **THEN** the check names both

#### Scenario: Inside a longer word

- **WHEN** a file of `boom` holds `gatekeeper` and no other form of `keeper` <!-- boom: allow keeper -->
- **THEN** the check finds no old word

#### Scenario: Two words

- **WHEN** a term replaces `drive log`, and a file holds `the Drive Log` on one line
- **THEN** the check names `drive log`

#### Scenario: A hyphenated old word in a name in code

- **WHEN** the term `origin` that replaces `came-from` is accepted for `boom`, and a file of `boom` holds `--came-from`, `came_from`, `CameFrom` and `cameFrom` <!-- boom: allow came-from -->
- **THEN** the check names each of the four, each as `came-from`, with `origin` <!-- boom: allow came-from -->

#### Scenario: The plain words of a hyphenated old word

- **WHEN** that term is accepted, and a file of `boom` holds `The file that it came from.`, `Came From` and `x.came.from`
- **THEN** the check finds no old word

#### Scenario: Both on one line

- **WHEN** that term is accepted, and a line holds `It came from the came-from flag.` <!-- boom: allow came-from -->
- **THEN** the check names `came-from` one time on that line <!-- boom: allow came-from -->
