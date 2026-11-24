## MODIFIED Requirements

### Requirement: The origin of a claim

An `origin` edge SHALL name a file that boom keeps, a file in a commit, or an elaboration. For `--origin <file>`, boom SHALL keep the bytes of the file under their SHA-256 fingerprint, with the name of the file. Boom SHALL store that fingerprint. For `--origin git:<repository>@<commit>:<path>`, boom SHALL store that reference. The commit SHALL be 40 hexadecimal characters. The path SHALL be relative, with no `..`. Boom SHALL NOT read a repository.

#### Scenario: A mockup kept by its fingerprint

- **WHEN** an agent proposes a claim with `--origin mockup.svg`
- **THEN** boom exits 0, and the entry stores the name `mockup.svg` and the fingerprint of its bytes

#### Scenario: A file in a commit

- **WHEN** an agent proposes a claim with `--origin git:sluice@<40 hexadecimal characters>:design/overview.svg`
- **THEN** boom exits 0, and the entry stores that reference. `boom show` gives the repository, the commit and the path

#### Scenario: A short commit

- **WHEN** an agent proposes a claim with `--origin git:sluice@1a2b3c:design/overview.svg`
- **THEN** boom exits 2 and says that a commit has 40 hexadecimal characters, and boom appends nothing

#### Scenario: A commit with no path

- **WHEN** an agent proposes a claim with `--origin git:sluice@<40 hexadecimal characters>`
- **THEN** boom exits 2 and says that a reference to a commit names the path of a file in it, and boom appends nothing

#### Scenario: A path out of the tree

- **WHEN** an agent proposes a claim with `--origin git:sluice@<40 hexadecimal characters>:../secret.txt`
- **THEN** boom exits 2 and says that the path is relative, with no `..`, and boom appends nothing

## ADDED Requirements

### Requirement: An origin names the elaboration that a version came from

`--origin elaboration:<repository>/<name>` SHALL name the elaboration that a version came from, as sounding names it. The repository SHALL be in kebab case. The name SHALL be lowercase letters, digits and hyphens, beginning and ending with a letter or a digit. A version SHALL name one elaboration at most. Boom SHALL NOT read the elaboration. `boom show` SHALL give its repository, its name and its reference.

#### Scenario: A narrative from an elaboration

- **WHEN** an agent proposes a narrative with `--origin elaboration:boom/the-design-book`
- **THEN** boom exits 0, and the entry stores the elaboration with the repository `boom` and the name `the-design-book`. `boom show <name>@<version> --json` gives the origin with `repository` `boom`, `elaboration` `the-design-book` and `reference` `elaboration:boom/the-design-book`

#### Scenario: An elaboration with no name

- **WHEN** an agent proposes a claim with `--origin elaboration:boom`
- **THEN** boom exits 2 and says that an elaboration is `elaboration:<repository>/<name>`, and boom appends nothing

#### Scenario: A name out of its form

- **WHEN** an agent proposes a claim with `--origin elaboration:boom/The_Book`
- **THEN** boom exits 2 and says what an elaboration's name holds, and boom appends nothing

#### Scenario: Two elaborations

- **WHEN** an agent proposes a claim with `--origin elaboration:boom/one` and `--origin elaboration:boom/two`
- **THEN** boom exits 2 and says that a version came from one elaboration at most, and boom appends nothing

#### Scenario: Two elaborations appended by hand

- **WHEN** another program appends a proposal whose origin names two elaborations, chained correctly
- **THEN** `boom log --json` gives the entry as not counted, saying why, and `boom show` has no such version
