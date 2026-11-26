## ADDED Requirements

### Requirement: Another tool appends its entry

`boom append <tool>.<type>` SHALL append an entry of a type that another tool owns. The act SHALL be two kebab names joined by a dot, and the tool SHALL NOT be `boom`. The signed part SHALL hold `format`, `act`, `at`, `rules`, `actor`, `links` and `content`, in that order. `--link <act hash>` SHALL name each link. `--content <file>` SHALL name one JSON object, `{}` when absent, which boom keeps as written and never reads.

#### Scenario: A landing appended

- **WHEN** the log holds the `recorded` entry `<r>`, and `boom append sluice.landed --link <r> --content landing.json --provenance "sluice host deneb"` runs, with `landing.json` an object
- **THEN** boom exits 0, and `boom log --json` gives an entry whose act is `sluice.landed`, whose `links` are `[<r>]`, and whose actor is an agent with that provenance

#### Scenario: A type of Boom

- **WHEN** `boom append boom.approved --provenance run-1` runs
- **THEN** boom exits 2 and says that `boom` names no other tool, and boom appends nothing

#### Scenario: A type with no tool

- **WHEN** `boom append landed --provenance run-1` runs
- **THEN** boom exits 2 and says that a type is `<tool>.<type>`, and boom appends nothing

#### Scenario: Content that is not an object

- **WHEN** `boom append sluice.landed --content list.json --provenance run-1` runs, with `list.json` a JSON list
- **THEN** boom exits 2 and says that the content is one JSON object, and boom appends nothing

### Requirement: Boom judges another tool's entry by its form, its actor and its links

Boom SHALL judge an entry of another tool's type by its form, its actor and its links, and by nothing else. Its actor SHALL be an agent with its provenance, or a person that the store registers, named with `--person <name>`. Boom SHALL refuse such an entry with a signature, and one of a person whom the store does not register.

#### Scenario: A person's hold

- **WHEN** the store registers ada, the log holds the `approved` entry `<a>`, and `boom append planning.held --link <a> --person ada` runs
- **THEN** boom exits 0, and the entry names ada and holds no signature

#### Scenario: A person the store does not register

- **WHEN** `boom append planning.held --person bo` runs, and the store does not register bo
- **THEN** boom exits 2 and names bo, and boom appends nothing

#### Scenario: A signed entry of another tool, appended by hand

- **WHEN** another program appends, chained correctly, an entry of the act `sluice.landed` with a signature
- **THEN** boom counts it for nothing, and `boom log --json` says that the key signs no entry of another tool

#### Scenario: Both an agent and a person

- **WHEN** `boom append planning.held --person ada --provenance run-1` runs
- **THEN** boom exits 2 and says that an entry has one actor, and boom appends nothing

### Requirement: The log names another tool's entry

`boom log` SHALL give the line of another tool's entry as its number, its act, each link by its first 12 characters, and its actor.

#### Scenario: The line of a landing

- **WHEN** entry 9 is `sluice.landed` with the link `<r>`, and `boom log` runs
- **THEN** line 9 starts with `9 sluice.landed <first 12 characters of r> by an agent`

### Requirement: boom lineage walks the links

`boom lineage <target>` SHALL give the target's entry, `back`, each entry that the target reaches through its links, and `forward`, each entry whose links reach the target. Each SHALL be each entry once, the nearest first, then by number. Each entry SHALL give its number, act, act hash, actor and links. With `--json`, the object's `format` SHALL be `boom-lineage/1`.

#### Scenario: Two entries that one step took in

- **WHEN** an `approved` entry links `<e1>` and `<e2>`, and `boom lineage <its act hash>` runs
- **THEN** `back` gives `<e1>` and `<e2>`, each once

#### Scenario: The claims of an approval out of date

- **WHEN** an approval covers `status-read-from-events@<v1>`, a person then accepts `<v2>`, and `boom lineage <the approval>` runs
- **THEN** the entry gives `status-read-from-events@<v1>`, marked out of date, naming `<v2>` as accepted

### Requirement: The target of a walk

The target of `boom lineage` SHALL be an act hash of 12 characters or more that starts one act hash alone, `<repository>:<name>` for the `recorded` entry of the newest version of a change record, or `<repository>:<name>@<version>`.

#### Scenario: From a change record back and forward

- **WHEN** the log holds, in order, `sounding.elaboration-ended` `<e>`, an `approved` `<a>` that links `<e>`, a `recorded` `<r>` of `boom:contexts-own-paths` that links `<a>`, and a `sluice.landed` `<l>` that links `<r>`, and `boom lineage boom:contexts-own-paths --json` runs
- **THEN** the entry is `<r>`, `back` is `<a>` then `<e>`, and `forward` is `<l>`

#### Scenario: A prefix that starts two act hashes

- **WHEN** two act hashes start with the 12 characters given to `boom lineage`
- **THEN** boom exits 2 and names both entries by their numbers

### Requirement: The views of the links

Boom SHALL keep the view `acts`, each entry's act hash, and the view `links`, each link of each counted entry, in step with the log in the same transaction as each append. `boom verify` SHALL build them again from the log, and give what the views hold.

#### Scenario: The views built again

- **WHEN** the views `acts` and `links` are deleted from the store file, and `boom verify` runs
- **THEN** boom builds them again from the log, and `boom lineage` gives what it gave before
