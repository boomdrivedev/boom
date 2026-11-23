# Spec Delta

## Purpose

What boom keeps of a narrative: an explanation in its author's words,
with a title, a body of Markdown text, and the claims that it includes,
each at the version that it meant. A person reads it, accepts it or
rejects it, and no one checks it
(`views-computed-from-the-ledger`, `a-link-remembers-the-version-it-meant`,
`claims-cite-claims-by-name`).

## ADDED Requirements

### Requirement: A narrative has a title and a body

`boom propose narrative <name> "<title>" --body <file>` SHALL propose a version of a narrative. The statement SHALL be the title. Boom SHALL refuse a narrative with no `--body`. Boom SHALL refuse a body that is not UTF-8 text. Boom SHALL keep the body as a blob, by its fingerprint. A narrative SHALL bind repositories, contexts or the whole system, as each other claim binds them.

#### Scenario: A narrative proposed

- **WHEN** `boom propose narrative boom-end-to-end "Boom, end to end" --body narrative.md --repository boom --provenance run-1` runs
- **THEN** boom exits 0, and `boom show boom-end-to-end --json` gives a narrative with one proposed version, the title as its statement, and the bytes of `narrative.md` as its body

#### Scenario: A narrative with no body

- **WHEN** an agent proposes a narrative with no `--body`
- **THEN** boom exits 2 and says that a narrative needs a body, and boom appends nothing

#### Scenario: A body that is not text

- **WHEN** an agent proposes a narrative whose body file holds bytes that are not UTF-8
- **THEN** boom exits 2 and says that the body of a narrative is UTF-8 text, and boom appends nothing

### Requirement: A narrative includes claims at the version it meant

`--includes <name>` and `--includes <name>@<version>` SHALL name a claim that the narrative includes. Boom SHALL resolve each as it resolves `--source`. Boom SHALL refuse `--includes` on a claim that is not a narrative. Boom SHALL refuse an include that names a narrative.

#### Scenario: A name stores its accepted version

- **WHEN** `status-read-from-events@<v1>` is accepted, and an agent proposes a narrative with `--includes status-read-from-events`
- **THEN** the entry of the proposal stores the edge `includes` to `status-read-from-events@<v1>`

#### Scenario: A claim that the store does not hold

- **WHEN** an agent proposes a narrative with `--includes record-0005`, and no claim has that name
- **THEN** boom exits 2 and names `record-0005`, and boom appends nothing

#### Scenario: Includes on a decision

- **WHEN** an agent proposes a decision with `--includes status-read-from-events`
- **THEN** boom exits 2 and says that only a narrative includes claims, and boom appends nothing

#### Scenario: A narrative included

- **WHEN** an agent proposes a narrative with `--includes boom-end-to-end`, and `boom-end-to-end` is a narrative
- **THEN** boom exits 2 and says that a narrative includes claims of other kinds, and boom appends nothing

### Requirement: The paragraphs of a body

Boom SHALL read a body as CommonMark with tables. Boom SHALL number the paragraphs of a body from 1, in order. A paragraph SHALL be one block at the top of the body, or one item of a list at the top of the body. A block is a heading, a paragraph of text, a code block, a block quote or a table. A thematic break SHALL NOT be a paragraph.

#### Scenario: A heading, text and a list

- **WHEN** a narrative's body is a heading, a paragraph of text, and a list of two items, and `boom show <name>@<version> --json` runs
- **THEN** `paragraphs` gives four paragraphs, numbered 1 to 4, of the kinds `heading`, `text`, `item` and `item`

#### Scenario: A nested list

- **WHEN** an item of a list at the top of a body holds a nested list of three items
- **THEN** `paragraphs` gives the item as one paragraph, and no paragraph for an item of the nested list

#### Scenario: A thematic break

- **WHEN** a body holds two paragraphs of text with a thematic break between them
- **THEN** `paragraphs` gives two paragraphs

### Requirement: boom show gives the paragraphs of a version

The fingerprint of a paragraph SHALL be the first 12 hexadecimal characters of the SHA-256 of its bytes in the body. `boom show <name>@<version> --json` SHALL give `paragraphs`, each with its number, fingerprint and kind.

#### Scenario: A paragraph's fingerprint

- **WHEN** paragraph 2 of a narrative's body is the bytes `Boom keeps claims.`, and `boom show <name>@<version> --json` runs
- **THEN** the fingerprint of paragraph 2 is the first 12 hexadecimal characters of the SHA-256 of those bytes

### Requirement: A narrative is judged by a person and never checked

A person SHALL accept or reject a version of a narrative with `boom accept` or `boom reject` and a key, as for each claim. One `boom accept` SHALL take a narrative with the claims that it includes. No hook SHALL give a narrative. Session start SHALL give no line for a narrative.

#### Scenario: A design accepted with one touch

- **WHEN** an agent proposes two invariants and a narrative that includes them, and ada runs `boom accept` with the three versions and ada's key
- **THEN** boom exits 0, the key signs one time, and the three versions are accepted

#### Scenario: Session start gives no narrative

- **WHEN** a narrative with two open notes and an invariant are accepted for `shop`, and a session starts in a worktree of `shop`
- **THEN** the text gives the statement of the invariant as a rule, and no line names the narrative or its notes

### Requirement: boom show gives a narrative

For a narrative, `boom show <name>` and `boom show <name>@<version>` SHALL give the title, the body and each included claim as `<name>@<version>`. They SHALL give each note on the version with its name, person, paragraph, words, and `open` or its answer. Boom SHALL mark an included version out of date when its claim has another accepted version.

#### Scenario: An include out of date

- **WHEN** a narrative includes `status-read-from-events@<v1>`, and a person then accepts `status-read-from-events@<v2>`
- **THEN** `boom show <narrative>` marks the include out of date, and names `<v2>`

#### Scenario: An open note shown

- **WHEN** chuck writes a note on paragraph 3 of a narrative's version
- **THEN** `boom show <name>@<version> --json` gives the note with its name, `chuck`, paragraph 3, its words and `open`
