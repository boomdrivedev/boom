## MODIFIED Requirements

### Requirement: A narrative has a title and a body

`boom propose narrative <name> "<title>" --body <file>` SHALL propose a version of a narrative. The statement SHALL be the title. Boom SHALL refuse a narrative with no `--body`. Boom SHALL refuse a body that is not UTF-8 text. Boom SHALL keep the body as a blob, by its fingerprint. A narrative SHALL bind one level.

#### Scenario: A narrative proposed

- **WHEN** `boom propose narrative boom-end-to-end "Boom, end to end" --body narrative.md --repository boom --provenance run-1` runs
- **THEN** boom exits 0, and `boom show boom-end-to-end --json` gives a narrative with one proposed version, the title as its statement, and the bytes of `narrative.md` as its body

#### Scenario: A narrative with no body

- **WHEN** an agent proposes a narrative with no `--body`
- **THEN** boom exits 2 and says that a narrative needs a body, and boom appends nothing

#### Scenario: A body that is not text

- **WHEN** an agent proposes a narrative whose body file holds bytes that are not UTF-8
- **THEN** boom exits 2 and says that the body of a narrative is UTF-8 text, and boom appends nothing

## ADDED Requirements

### Requirement: A narrative binds one level

A narrative SHALL bind one of these: the whole system, one repository, or one context. Boom SHALL refuse a narrative whose attachment names two repositories, two contexts, or a repository and a context, and SHALL say that a narrative binds one level. A narrative SHALL name no view: an outline holds it by name.

#### Scenario: A narrative of one repository

- **WHEN** an agent proposes a narrative with `--repository boom`
- **THEN** boom exits 0

#### Scenario: A narrative of two repositories

- **WHEN** an agent proposes a narrative with `--repository boom --repository sluice`
- **THEN** boom exits 2 and says that a narrative binds one level, and boom appends nothing

#### Scenario: A narrative of a repository and a context

- **WHEN** `sluice-host-process` is a context of the store, and an agent proposes a narrative with `--repository sluice --context sluice-host-process`
- **THEN** boom exits 2 and says that a narrative binds one level, and boom appends nothing

#### Scenario: An amendment keeps one level

- **WHEN** `boom-end-to-end@<v1>` binds `boom`, and an agent proposes a narrative with `--amends <v1>` and no `--repository`
- **THEN** boom exits 0, and the new version binds `boom`
