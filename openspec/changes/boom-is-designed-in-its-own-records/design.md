# Design

## Context

See proposal.md for why. The repository holds one decision record,
`docs/adr/0001-boom-is-an-open-source-repository-of-its-own-and-its-binary-is-boom.md`,
in MADR's bare variant: frontmatter giving `number`, `title`, `status`,
`date` and `decision-makers`; an H1 repeating the title; `Status:`,
`Deciders:` and `Sources:` lines; then `## Context and problem
statement`, `## Considered options`, `## Decision outcome` and
`### Consequences`. `check-adrs.sh` (`root:adrs`) holds every record to
a number of its own and tells the later of two to take the next free
one. `adrs.toml` configures a tool that writes a new record numbered
after the last. `docs/architecture/core.md` holds six rules, core.1 to
core.6, each opening `- **[core.N]**` with a details entry below, and
`CLAUDE.md` tells an agent to give a new rule "the next free number in
its area". `interview.md`, on main since sluice landed it as submission
2 and merged into this branch, holds the constitution interview's notes:
each requirement, rule and open question under a name, and the section
"What the change must take in" that this design carries out.

The site (`site/`, Astro with Starlight) has seven pages and four
walkthroughs that sketch boom; its build checks every internal link,
anchors included: `starlight-links-validator` 0.27.0 counts any element
with an `id` as a target, not headings alone. Its diagrams are
TypeScript under `site/src/diagrams/`, drawn at build time and checked
by `site:test` (`scripts/diagrams.ts --check`), which also holds the
README's copy of the start page's diagram, `docs/diagrams/boom.svg`, to
what the site draws. Its prose is checked by Vale (`site:prose`, `vale
src/content/docs`). It is published by `.github/workflows/pages.yml`,
which checks out the pushed commit alone. sluice lands each branch on
main as a merge commit, so main's first-parent history is the order
things landed, and a landing's own commits sit behind its merge.

The design this change writes down comes from these sources, cited by
date and words (`core.private-sources-by-words`), all but the
constitution interview kept outside this repository:

- the design session of 2026-10-31 on claims, verdicts and
  reconciliation, and its rulings;
- the design's log, decisions 18 to 42, of which 39 to 42 were accepted
  on 2026-11-01;
- the family's storage convention, by its words: five layers (host,
  worktree, clone, shared branch, content), a piece of state kept in the
  lowest layer that holds every reader it has; one protocol for writing
  a shared branch (its 5.2); recutils at rest on a shared branch, each
  record naming its format, JSON only in flight; a repository's
  declaration in `.config/<tool>.toml`; its rule 2, that a tool's
  records about one repository stay with that repository; and its
  decisions 2, 5, 14 and 15;
- the user's ruling of 2026-11-01 on millpond (request `req_ee1f3b7c`);
- the constitution interview of 2026-11-01, `interview.md`, in this
  repository;
- the awesome-lists research of 2026-11-01, three of whose items the
  user took into this design the same day (request `req_ee5dd28f`),
  cited with the public projects it names:
  [radicle-cob](https://github.com/radicle-dev/heartwood), whose
  collaborative objects order their changes in git by the graph they
  form, for an event's order;
  [rmcp](https://github.com/modelcontextprotocol/rust-sdk), the Rust MCP
  SDK, for `boom mcp`, with whether `boom mcp` offers an accept; and
  gitoxide's
  [`crate-status.md`](https://github.com/GitoxideLabs/gitoxide/blob/main/crate-status.md),
  which marks push and rebase not done, for how boom writes to git.

The session's rulings, as this change cites them:

| ruling | what the user ruled on 2026-10-31 |
|---|---|
| the status ruling | boom is designed on its own terms: a cell's status is a function of boom's records, and nothing runs between commands |
| Q1 | boom reconciles and judges on a host; a planner reads the judged backlog |
| Q2 | amend and supersede, both: an amendment is a new version of the same claim, a supersession a new claim |
| Q3 | verdicts are judged at shared heads only; a landing that cites a claim makes its cell due |
| Q4 | the gate reads its claims from the repository's committed slice; a claim is gated there once its sync lands |
| Q5 | an unclear check fails as unclear until a person answers with `boom check --answer` |
| Q6 | no map of what is built is kept beside the target: the accepted records are the target, and what is built is read off the ledger |

The decisions of the design's log this change cites, in open terms:

| decision | what it decides |
|---|---|
| 18 | a claim is any assertion or decision about the system |
| 24 | the ledger is plain-text recfiles in git, written by boom directly, a rejected push applied again and retried |
| 26 | a decision's reasoning is written when it is made |
| 27 | the design book is a view of narratives and claims |
| 28 | the map is records |
| 32 | a status is derived from events |
| 33 | existing MADR records are imported once, and boom writes them afterwards |
| 39 | verdicts are boom records, kept beside the claims |
| 40 | boom reconciles per repository: for one repository at a commit it reports which claims in scope have no current verdict, an agent on a host judges them with `boom judge`, it works with no other service, and a planner reads the judged backlog |
| 41 | a person whose role carries the permission accepts a record; never an agent, never the record's author |
| 42 | a proposal awaiting a person shows as a millpond signal where no other planner is connected |

The rules this design relies on: core.1 and core.2, which the records
restate for boom itself and which hold the design pages to being
computed; core.3, by which boom holds no model's credential;
`core.no-commercial-strategy`, which this change writes in core.4's
place and which bounds every word written here, with
`core.private-sources-by-words`; core.5, for the new shell scripts
(shellcheck and shfmt own `*.sh`), the fixtures (`*.md`, excluded as
prose) and the generator and its test (Biome owns `site/*.ts`); and
core.6, since every tool the new tasks run (sh, awk, git, bun, Vale) is
one the tree already pins or the host's own shell. The rule this change
writes first, `core.names-not-numbers`, binds everything after it.

## Goals / Non-Goals

**Goals:**

- Every decision of boom's design that the user has made, the
  interview's among them, is a record a reader can open here, and every
  requirement it implies is a clause a later change can cite by name.
- Nothing here is known by a number taken on a branch: a record, a rule
  and an item of the statement each have a name fixed when it is
  written, every citation names it, and two branches landing side by
  side never renumber or rename anything that landed.
- A site page states nothing about the design that it cannot link to a
  clause or a record, and the build proves each link.
- No commercial strategy reaches the repository
  (`core.no-commercial-strategy`), and no private source is cited but by
  its date and words (`core.private-sources-by-words`).

**Non-Goals:**

- Drawing `core.md` from these records beyond the seven rules the
  interview and the user's word on the research send there; the rules
  the records imply are the separate step the user decided on.
- Renaming record 0001 or core.1 to core.6: they keep their ids, cited
  only as legacy.
- Answering the statement's open questions, `open-signature-format`,
  `open-sync-without-pull-requests` and `open-after-landing-step` among
  them.
- Vale over the records and the statement: they are read by people and
  reviewed; the gate's prose check stays on the site's written pages.
- Any change in millpond, flume, sluice or the family's requirements
  style (Outside this change, below).

## Decisions

### What stays out, and the word owner

What stays out of every file this change writes is commercial strategy,
as `core.no-commercial-strategy` has it: no pricing, no plan for a paid
tier, no customer, and nothing saying how boom's design limits what it
does alone or leads its users to a paid product. The family's umbrella
name and its products, paid ones among them, may be named, and a paid
product may be said to work with boom by boom's command line, as any
caller does. A private source is cited by its date and its words, and
any number it gives, never by a path, a repository's name or a link a
reader cannot open (`core.private-sources-by-words`): "the design
session of 2026-10-31", "decision 40 of the design's log", "the family's
storage convention, its 5.2". No check holds either yet; review does,
as the user decided, and no word list is added to the tree, since one
would name what it forbids.

The people who grant roles and register keys in a ledger are its
**owners** (`owner-is-the-word`). **Operator** keeps the family's
meaning, the person who runs a host. No file this change writes says
"keeper".

### Names in this repository

`core.names-not-numbers` binds this repository as boom binds a project
(`boom-follows-its-own-scheme`, `boom-names-everything-now`).

- **Records.** Each new record is `docs/adr/<name>.md`. A name here is
  two or more lowercase words of letters and digits joined by hyphens,
  chosen when the record is written, so a citation of it in prose is
  never read as an ordinary word (`record a-person-accepts`, never
  `record accepts`). The frontmatter holds `name`, the file's name,
  then `title`, `status`, `date` and `decision-makers`; never `number`.
  Record 0001 keeps its file and its frontmatter as they are, legacy,
  and is cited as `record 0001`.
- **Rules.** Each new rule in `core.md` opens `- **[core.<name>]**`,
  the name as a record's, and its details entry opens
  `- **core.<name>**`. core.1, core.2, core.3, core.5 and core.6 keep
  their ids, legacy. core.4 is reworded as `core.no-commercial-strategy`
  and is its legacy alias, the details entry ending `Legacy id: core.4.`
  A rule that moves to an area page takes that area's prefix, its name
  unchanged.
- **The statement.** Each clause, global invariant, question and
  scenario opens with its name in bold brackets, `**[cite-by-name]**`,
  and is anchored by it on the site; nothing in the file is numbered
  (The statement's shape).
- **Citations.** A record is cited `record <name>`, or `record 0001`; a
  rule by its id, `core.names-not-numbers` or `core.5`; an item of the
  statement, inside the statement, by its bare name in a parenthesis,
  `(status-from-events, record five-records)`. Each is written plain so
  the checks read it; a code span quotes a form without citing it.
  Several records are cited each in its own form, `record seven-events
  and record a-person-accepts`, since `root:adrs` reads `record <name>`
  and nothing in "records a and b" after the first; this design's own
  paragraphs, which no check reads, write the shorter form. An id
  from outside this repository is cited as its place gives it, in a code
  span where it would read as one of this repository's: flume's
  `core.21`, sluice's statement, 114, decision 40, request
  `req_ee1f3b7c`. core.md's details of core.5 and core.6 quote flume's
  rules so.
- **The numbers a reader sees** are computed by the site at build from
  the order each record and item landed on main ("Record 0002",
  "Clause 12"), and are written into no file (The numbers the site
  shows).
- **`CLAUDE.md`.** Its rules paragraph says the decisions are
  `docs/adr/`, one MADR record each, its file named by its name, record
  0001 keeping its numbered file as legacy; and that a rule is sent to
  `core.md` or its area page with an id `<area>.<name>`, a name chosen
  when it is written and never the next free number
  (`core.names-not-numbers`), amended in the same commit as the decision
  that changes it. Its checks table lists the root's format, toolchain,
  records, statement, tests, specs and open changes.
- **`openspec/config.yaml`.** Its context says each record's file is
  named by its name, and names no `adrs.toml`.
- **`adrs.toml` goes**: the tool it configures takes the next number on
  a branch.

Where this repository cannot follow the scheme yet, said plainly in
record `names-not-numbers`:

- Record 0001 and core.1 to core.6 were numbered before the scheme.
  Renaming them would rewrite every citation of them, the churn the
  scheme exists to stop, so they keep their ids, legacy, and nothing new
  takes a number.
- Commits already on main cite by number; a landed commit cannot change
  without rewriting main.
- Ids from outside the repository are cited as their places give them
  until those places follow the scheme themselves.
- The family's requirements style gives a new clause the next free
  number and an insert a letter. boom's statement departs from that
  section of the style and says so in its preamble; changing the style
  is the style's own change.
- A ledger holds one name space across its kinds (`one-name-space`).
  This repository is not yet one ledger: a record, a rule and an item of
  the statement are three name spaces, and every citation carries its
  kind (`record`, `core.`, or a bare name inside the statement). The
  interview names the record `sync-by-one-pull-request` and the clause
  stating its outcome alike; nowhere else does a record share an item's
  name. Once boom keeps this repository, the second of two such names
  is refused when it is brought in, as any clash is.
- The numbers the site shows need main's whole history; a build without
  it fails rather than guess.

Set aside: giving record 0001 and core.1 to core.6 names now, which
rewrites every citation in the tree for no clash anyone has; a check of
commit messages, which the interview does not ask for and review holds.

### The rules this change sends to `core.md`

The interview sends five rules to `core.md` "with them, or in the
separate step that draws `core.md` from the records, as the user
chooses", and the user's word on the awesome-lists research (request
`req_ee5dd28f`) sends two more, an event's order under the ledger and
no accept from a surface an agent calls. This design sends all seven
with their records, each in its record's commit, as `CLAUDE.md` asks of
a rule and the decision behind it. `core.md`'s preamble gains: "Each rule is known by its id,
`core.<name>`, a name fixed when the rule is written. core.1, core.2,
core.3, core.5 and core.6 keep the numbers they were given before
`core.names-not-numbers`, as legacy ids, and core.4 is
`core.no-commercial-strategy`'s legacy alias; each is cited only as
legacy."

- **`### Names`**, a new section after `### The ledger`:

  > **[core.names-not-numbers]** A record, a rule, a clause, an
  > invariant or a scenario is identified by a name fixed when it is
  > written and unique in its repository, and every citation names it;
  > a number is given only from the order things were accepted or
  > landed on a line never rewritten, never taken on a branch, never
  > changed once given, and never used as an id or a citation.

  Details: Rules out: a branch's records numbered "after the last" and
  renumbered when main moves, as another repository's interview
  renumbered 22 records and 75 rules twice, rewriting every citation;
  the `adrs` command line's next number taken on a branch; `core.12`
  as a new rule's id; a citation by number. Not ruled out: record 0001
  and core.1 to core.6, cited only as legacy; a number on a page,
  computed from main's order; boom writing `0007-<name>.md` with the
  number it gave at acceptance. Source: record names-not-numbers. (The
  examples stay in code spans, so `root:adrs` reads them as quoted.)

- **`### The ledger`** gains, after core.2:

  > **[core.events-carry-their-order]** Each event carries an ordering
  > key of its own, fixed when it is written, so that the appends two
  > hosts make to one record, merged in either order, derive the same
  > status.

  Details: Rules out: a status read from an event's place in the
  ledger's history or in a file; an order taken from a clock, or from a
  count one host keeps; a write applied again to a new tip with its key,
  or a person's signature over it, made anew. Not ruled out: the
  ledger's history kept one line by the family's protocol; the check at
  the tip refusing an append that no longer holds; two hosts' lines of
  one file joined as `merge=union` joins them, read as the same status.
  Source: record events-carry-their-order; the awesome-lists research,
  2026-11-01, and radicle-cob.

- **`### Secrets and records`**: core.4 becomes

  > **[core.no-commercial-strategy]** No commercial strategy appears in
  > this repository: no pricing, no plan for a paid tier, no customer,
  > and nothing saying how boom's design limits what it does alone or
  > leads its users to a paid product. The family's umbrella name and
  > its products, paid ones among them, may be named.

  Details: Rules out: a tier's price or what a tier holds; a design
  choice explained by what it sells; a customer named. Not ruled out:
  naming the umbrella or a paid product; saying that a paid product
  works with boom by boom's command line, as any caller does. Source:
  record no-commercial-strategy-here; record 0001. Legacy id: core.4.

  And after it:

  > **[core.private-sources-by-words]** A source kept in a private
  > repository is cited by its date and its words, and any number it
  > gives, never by a path, a repository's name or a link a reader
  > cannot open.

  Details: Rules out: a path under a home directory, or a private
  repository's name, in a record, a page or a commit; a link a reader
  cannot open. Not ruled out: "the design session of 2026-10-31"; a
  public repository's path and clause, such as sluice's statement, 122.
  Source: record no-commercial-strategy-here.

- **`### Callers and plugins`**, a new section after `### The gate`:

  > **[core.callers-pin-boom]** A tool that calls boom pins boom's
  > release where it pins its other tools, and refuses, naming both, an
  > answer whose format major it was not built for.

  Details: Rules out: a caller running whatever boom is on the path; a
  caller reading a newer major as if it were its own. Not ruled out: a
  caller accepting fields it does not know within its major. Source:
  record the-command-line-contract; core.6, across tools.

  > **[core.plugins-in-this-repository]** The Claude Code plugin and the
  > herdr plugin live in this repository, each under `plugins/<name>` as
  > a moon project of its own with its language declared and a format,
  > lint, type check and test task each; each calls boom by its command
  > line or `boom mcp`, never its code, names the boom release it needs
  > and ships with that release.

  Details: Rules out: a plugin with no project of its own; a plugin
  linking boom's library. Not ruled out: a marketplace entry that points
  here. Source: record the-surfaces; core.5.

  > **[core.no-agent-surface-accepts]** No surface an agent calls
  > accepts a record: `boom mcp` offers reading and proposing, and an
  > accept only as a link a person opens to make it.

  Details: Rules out: an accept tool in `boom mcp`; a plugin's skill or
  hook that accepts; an agent's call that sets off a person's key
  prompt; a link that accepts when it is opened, or carries a
  signature. Not ruled out: `boom mcp`'s other tools, each an agent's
  act and none a person's, as record the-surfaces serves them; the link
  shown to the person by the agent, a plugin or a planner; the person
  opening it in their own pane, where the accept is signed with them
  present. Source: record the-surfaces; record a-person-accepts; the
  awesome-lists research, 2026-11-01.

### The records' form

One record per decision, in record 0001's form, each with `name`,
`title`, `status: accepted`, `date: 2026-11-01` and `decision-makers:
[Chuck Swanberg]` in its frontmatter, an H1 repeating the title, a
`Status: accepted by the user on 2026-11-01.` line, a `Deciders:` line
naming the dates of the rulings it rests on, and a `Sources:` line
citing them by their words (The records, below, gives each). A title has
no colon followed by a space, so the frontmatter stays a plain YAML
scalar. The considered options are a bulleted list, each option in
words, and the outcome opens "Chosen option: " and the option's words,
since an option cited by its place in a list is a citation by number. A
record names the records it leans on as `record <name>` and the rules by
id, and quotes a number given as an example, another repository's
`core.12` among them, in a code span. The records cite no clause: they are the decisions, and the
statement is drawn from them. The interview is cited as "the
constitution interview of 2026-11-01 (`interview.md`)" and its topic,
and the user's own words, where a record quotes them, as the interview
gives them.

The records are committed one at a time, in the order of the section
The records below, so the order the site numbers them in is the order
they are read in (The numbers the site shows). `names-not-numbers` is
first, in the commit that writes its rule and `CLAUDE.md`'s line; each
record that sends a rule to `core.md` writes it in its own commit. A
record cites records later in that order, `names-not-numbers` citing
`import-proposes-for-a-person` and `seven-events` citing
`a-person-accepts`, so `root:adrs` resolves every citation only once the
last record is committed, and the tasks prove the records there. The
commits between are a landing's own, behind its merge, and the gate
reads the candidate alone.

Set aside: one record for the whole design, which a citation could not
narrow to the decision it rests on; a record per interview item, which
splits one decision's reasoning across files.

### The product's design is the statement, not OpenSpec specs

`openspec/specs` holds what this repository's checks prove, and boom is
not built. The design of the product is the statement,
`design/requirements.md`, which the family's requirements style writes
for a model not yet designed. A later change that builds a part of boom
writes that part's specs, citing the clauses they satisfy by name. This
change's own specs, `record-identity`, `design-statement` and
`design-pages`, are the checks that hold the records, the rules, the
statement and the site to their names and citations.

### The statement's shape

`design/requirements.md`, titled `# boom — requirements`, in the order
the style sets and sluice's statement follows, each section named and
none numbered: a preamble, then `## Purpose`, `## Actors`,
`## Vocabulary`, `## Part A. What must hold of boom`,
`## Part B. Contracts across the seams`, `## Part C. Profiles`,
`## Global invariants`, `## Non-goals`, `## Environment givens`,
`## Questions the model must answer`,
`## Scenarios the model must satisfy` and `## What to deliver`. The
three parts keep the letters the style fixes; their sections are
`###` headings named for their concern (`### Names and numbers`), never
lettered or numbered.

A clause is a paragraph opening with its name in bold brackets,
`**[identity-fixed-at-proposal]**`, then its words. A global invariant,
a question and a scenario are list items opening the same way,
`- **[status-never-stored]**`. A name is two or more lowercase words
saying what the item holds, fixed when it is written: an amended item
keeps its name, and a withdrawn one's name is never held again. Every
item the interview names keeps the interview's name. A clause cites the
items it leans on by bare name, and the records behind it, in one
parenthesis at the point it leans on them: `(status-from-events, record
seven-events)`. Nothing in the file is numbered.

The preamble gives the mechanism rule, as the style does, and the rule
on names: every item is known by its name and cited by it; a number the
site shows beside one is computed when the site is built, from the order
the items first landed on main, and is written nowhere in the file; this
departs from the style's section on numbers, inserts and citations,
which gives a new clause the next free number, for the reason record
names-not-numbers gives, and the rest of the style holds.

Parts A and B name no command, file, format, field, service or tool; the
one thing of the world they name freely is git, which the givens fix.
Where an interview item names a mechanism, its mechanism-free invariant
is the clause in Part A or B, and the item itself, under its name, is
the clause of the profile in Part C that binds it. The statement's
content is the section The statement below.

### `root:adrs` and the root's tests

`check-adrs.sh`, POSIX sh and awk, with no argument reads the files git
tracks, from the workspace root; given a directory, it reads every file
under it as the tree, which is how its cases run. It fails, naming the
file and line of each, where:

- a file under `docs/adr/` is neither a legacy record, which the script
  names with its reason (`0001-boom-is-an-open-source-repository-of-its-own-and-its-binary-is-boom.md`,
  numbered before `core.names-not-numbers`), nor `<name>.md` with a name
  as above, a name opening with a number among those it refuses ("a
  record takes a name, never a number");
- a named record's frontmatter lacks `name`, `title`, `status` or
  `date`, its `name` is not its file's, or it holds `number`;
- two records hold one name, naming both files;
- a rule on a page under `docs/architecture/` does not open
  `- **[<id>]**` with an id that is the page's area, a full stop and a
  name, or a legacy id the script names with its reason (core.1, core.2,
  core.3, core.5, core.6); two rules hold one id, naming each line; a
  rule has no details entry `- **<id>**`; or a legacy id the script
  names is held neither as a rule's id nor by exactly one details
  entry's `Legacy id:`;
- a citation resolves to nothing. Outside code spans and fenced blocks,
  with lines joined so a citation that crosses a line break is read as
  one: `record <name>` (either case of its first letter) naming no
  record; `record` and four digits naming no legacy record; `core.` and
  a name naming no rule; `core.` and digits naming no legacy id or
  alias. The failure says what the citation should name: "a record
  takes a name" for a number no legacy record holds.

Nothing in it holds a number unique. It reads every tracked text file
but these, each exempt by name with its reason in the script:
`openspec/changes/` (a plan quotes the ids it changes);
`interview.md` (notes that quote the numbers their rules rule out, and
other repositories' ids); `tests/` (fixtures that break the rules on
purpose); lockfiles and `*.svg` (written by tools). Passing, it prints
how many records, rules and citations it read.

`check-adrs.test.sh` runs the script over the cases under `tests/adrs/`,
each a small tree with `docs/adr/`, `docs/architecture/core.md` and a
file that cites, asserting each exit status and that each failure names
what its case breaks:

- `holds/`, which passes: the legacy record, a named record, a legacy
  rule, a named rule, core.4 held as a details entry's legacy id,
  citations of each kind resolving, one split across a line, and
  `core.7` and `record 0002` inside code spans;
- `numbered/`: `docs/adr/0002-amounts-as-floats.md`, and a rule
  `[core.7]`;
- `frontmatter/`: a named record holding `number: 2`, and one whose
  `name` is not its file's;
- `shared/`: two records whose frontmatter holds one name, and one rule
  id on two lines of `core.md`;
- `unresolved/`: `record no-such-record`, `core.no-such-rule`, `record
  0002` and `core.7`, outside code spans.

The root's `moon.yml`: `adrs` keeps its command, its inputs become
`/**/*`, since any file may cite, and its comment says it holds every
record and rule to a name no other holds and every citation of one to
something that exists; `test` (inputs the two check scripts, the two
test scripts and `/tests/**/*`), `check-fast`, run from the workspace
root, running both test scripts.

Set aside: a check in `crates/boom`, which is boom itself and not a
check of this repository's prose; reading commit messages, which would
make the task depend on git's history and never replay.

### `root:statement`

`check-statement.sh`, POSIX sh and awk, takes the statement's path
(default `design/requirements.md`) and reads it whole, joining lines.
An item is a paragraph opening `**[<name>]**` in Parts A to C (from the
`## Part A` heading to the next `## ` heading after Part C), and a list
item opening `- **[<name>]**` under `## Global invariants`,
`## Questions the model must answer` and
`## Scenarios the model must satisfy`. A citation list is a
parenthesis, outside code spans, every comma-separated part of which is
a name, `record` and a name or four digits, or `core.` and an id. It
fails, naming each line, where:

- two items share a name, whatever their kinds;
- a line of those sections opens with a number and a full stop (`12.`,
  `93a.`: "an item takes a name");
- a name in a citation list names no item;
- a parenthesis every part of which is a citation form or a bare clause
  number (`12`, `93a`), one at least a number, cites by number:
  `(12)`, `(3, a-name)`.

Record and rule citations are `root:adrs`'. Passing, it prints how many
clauses, invariants, questions and scenarios it read.

`check-statement.test.sh` runs it over the cases under
`tests/statement/`, asserting each exit status and that each failure
names what its case breaks: `holds.md`, which passes (Parts A to C with
three clauses, a citation list `(first-clause,` ending a line and
`second-clause)` opening the next, two invariants, a question, two
scenarios, a parenthesis of prose holding a hyphened word among other
words, and a numbered list `1.` under `## What to deliver`, which holds
no items); `shared-name.md` (two clauses named `same-name`, and a clause
and a scenario both `other-name`); `unresolved.md` (`(no-such-clause)`);
and `numbered.md` (a line `12. …` in Part A, and `(12)` in another
clause). The root's `moon.yml` gains `statement` (inputs
`/design/requirements.md`, `/check-statement.sh`), `check-fast`, run
from the workspace root, and `.moon/workspace.yml`'s list of the root's
checks names records, statement and tests.

### The design pages, computed at build

`site/scripts/design.ts`, run by bun, writes
`site/src/content/docs/design/statement.md` from the statement and
`site/src/content/docs/design/decisions/<slug>.md` from each record, the
slug a record's name, or a legacy record's number (`0001`), emptying
that directory first. `.gitignore` names
`/site/src/content/docs/design/`; nothing commits it (core.2's reasoning
holds for these: written to a build directory, never read back as
input). The site's `moon.yml` gains `design-pages`, with no layer and
`cache: false` like `install`, since the numbers it writes come from
git's history; `typecheck` and `build` depend on it and take
`/design/requirements.md` and `/docs/adr/*.md` as inputs; `prose` runs
`vale --glob='!**/design/**' src/content/docs`. `package.json`'s `dev`
runs the script before `astro dev`.

- **The statement's page**, titled "The statement", its H1 dropped.
  Each item is written as a paragraph opening with an empty element
  whose `id` is its name, then its number in bold, as The numbers the
  site shows gives it ("Clause 12", "Invariant 3", "Question 2",
  "Scenario 4"), or "new" where main lacks it, then its name in a code
  span, then its words. Each name in a citation list becomes a link to
  `#<name>`, and each `record <name>` a link to that record's page. An
  item whose name is the id one of the page's headings takes fails the
  script, naming both, since the two ids would collide.
- **Each record's page**, at `/boom/design/decisions/<slug>/`, titled
  with the record's title, its description `<status>, <date>`, its
  `editUrl` the record's file on GitHub, its H1 dropped. It opens with a
  line giving its number ("Record 0002", or "Not yet on main") and how
  to cite it (`record names-not-numbers`); a legacy record's says it was
  numbered before `core.names-not-numbers` and is cited as `record
  0001`. Its sidebar order is its number, and records main lacks follow
  the rest by name. Each `record <name>` in it, and each link to
  another record's file, becomes a link to that record's page. A record
  whose frontmatter lacks its title, status or date fails the script,
  naming the file.
- **The sidebar** gains a group "Design", after "boom check", holding
  "The statement", by `slug: "design/statement"`, which Starlight
  refuses to build when no page has it, and "Decisions", generated from
  `design/decisions`, each label the record's number and title.

Set aside: links to the files on GitHub, where no clause has an anchor
and nothing checks a citation; a content loader composing Starlight's,
whose pages the link checker's pass would have to be shown to reach,
where writing files first is the way sluice's site already works.

### The numbers the site shows

A number on a design page is computed by `design.ts` each time it runs,
from main's first-parent history, and written nowhere else. The
computation is a pure function, `ordinals`, from a list of landings,
each the names it holds in order, to a number for each name, with git
read by a thin layer around it:

- **Which history.** The default branch is `main`, as moon's
  `vcs.defaultBranch` says; the script reads `refs/heads/main`, else
  `refs/remotes/origin/main`. Where neither exists, or the repository is
  shallow, it fails, naming the ref and saying the whole history is
  needed.
- **Landings.** The commits of `git log --first-parent --reverse main`
  that touch `docs/adr` or `design/requirements.md`, oldest first. A
  landing's own commits are those its merge brings, `<landing>^1..<landing>^2`,
  oldest first; a commit made on main directly is a landing of one.
- **Records** (`number-given-at-acceptance`,
  `ordinal-per-kind-ledger-wide`). A record is accepted at a commit when
  its file there says `accepted` or `superseded`. A legacy file,
  `NNNN-<slug>.md`, keeps the number its name gives
  (`legacy-numbers-kept-count-goes-on`), and the count goes on from the
  highest such number. Every other record takes the next number in the
  order records were first accepted: by landing, within a landing by the
  first of its own commits that holds it accepted, and within a commit
  by name. A record never accepted on main has none; a superseded one
  keeps its own.
- **Statement items.** Clauses, invariants, questions and scenarios are
  numbered apart, each from 1, in the order their names first appear in
  the statement on main: by landing, and within a landing in reading
  order. An item main lacks has none.
- **A name held again.** A name main's history once held and later lost
  is never held again: where a later landing, or the working tree, holds
  it, the script fails, naming it and the landing that withdrew it.

Since a landing only adds to the end of main's history, no landing
changes a number an earlier one gave, and two branches landing side by
side change nothing the other was shown. A build on a branch shows its
own new records and items with no number; the numbers come when they
land.

`site/scripts/design.test.ts`, under `bun test`, holds `ordinals` to
that with landings written as data: two branches each adding a record
and landing one after the other; one landing bringing three records in
two commits; a record proposed in one landing and accepted in a later
one, after another; a legacy file whose number the count goes on from; a
statement whose second landing adds a clause between two of the
first's, numbered 3 while the others keep 1 and 2, its invariant,
question and scenario numbered apart; and a name withdrawn and held
again, which fails. `site:test` runs it beside the diagrams' check, its
inputs gaining `scripts/design.ts` and `scripts/design.test.ts`.

`.github/workflows/pages.yml` checks out with `fetch-depth: 0`, with a
comment saying the design pages' numbers need the whole history, and
builds on a push that touches `design/**` or `docs/adr/**` as well as
the site.

### How a page cites

A page cites at the end of the sentence, row or item that states what it
cites, by a link named by the name it cites:
`([identity-fixed-at-proposal](/boom/design/statement/#identity-fixed-at-proposal))`,
several as `([cite-by-name](…), [one-name-space](…); [record
names-not-numbers](/boom/design/decisions/names-not-numbers/))`. A rule
is cited by its id as plain text, `core.no-commercial-strategy`, which
`root:adrs` resolves. A page that only shows a command's output, the
commands page's examples, cites once per command, in its prose. A page
writes a name of boom's examples, a claim such as `amounts-in-cents`, in
a code span, so no check reads it as one of this repository's records.
The roadmap links record 0001 to its page rather than to GitHub.

### The walkthroughs follow record `a-person-accepts`

Record a-person-accepts rules out a person accepting a version they
proposed, and record a-persons-act-is-signed has every person's act
signed with them present. Every walkthrough where Ada proposes and
accepts changes so that her coding agent proposes, with the agent's
session as provenance, and Ada accepts, her key's touch signing it.
`first-claims` makes the ledger as a branch of `shop` and registers
Ada's key as its first owner, and shows once what boom says when she
runs `boom accept` on a claim she proposed herself: refused, exit 2,
since she is its author, naming who may accept it. `importing-adrs` has
the agent run the import, which proposes, and Ada accept the batch.

### The ledger's settings are events an owner signs

The repositories a ledger describes, each one's remote and planner, and
the reviewer's and the judge's commands and models are the ledger's
settings, each changed by an event an owner signs with them present
(`ledger-settings-owner-signed`). A repository's planner setting names
the one planner that takes its backlog and shows its proposals awaiting
a person: `millpond`, or the name another planner asks under. `boom
backlog --planner <name> --json` lists the backlog of the repositories
naming that planner and no other; `boom query --status proposed
--planner <name> --json` lists their proposals. A repository naming none
is planned by no tool; `boom backlog` with no `--planner` lists every
repository's for a person. A repository's `.config/boom.toml` names its
ledger, its repository and branch, and nothing the check trusts; where
each repository is checked out is each host's own setting, under
`$XDG_CONFIG_HOME/boom/` (`boom-state-by-layer`). The site sketches an
owner's acts as `boom owner set`, `boom owner grant` and `boom owner
key`, run in the owner's own pane.

Set aside: the settings as a table in a committed file, which a change
could edit to swap its own reviewer or an agent to name a planner;
millpond's own settings saying which repositories it plans, which
another planner would have to read to stay out; millpond as the planner
wherever none is named, which would plan a repository nobody chose to
have planned and have boom name a caller in its code.

### A backlog entry's id names its cell and its verdict

Each entry of `boom backlog --json` carries an id, the same on every
host and every run, naming the repository, the claim and the verdict
that put the cell in the backlog: `shop/invoices-final/7c1e0a4`, the
last part the verdict's id, derived from what the verdict records. A
caller takes each entry once. A cell judged again, not satisfied still,
is a new entry with a new id, which millpond's curation may attach to
the item it already sent; a cell judged satisfied is in no entry. The
statement calls it the entry's id, keeping "key" for a person's key.

### An event's order

Each event, when boom makes it, names the events of its record that the
writer had read and that no other read event follows, and carries an id
derived from what it says, those names among it
(`event-carries-its-order`). A claim's record, for this, is its events,
its verdicts and the due events naming them, so a verdict names the
claim's latest events and the cell's previous verdict as the judge's
host read them. Both are fixed when the event is made and covered by a
person's signature; a write the family's protocol applies again to a new
tip carries the same event. One event is after another when it names
it, or names an event after it; two neither of which is after the other
are concurrent, and where a status needs them in one order, as the
latest verdict of a cell, it takes them by their ids
(`after-is-what-the-writer-read`). A cell is stale when a `challenged`
event names its claim and its verdict is not after it, so a challenge
concurrent with a judgment stales the cell whichever lands first. An
event naming an event the ledger does not hold before it counts for
nothing (`validated-on-read`). The site's sketch writes the order as
`Id:` and one `After:` per event followed, and keeps `At:` as a note no
status reads; where the fields go is the layout's question
(`open-ledger-layout`).

Set aside: `merge=union` for the ledger's recfiles, the research's
cheaper trick, which keeps both sides' appended lines unordered: boom
applies its own write again to the new tip (record
the-ledger-on-a-shared-branch) and needs no textual merge, and with each
event carrying its order a union would derive the same status anyway.

### An accept offered as a link

An answer that names a version waiting on a person's accept, `boom
propose`'s, `boom supersede`'s and `boom query`'s for a proposed version
among them, carries `accept_link` beside the words naming the command:
`boom:accept?ledger=<ledger>&record=<name>@<version>`, a link of boom's
own scheme naming the ledger as a repository's `.config/boom.toml`
names it, the record by its name and the version, and nothing that acts
(`accept-offered-as-a-link`). `boom accept` takes the link as it takes
`<name>@<version>`: it shows the version, its scenarios and who
proposed it, then the key's prompt signs, with the person present
(record a-persons-act-is-signed). Opening a link accepts nothing; it
carries no signature and no token; an agent hands it to the person and
never opens it, and its call sets off no key prompt
(`core.no-agent-surface-accepts`). The herdr plugin opens a link in the
person's own pane; a planner may show it beside a proposal it lists
(record proposals-as-signals). boom serves no page and no network for
it. What opens a link on a host where no plugin does, a handler for
boom's scheme registered with the host that starts the person's
terminal on `boom accept`, or the person handing it to `boom accept`
themselves, is a question the statement keeps
(`open-accept-link-opener`).

Set aside: an accept tool in `boom mcp` that refuses an agent, which
would set off the person's key prompt from an agent's call; the words
naming the command alone, which a surface cannot open for the person.

### The design notes are clauses of Part C

Two choices of the stack come from the research: `boom mcp` built on
rmcp (`mcp-on-rmcp`), and every write boom makes to git through the
host's own `git` command, a library only reading
(`writes-through-the-hosts-git`). A choice of library is no rule and no
invariant: a model may displace it by saying why (the style's section
on givens). Each is a named clause of the Part C profile it binds, the
surfaces' and the store's, which the givens' list of the stack cites,
and records the-surfaces and the-ledger-on-a-shared-branch state each
among their outcomes.

### The millpond loop as a diagram

`site/src/diagrams/millpond.ts`, registered as `millpond` in
`src/diagrams/index.ts`, drawn with `kit.ts` like `fits`, wide and
narrow, CSS keyframes only, one still frame for a reader who asks for
less motion, millpond's, flume's and sluice's accents from the family
list as `fits` takes them. Its one claim: the loop closes in boom. The
stations, in order: `boom judge` judges a claim not satisfied; `boom
backlog --json` lists the entry; millpond takes it in as an item; the
item's send launches the work through flume, the launch naming the
claim, and the place's commits carry `Claim: <name>@<version>`; sluice
lands it; `boom reconcile` writes the cell due; `boom judge` judges it
satisfied, and the entry leaves the backlog. Beside the loop, a
millpond signal reaches boom as `boom challenge` and its cells return to
be judged. `fits`'s millpond lines become "boom challenge, boom propose"
and "boom backlog --json". `site/README.md`'s diagrams paragraph names
the new one.

## The records

Each paragraph is a record's substance, in the order they are
committed: its name and title, the question its context asks, the
options it weighs, the outcome and its consequences, and its sources.
The record says it in the voice of record 0001 and says nothing the
paragraph does not. An interview item named in a paragraph is its
substance, written into the record in words; the item's name stays the
statement's.

**`names-not-numbers`, A record is known by a name fixed when it is
proposed, and numbered only at acceptance.** What identifies a record,
and what does a citation name, so that work on two branches never
collides? In another repository of the family, a constitution
interview's branch numbered its records and rules after main's last; a
parallel branch landed records 0002 to 0016 and rules core.12 to
core.18 first, every number on the branch collided, and 22 records and
75 rules were renumbered twice, each citation rewritten. The user: "A
well-designed boom must make that impossible." Options: the next free
number, taken on the branch where the record is written, as the `adrs`
command line and the family's statements do; a number taken when the
work lands, the landing rewriting the branch's numbers and citations; a
name chosen by the proposer and fixed when it is proposed, held unique
when it reaches the default branch, every citation naming it, and a
number given only from the order of first acceptance on the ledger's
one-line history; an identifier no two proposers can share, random or
derived from the record's words. Outcome: the name. A record's identity
is its name, in kebab case, chosen by whoever proposes it and fixed when
it is proposed; it never depends on what else exists and never changes:
an amendment keeps it, a supersession is a new record with a new name
(`identity-fixed-at-proposal`). No two records of one ledger share a
name, whatever their kinds, since a citation carries no kind; the same
name at the same version brought by two branches is one proposal, and
the second adds nothing (`one-name-space`). A second record under a
taken name fails the check when it reaches the default branch, naming
the record that holds it, so a clash is caught at the merge and never
mended by a rename after it (`name-clash-fails-at-merge`). Every
citation, in a claim, a rule, a statement, code or a commit, names the
record, a claim at a version as `<name>@<version>` (`cite-by-name`). A
number a reader sees, a decision's 0007 or a clause's 12, is given from
the order of first acceptance on the ledger's one-line history, never
taken on a branch, never changed once given, and never used as an id or
in a citation (`number-given-at-acceptance`); it is a record's place
among the records of its kind across the ledger, counted on from the
highest number the ledger holds; a proposed or rejected record has
none, a superseded one keeps its own, and an amendment takes none
(`ordinal-per-kind-ledger-wide`). A record brought from another tool
keeps its old number, cited only as legacy (`legacy-number-alias`;
record import-proposes-for-a-person). Every other identity boom gives is
derived from what it identifies: a version from a record's words, a
verdict's id from what it records, a backlog entry's id from its
repository, claim and verdict. The first option falls on the
renumbering. The second because a branch's commits already carry the
numbers the landing would replace, and a gate that rewrites what it
lands lands something other than what it checked. The fourth because a
citation of it tells a reader nothing, and an identifier derived from
the words changes with an amendment, which a name must survive.
Consequences: boom's own repository follows the same scheme, or says
plainly where it cannot yet and why (`boom-follows-its-own-scheme`): in
it a decision record, a core rule and a clause, invariant, question or
scenario of the statement each carry a short kebab name; a record's file
is `docs/adr/<name>.md`, its frontmatter naming it and holding no
number; a rule's id is `core.<name>`; a clause opens with its name and
is anchored by it; a citation names it. The site shows numbers ("Record
0002", "Clause 12") computed at build from the order each landed on
main. A check in `check-fast`, run on the candidate, fails when two
records, two rules or two items of the statement share a name, and
nothing holds a number unique. Record 0001 keeps its file and core.1 to
core.6 their ids, cited only as legacy. Once boom keeps these records,
it writes them as any project's (record import-proposes-for-a-person)
(`boom-names-everything-now`). Rule core.names-not-numbers holds it.
Where this repository cannot yet: the legacy ids, which renaming would
rewrite every citation of; commits already on main; ids from outside the
repository, cited as their places give them; the family's requirements
style, which numbers clauses, and from which boom's statement departs;
the three name spaces of record, rule and statement item, each citation
carrying its kind, where a ledger holds one; the numbers the site shows,
which need main's whole history. Sources: the user, 2026-11-01, sending
the first plan of this change back, with the renumbering as its example;
the constitution interview of 2026-11-01 (`interview.md`), its topics on
identity and citation when branches race and on importing existing
decision records.

**`no-commercial-strategy-here`, No commercial strategy appears here,
and a private source is cited by its date and words.** What must stay
out of this repository, the public one that holds boom, and how does it
cite a source a reader cannot open? core.4 kept out pricing, a tier, a
customer and a hosted service, and this change's first brief kept out
more, the family's umbrella name and its products among them, though the
site is part of the family's. The user: "The things that should be kept
out would be strategy for paid tiers or how things we do in Boom might
limit or get people to a paid product." Options: core.4 as written; core.4
naming by kind everything the brief kept out; no commercial strategy,
the umbrella and the family's products named freely. Outcome: the
third: no pricing, no plan for a paid tier, no customer, and nothing
saying how boom's design limits what it does alone or leads its users to
a paid product; the umbrella name and the products, paid ones among
them, may be named, and a paid product may be said to work with boom by
boom's command line, as any caller does. For a private source, options:
its path; a link; its date and words and any number it gives. Outcome:
the last, never a path, a repository's name or a link a reader cannot
open; a public repository's path and clause is cited as it stands. The
first options fall because core.4 as written rules out naming the
family's products the site belongs beside, and a list by kind names
what it forbids; a path or a link tells a reader nothing they can open.
Consequences: core.4 is reworded as rule core.no-commercial-strategy and
stays its legacy alias; rule core.private-sources-by-words is added; no
check holds either yet, and review does ("we don't need a check yet");
no word list is added to the tree; commits already written stay as they
are. Sources: the user, 2026-11-01, in the constitution interview
(`interview.md`), its topic on what stays out of this repository; record
0001.

**`five-records`, The ledger holds claims, narratives, contexts,
relationships and terms.** What does boom record? Options: a store per
form (ADR files, a rules file, a map, a glossary); one ledger of five
kinds of record, every other form a view; one kind, the claim, with
narratives, the map and terms as claims. Outcome: five kinds. A
**claim** has a name, a kind (invariant, decision or assumption), a
statement, at least one scenario saying how one would know it holds, for
a decision a body (context, drivers, options, outcome, consequences,
written when it is proposed), what it attaches to, an optional check
rule, the claim it supersedes, and provenance. A **narrative** is an
explanation in its author's words at the time, scoped to a context,
including claims by name; a person judges it, and it is never given a
verdict. A **context** is a part of the system with its own model and
language: a name, its responsibilities, and its home, a repository
optionally narrowed by paths. A **relationship** names two contexts,
which is upstream, its pattern (customer and supplier, conformist and
the other fixed patterns), and the direction a dependency may run. A
**term** is a word as one context means it. Each record is known by its
name, fixed when it is proposed, in one name space across the five kinds
(record names-not-numbers), and versions: a version is the hash of what
it says, for a claim its statement, scenarios and body; what it attaches
to and its check rule sit outside the hash, and changing either is a
proposal a person accepts (record seven-events). A claim is written at
the destination's grain: it survives a rewrite of the code, names what
someone could observe, and can be judged from the repository alone; it
never names a file, a colour, a setting or a step of a pipeline, which
is a check rule's business. The one kind not versioned, the verdict, is
record cells-and-verdicts'. The third option falls because a verdict on
a narrative is a category error and a term needs its own proposal and
acceptance. Consequences: an ADR is a decision claim, numbered when it
is first accepted (record names-not-numbers); rules, decision records,
the design book and the map are views (core.2); a decision's reasoning
is never written after the fact; an imported MADR file is proposed with
a drafted scenario (record import-proposes-for-a-person). Sources:
decisions 18, 26, 27 and 28 of the design's log; the design session of
2026-10-31, its table of records; the constitution interview
(`interview.md`), its topic on identity.

**`seven-events`, A status is read from seven events, and a standing
claim changes only by a proposal a person accepts.** How does a record
move from proposed to standing and beyond, how does a standing claim
change, and where is its status kept? Options: a status field, edited;
events appended, the status read from them; a status given by where a
file sits. Outcome: seven events, appended, never changed or removed
(core.1), each naming the record and version it acts on, its actor, a
person or an agent, and its provenance (a session, a run, a transcript,
a signal); a person's event carries their signature (record
a-persons-act-is-signed). `proposed`: a new record; a new version of a
standing one; or a change that keeps a standing claim's version, its
retirement with nothing in its place, a move of what it attaches to, or
a new or changed check rule, each proposed by anyone and taking effect
only on a person's accept (`retire-attach-and-rules-need-a-person`);
visible, never checked, never judged. `accepted`: by a person (record
a-person-accepts); the version stands, and the one it amends stays in
the history. `rejected`: by a person; the proposed version is closed.
`superseded`: written when a person accepts a claim naming one it
replaces; the old claim no longer stands. `annotated`: a note on one
version; no status changes. `challenged`: names a claim and the signal
that bears on it; it stales the claim's verdicts in every repository,
and an assumption goes back to a person; written by any caller through
boom's command line (record millpond-in-both-directions). `due`:
written by boom when it reconciles a repository (record
judged-on-a-host); it names one verdict and stales that cell alone. A
standing claim changes by amendment, a new version of the same name
whose cells are judged again, or by supersession, a new name naming the
one it replaces, whose cells start unjudged as the old one's leave. An
amendment names the version it amends, and accepting it is refused
unless that version still stands; the refusal names the standing
version and records nothing; two amendments may wait side by side, a
person may reject one and accept the other, and the proposer may propose
the same words again on the new base, as a new version; boom never
merges two amendments into one text (`amend-names-its-base`). "After" is
the order each event carries itself, never its place in the ledger's
history or a file (record events-carry-their-order), and no status
depends on a clock. boom holds every event it reads to the checks it holds a
write to, its form, its actor's role and its signature; an event that
fails counts for nothing, and a change to what was already written is
reported as tampering, naming where, and never read as its new text;
the files stay readable with their format's own tools, and an owner
reverts the bad change (`validated-on-read`). The edited field falls
because edits meet in conflicts and lose who agreed; the file's place
because acceptance would be a side effect of moving a file and not a
person's act. Consequences: a status is computed every time it is
asked; two people adding events rarely meet; the history is the record
of who agreed to what; withdrawing a claim and moving its attachment
are proposals, like any change to what stands; an agent's hand-written
accept counts for nothing. Sources: decision 32; ruling Q2; the design
session of 2026-10-31, its table of events; the constitution interview
(`interview.md`), its topics on identity, on who may accept, and on the
ledger's storage.

**`the-ledger-on-a-shared-branch`, The ledger is a shared branch under
boom's prefix, written by the family's protocol.** Where does a ledger
live, how is it written so that no append is lost, and what may boom
keep besides it? The user: "As long as this is grounded in the storage
rules we established for the flywheel tools." Options: the ledger's
files in each repository's working tree, landed with the code; a branch
under boom's own prefix, `boom/`, in a repository the operator names,
written by the family's storage convention's protocol for a shared
branch; a database or a service of boom's own. Outcome: the shared
branch, layer 4 of the convention: a branch the operator names, in a
repository the operator names; a project of one repository may name a
branch of that repository; a ledger may share a state branch with
another tool's state, such as millpond's `inbox/`, each under its own
prefix; never ledger files on a working branch, never a merge commit on
the ledger's branch, never a write to a gated main
(`ledger-on-a-shared-branch`). boom writes it by the convention's 5.2:
it fetches the tip into `$XDG_CACHE_HOME/boom/git/`, applies the write
to that tip and checks it with `recfix` and its own rules (a name now
taken, an amendment's base no longer standing, an actor's role), commits
it as one commit through a temporary index, touching no working tree,
and pushes without force from the tip it read; when refused, it applies
the write again to the new tip, a bounded number of times, then reports;
a branch with no remote takes the same steps with an `update-ref`. The
history stays one line, and a clash between two writers is caught by the
check at the new tip, which refuses the second; never a forced push, a
merge, or a lock standing in for the push
(`ledger-written-by-the-family-protocol`). Every step of it that
changes a repository, the fetch into the cache, the commit through a
temporary index, the push and the `update-ref`, runs the host's own
`git` command, so the host's credential helpers and ssh configuration
apply; a library may read, gix as sluice reads with it, and changes
nothing, gitoxide's own status of its crates marking push and rebase not
done; `boom sync`'s push to a repository's `boom/sync` takes the same
way (`writes-through-the-hosts-git`). Every record at rest is
recutils and names its format and major version, `format:
boom-claim/1`; JSON travels only in flight; boom reads every earlier
version of a record as it stands (`records-name-their-format`). Where
each repository is checked out on a host, and the host's other
settings, are layer 1, under `$XDG_CONFIG_HOME/boom/`; caches under
`$XDG_CACHE_HOME/boom/`; a repository declares itself to boom in
`.config/boom.toml`, read at the branch's revision; what boom puts in a
repository it describes, the slice among it, is layer 5 and lands as
that repository's other work does; never a checkout path in a committed
file, never boom committing straight into a repository it describes
(`boom-state-by-layer`). Anything boom keeps besides the ledger is
computed from it, kept outside it, and never read as the truth:
deleting it changes no answer, and a cache built from an older head is
rebuilt before it answers (`caches-derived-only`). The working tree
falls because the ledger's events would ride a gate built for code and
land by merge, and a change could edit the claims it is checked
against; a database because the format must be open so a project trusts
that it owns its history (record 0001), and a service would run between
commands. Consequences: the ledger's branch keeps one line of history,
and the order a status reads is each event's own (record
events-carry-their-order); verdicts are appended there too, a named
departure from the convention's rule 2 (record cells-and-verdicts); the
layout of boom's files under `boom/` is a question the statement keeps.
Sources: decision 24; the family's storage convention, its five layers,
its 5.2, its rule 4 and its decisions 2, 5, 14 and 15, cited by their
words; the constitution interview (`interview.md`), its topic on the
ledger's storage and merge behaviour; the awesome-lists research,
2026-11-01, and gitoxide's `crate-status.md`
(https://github.com/GitoxideLabs/gitoxide/blob/main/crate-status.md);
record 0001.

**`events-carry-their-order`, Each event carries its own place in its
record's order, so two hosts' appends give one status in either
order.** How are a record's events ordered, so that the appends two
hosts make to one record derive the same status whichever lands first?
Each host writes the ledger by the family's protocol and applies a
refused write again to the new tip (record
the-ledger-on-a-shared-branch), so which of two hosts' appends comes
first in the history is a race; a person's event is signed when it is
made (record a-persons-act-is-signed) and cannot be made anew where it
lands; and each host appends verdicts to files of its own (record
cells-and-verdicts), so no one file holds a cell's order. Options: the
event's place in the ledger's history; a time each event carries; a
count each host keeps; each event naming the events of its record that
its writer had read, with an id derived from its content, as
radicle-cob orders the changes of a collaborative object in git by the
graph they form. Outcome: the last. An event names, when it is made,
the events of its record its writer had read and no other read event
follows, and carries an id derived from what it says, those names among
it; both are fixed when it is made, covered by a person's signature
where it is a person's act, and never change, a write applied again to a
new tip carrying the same event (`event-carries-its-order`). One event
is after another only when it names it, or names an event after it; two
events neither of which is after the other are concurrent, and where a
status needs them in one order it takes them by their ids
(`after-is-what-the-writer-read`). A claim's record, for its order, is
its events, its verdicts and the due events naming them. A status reads
that order and nothing else: whichever of two hosts' appends to one
record lands first, or however their lines are joined in a file, the
status derived is the same; never an event's place in the history or a
file, never a clock (`either-order-same-status`). The check at the tip
stays: an append that no longer holds where it lands, an accept on a
base no longer standing or a name now taken, is refused there, and the
order decides only among appends that hold. An event naming an event
the ledger does not hold before it counts for nothing (record
seven-events). The history's place falls because a race decides it and
a signed event cannot be moved; a time because hosts' clocks disagree
and no status depends on a clock (ruling Q2); a count per host because
two hosts' counts say nothing of each other. Consequences: a cell is
stale when a `challenged` event names its claim and the cell's verdict
is not after it, so a challenge concurrent with a judgment stales the
cell (record cells-and-verdicts); a cell's latest verdict is the latest
in this order; the ledger's history stays one line, and no status rests
on it; two hosts' lines of one file joined as `merge=union` joins them
would derive the same status, though boom applies its own write again
and joins nothing; where the order is written is the layout's question
(`open-ledger-layout`). Rule core.events-carry-their-order holds it.
Sources: the user, 2026-11-01 (request `req_ee5dd28f`), taking it from
the awesome-lists research of 2026-11-01; radicle-cob, in
radicle-dev/heartwood (https://github.com/radicle-dev/heartwood);
decision 32; ruling Q2; the constitution interview (`interview.md`), its
topic on the ledger's storage and merge behaviour.

**`cells-and-verdicts`, A verdict is a record beside the claims, and a
cell's status and its two queues are computed.** Where does a
repository's standing against a claim live, and how is it kept current
with nothing running? Options: a status per cell, kept by a process
that runs a state machine for each; verdicts kept with the repository
they judge, as the storage convention's rule 2 keeps a tool's records
about one repository; verdicts appended as boom records on the ledger's
branch beside the claims, a cell's status a function of the records,
the queues computed; no verdicts, the gate's checks the only reading.
Outcome: beside the claims. A **cell** is one standing claim and one
repository in its scope (record one-context-per-repository), derived and
with no record until judged; it is never a change, a revision or a piece
of work, and work that cites no claim has none. A **verdict** names the
claim, the version judged, the repository, the revision judged, a value
(satisfied, partial, not satisfied, not applicable), the evidence at
that revision, a few words, and who judged it (the session and its
model), and is appended, never edited; its id is derived from what it
records. A verdict is a judgment stored with its inputs, never a
computation; a marker naming the claim at its version is evidence the
judge is handed, never the verdict. Verdicts and due events are appended
on the ledger's branch, beside the claims, a named departure from the
convention's rule 2: a verdict names a claim's version, which is the
ledger's history, and anyone holding the ledger computes every cell's
status from it alone; never a verdict under `refs/boom/` in the judged
repository, never a status that needs every repository's refs fetched
first; a verdict names the revision it judged, and each host may append
only to files of its own, so writers never meet
(`verdicts-beside-the-claims`). The status, in order: the claim does not
stand, no cell; the repository homes nothing it attaches to, out of
scope, its verdict reading not applicable and kept; no verdict,
unjudged; the latest verdict judged an earlier version, stale; a
`challenged` event naming the claim that the verdict is not after,
stale; a `due` event naming it, stale; otherwise current, with its
value. "Latest" and "after" are the order each event carries (record
events-carry-their-order), so a challenge concurrent with a judgment
stales the cell whichever lands first. A stale cell lists every reason that
holds. The queues: **to judge**, every cell unjudged or stale; the
**backlog**, every current cell judged partial or not satisfied. Neither
is stored. Nothing else moves a verdict: not a tag, a kind, a term, a
narrative, a path inside a home, a check rule or a check's result; a
repository changing does not by itself stale one. An assumption is about
the world, not a repository, and has no cells. The process falls on the
status ruling: nothing runs between commands, and a stored status breaks
core.1; the repository's own refs because a status would need every
repository fetched before it could be read; no verdicts because the
gate asks another question (record the-gate-blocks-regressions) and
without verdicts there is no backlog. Consequences: `boom judge` takes
the queue to judge and `boom backlog` lists the backlog; a planner reads
the backlog and nothing else (records judged-on-a-host,
millpond-in-both-directions). Sources: decision 39; the status ruling;
the design session of 2026-10-31, its status function and queues; the
constitution interview (`interview.md`), its topic on the ledger's
storage; the family's storage convention, its rule 2.

**`one-context-per-repository`, A claim's scope is derived, and each
repository is one context until someone models the system.** How does a
claim reach the repositories it binds, and must a project model its
system first? Options: a claim names its repositories or paths; a claim
attaches to a context, a relationship or the whole system, its scope the
repositories homing them, each repository one context until a map is
drawn; a map before any claim. Outcome: the second. Scope is derived,
never chosen: a claim reaches a repository only through the homes of
what it attaches to; a relationship's claim has a cell in each end's
repository, a system claim one in every repository the map homes
anything in, each with its own verdict (`scope-crosses-by-homes`). Only
three things move a cell through scope: a home moves, an attachment
moves, or the claim's version moves; an attachment moves only by a
proposal a person accepts (record seven-events). A home may name paths
in its repository; `boom check` reads them to match a diff to claims
(record the-gate-blocks-regressions), and scope never does, so moving a
path stales nothing. Until someone draws the map, each repository is one
context, homed in itself, and a new claim attaches to the context of the
repository it was written for. Drawing the map changes no verdict: a
claim keeps its repository's context until someone attaches it
elsewhere, and that context lasts while any claim is attached to it.
Contexts and relationships are records, proposed and accepted like
claims. No second map of what is built is kept (Q6): the accepted
records are the target, and what is built is read from the verdicts.
The first option falls because a claim would stale whenever code moves;
the third because a person on one repository would have to model a
system before writing one claim. Consequences: whether the map needs
parts finer than a context is a question the statement keeps. Sources:
decision 28; ruling Q6; the design session of 2026-10-31, its section
on the map; the constitution interview (`interview.md`), its topic on
reconciliation across repositories.

**`judged-on-a-host`, boom reconciles and judges each repository at its
shared head, on a host, with no other service.** Where and when are
verdicts judged and kept current, and who judges? Options: inside a
planner's run, judging and planning together; boom on a host where the
repository is checked out, reconciling and judging, a planner reading
the judged backlog; in a change's own place before it lands. Outcome:
the second, at shared heads only. boom reconciles and judges one
repository at its shared head, on a host where it is checked out, with
no other service; a landing that cites a claim makes its cell due; a
repository first homed is judged once, as a baseline
(`judged-per-repository-at-its-head`). `boom reconcile` writes a `due`
event for a current verdict whose cited evidence is gone, or whose claim
a landing since cites (a `Claim:` trailer or a new marker), and nothing
else. `boom judge` hands the judge, for each cell to judge, closed
inputs: the claim at its standing version with its scenarios and body,
the narrative that includes it, the cell's previous verdict and its
evidence, whether the shared head carries a marker naming the claim and
at which version, and the repository at its shared head, read-only;
nothing else reaches it. The judge is an agent program's command, with
its exact model, named in the ledger's settings, which an owner signs
(record a-persons-act-is-signed), given the closed inputs on standard
input and answering JSON on standard output; boom holds no model's
credential (core.3). boom writes the one verdict per cell the judge
returns, checked against the verdict's form; anything else is refused
and the cell stays to judge. Never a verdict for a proposed claim, never
one from a person (a person answers an unclear check, record
the-gate-blocks-regressions, and never asserts that work is done), never
one on a revision that has not landed. Both commands ask the status
first, so either, run again with nothing changed, writes nothing, and
any trigger may run them (`repeat-writes-nothing`). boom holds no lease
or claim on a cell: every verdict and due event is checked again against
the status at the tip it is applied to, and one for a cell no longer to
judge is dropped and reported, never appended; a second model session
spent on a cell another host judged first is the accepted cost
(`no-lease-recheck-at-the-tip`). In the family, what runs after a
landing on a repository's main runs `boom reconcile` and then `boom
judge` for that repository, in the background, a failure reported
beside the landing and never making it not landed, and a host's schedule
runs both for every repository it holds, so a quiet one converges
(`reconcile-and-judge-after-each-landing`). The planner's run falls
because the ledger would be complete only while something plans, and a
planner holds no code to judge; the change's place because a verdict
written there stands for code that never lands when the change is
dropped. Consequences: a landing that cites a claim makes its cell due,
so work closes its own loop (record millpond-in-both-directions); a
person, millpond or flume may run either command too; which step runs
them after a landing is sluice's to settle, a question the statement
keeps. Sources: decision 40; rulings Q1 and Q3; the constitution
interview (`interview.md`), its topics on the check, on reconciliation
across repositories, and on sluice, flume and millpond.

**`the-gate-blocks-regressions`, boom check reads a change in three
layers, and the gate blocks regressions, never the backlog.** What does
the merge gate ask of a change, from what, and with whom reviewing?
Options: whether the repository satisfies every claim in scope after
the change; whether the change moves the repository away from a
standing claim it touches, in three layers, cheapest first; a model
reviewing every claim for every change. Outcome: the second. The check
asks whether a change moves the repository away from a standing claim
the change touches, never whether the repository satisfies one; a claim
the change leaves alone holds, so the gate blocks regressions and never
the backlog (`check-asks-direction`). **Scope**: the changed files,
mapped to the contexts homed there, keep the standing claims attached to
them and to the whole system. **Rules**: a claim's check rule passes or
fails with no judgment; the map's free rule fails a new dependency
between contexts that no relationship allows; a marker naming no
standing claim fails. **Prose**: the reviewer gets the diff and the
claims still to settle, and nothing else, and returns holds, violated or
unclear for each, with a few words. A claim an earlier layer settles
reaches no later one (`three-layers-cheapest-first`). The check reads
its claims from the slice committed in the change's own tree, so a claim
is checked once its sync lands, and a gate that caches by content
replays an unchanged check (`check-reads-its-tree`). A change that
touches the slice fails unless its new slice is exactly what the
ledger, at the commit the slice names, gives for this repository, that
commit on the ledger's branch and no older than the one the replaced
slice names; where the gate cannot read the ledger, that check is not
run, never passed (`slice-only-as-the-ledger-gives`). The reviewer is an
agent program's command, with its exact model, named in the ledger's
settings and carried into each repository by its slice, given the
closed inputs on standard input and answering JSON on standard output;
the model is an input of the check, so a new model runs it again; boom
holds no model's credential (core.3); never "latest" or an unpinned
model, never a change choosing its own reviewer
(`reviewer-is-an-agent-command`). A claim the check could not have
reviewed, its reviewer unreachable, has no outcome: `boom check` exits
the not-run code the gate's policy names, so the gate records no
outcome, and never passes the claim; an answer not of the reviewer's
form reads unclear (`unreached-is-not-run`). Violated blocks the merge:
the agent fixes the code, or proposes an amendment or a claim to replace
it, which waits on a person (record a-person-accepts). Unclear fails
until a person answers for that change, and the answer is never a
verdict; a check writes no verdict and moves none (`what-blocks`). The
answer is the person's signed statement, made with them present, written
by `boom check --answer` into the change's own tree under
`.boom/answers/`, naming the claim at its version and the digest of the
change it answers, the diff of the files in that claim's scope, answers
left out; the check reads it there, and an answer whose digest no longer
matches the change is void; it may land with the change, as the record
of who answered (`answers-signed-in-the-tree`). boom calls its result a
check and keeps "verdict" for the ledger; in a gate's reading of all a
change's checks, `boom check` is one row. A prose claim checked often is
a candidate for a rule, which changes how it is checked and never its
version. The first option falls because every merge would wait on every
claim nobody has built; the third on cost, and on certainty where a
rule can say it. Consequences: a change cannot edit the claims it is
checked against, nor choose its reviewer; a gate that allows only
localhost runs a prose claim as not run rather than passing it; what a
check rule looks like is a question the statement keeps. Sources:
rulings Q4 and Q5; the design session of 2026-10-31, its section on the
gate; the constitution interview (`interview.md`), its topic on `boom
check`; sluice's statement, 83, 84 and 114.

**`a-person-accepts`, A person whose role allows it accepts, never an
agent and never the record's author.** Who may make a proposed version
stand? Options: anyone who runs `boom accept`; any person, never an
agent; a person whose role carries the permission, never an agent,
never the version's author. Outcome: the third. A proposed version of
any record stands only by the accept of a person whose role carries the
permission; an accept by an agent, by the actor of that version's
`proposed` event, or by a person without the role is refused, says why
and records nothing; the accept that supersedes a claim is held to the
same (`accept-by-role-never-agent-never-author`). Only a person rejects,
never an agent (`reject-is-a-persons`). An agent proposes, annotates,
and proposes amendments and supersessions, and never gets a change
through on its own; a person working alone accepts what an agent
proposed, and a version they proposed themselves waits on another person
(`agents-propose`). Retiring a claim, moving its attachment and changing
its check rule take effect on the same accept (record seven-events).
Roles, and who holds each, are the ledger's settings, which its owners
change by signed events, and every accept and reject is signed by the
person with them present (record a-persons-act-is-signed). The first
option falls because an agent could rewrite what the project claims to
get its own work through; the second because an author's own accept is
one person's word, not an agreement. Consequences: `boom mcp`, which
serves an agent, offers no accept and no reject, and no act that needs a
person's signature, an accept only as a link a person opens (record
the-surfaces); the walkthroughs show an agent proposing and a person
accepting. Sources: decision 41; the
constitution interview (`interview.md`), its topic on who may propose,
accept, reject and supersede.

**`a-persons-act-is-signed`, A person's act is signed with them present,
by a key an owner registers.** What proves an accept or a reject was the
person's act, when an agent on the same host runs as the same user, and
who holds the power to say whose key is whose? Options: the event's
actor field as written; the host's user; a signature over the event by a
key the ledger registers to the person, one that signs only with the
person present; an outside authority vouching for each person. Outcome:
the signature. An event whose actor is a person carries that person's
signature over the event, by a key the ledger registers to them that
signs only with them present (a hardware key's touch, a passkey or a
biometric prompt); boom verifies it before it writes the event and again
whenever it reads it (record seven-events); an agent's event is unsigned
and names its session or run as provenance; never an accept unsigned or
signed by a key not registered to its actor, a key that signs
unattended, a plain key file or a forwarded agent with no prompt, or an
agent's event naming a person as its actor; a signature verifies with
its format's own tools and no boom (`persons-acts-signed-with-presence`).
The people who grant roles and register keys are the ledger's
**owners**; the operator keeps the family's meaning, the person who runs
a host (`owner-is-the-word`). The ledger's first commit registers its
maker's own key as owner, verified by nobody else; after it, a role, a
grant of a role or a registered key changes only by an event an owner
signs, with the owner present; boom proves that each person's act was
signed by the key registered to them with them present, and never claims
to prove who that person is in the world; never roles or keys changed by
a commit to a file, `.config/boom.toml` among them, never an agent
registering a key or granting a role; several owners, an owner revoking
a key, the events it signed before staying valid, and an outside
authority added later beside the self-registered key are not ruled out
(`self-registered-first-owner`). The repositories a ledger describes,
each one's remote and planner, and the reviewer's and the judge's
commands and models are ledger settings, changed only by an event an
owner signs; the check takes its reviewer from the slice, which only the
ledger gives; a repository's `.config/boom.toml` names its ledger and
nothing the check trusts; where a repository is checked out stays each
host's own setting (`ledger-settings-owner-signed`). The actor field and
the host's user fall because an agent on the same host as the same user
writes the same; the outside authority because a project run alone,
open source, has nobody to vouch for it (the user: "the user would be
able to define self-sign or something without having the verified
owner"). Consequences: the signature's format is a question the
statement keeps; a person's key prompt is the key's, never boom's
(record the-command-line-contract); the planner is a ledger setting
(records millpond-in-both-directions, proposals-as-signals). Sources:
decision 41; the constitution interview (`interview.md`), its topics on
who may propose, accept, reject and supersede, and on reconciliation
across repositories.

**`the-command-line-contract`, Every command exits 0 to 3, answers
versioned JSON, never prompts, and a repeat adds nothing.** What does a
caller, a person, an agent or another tool, rely on when it runs boom?
Options: exit codes and output each command chooses; one contract for
every command, its exits, its JSON and its repeats. Outcome: one
contract. Every command exits 0 when it did what was asked, or the check
found every claim holds; 1 on a finding, the check finding a claim
violated or unclear; 2 when it refuses, as for an agent's accept, a name
already taken, an amendment on a base no longer standing or an unknown
name; 3 when it could not run, as when the reviewer or judge is
unreachable, the ledger's push is refused past its bound or the ledger
is unreadable; on 2 and 3 nothing is written; 3 is the code a gate's
policy names as not run; no other code (`exit-codes`). With `--json` a
command prints exactly one JSON object on standard output, naming its
format and major version, `"format": "boom-backlog/1"`; within a major a
field is only added, never renamed, removed or given a new meaning; a
refusal or a not-run is an object with its reason, in the shape flume's
answers use; every word for a person goes to standard error; a new
major may be served beside the old for a release
(`json-versioned-additive`). Every command runs from its arguments, its
settings and the ledger, and never reads an answer from its terminal; a
write run again with the same arguments (the same name at the same
version proposed, the same signal challenging the same claim) adds
nothing and answers as the first did; a hardware key's touch or a
biometric prompt for a person's signature is the key's, not boom's, and
every write takes `--dry-run` (`no-prompts-repeats-add-nothing`). A tool
that calls boom pins boom's release where it pins its other tools, and
refuses, naming both, an answer whose format major it was not built for
(`callers-pin-boom`). The first option falls because a caller would
parse words, and an outage read as a refusal hides that nothing ran.
Consequences: rule core.callers-pin-boom; a gate reads exit 3 as not
run; an agent never waits on a prompt it cannot answer. Sources: the
constitution interview (`interview.md`), its topics on the command line
and on sluice, flume and millpond; the family's storage convention, its
rule 4; sluice's statement, 114.

**`the-surfaces`, boom mcp is boom's commands, less every person's act,
and the plugins live here.** What does an agent meet of boom, and where
does each surface live? Options: an MCP server with logic of its own and
the plugins in repositories of their own; one engine with thin
surfaces, the MCP server boom's commands less every person's act, the
plugins here. Outcome: the second. boom is one engine, a library and
the `boom` command, with thin wrappers around it, every command taking
`--json` (`one-engine-every-command-json`). `boom mcp` is an MCP server
on standard input and output that an agent's harness starts, its tools
boom's commands, each taking the command's arguments and returning its
`--json` object (`mcp-is-the-commands`); it is built on rmcp, the Rust
MCP SDK (https://github.com/modelcontextprotocol/rust-sdk), each tool a
thin call of its command, so the protocol is the SDK's and the logic
boom's (`mcp-on-rmcp`). Each tool is one of boom's commands, run by the
same code and answering the same JSON, and no tool exists without its
command; no tool performs an act that needs a person's signature.
Served: propose, supersede, annotate, challenge, query, check, backlog,
render, judge, reconcile, sync. Absent: accept, reject, `check
--answer`, and every owner's act; a tool may answer, as text, that a
person must accept and with which command (`mcp-serves-no-persons-act`).
No surface an agent calls accepts a record: `boom mcp` offers reading
and proposing, and an accept only as a link a person opens to make it
(`no-agent-surface-accepts`). An answer naming a version that waits on a
person's accept carries that link beside the words: a link of boom's
own scheme naming the ledger, the record by its name and the version,
and nothing that acts; `boom accept` takes it as it takes
`<name>@<version>`, shows the version and its scenarios, and the key's
prompt signs with the person present; opening it accepts nothing, and
boom serves no page for it (`accept-offered-as-a-link`). The Claude Code plugin is `boom mcp` and
the three moments an agent meets the ledger: drawing out claims and
narratives while it plans, at the destination's grain; looking up what
applies before it edits; running `boom check` before it hands off
(`claude-code-plugin-three-moments`). Before an agent edits a file, the
plugin's hook gives it the standing claims whose scope holds that file,
by `boom query`, with no model; drawing out claims and running the
check before hand-off are skills; the hook stays silent where no claim
applies, calls no model and writes nothing (`hook-before-each-edit`).
The herdr plugin shows, for each agent pane, the claims in scope of its
last edit and its last check's outcome, and for the workspace, the
proposals waiting on the person and the unclear checks to answer; its
one acting key opens the waiting list in the person's own pane, where
`boom accept` runs and the key's presence prompt signs it, and it opens
an accept link there the same way; it reads boom only by `boom …
--json`, never the ledger's files
(`herdr-shows-what-waits-and-applies`). Both plugins live in this
repository, each under `plugins/<name>` as a moon project of its own,
each calling boom by its command line or `boom mcp`, never its code,
naming the boom release it needs and shipping with it
(`plugins-in-this-repository`). For the accept, options: an accept tool
in `boom mcp` that refuses an agent; no accept at all, the words naming
the command; an accept only as a link a person opens. Outcome: the link,
since a tool that refuses an agent still sets off the person's key
prompt from an agent's call, and words a surface cannot open leave the
person to copy a command. The first option falls because logic in a surface is logic
its command lacks, and a plugin elsewhere drifts from the release it
calls. Consequences: rules core.plugins-in-this-repository and
core.no-agent-surface-accepts; no agent's call sets off a person's key
prompt; what opens a link on a host where no plugin does is a question
the statement keeps (`open-accept-link-opener`). Sources: the design
session of 2026-10-31, its surfaces; the constitution interview
(`interview.md`), its topic on `boom mcp` and the Claude Code and herdr
plugins; the user, 2026-11-01 (request `req_ee5dd28f`), taking the link
and rmcp from the awesome-lists research of 2026-11-01; record
a-person-accepts.

**`sync-by-one-pull-request`, boom sync hands a repository its files on
one ready pull request.** How does what boom writes for a repository,
its slice, rules and decision records, reach that repository? Options:
boom pushing to the repository's main; a branch only, for someone to
land; one commit on boom's own branch there with one pull request,
opened through the forge's own command line and marked ready; boom
running the repository's merge tool itself. Outcome: the pull request.
`boom sync` writes a repository's files only on its own branch there,
`boom/sync`, as one commit from the ledger, and opens or updates one
pull request for it through the forge's own command line, marked ready,
never a draft, so a merge path that takes pull requests lands it as it
lands any other; with output unchanged it pushes nothing and leaves the
request alone; boom never merges it itself; never a second sync request
open at once. The user: "We just need to make sure this is compatible
with how we use sluice." It is: sluice's pull-request profile finds a
request it did not open as one opened by hand, submits it once it is out
of draft and the branch's conditions hold, and lands it; a new head
withdraws the entry and the request waits for its conditions on that
head; with `reviews = 0`, a sync request lands once the gate passes. A
push to main falls because it skips the gate the repository's other
work passes; boom running sluice because boom runs no other tool (record
millpond-in-both-directions); a branch alone because nobody is asked to
land it, though it is what remains where the forge has no pull
requests. Consequences: the repository's gate checks a sync like any
change, and `slice-only-as-the-ledger-gives` lets it change the slice;
where a repository's merge path takes no pull requests, the sync branch
waits for whoever lands work there, a question the statement keeps.
Sources: the constitution interview (`interview.md`), its topic on
reconciliation across repositories; sluice's statement, the pull-request
row of its Part C, 118, 120 and 136.

**`millpond-in-both-directions`, millpond and boom work both ways, and
millpond takes in a backlog only where no other planner plans it.** How
does what millpond hears reach boom's claims, and how does boom's
backlog reach work? Options: one way, millpond proposing and
challenging and the backlog left to a person; both ways, millpond taking
in every repository's backlog; both ways, millpond taking in a
repository's backlog only where no other planner plans it, the
repository's planner setting saying which. Outcome: the third, the
user's ruling of 2026-11-01 (request `req_ee1f3b7c`). (a) A millpond
signal that bears on a claim reaches boom as a `challenged` event, `boom
challenge <name> --signal <key>`, or as a proposed claim, `boom propose`
with the signal as its provenance; the challenged claim's cells are
judged again (record cells-and-verdicts), and an assumption goes to a
person. (b) millpond takes `boom backlog --planner millpond --json` in
as items, each entry once by its id, only for the repositories whose
planner setting names it; an item's send launches the work through
flume, the launch naming the claims the work serves, each at its
version, and flume puts `Claim: <name>@<version>` on every commit the
place lands (`claims-travel-in-the-launch`); the landing makes the cell
due and `boom judge` judges it again (record judged-on-a-host). The loop
runs claim, judged not satisfied, item, work, landed, judged satisfied,
and the entry leaves the backlog. (c) Where a repository's planner
setting names another planner, that planner reads the backlog itself
and millpond takes none of it in, so nothing is planned twice. The
planner setting is a ledger setting an owner signs (record
a-persons-act-is-signed), never a line of a repository's file. boom
reads no other tool's files, every exchange with another tool is a
command and its output, and boom runs none of sluice, flume or millpond
(`tools-call-commands`); it never plans, starts or lands work, and a
planner never writes a verdict. The first option falls because a
backlog nobody takes in waits on a person to notice it; the second
because a repository another planner plans would be planned twice.
Consequences: `boom backlog --planner`; a challenge naming the same
signal twice adds nothing; the citation rests on the launch, never on
an agent copying a line from a brief; flume's launch field and
millpond's send filling it are those tools' own changes. Sources: the
ruling of 2026-11-01; decision 40 (a planner reads the judged backlog);
the design session of 2026-10-31, which brings a challenge into boom as
an event because no tool reads another's files; the constitution
interview (`interview.md`), its topic on sluice, flume and millpond.

**`proposals-as-signals`, A proposal awaiting a person shows as a
millpond signal where no other planner is connected.** Where does a
person see what waits on them? Options: only by asking boom, `boom
query --status proposed`; as millpond signals for every repository; as
millpond signals where no other planner is connected to the repository,
the planner that is connected showing them where one is. Outcome: the
third (decision 42). millpond lists the versions awaiting a person in
the repositories whose planner setting names it, each once by its
record's name and version, as a signal; where the setting names another
planner, that planner shows them and millpond carries none. The same
ledger setting decides this and record millpond-in-both-directions'
(c). Showing a proposal anywhere accepts nothing: only a person's
signed accept does (record a-person-accepts), and a version accepted or
rejected leaves the listing. The first option falls because a proposal
nobody is shown waits unseen; the second because a person would be
asked twice. Consequences: the herdr plugin shows the same waiting list
in the workspace, and a signal may carry each proposal's accept link
for the person to open (record the-surfaces); what a proposal whose scope
spans repositories naming different planners shows where is a question
the statement keeps. Sources: decision 42; the ruling of 2026-11-01.

**`import-proposes-for-a-person`, An import proposes for a person to
accept, and keeps the numbers it brings.** How do a project's existing
decision records come into the ledger, under what names and numbers,
and whose accept do they stand by? Options: boom reading the old files
where they lie, beside the old tool; an import that accepts what it
reads; an import that proposes, a person accepting. Outcome: the third.
`boom import adr <dir>` reads a folder of MADR files once: each file's
title becomes a decision's statement, its sections the body, its status
and date its history, and a "superseded by" link a supersession;
afterwards boom writes the records and the old tool writes none
(`import-once`). An imported decision's name is its file's title part,
`amounts-as-floats` from `0002-amounts-as-floats.md`, held to the one
name space: an import that would bring a name the ledger already holds
is refused, naming it, for the importer to name it otherwise
(`import-names-from-files`). The import proposes and accepts nothing:
each decision is proposed with the file's own status, date, deciders and
path as its provenance, and stands only by a person's signed accept
(record a-person-accepts); one command may accept the batch, one signed
event each; a file the old tool marked superseded comes in as history
only, named by its successor, and never stands
(`import-proposes-a-person-accepts`). Each imported decision is proposed
with at least one scenario an agent drafted from its outcome and
consequences, marked as drafted at import, which the person who accepts
it reads with the rest; a superseded file, history only, has none
(`import-drafts-scenarios`). A decision's number is given when it is
first accepted, from the ledger's one-line history, and never changes;
`boom sync` writes the decision as `NNNN-<name>.md`, MADR's naming, with
MADR's frontmatter and sections; citations name the record,
`money-has-currency`, never its number
(`decision-files-numbered-at-acceptance`). An imported decision keeps its
old number as its number, and its file its name; boom's count goes on
from the highest number the ledger holds, so with one repository
imported, 0001 to 0004 keep theirs and the next decision accepted is
0005; an import whose old numbers the ledger has already given is
refused, naming them, unless the importer has them numbered on from the
count, their old numbers kept as aliases
(`legacy-numbers-kept-count-goes-on`). The user, asked about the files'
names: "I thought an ADR was supposed to have a number because we're
creating them with the CLI tool now and they're all getting numbered."
The collision comes from a number taken on a branch, as the `adrs`
command line takes the next one; boom gives a number at acceptance on a
line never rewritten, so it need not take one there, and its files keep
MADR's numbered names. Reading the files where they lie falls because
the project would have two sources for its decisions and no record of
who accepted each; an import that accepts because it would write an
unsigned accept, or the importer's own. Consequences: an agent may run
the import for a person to accept; a person may rewrite a drafted
scenario later, as an amendment; this repository, meanwhile, names its
records and numbers them on the site (record names-not-numbers).
Sources: decision 33; the constitution interview (`interview.md`), its
topic on importing existing decision records.

## The statement

What each part holds, in substance, written to the style's rules (one
invariant each, present tense, the strong words, checkable, no
mechanism in Parts A and B). Each item is listed by its name, the
interview's where it gives one; the clause's words carry the substance
the record gives under that name, and each clause cites the record
behind it.

**Preamble and purpose.** A statement of what boom must always do and
never do; the three parts; the mechanism rule; the rule on names (The
statement's shape). boom keeps a ledger of what a project claims about
itself, judges each repository against the claims that stand, computes
every view and queue from the ledger, and reads each change against the
claims it touches; it never plans, starts or lands work.

**Actors**, a table, may decide and may not: **a person** (accepts and
rejects where their role allows, signing with them present; answers an
unclear check; names what they propose; may not accept their own
proposal or assert a verdict); **an agent** (proposes, naming what it
proposes, annotates, proposes amendments, supersessions, retirements,
attachments and check rules, runs the check; may not accept, reject,
answer a check, or act as an owner); **an owner** (the repositories a
ledger describes, each one's remote and planner, the reviewer and the
judge, the roles and who holds them, whose key is registered; may not
act unsigned or prove who anyone is in the world); **the operator** (the
person who runs a host: where each repository is checked out there, and
what runs when; may not change the ledger's settings but as an owner);
**the judge**, an agent program boom starts (a cell's value, evidence
and words from its closed inputs; may not read beyond them or judge a
proposed claim); **the reviewer**, an agent program the check starts
(holds, violated or unclear for a claim against a change; may not write
a verdict); **a planner** (what work a backlog entry becomes, and when;
may not write a verdict or take a repository's backlog its setting does
not name it for); **a caller**, any tool or person running boom's
commands (what to ask and hand in; may not write the ledger but through
boom); **boom** (what its commands compute and the events and verdicts
they append; may not accept, choose or change a record's name, decide a
verdict for the judge, plan, start, land or merge work, or read another
tool's files).

**Vocabulary**: ledger, record, name, number (a record's, given at
first acceptance), legacy number, version, event, event id, after (of
two events), concurrent, actor, provenance, signature, key (a
person's), accept link, owner, operator, role, setting, claim,
kind, invariant, decision, assumption, statement (a claim's), scenario,
body, check rule, narrative, context, home, relationship, term, map,
attachment, scope, standing, proposed, amendment, base (an amendment's),
supersession, retirement, signal, challenge, cell, verdict, value,
evidence, revision, shared head, landing, citation, marker, status (a
record's and a cell's), stale, to judge, backlog, entry, entry id,
reconcile, judge, change, check, layer, outcome, not run, answer, slice,
view, cache, import. "Terms are defined by meaning. A model may add
terms; it may not redefine these." "Key" is a person's key alone; an
entry is known by its id.

**Part A. What must hold of boom.** One `###` section per concern, each
clause citing its record:

- **Names and numbers** (record names-not-numbers):
  `identity-fixed-at-proposal`; `one-name-space`;
  `name-clash-fails-at-merge`; `cite-by-name`;
  `number-given-at-acceptance`; `ordinal-per-kind-ledger-wide`;
  `legacy-number-alias`; `other-identities-derived`, every other
  identity boom gives, a version, an event's id, a verdict's id, an
  entry's id, is derived from what it identifies and depends on nothing
  else the ledger holds.
- **Records and versions** (record five-records): `version-is-the-words`,
  a version is fixed by what a record says alone, so the same text has
  the same version and new text a new one; `version-leaves-out-attachment`,
  a claim's version covers its statement, scenarios and body and never
  what it attaches to or its check rule; `one-kind-a-scenario`, every
  claim has exactly one kind and at least one scenario;
  `reasoning-as-proposed`, a decision's reasoning is the reasoning as
  proposed, and nothing added later is reasoning;
  `narrative-includes-by-name`, a narrative includes a claim by name,
  never by copy, and is never given a verdict; `term-in-one-context`;
  `relationship-names-its-direction`, a relationship names its two
  contexts, the upstream one and the direction a dependency may run.
- **Events and standing** (records seven-events,
  events-carry-their-order): `status-from-events`, a status is read from
  events alone and stored nowhere; `events-only-appended`;
  `event-names-version-actor-provenance`; `event-carries-its-order`, in
  mechanism-free words: each event carries, from when it is made, its
  place in its record's order, the events its writer had read and an id
  derived from what it says, and keeps both wherever it is appended;
  `after-is-what-the-writer-read`, one event is after another only when
  its writer had read the other, and two neither of which had read the
  other are concurrent, taken by their ids where a status needs one
  order; `either-order-same-status`, whichever of two appends to one
  record lands first, the status derived is the same, and no status
  depends on an event's place in the store or on a clock;
  `proposed-is-never-checked`;
  `stands-only-by-accept` (accept-by-role-never-agent-never-author);
  `amend-names-its-base`; `amendment-stands-in-place`, an accepted
  amendment stands in place of the version it amends, which stays in
  the history; `supersession-ends-standing`, the accept of a claim
  naming one it replaces ends that one's standing in the same act;
  `annotation-moves-nothing`; `challenge-once-per-signal`, a challenge
  names one claim and one signal, and a second naming the same pair
  adds nothing; `due-once-per-verdict`;
  `retire-attach-and-rules-need-a-person`; `validated-on-read`, in
  mechanism-free words: a change to what was already appended is
  reported as tampering and never read as its new text.
- **Acceptance and owners** (records a-person-accepts,
  a-persons-act-is-signed, the-surfaces, proposals-as-signals):
  `accept-by-role-never-agent-never-author`; `reject-is-a-persons`;
  `agents-propose`; `no-agent-surface-accepts`, in mechanism-free words:
  no surface an agent calls makes an accept, and an accept it would
  need is offered only as a link that names the version and acts on
  nothing, which a person opens to make the accept;
  `persons-acts-signed-with-presence`, in
  mechanism-free words: a person's act carries their signature by a key
  the ledger registers to them, made only with them present, verified
  when written and whenever read, and an agent's act is never taken for
  a person's; `self-registered-first-owner`;
  `ledger-settings-owner-signed`, in mechanism-free words: a
  repository's own declaration names its ledger and nothing the check
  trusts; `showing-accepts-nothing`, showing a proposal anywhere
  accepts nothing.
- **Scope and the map** (record one-context-per-repository):
  `scope-crosses-by-homes`; `three-things-move-cells`, only a home
  moving, an attachment moving or a version moving changes a claim's
  cells; `paths-never-move-scope`; `one-context-until-a-map`;
  `a-map-moves-no-verdict`; `no-map-of-what-is-built`.
- **Cells and verdicts** (record cells-and-verdicts):
  `cell-is-claim-and-repository`, a cell is exactly one standing claim
  and one repository in its scope, never a change, a revision or a piece
  of work; `cell-unrecorded-until-judged`; `verdict-appended-whole`, a
  verdict is appended and never edited, and names the claim, the version
  judged, the repository, the revision judged, its value, its evidence
  at that revision, its words and its judge; `four-values`;
  `verdict-is-a-judgment`, never computed, the mechanical facts being
  inputs; `assumption-has-no-cells`; `verdicts-beside-the-claims`, in
  mechanism-free words: verdicts and due events are kept in the ledger
  beside the claims, so any holder of the ledger computes every cell's
  status from it alone.
- **Status and the queues** (record cells-and-verdicts):
  `status-from-the-ledger-alone`, any holder of the ledger computes the
  same status for every cell; `status-order`, a cell's status is the
  first of the record's steps that holds, in its order;
  `stale-lists-every-reason`; `queues-computed`, the queue to judge and
  the backlog are computed, never stored; `what-never-moves-a-verdict`;
  `nothing-runs-between-commands`, a status changes only by a record
  appended.
- **Reconciling and judging** (record judged-on-a-host):
  `judged-per-repository-at-its-head`; `reconcile-writes-due-only`, a
  reconcile writes a due event only for a current verdict whose evidence
  is gone at the shared head or whose claim a landing since cites;
  `judge-writes-only-to-judge`, a judge writes a verdict only for a
  cell to judge; `repeat-writes-nothing`; `no-lease-recheck-at-the-tip`,
  in mechanism-free words: every verdict and due event is checked again
  against the status where it is appended, and one for a cell no longer
  to judge is dropped and reported; `judge-inputs-closed`;
  `judge-output-checked`, an output not of a verdict's form is refused
  and the cell stays to judge; `no-verdict-unearned`, never a verdict
  for a proposed claim, from a person, or on a revision that has not
  landed; `model-is-an-input`, the judge's and the reviewer's programs
  and exact models are the ledger's settings, a new model judges and
  reviews again, and no change chooses its own.
- **The check** (record the-gate-blocks-regressions):
  `check-asks-direction`; `three-layers-cheapest-first`; `what-blocks`;
  `check-reads-its-tree`; `slice-only-as-the-ledger-gives`;
  `unreached-is-not-run`, in mechanism-free words: a claim the check
  could not have reviewed has no outcome and is never passed;
  `answers-signed-in-the-tree`, in mechanism-free words: a person's
  answer is signed with them present, kept in the change's own tree,
  names the claim at its version and the change it answers, and is
  void once that change moves; `violated-changes-by-proposal`, a
  violated claim changes only by a proposal a person accepts;
  `rule-changes-how-not-what`, a check rule changes how a claim is
  checked, never its version; `a-number-cites-nothing`, a citation by a
  number names no record, and the check fails it, naming the name to
  cite.
- **Bringing records in** (record import-proposes-for-a-person):
  `import-once`, in mechanism-free words: records brought from another
  tool are read once, and afterwards boom writes them and that tool
  none; `import-proposes-a-person-accepts`; `import-drafts-scenarios`;
  `legacy-numbers-kept-count-goes-on`.
- **Views** (records five-records, names-not-numbers,
  the-ledger-on-a-shared-branch): `views-computed`, every view is
  computed from the ledger, and the same ledger gives the same view;
  `views-never-read-back`; `rendering-arranges`, rendering arranges and
  never reasons; `book-names-its-records`, every part of the design book
  names the record it shows, so a note on it reaches the record;
  `views-show-names`, a view shows each record under its name, and any
  number it shows is the one given at acceptance;
  `caches-derived-only`.
- **What boom never does** (records judged-on-a-host,
  millpond-in-both-directions, sync-by-one-pull-request):
  `never-plans-starts-or-lands`, boom never plans, starts, lands or
  merges work, and never changes work that cites a claim;
  `gives-the-standing-version`, it gives any caller the standing
  version of each claim, so work citing an earlier one can be told;
  `tools-call-commands`, in mechanism-free words: boom reads no other
  tool's state, every exchange with another tool is a command and its
  output, and boom starts no other tool;
  `writes-a-repository-only-by-proposing`, boom writes into a
  repository it describes only by proposing a change there, which lands
  as that repository's other work does.

**Part B. Contracts across the seams.** One `###` section per seam, its
operations in abstract names and its guarantees as named clauses, the
writer naming each:

- **The ledger's store** (record the-ledger-on-a-shared-branch),
  carrying the shared-branch protocol's guarantee: append; read at a
  point of its history; its tip. Appends are never lost or changed and
  its history is one line; a write is checked against the tip it lands
  on and refused there when it no longer holds, a name now taken, a base
  no longer standing, a role not held; a write applied again to a new
  tip appends the same event, its order and signature unchanged
  (event-carries-its-order); two writers both land, or the second is
  refused, and neither overwrites the other; nothing is forced over what
  another wrote; every record names its form and major version, and
  every earlier one is read as it stands; the order a status reads is
  the events' own, never the store's (either-order-same-status).
- **The repository**: its shared head; what a revision holds; the
  landings since a revision, with what each cites; a change against a
  base; propose a change, which lands only by the repository's own
  path. What is read at a revision is that revision's.
- **The judge** (record judged-on-a-host): judge, from closed inputs, to
  one verdict or none; it reads nothing else; its program and exact
  model are the ledger's settings.
- **The reviewer** (record the-gate-blocks-regressions): review, a
  change and the claims to settle, to an outcome and words per claim,
  or unreached; an answer not of that form reads unclear, and unreached
  is no outcome (unreached-is-not-run).
- **The actor** (record a-persons-act-is-signed), bound to
  persons-acts-signed-with-presence: who acted, a person or an agent,
  and the roles they hold; prove a person's act, a signature by the key
  registered to them, made with them present; register and revoke a
  key, by an owner's signed act; an agent is never taken for a person,
  and a revoked key leaves the events it signed before valid.
- **The planner** (records millpond-in-both-directions,
  proposals-as-signals): backlog, for a planner, every entry of each
  repository whose planner setting names it, each with its id, the claim
  by name at its standing version, its scenarios, the verdict's value,
  evidence and words and why the cell came back; proposals, for a
  planner, every version awaiting a person in those repositories. An
  entry's id is the same on every host and run and names one cell and
  one verdict, and no repository's entries are given to two planners.
- **Signals** (record millpond-in-both-directions): challenge, a claim
  by name and a signal; propose, a record, its name and its provenance;
  standing, whether a named claim stands. The same signal challenging
  the same claim twice adds nothing.
- **The gate** (record the-gate-blocks-regressions), taking
  slice-only-as-the-ledger-gives and unreached-is-not-run: check, a
  candidate's tree, to pass, fail or not run; everything it reads is in
  that tree, or the ledger at a commit the tree names; not run is never
  pass.
- **The caller** (record the-command-line-contract): every answer boom
  gives is one of done, a finding, refused, or could not run; refused
  and could not run write nothing; an answer names its form and major
  version, within which a field is only added; nothing waits on an
  answer from a terminal; a write repeated with the same arguments adds
  nothing and answers as the first did; an answer naming a version that
  waits on a person's accept says how a person makes it and makes
  nothing itself (no-agent-surface-accepts).
- **Binding the contracts**: each seam is bound by naming profiles of
  Part C; a ledger binds one store and one actor profile; any number of
  planners and signal sources; one conformance suite holds every
  profile.

**Part C. Profiles.** Opening "Parts A and B do not change to admit a
profile." One `###` section per profile, mapping its seam's operations
to how it satisfies them, citing Part B's clauses by name, the
interview's items as its clauses:

- **The store on a shared branch** (record
  the-ledger-on-a-shared-branch): `ledger-on-a-shared-branch`;
  `ledger-written-by-the-family-protocol`; `records-name-their-format`;
  `boom-state-by-layer`; verdicts and due events appended on the
  ledger's branch beside the claims, the storage convention's rule 2
  departed from by name (verdicts-beside-the-claims); tampering read as
  a commit that changes or removes a line already written, naming the
  commit; the history the branch's own, one line, with no merge; an
  event's order written with it, its id and the ids of its record's
  events it follows, covered by a person's signature
  (event-carries-its-order); `writes-through-the-hosts-git`, every write
  boom makes to a repository, the ledger's branch or a sync branch, the
  fetch, the commit through a temporary index, the push and the
  `update-ref` among it, run by the host's own `git` command so its
  credential helpers and ssh configuration apply, a library, gix by
  default, only reading; the layout of boom's files under `boom/` a
  question (open-ledger-layout).
- **The repository in git**: the shared head is the default branch at
  its remote; a landing cites a claim by a `Claim: <name>@<version>`
  trailer; a marker is the same words in an OpenSpec requirement, a
  test, a doc comment or a spec file; a repository's `.config/boom.toml`
  names its ledger and nothing else boom trusts;
  `sync-by-one-pull-request`.
- **The judge and the reviewer**: `reviewer-is-an-agent-command`, for
  both.
- **The actor**: a key registered to each person that signs with them
  present, a hardware key's touch, a passkey or a biometric prompt, its
  format open (open-signature-format) and verifiable with that format's
  own tools; an agent's event unsigned, naming its session or run.
- **The gate in sluice**: `sluice-gates-with-check`; not run as exit 3,
  the code the branch's policy names for a check that could not run
  (sluice's statement, 114), so sluice records no outcome; an answer
  written by `boom check --answer` under `.boom/answers/`; the slice,
  `.boom/slice.rec`, naming the ledger commit it was written from, which
  the check reads from the ledger's branch, not run where the gate's
  network cannot reach it or the model, as under a policy that allows
  only localhost.
- **The planner in millpond**: `millpond-both-ways`;
  `reconcile-and-judge-after-each-landing`, the step that runs them a
  question (open-after-landing-step); `claims-travel-in-the-launch`;
  proposals awaiting a person as millpond signals where millpond is the
  planner; another planner, named by its own name, reading the same
  commands for its repositories.
- **Signals from millpond and flume**: `flume-asks-what-applies`;
  millpond's challenge move running `boom challenge <name> --signal
  <key>`; a signal asserting something new running `boom propose` with
  a name and the signal's key as provenance; "does this claim stand"
  read with `boom query <name> --json`.
- **The surfaces** (records the-command-line-contract, the-surfaces):
  `one-engine-every-command-json`; `exit-codes`;
  `json-versioned-additive`; `no-prompts-repeats-add-nothing`;
  `mcp-is-the-commands`; `mcp-on-rmcp`, `boom mcp` served by rmcp, the
  Rust MCP SDK, on standard input and output, each tool a thin call of
  its command (mcp-is-the-commands); `mcp-serves-no-persons-act`, which
  holds decision 41's consequence that `boom mcp` offers no accept and
  no reject (the interview's `mcp-offers-no-accept`, stated once here);
  `accept-offered-as-a-link`, an answer naming a version that waits on
  a person's accept carrying `accept_link`,
  `boom:accept?ledger=<ledger>&record=<name>@<version>`, which `boom
  accept` takes as it takes `<name>@<version>`, showing the version
  before the key's prompt signs, and which the herdr plugin opens in the
  person's own pane, what opens it elsewhere a question
  (open-accept-link-opener); `claude-code-plugin-three-moments`;
  `hook-before-each-edit`; `herdr-shows-what-waits-and-applies`.
- **The renderer and the import**: mdBook, run by command where
  installed, none of its code embedded; `decision-files-numbered-at-acceptance`;
  `import-names-from-files`; `boom import adr` reading a folder of MADR
  files.

**Global invariants**, each a named item citing its clauses:
`status-never-stored`, a status, a record's or a cell's, is read from
the ledger and never stored; `every-view-is-computed`, every view,
queue and cache is computed; `only-a-person-makes-a-change-stand`, no
agent accepts, no surface an agent calls accepts, no author accepts
their own, and every person's act is signed with them present;
`only-regressions-block`, the gate blocks only
a change that moves away from a standing claim, and a claim it could not
review is not run, never passed; `unchanged-inputs-write-nothing`,
reconciling, judging and every repeated write, on unchanged inputs,
write nothing; `every-holder-computes-the-same`, any holder of the
ledger, whichever order two hosts' appends landed in, computes the same
status for every record and cell; `tools-exchange-only-commands`; `no-backlog-planned-twice`;
`no-landing-renumbers-or-renames`, no landing renames a record or
changes a number already given; `nothing-trusted-is-hand-edited`, no
setting, slice or event the check trusts is changed by editing a file.

**Non-goals**: planning, ordering, starting or landing work; approving
work; merging anything, its own sync among it; running checks other than
its own, or deciding which a repository requires; running another tool;
hosting anything or serving a network; holding a model's credential;
proving who a person is in the world; rendering a book itself; keeping a
map of what is built; judging a claim from anything but a repository at
its shared head; choosing a record's name for its proposer.

**Environment givens.** Constraints: repositories are git repositories;
boom is free and open source under MIT, written in Rust, its binary
`boom`; each tool of the family keeps its own state in git and reaches
another only by its command line; the family's storage convention, its
layers, its protocol for writing a shared branch, recutils at rest each
naming its format and JSON only in flight, and a repository's
declaration in `.config/<tool>.toml`; a gate's policy names the code by
which a check says it could not run (sluice's statement, 114). The rest
of the stack, which a model builds on unless it says why not: moon and
sluice for the gate; a forge's pull requests, through its own command
line; mdBook for the book; millpond and flume as the family's inbox and
launcher; Claude Code and herdr for agents; rmcp for `boom mcp`
(mcp-on-rmcp); the host's own `git` for every write to a repository,
and a library, gix as in sluice, for reads
(writes-through-the-hosts-git).

**Questions the model must answer**, marked as not requirements, each a
named item: `open-ledger-layout`, the layout of boom's files under
`boom/`; `open-signature-format`, the format of a person's signature,
the family's attestation format (an in-toto statement in a DSSE
envelope), git's SSH signatures or another, so that a reader can verify
it with no boom; `open-sync-without-pull-requests`, where a
repository's merge path takes no pull requests, whether the sync branch
waiting for whoever lands work there is enough, or such a repository
needs a planner or a hook to submit it; `open-after-landing-step`,
whether sluice offers a step that runs a repository's command after a
landing, or a host's watcher runs boom on each landing it sees, which
is sluice's to settle; `open-finer-map`, whether the map needs parts
finer than a context, elements inside one and the links between them;
`open-check-rule-form`, what a check rule looks like;
`open-proposal-across-planners`, what a proposal whose scope spans
repositories naming different planners shows where;
`open-accept-link-opener`, what opens an accept link on a host where no
plugin does, a handler for boom's scheme registered with the host that
starts the person's terminal on `boom accept`, or the person handing the
link to `boom accept` themselves; `open-the-name`, the name.

**Scenarios the model must satisfy**, each a named item, as the
operator lives them, citing the clauses it exercises:
`alone-with-no-map`, a person alone on one repository, with no map,
makes a ledger on a branch of it, her key its first owner, has an agent
propose an invariant, accepts it, and after `boom judge` sees it in the
backlog; `an-agents-accept-is-refused`; `an-authors-accept-is-refused`;
`a-hand-written-accept-counts-for-nothing`, an agent appends an accept
to the files itself, and the status is unchanged and the commit
reported; `the-backlog-never-blocks`, a change that leaves a claim
nobody has built alone passes the gate; `a-break-waits-on-a-person`, a
change that breaks a standing claim is blocked, the agent proposes an
amendment, and the gate holds the old version until a person accepts;
`an-amendment-on-an-old-base`, two amendments of one claim wait, one is
accepted, and the other's accept is refused naming the standing
version; `unclear-waits-on-an-answer`, an unclear check fails until a
person answers it, and the answer is void once the change moves;
`the-reviewer-unreached`, a gate that allows only localhost reports a
prose claim not run and records no outcome; `a-change-swaps-its-reviewer`
and `a-slice-edited-by-hand`, each failing the check; `forty-quiet-commits`,
forty commits that leave a verdict's evidence alone judge nothing again;
`evidence-removed-makes-due`; `two-hosts-judge-one-cell`, the second
verdict is dropped and reported; `two-hosts-append-either-order`, from
one tip, one host appends Ada's signed annotation of a claim and
another a millpond signal's challenge of it, and whichever lands first,
the claim's status and every cell's are the same and Ada's signature
still verifies after her write was applied again to the new tip;
`a-challenge-beside-a-judgment`, a challenge and a verdict made from
one tip leave the cell stale, whichever lands first;
`amend-and-supersede-cells`, an
amendment makes every cell of the claim stale, a supersession retires
the old cells and starts the new claim's unjudged;
`a-map-changes-no-verdict`; `a-signal-challenges`, a millpond signal
challenges a claim and its cells are judged again, and a challenged
assumption goes to a person; `the-loop-closes`, the millpond loop from
not satisfied to satisfied, the work's commits citing the claim from
the launch; `another-planner-plans`, a repository naming another
planner has none of its backlog taken in by millpond;
`proposals-wait-as-signals`; `twice-with-nothing-changed`, reconcile,
judge and a repeated proposal write nothing the second time;
`sync-lands-through-sluice`, a sync's ready pull request lands under
sluice's pull-request profile once the gate passes;
`two-branches-land-in-turn`, two branches each propose a decision and
land one after the other, neither is renamed, the one accepted first is
given the earlier number, and every number given before either landed
is unchanged; `one-name-from-two-branches`, two branches each propose a
record named `money-has-currency`, the first lands, the second is
refused at the merge naming both, its proposer names theirs
`amounts-carry-currency` on the branch and it lands, and nothing landed
was renamed; `an-import-keeps-its-numbers`, a project imports ADRs 0001
to 0004 for Ada to accept, each with a drafted scenario, and the next
decision accepted is 0005, written `0005-money-has-currency.md`;
`the-hook-before-an-edit`, an agent about to edit a file is given the
standing claims whose scope holds it; `an-accept-reaches-a-person-as-a-link`,
an agent proposes through `boom mcp`, its answer carries the accept
link and no tool accepts, Ada opens the link in her own pane, reads the
version and her key's touch signs the accept, and the agent's call set
off no prompt.

**What to deliver**: a model of boom satisfying Parts A and B: its
records and events, the status function, the queues, the commands and
their output; a conformance suite whose scenarios cite the items they
exercise by name; the profiles of Part C, each passing it.

## The site

Each page, what changes on it and what it cites (How a page cites).
Every claim on a page about the design links the clause or record behind
it, by the names the statement and the records hold. A page that
sketches what the design leaves open says so and links the question.

- **index.mdx**: "Where it stands" says the repository holds its rules,
  its statement and its decision records, linked, and a binary that
  prints its version; the cards' "Status read from events" and "Checked
  at the merge" cite their records; "Who it is for" says agents propose
  and a person whose role allows it accepts, signing with them present.
- **how-its-used.mdx**: "Where the ledger lives" becomes a branch under
  `boom/` in a repository the operator names, a branch of the project's
  own repository where there is one, citing record
  the-ledger-on-a-shared-branch and `ledger-on-a-shared-branch`; the
  records table cites record five-records; a new section, "Names, not
  numbers", says a record is known by a name its proposer chooses, cited
  by it, and that its number is given at first acceptance, citing record
  names-not-numbers and its clauses; the events table gains `challenged`
  and `due`, each row citing its clause, the proposed row names
  retirements, attachments and check rules
  (`retire-attach-and-rules-need-a-person`), and the closing sentence on
  later events goes; the paragraph on two people adding events on two
  branches becomes two hosts appending to one record, each event
  carrying its own order, so the status is the same whichever lands
  first, citing record events-carry-their-order and
  `either-order-same-status`; a new paragraph says who may accept and that a
  person's act is signed with them present (records a-person-accepts,
  a-persons-act-is-signed); amend and supersede cite record seven-events
  and `amend-names-its-base`; "Verdicts" becomes cells, verdicts and the
  two queues, citing record cells-and-verdicts and its clauses, and
  where they are judged, citing record judged-on-a-host; "Contexts"
  cites record one-context-per-repository; the views table's decision
  records are MADR files `NNNN-<name>.md`, numbered at acceptance
  (`decision-files-numbered-at-acceptance`), and caches are derived only
  (`caches-derived-only`).
- **configuration.md**: "Two repositories" becomes where the ledger
  lives: a branch under `boom/`, `boom/ledger` of `shop` in the sketch,
  each record naming its format, `format: boom-claim/1`
  (`records-name-their-format`), the layout a sketch linked to
  `open-ledger-layout`; the ledger's settings, the repositories it
  describes, each one's remote and planner, the reviewer's and the
  judge's commands and models, shown as events an owner signs, set with
  `boom owner set`, never as a `[repositories]` table
  (`ledger-settings-owner-signed`); roles, grants and keys as owner's
  signed events, the first owner's key registered by the ledger's first
  commit (`self-registered-first-owner`); each host's settings, where
  `shop` is checked out, under `$XDG_CONFIG_HOME/boom/`, and caches
  under `$XDG_CACHE_HOME/boom/` (`boom-state-by-layer`); `shop`'s
  `.config/boom.toml` naming only its ledger, its repository and
  branch; `events.rec`'s `%type` gaining `challenged` and `due`, with
  one example of each, and a person's event its signature; each event
  its `Id:` and an `After:` for each event of its record it follows, the
  intro saying a status reads that order and no event's place in the
  file, and that `At:` is a note no status reads
  (`event-carries-its-order`, record events-carry-their-order); the verdict
  example gaining its id and its judge; what sync writes,
  `docs/adr/0001-amounts-in-cents.md` among it, on `boom/sync`; the
  caution keeping the format a sketch.
- **commands.md**: the opening says every command exits 0, 1, 2 or 3,
  with a table of what each means and that 2 and 3 write nothing
  (`exit-codes`); that `--json` prints one object naming its format and
  major version, a refusal or a not-run an object with its reason, and
  every word for a person on standard error (`json-versioned-additive`);
  that no command prompts and a repeated write adds nothing
  (`no-prompts-repeats-add-nothing`). `boom judge`, `boom reconcile`,
  `boom backlog` (with `--planner` and each entry's id), `boom
  challenge` and `boom annotate` join the table with their own
  sections, arguments, an example and its JSON, each citing its record
  and clauses; `boom query` gains `--planner`; `boom propose` shows the
  refusal of a taken name, exit 2, and its JSON carrying `accept_link`
  (`accept-offered-as-a-link`); `boom accept` says who may accept,
  takes `<name>@<version>` or an accept link, shows the version and the
  key's prompt signing it, and shows the refusal of an author's accept
  as JSON, exit 2, citing records a-person-accepts and
  a-persons-act-is-signed; `boom check` shows `--answer` writing a
  signed answer under `.boom/answers/` and a not-run, exit 3; `boom
  render adrs` and `boom sync` write `NNNN-<name>.md`, numbered at
  acceptance, and `boom sync` opens or updates one ready pull request
  (record sync-by-one-pull-request); `boom import adr` proposes each
  decision with a drafted scenario, keeping its number, for a person to
  accept (record import-proposes-for-a-person); `boom mcp` lists its
  tools as `mcp-serves-no-persons-act` has them, served and absent, an
  accept offered only as a link (core.no-agent-surface-accepts);
  "Also in the design" goes.
- **check.mdx**: each section cites record the-gate-blocks-regressions
  and its clauses: the three layers, what it reads, regressions not the
  backlog, the outcomes table, which gains not run, exit 3, for a
  reviewer it could not reach (`unreached-is-not-run`), the answer
  signed and kept in the change's tree (`answers-signed-in-the-tree`),
  the slice only as the ledger gives it
  (`slice-only-as-the-ledger-gives`), the reviewer an agent program's
  command from the ledger's settings (`reviewer-is-an-agent-command`), a
  check is not a verdict, and the two meanings of verdict; the rules
  layer names a citation by number among what fails
  (`a-number-cites-nothing`).
- **integrations.mdx**: `boom mcp` is built on rmcp (`mcp-on-rmcp`) and
  lists its tools, served and absent (`mcp-serves-no-persons-act`,
  record the-surfaces); the paragraph leaving whether `boom mcp` offers
  an accept open becomes the rule: no surface an agent calls accepts,
  and an answer waiting on a person carries an accept link the person
  opens (core.no-agent-surface-accepts, `accept-offered-as-a-link`),
  what opens it where no plugin does linked to `open-accept-link-opener`;
  the Claude Code plugin is its skills and the hook before each edit
  (`hook-before-each-edit`); the herdr plugin shows what waits on the
  person and what applies, its key opening the waiting list, and any
  accept link, in the person's own pane
  (`herdr-shows-what-waits-and-applies`); sluice gates
  with `boom check` (`sluice-gates-with-check`), reads exit 3 as not
  run, lands a sync's pull request under its pull-request profile, and
  runs `boom reconcile` and `boom judge` after each landing
  (`reconcile-and-judge-after-each-landing`, the step linked to
  `open-after-landing-step`); flume asks what applies before work
  (`flume-asks-what-applies`) and carries the claims in the launch
  (`claims-travel-in-the-launch`); millpond's section becomes both ways:
  (a) a signal to `boom challenge` or `boom propose`, (b) the backlog
  taken in as items, the loop drawn as the `millpond` diagram with a
  caption naming each station, (c) another planner reading the backlog
  itself, the planner setting deciding, and proposals as signals where
  millpond is the planner, citing records millpond-in-both-directions
  and proposals-as-signals and `millpond-both-ways`; each tool's section
  says it calls boom by its command line (`tools-call-commands`).
- **roadmap.md**: "Settled" lists each record, linked to its page, with
  the clauses that state it, and keeps core.1 and core.2; "Leaning" and
  "Proposed" keep only what no record decided; "Open questions" become
  the statement's nine questions, each linked by its anchor, with the
  build order kept as the roadmap's own.
- **guides/first-claims.md**: the ledger made as `boom/ledger`, a branch
  of `shop`, Ada's key registered as its first owner, the host's
  checkout under `$XDG_CONFIG_HOME/boom/`, `shop`'s `.config/boom.toml`
  naming its ledger; Ada's agent proposes, Ada accepts, her key's touch
  signing it; the refusal of her own accept shown once; `boom render
  adrs` writes `0001-amounts-in-cents.md`, the ledger's first decision,
  and `boom sync` opens one ready pull request; cites records
  one-context-per-repository, a-person-accepts,
  a-persons-act-is-signed, the-ledger-on-a-shared-branch and
  names-not-numbers.
- **guides/an-agent-proposes.md**: the agent's session shows each
  proposal's answer carrying its accept link, and Ada opens the one she
  accepts in her own pane, where her key's touch signs it
  (`accept-offered-as-a-link`); the last paragraph's open question
  becomes the rule, citing record a-person-accepts and
  core.no-agent-surface-accepts.
- **guides/a-change-breaks-a-claim.md**: cites record
  the-gate-blocks-regressions for the block and the replacement that
  waits, record a-person-accepts for who accepts it, and
  `amend-names-its-base` for the amendment.
- **guides/importing-adrs.md**: the agent runs `boom import adr`, which
  proposes each decision, keeping its number and naming it by its
  file's title part, each with a drafted scenario; Ada reads them and
  accepts the batch, one signed event each, the superseded file coming
  in as history; the table's number row says the number is kept as the
  decision's number; the scenario question becomes the rule
  (`import-drafts-scenarios`); after the import the agent proposes the
  new decision and Ada accepts it, and `boom sync` writes
  `docs/adr/0005-money-has-currency.md`, numbered at its acceptance,
  counting on from 0004 (`legacy-numbers-kept-count-goes-on`); cites
  record import-proposes-for-a-person.
- **README.md**: the table gains `design/requirements.md`, the
  statement, and says the records in `docs/adr/` are each named by its
  name; the "What it does" line on acceptance says an agent proposes and
  a person whose role allows it accepts, signing with them present.
- **styles/config/vocabularies/Boom/accept.txt** gains the words the
  pages now use that Vale does not know, each one the pages use.

`hero.ts` and `docs/diagrams/boom.svg` stay as they are:
`0001-money-has-currency.md` is the hero ledger's first decision,
numbered at its acceptance.

## Outside this change

Each is another tool's change, which this design names and does not
make:

- flume: a launch field naming the claims a place's work serves, each
  at its version, and `Claim: <name>@<version>` on every commit the
  place lands (`claims-travel-in-the-launch`). Until it lands, the loop
  of record millpond-in-both-directions closes only where an agent cites
  the claim itself.
- millpond: its send filling that field from a backlog entry, its
  adapter for `boom backlog --planner millpond --json`, and its
  challenge move calling `boom challenge`.
- sluice: a step that runs a repository's command after each landing,
  or a host's watcher doing it (`open-after-landing-step`). Until then,
  a host's schedule runs `boom reconcile` and `boom judge`.

## Risks / Trade-offs

- [A name chosen on one branch is chosen on another] → `root:adrs` and
  `root:statement` fail at the merge's candidate, naming both, and the
  branch landing second renames its own record, rule or item before it
  lands; nothing landed is renamed. Two records of one name are two
  files of one path, which git refuses to merge unless they are the
  same.
- [A reader cites the number the site shows] → each page says how to
  cite what it shows, by name; `root:adrs` fails a citation by a number
  no legacy record holds, and `root:statement` one by a clause number;
  the number shown never moves, so a reader who wrote it down finds the
  same thing later.
- [The numbers shown need git's history] → `design-pages` fails on a
  shallow or missing history rather than guess, and the pages workflow
  fetches the whole history. A local build that replays `site:build`
  after main moved may show an older number until the design pages
  change; nothing cites a number, and the published build runs fresh.
- [The citation checks read an ordinary phrase as a citation] → a name
  here is two or more hyphened words; a citation names its kind or sits
  in a citation list; code spans are quoted, not read; a page writes
  boom's example claims in code spans; a hyphened word alone in a
  parenthesis of the statement is rephrased.
- [A word of commercial strategy or a private path reaches the
  repository] → the records and the statement are written from this
  design alone, and review reads every new file against
  `core.no-commercial-strategy` and `core.private-sources-by-words`. No
  word list is added: one in the tree would name what it forbids.
- [A clause states more than a record decided] → each clause cites its
  record; what no record decides goes to the questions, never a clause.
- [Record a-person-accepts leaves a person working alone unable to
  accept what they write themselves] → the record says so among its
  consequences, and the walkthroughs show the agent proposing.
- [An accept link is read as an accept, or an agent opens one itself]
  → opening a link accepts nothing: it names a version and carries no
  signature, `boom accept` shows the version before the key's prompt,
  and the prompt signs only with the person present, so an agent that
  runs `boom accept` on a link meets the refusal any agent's accept
  meets.
- [A writer names fewer events than it read, to place its event before
  one it saw] → boom makes each event and names what it read; an event
  written by hand is held to the same checks on read, and two events
  neither of which names the other are concurrent, so leaving one out
  makes a judgment concurrent with a challenge, which stales the cell,
  never makes it current.
- [Generated pages under `src/content/docs` could be read by a check as
  written ones] → git ignores them, `site:prose` leaves them out by
  glob, and `root:adrs` reads tracked files only; `design-pages` empties
  its directory before it writes.

## Open Questions

- The interview leaves it to the user whether
  `core.no-commercial-strategy`, `core.private-sources-by-words`,
  `core.callers-pin-boom` and `core.plugins-in-this-repository` go to
  `core.md` with their records or in the separate step that draws
  `core.md` from the records. This design sends them with their
  records, as `CLAUDE.md` asks of a rule and its decision, beside the
  two the user sent as rules, `core.events-carry-their-order` and
  `core.no-agent-surface-accepts`; sending the four to the separate step
  moves four `core.md` edits out of the records task and changes
  nothing else.

The design's own open questions are the statement's questions.
