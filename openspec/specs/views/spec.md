# views Specification

## Purpose
What boom computes as a view of its narratives: the outline of a view,
a claim that a person accepts and that holds narratives in order, and
the design book, whose parts come from the level that each narrative
binds and whose order comes from the outline. A narrative names no
view, so many views can hold it
(`views-computed-from-the-ledger`, `a-link-remembers-the-version-it-meant`,
`claims-cite-claims-by-name`).

## Requirements

### Requirement: An outline holds the narratives of a view

`boom propose outline <name> "<title>" --view <view> --narrative <name>` SHALL propose a version of an outline. The statement SHALL be the title. `--view` SHALL be required, and Boom SHALL refuse each view but `book`. Boom SHALL refuse an outline with no `--narrative`. A body MAY be given with `--body`.

#### Scenario: An outline proposed

- **WHEN** the store holds the narratives `boom-end-to-end` and `the-merge-queues-flow`, and `boom propose outline boomdrive-design-book "The design book of Boomdrive" --view book --narrative boom-end-to-end --narrative the-merge-queues-flow --provenance run-1` runs
- **THEN** boom exits 0, and `boom show boomdrive-design-book --json` gives an outline with one proposed version, the view `book`, and the narratives `boom-end-to-end` and `the-merge-queues-flow` in that order

#### Scenario: No view

- **WHEN** an agent proposes an outline with no `--view`
- **THEN** boom exits 2 and says that an outline names its view, and boom appends nothing

#### Scenario: A view that boom does not compute

- **WHEN** an agent proposes an outline with `--view context-map`
- **THEN** boom exits 2 and says that the view is `book`, and boom appends nothing

#### Scenario: No narrative

- **WHEN** an agent proposes an outline with `--view book` and no `--narrative`
- **THEN** boom exits 2 and says that an outline holds one narrative at least, and boom appends nothing

### Requirement: An outline names each narrative at the version it meant

Boom SHALL resolve each `--narrative <name>` and `--narrative <name>@<version>` as it resolves `--source`, and SHALL keep the narratives in the order given. Boom SHALL refuse a name that no claim has, a version that its claim does not have, a claim that is not a narrative, and a narrative given two times. Boom SHALL refuse `--narrative` on a claim that is not an outline.

#### Scenario: A name stores its accepted version

- **WHEN** `boom-end-to-end@<v1>` is accepted, and an agent proposes an outline with `--narrative boom-end-to-end`
- **THEN** the entry of the proposal stores the edge `narratives` to `boom-end-to-end@<v1>`

#### Scenario: A name that does not resolve

- **WHEN** an agent proposes an outline with `--narrative what-boomdrive-delivers`, and no claim has that name
- **THEN** boom exits 2 and names `what-boomdrive-delivers`, and boom appends nothing

#### Scenario: A claim that is not a narrative

- **WHEN** an agent proposes an outline with `--narrative status-read-from-events`, and `status-read-from-events` is an invariant
- **THEN** boom exits 2 and says that an outline holds narratives, and boom appends nothing

#### Scenario: A narrative given two times

- **WHEN** an agent proposes an outline with `--narrative boom-end-to-end --narrative boom-end-to-end@<v1>`
- **THEN** boom exits 2 and says that the outline holds `boom-end-to-end` two times, and boom appends nothing

#### Scenario: Narratives on a decision

- **WHEN** an agent proposes a decision with `--narrative boom-end-to-end`
- **THEN** boom exits 2 and says that only an outline holds narratives, and boom appends nothing

#### Scenario: Another order is another version

- **WHEN** an agent proposes an outline with two narratives, and then the same outline with the two narratives in the other order
- **THEN** the two proposals give two versions

### Requirement: An outline binds the whole system

Boom SHALL attach each version of an outline to the whole system. Boom SHALL refuse `--repository` and `--context` on an outline.

#### Scenario: Proposed in a worktree of a repository

- **WHEN** an agent proposes an outline in a worktree of `boom`, with no `--system`
- **THEN** the entry of the proposal holds no `attach`

#### Scenario: An outline with a repository

- **WHEN** an agent proposes an outline with `--repository boom`
- **THEN** boom exits 2 and says that an outline binds the whole system, and boom appends nothing

### Requirement: A store has one design book

Boom SHALL refuse the first version of an outline of the view `book` while another outline of the view `book` has a version that is not rejected, and SHALL name that outline.

#### Scenario: A second book

- **WHEN** a version of the outline `boomdrive-design-book` is proposed, and an agent proposes a first version of the outline `another-book` with `--view book`
- **THEN** boom exits 2 and names `boomdrive-design-book`, and boom appends nothing

#### Scenario: A book after a rejected book

- **WHEN** a person rejected each version of the outline `boomdrive-design-book`, and an agent proposes the outline `another-book` with `--view book`
- **THEN** boom exits 0

### Requirement: An outline is judged by a person and never checked

A person SHALL accept or reject a version of an outline with `boom accept` or `boom reject` and a key, as each claim. No hook SHALL give an outline. Session start SHALL give no line for an outline.

#### Scenario: An outline accepted with its narratives

- **WHEN** an agent proposes a narrative and an outline that holds it, and ada runs `boom accept` with the two versions and ada's key
- **THEN** boom exits 0, the key signs one time, and the two versions are accepted

#### Scenario: Session start gives no outline

- **WHEN** an outline and an invariant of the whole system are accepted, and a session starts in a worktree of `shop`
- **THEN** the text gives the statement of the invariant as a rule, and no line names the outline

### Requirement: The parts of the design book

Boom SHALL compute the design book from the accepted version of the outline of the view `book`, or its newest version. Part I SHALL give the narratives that bind the whole system, then a section for each context of two repositories or more. Part II SHALL give a chapter for each repository that a narrative binds, directly or through a context: the narratives that bind the repository, then a section for each context of that repository alone.

#### Scenario: Three levels

- **WHEN** the narrative `the-drives-log` binds the whole system, `boom-end-to-end` binds `boom`, `the-merge-queue` binds the context `sluice-host-process`, whose globs name `sluice`, the outline holds the three, and `boom show boomdrive-design-book --json` runs
- **THEN** `book` gives `the-drives-log` in Part I, a chapter `boom` with `boom-end-to-end` in Part II, and a chapter `sluice` with a section `sluice-host-process` that gives `the-merge-queue`

#### Scenario: A context of two repositories

- **WHEN** a narrative binds a context whose globs name `flume` and `sluice`, and the outline holds it
- **THEN** `book` gives the context as a section of Part I, after the narratives that bind the whole system, and no chapter of Part II gives it

#### Scenario: A repository with no narrative

- **WHEN** `flume` is registered, and no narrative binds `flume` or a context of `flume`
- **THEN** `book` gives no chapter `flume`

### Requirement: A narrative's level in the book

A narrative's level SHALL be what its accepted version binds, or its newest version when none is accepted. A context's repositories SHALL be those that its accepted version binds, or its newest version when none is accepted. The book SHALL give each narrative at that version, and not at the version that the outline meant.

#### Scenario: A narrative that changes its level

- **WHEN** the outline holds `drive-architecture-model@<v1>`, which binds `boom`, and a person accepts `drive-architecture-model@<v2>`, which binds the whole system
- **THEN** `book` gives `drive-architecture-model@<v2>` in Part I, and the chapter `boom` does not give it

### Requirement: The order of the design book

Inside a level, the book SHALL give the narratives that the outline holds in the outline's order, then each other narrative of that level by name. The chapters of Part II, and the sections of a part or a chapter, SHALL come in the order of the outline's first narrative in each, then by name.

#### Scenario: The outline's order

- **WHEN** `boom-end-to-end` and `change-records` bind `boom`, and the outline holds `change-records`, then `boom-end-to-end`
- **THEN** the chapter `boom` gives `change-records` first, then `boom-end-to-end`

#### Scenario: The order of the chapters

- **WHEN** the outline holds `the-merge-queues-flow`, which binds `sluice`, before `boom-end-to-end`, which binds `boom`
- **THEN** Part II gives the chapter `sluice` first, then the chapter `boom`

#### Scenario: The same book from two runs

- **WHEN** `boom show boomdrive-design-book --json` runs two times, and `boom verify` drops and builds the views between the two runs
- **THEN** both runs print the same bytes

### Requirement: A narrative that the outline does not hold

The book SHALL give a narrative that the outline does not hold at the end of its level, marked as not in the outline. The book SHALL NOT give a narrative whose every version is rejected and that the outline does not hold. `boom show <outline>` SHALL name each narrative of the book that the outline does not hold, and `--json` SHALL give them as `unheld`.

#### Scenario: A narrative not in the outline

- **WHEN** `the-design-book` binds the whole system, the outline does not hold it, and `boom show boomdrive-design-book --json` runs
- **THEN** `book` gives `the-design-book` last in Part I, marked as not in the outline, and `unheld` gives `the-design-book`

#### Scenario: A rejected narrative

- **WHEN** a person rejected each version of `the-design-book`, and the outline does not hold it
- **THEN** `book` does not give `the-design-book`, and `unheld` does not name it

### Requirement: boom show gives an outline

`boom show <name>` and `boom show <name>@<version>` SHALL give an outline's title, view, body and each narrative that it holds as `<name>@<version>`, in order. Boom SHALL mark a narrative out of date when another version of it is accepted. `boom show <name>` SHALL also give the book as Boom computes it, and the narratives that the outline does not hold. With `--json`, it SHALL give `view`, `narratives`, `book` and `unheld`.

#### Scenario: A narrative out of date

- **WHEN** the outline holds `boom-end-to-end@<v1>`, and a person then accepts `boom-end-to-end@<v2>`
- **THEN** `boom show boomdrive-design-book` marks the narrative out of date, and names `<v2>`

#### Scenario: The book in words

- **WHEN** `boom show boomdrive-design-book` runs on the store of "Three levels"
- **THEN** boom gives `Part I. The system` with `the-drives-log`, and `Part II. The repositories` with `boom` and `sluice`, each narrative as `<name>@<version>` with its status
