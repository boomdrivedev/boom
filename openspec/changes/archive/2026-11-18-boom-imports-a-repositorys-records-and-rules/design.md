# Design

## Context

`crates/boom` keeps a store's claims in a log in one SQLite file: an
agent proposes a version of a claim, a person accepts or rejects it with
a signature by their security key (record 0006), and every answer is
replayed from the log by one judge (`src/judge.rs`). Its commands are
`init`, `register`, `propose`, `accept`, `reject`, `show` and `log`
(`src/cli.rs`). A store's claims are invariants, decisions and
assumptions; each version holds a statement, scenarios and a body.

A repository coming to boom has records and rules already. This one has
MADR files under `docs/adr/`, their frontmatter `number`, `title`,
`status`, `date` and `decision-makers`, and an unnumbered one named in
its frontmatter (`the-rename-to-boomdrive.md`); and `core.md`, its
rules bullets `- **[core.1]** ...` under `###` headings in `## Rules`,
their details `- **core.1** Rules out: ...` under `## Details`. The
family's other repositories keep the same two forms: Flume's records
carry the same frontmatter, and Flume's `core.md` keeps its details on
`core-details.md` beside it (its core.61). The import reads those two
forms, and MADR as its own template writes it.

The interview of 2026-11-01 settled the import's shape: it proposes and
a person accepts, one command accepting the batch, one signed event each
(`import-proposes-a-person-accepts`); each decision is named by its
file's title part, a name the store holds refused for the importer to
name it otherwise (`import-names-from-files`, `one-name-space`); each
gets a scenario an agent drafted, marked drafted
(`import-drafts-scenarios`); a superseded file comes in as history,
named by its successor, and never stands; old numbers are kept, and an
import whose numbers the ledger holds is refused
(`legacy-numbers-kept-count-goes-on`); afterwards boom keeps the
records and the old tool writes none (`import-once`). Record 0005 keeps
these. The rest, and the rules, which the interview did not take up,
record 0007 settles, the construct stage's own, cheap to change.

The rules this design relies on: core.1 (a status read from events,
nothing edited), core.2 (no view stored as the truth: boom writes
nothing back into the repository), core.3 (the tests' keys made when
they run), core.5 (no test reads a Markdown file of the tree) and core.6
(every dependency locked). It reaches no control plane.

## Goals / Non-Goals

**Goals:**

- `boom import adr <dir>` and `boom import rules <page>` propose a
  repository's records and rules in their files' own words, with what
  the files lack drafted by the agent that runs them.
- A person accepts an import's versions with one command.
- Once imported, a record is boom's: its file is never taken again as
  the record.

**Non-Goals:**

- Numbers given at acceptance (`number-given-at-acceptance`,
  `ordinal-per-kind-ledger-wide`), and an import numbered on from the
  count where its numbers are held. The import keeps each number; the
  change that gives numbers computes them from the log and takes that
  option. Until then a second repository whose decision numbers clash
  is refused.
- Writing records or rules back into a repository: a view, `boom
  render` and what hands a repository its files, later changes.
- Records in Nygard's form or any but MADR's; an area page's `paths`,
  which attachments will take; the site's pages.
- Rejecting several versions at once.

## Decisions

### 1. Two commands over one module

`src/import.rs` reads a folder or a page into `Record`s, each with its
`source` (an ADR's file name, a rule's id), kind, name where the file
gives one, statement, body, `imported` and `history`. `cli` adds
`boom import adr <dir>` and `boom import rules <page>`, each taking
`--repository <name>`, which it requires, `--drafts <file>`,
`--provenance <run>` and `--dry-run` beside the global `--store` and
`--json`, and answering `boom-import/1`: the
`records`, each with its `source`, kind, name, version (none on a dry
run), statement, `imported`, `history`, scenarios and `needs`; the files
`skipped`, each with why; and `accept`, the versions to accept together
and the command that accepts them (decision 9). Both commands read and
check everything before anything is written, so a file that cannot be
read is refused, exit 2, with nothing appended. The repository's name,
kebab case, is the operator's: a store holds no list of repositories
yet, and the name is what tells two repositories' `0001` or `core.1`
apart. Not taken: one command that finds a repository's folders itself,
guessing a layout repositories do not share; a record's source as its
path on the host, which differs from host to host.

### 2. A folder of MADR files

A record is a `.md` file directly in the folder whose name begins with
digits and a hyphen, or whose frontmatter has `name`; any other file is
skipped. Its frontmatter is the YAML between a first line `---` and the
next, read with `yaml-rust2`, and only `name`, `number`, `title`,
`status`, `date` and `decision-makers` (a string or a list) are read
from it. Its statement is the first line outside the frontmatter and
outside a fenced block that opens `# `, without the `# `; its body the
bytes after that line, the blank lines that open it left out. Its name
is the frontmatter's `name`, else the file's name less `.md` and its
leading digits and hyphen. Its number is the file name's leading
digits, else the frontmatter's `number`, kept as four digits; both
given and differing is refused.

Its status is the frontmatter's `status`, its first word read without
case: `accepted` or `proposed` a version to stand, none read as
`accepted`; `superseded`, `deprecated` or `rejected` history (decision
6); any other refused, naming the file. A superseded status's successor
is the file a Markdown link in it names, else the record of the folder
whose number is the first number in it (`superseded by 0003`,
`superseded by ADR-0003`); none found is refused.

Lines are read as text, with no Markdown parser: a title, a status and
a bullet are each one line in the forms read. Not taken:
`pulldown-cmark`, whose events carry nothing these forms need.

### 3. A rules page

The rules are the bullets of the page's `## Rules` section, up to the
next `## `, that open `- **[<id>]** `; a line indented under a bullet
continues it, joined with one space, so a statement is one line. Each
rule's section is the `### ` heading above it. Its details are the
bullet opening `- **<id>** ` under the page's `## Details`, or, where
the page has none, on `<stem>-details.md` beside it, with its
continuation lines as written; details for an id that names no rule are
skipped. Each rule is an invariant whose statement is its words, whose
body is its details, and whose `imported` holds the page's file name,
the id and the section. A page with no `## Rules`, or no rule in it, or
one id twice, is refused naming it.

### 4. The agent that runs the import drafts; boom runs no model

A MADR file has no scenario, and a rule has an id where a claim needs a
name. The agent that runs the import drafts both, after reading the
files, into a drafts file:

```json
{"format": "boom-import-drafts/1",
 "records": {"0003-amounts-in-cents.md": {"scenarios": ["Given ..., then ..."]},
             "core.1": {"name": "status-from-events", "scenarios": ["..."]}}}
```

keyed by each record's `source`. A draft gives `scenarios` and may give
`name`; a rule needs both, a decision to stand needs scenarios, and a
history record needs neither. boom takes from a draft its scenarios and
name and nothing else: the statement and body are always the file's,
so what a person accepts says what the record said. Each imported
version's `imported.drafted` is true. A draft whose key names no record
of the import is refused, so drafts written for another folder are not
taken. `--dry-run` writes nothing and answers each record with its
`needs`, `a name` and `a scenario`, those its draft still lacks, exit 0
while the files read; without `--dry-run` an import with any need is
refused, naming each record and what it needs.

Not taken: boom running a drafting command, as the check's reviewer will
be run (`reviewer-is-an-agent-command`), which needs the owner-signed
settings that name the program (`ledger-settings-owner-signed`), not
built, and would have boom start a model for an agent already present.
Not taken: a rule named from its id, `core-1`, a number standing as a
name, which `cite-by-name` and `core.names-not-numbers` in the
interview rule out; the id stays as the rule's legacy alias.

### 5. What an imported entry holds

The `proposed` act gains `imported`, written after `claim` in the
signed part and left out where the proposal is not an import:

| field | holds |
|---|---|
| `repository` | the repository `--repository` names |
| `from` | the file's name, `0003-amounts-in-cents.md` or `core.md` |
| `form` | `madr` or `rules` |
| `legacy` | the number, `0003`, or the id, `core.1`; none for an unnumbered record |
| `status` | the file's status as written, or none |
| `date` | the file's date, or none |
| `deciders` | the file's decision-makers, a list |
| `section` | a rule's `###` heading, or none |
| `drafted` | true: the scenarios were drafted at import |

A claim's version gains `history`, `{"status": "superseded" |
"deprecated" | "rejected", "by": <successor's name> or none}`, left out
where it has none. `history` is part of a version's content and so of
its fingerprint, written last and only where present, so every version
proposed before keeps the fingerprint it has; `imported` is where a
version came from, not what it says, and is not. Both fields are
optional and added, so entries stay `boom-entry/1`.

The judge keeps each version's `imported` in the state it replays, for
the rules of decision 7. `boom show <name>@<version>` gives `imported`
and `history`, read from the entry that proposed the version, which the
`versions` view names, so the store's schema stays 1. Not taken: two
view columns, which would change the schema of every store made.

### 6. History

A version with `history` may hold no scenario, has no base, and is
proposed only by an import: `boom propose` has no flag for it. A person
accepts it as any version, by an accept the judge holds to its rules;
once accepted its status is `history`. It never stands, replaces no
version and is no claim's standing version, so a later first version
under its name is proposed as under any name with none standing. A
superseded version's `by` names its successor; the import checks it is
a record of the same folder, and the judge that it is a name, kebab
case, not its own.

Not taken: leaving such records out, which loses their numbers, so a
later decision could take one, and their place in the history; a
version that stands and is then replaced, which would stand for a
moment, and needs one claim superseding another, not built.

### 7. The import's rules are the judge's

The rules an imported proposal is held to are the judge's, so they hold
inside the write's transaction, where a second import racing the first
is judged again, and on every read (`validated-on-read`). A record's
source is its `repository`, `from` and `legacy`. For a proposal with
`imported`, after the rules every proposal keeps:

1. a version of some claim came in from the same source: the same name
   and fingerprint is a repeat, adding nothing, as any repeat is; any
   other is refused, naming the file and that version and saying to
   amend it with `boom propose --amends`, since boom keeps the record
   now and never takes its file again as the record;
2. else its name held by any claim, whatever its versions' statuses:
   refused, naming the claim and saying to name it otherwise in its
   draft;
3. a decision's number held by a version of another claim that came in
   with that number: refused, naming the number and that claim.

Two records of one import under one name are refused by the import
before the judge, naming both files. A new file in a folder imported
before is a new record, proposed as any other.

### 8. All or nothing

`Store::append_all` takes several entries, each with its signature and
blob, and appends them in one `IMMEDIATE` transaction, judging each in
turn against the state the ones before it built; a refusal of any rolls
back the transaction, so nothing is written, exit 2, and a repeat among
them adds nothing. `Store::append` becomes `append_all` of one. The
import's proposals and an accept of several versions both write through
it.

### 9. Accepted together

`boom accept` takes one or more `<name>@<version>` targets with one
`--key`. Every target is judged against the state, each after the ones
before it, before the key signs any, so a person is never asked to touch
their key for a batch that would be refused; each is then signed, an
entry of its own, and all are appended through `append_all`. Its answer
is `boom-accept/1` with `accepted`, a list holding each version as an
accept of that one version answers; with one target the fields it gives
now are given too, unchanged.

An import's `accept` lists every version it proposed or found repeated
whose file's status was not `proposed`, the history among them, and the
command that accepts them, `boom accept <name>@<version>... --key
<key>`. A file the old tool left proposed waits to be accepted on its
own.

Not taken: one entry accepting the batch, signed once, where the
interview asks one signed event each; an accept naming an import by an
id, a set an agent could add to after the person read it, where naming
each version by its fingerprint accepts exactly what was read.

### 10. The tests

`crates/boom/tests/import.rs` holds the import's tests, each named for
its scenario and run through `boom::cli::run` in a temporary folder,
with the tests' `SoftKeys`. Every MADR file, rules page and drafts file
a test reads is written by the test from a string in its code into its
own folder, data that only boom's import reads: no test reads a
Markdown file of the tree (core.5; boomdrivedev/blueprints principle 16,
whose amendment of 2026-11-15 leaves a tool reading its own input form,
as core.7 lets the change check's cases write changes into scratch trees
OpenSpec alone reads). `SoftKeys` counts the signatures it makes, so a
test asserts a refused batch signed nothing.

## Risks / Trade-offs

- [A batch of forty records asks forty touches of the person's key] →
  one signed entry each is the interview's; a person accepts in parts by
  naming fewer versions, and a single signature over a batch would be a
  new decision of the user's.
- [Reading lines and not a Markdown tree misreads an unusual file, a
  title in HTML or a rule's bullet in another form] → the dry run shows
  what was read before anything is proposed, and every refusal names its
  file; the person reads each version before accepting it.
- [A file with no status reads as accepted, so a Nygard-form record
  whose status is a section reads accepted] → the dry run gives its
  status as none, the body holds the section, and the person reads it
  before accepting.
- [An agent's drafted scenario misstates a decision] → it is marked
  drafted and read by the person who accepts it, and amended like any
  version later.
- [A boom older than this change counts an entry with `imported` or
  `history` for nothing, since it refuses fields it does not know] → the
  entries are read by a boom at least as new as the one that wrote them;
  a store is kept on its machine, written and read by the boom there.
- [A second repository's decisions whose numbers the store holds are
  refused] → as the user's `legacy-numbers-kept-count-goes-on` has it;
  numbering them on comes with the numbers.
