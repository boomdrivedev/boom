# Design

## Context

See proposal.md for why. The repository holds one decision record,
`docs/adr/0001-boom-is-an-open-source-repository-of-its-own-and-its-binary-is-boom.md`,
in MADR's bare variant: frontmatter giving `number`, `title`, `status`,
`date` and `decision-makers`; an H1 repeating the title; `Status:`,
`Deciders:` and `Sources:` lines; then `## Context and problem
statement`, `## Considered options`, `## Decision outcome` and
`### Consequences`. `check-adrs.sh` (`root:adrs`) holds every record to
a number of its own and, where two share one, tells the later to take
the next free number. `adrs.toml` configures a tool that writes a new
record numbered after the last. `docs/architecture/core.md` holds six
rules, each opening with an id made of its area and the next free number
in it, and `CLAUDE.md` tells an agent to give a new rule that id.

The site (`site/`, Astro with Starlight) has nine pages and four
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

The design this change writes down comes from the design session of
2026-10-31 on claims, verdicts and reconciliation, its rulings (on how a
cell keeps its status, and Q1 to Q6 below), decisions 39 to 42 of the
design's log, accepted on 2026-11-01, the user's ruling of 2026-11-01 on
millpond (request `req_ee1f3b7c`), and the user's direction of
2026-11-01 on how a record is known. The first sources are kept outside
this repository and carry more than the open design. This design
carries only the open part, so the code stage writes from this file
alone (rule `nothing-commercial`).

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

And the decisions and direction of 2026-11-01, in open terms:

| source | what it decides |
|---|---|
| decision 39 | verdicts are boom records, kept beside the claims |
| decision 40 | boom reconciles per repository: for one repository at a commit it reports which claims in scope have no current verdict, an agent on a host judges them with `boom judge`, it works with no other service, and a planner reads the judged backlog |
| decision 41 | a person whose role carries the permission accepts a record; never an agent, never the record's author |
| decision 42 | a proposal awaiting a person shows as a millpond signal where no other planner is connected |
| the ruling on millpond | both directions, and millpond takes a repository's backlog in only where no other planner plans it |
| the direction on names | a record is known by a name fixed when it is proposed, held unique by a check at the merge; every citation names it; a number a reader sees is derived on render from the order of acceptance on the default branch; an imported ADR keeps its number as a legacy alias; boom's own statement and rules follow the same, or say where they cannot yet and why |

The rules this design relies on, by the names this change gives them:
`status-read-from-events` and `views-are-computed`, which the records
restate for boom itself; `nothing-commercial`, which bounds every word
written here; `every-part-is-checked`, for the new shell scripts
(shellcheck and shfmt own `*.sh`), the fixtures (`*.md`, excluded as
prose) and the generator and its test (Biome owns `site/*.ts`); and
`tools-are-pinned`, since every tool the new tasks run (sh, awk, git,
bun, Vale) is one the tree already pins or the host's own shell. The
rule this change adds, `identity-is-a-name`, lands in the commit of the
record behind it, as `CLAUDE.md` asks. The rules the other records imply
are drawn into `core.md` in a separate step, as the user decided on
2026-11-01.

## Goals / Non-Goals

**Goals:**

- Every decision of boom's design that the user has made is a record a
  reader can open here, and every invariant it implies is a clause a
  later change can cite.
- Nothing here is known by a number allocated on a branch: a record, a
  rule and an item of the statement each have a name fixed when it is
  written, every citation names it, and two branches landing side by
  side never renumber or rename anything that landed.
- A site page states nothing about the design that it cannot link to a
  clause or a record, and the build proves each link.
- Nothing commercial reaches the repository: no pricing, tier, customer
  or hosted service (rule `nothing-commercial`). Where the sources go
  past the open design, this design says what boom offers any caller,
  its commands and their JSON, and stops.

**Non-Goals:**

- Drawing `core.md` from these records, beyond the one rule that holds
  names and the names of the six that stand.
- A record for anything the user has not decided: signing, sync's
  rolling pull request, rendering and import stay proposed on the
  roadmap, and the statement describes them only where a decided record
  leans on them.
- Vale over the records and the statement: they are read by people and
  reviewed; the gate's prose check stays on the site's written pages.
- Any change in millpond, flume, sluice or the family's requirements
  style.

## Decisions

### Names for what this repository holds

Rule `identity-is-a-name` binds this repository as it binds boom.

- **Records.** Each record is `docs/adr/<name>.md`. A name here is two
  or more lowercase words of letters and digits joined by hyphens, so a
  citation of it in prose is never read as an ordinary word (`record
  a-person-accepts`, never `record accepts`). The frontmatter holds
  `title`, `status`, `date`, `decision-makers` and, for a record that had
  a number before the rule, `alias`, those four digits in quotes; never
  `number`. The first record moves to
  `docs/adr/an-open-repository-of-its-own.md` with `alias: "0001"`, its
  words unchanged but for its citations, which name the rules. A link to
  its old path at main stops resolving; the roadmap, the one page that
  links it, links its page instead.
- **Rules.** Each rule in `core.md` opens `- **[<name>]**`, the name as
  a record's and held by no other rule on any page of
  `docs/architecture/`; its details entry opens `- **<name>**`, and the
  six that stand end theirs with `Legacy id: ` and their old id in a
  code span. A rule that moves to an area page keeps its name.

  | name | legacy id | holds |
  |---|---|---|
  | `status-read-from-events` | `core.1` | a status read from events; an event appended, never changed |
  | `views-are-computed` | `core.2` | every view computed from the ledger, never stored as the truth |
  | `no-credentials` | `core.3` | no token, key or credential in the repository |
  | `nothing-commercial` | `core.4` | nothing commercial in the repository |
  | `every-part-is-checked` | `core.5` | every file owned by a formatter; every crate and the site a moon project, one task per check |
  | `tools-are-pinned` | `core.6` | every tool a gate task runs is the version the tree pins |
  | `identity-is-a-name` | none | the rule this change adds |

  `core.md`'s preamble gains: each rule is known by its name, the
  bracketed words that open it, and cited as `rule <name>`. The new rule
  sits in a new section, `### Names`, after `### The ledger`:

  > **[identity-is-a-name]** A record, a rule, and a clause, invariant
  > or scenario of a statement is known by a name chosen when it is
  > proposed and cited by that name; a number a reader sees is a display
  > ordinal derived on render from the order of acceptance on the
  > default branch, never written into a file's name, a citation or a
  > record.

  Its details entry: Rules out: an identity allocated as the next
  number on a branch, a record file `0007-….md`, a rule `[core.7]`, a
  clause `12.` or an import numbering new records after the last; a
  citation by a number, `record 0007`, `core.7` or `(12)`; a number in a
  record's file name or frontmatter. The renumbering it exists to stop:
  in another repository of the family, a constitution interview's branch
  numbered its records and rules after main's last, a parallel branch
  landed records 0002 to 0016 and rules `core.12` to `core.18` first,
  every number on the interview's branch collided, and 22 records and 75
  rules were renumbered twice, each citation rewritten. Not ruled out:
  a number shown on a rendered page, derived from main's first-parent
  history; a legacy id or alias kept on what had a number before this
  rule, resolving citations already written; another place's id, quoted
  as that place gives it; a sequence inside one file that only one
  branch writes and nothing outside it cites. Source: record
  `names-not-numbers`. (In `core.md` the examples stay in code spans, as
  here, so the citation check reads them as quoted.)
- **Citations.** A record is cited as `record <name>`, a rule as `rule
  <name>`, and in the statement an item of it as `§<name>`, each written
  plain, so the checks read it: `Source: record names-not-numbers.`,
  never with the name in a code span. A code span quotes a form without
  citing it, which is how this design, the rule's details and the
  fixtures show a citation by number. An id from
  outside this repository is cited as the place it comes from gives it:
  a decision of the design's log as `decision 40`, a request by its id,
  another repository's rule in a code span, flume's `core.21`, so no
  check reads it as one of this repository's. The details of
  `every-part-is-checked` and `tools-are-pinned` cite flume's rules so.
- **Every citation, rewritten once.** Every citation of a rule or a
  record by number in the tree names it instead: the comments of
  `Cargo.toml`, `crates/boom/moon.yml`, `moon.yml`,
  `.moon/workspace.yml`, `site/moon.yml`, `devenv.nix`, `treefmt.nix`,
  `rust-toolchain.toml`, `check-toolchain.sh` and
  `.github/workflows/pages.yml`; the first record's consequences;
  `core.md`'s sources; `openspec/config.yaml`; the roadmap. Nothing
  outside the tree is rewritten.
- **`CLAUDE.md`.** Its rules paragraph says: the decisions are
  `docs/adr/`, one MADR record each, its file named by its name; a
  design or a commit that relies on a rule cites it as `rule <name>`,
  and a record as `record <name>`; a rule is sent to `core.md` or its
  area page under a name of its own, chosen when it is proposed, never
  the next free number, and amended in the same commit as the decision
  that changes it (rule `identity-is-a-name`). Its checks table lists
  the root's format, toolchain, records, commits, statement, tests,
  specs and open changes.
- **`openspec/config.yaml`.** Its context says each record's file is
  named by its name, and names no `adrs.toml`; its citations name the
  rules;
  the design rule on citing reads "Cite by name, as `rule <name>`, the
  rules in docs/architecture/ that this design relies on. Where the
  design departs from a rule, name the rule and say why."; the design
  rule on a control plane goes, since boom has none and the rule it cited
  holds views.
- **`adrs.toml` goes**, with `check-adrs.sh` and `root:adrs`: the tool
  it configures numbers a new record after the last.

Where this repository cannot follow the rule yet, said plainly in the
record's consequences:

- Commits already on main cite rules by their legacy ids. A landed
  commit cannot change without rewriting main; the legacy ids resolve
  them, and `root:commits` reads only the commits main lacks.
- Ids from outside the repository, flume's rules, the design log's
  decisions, a request's id, are cited as their places give them until
  those places follow the rule themselves.
- The family's requirements style gives a new clause the next free
  number and an insert a letter (its section on numbers). boom's
  statement departs from that section and says so in its preamble;
  changing the style is the style's own change.
- OpenSpec lets a task carry a number. This change's tasks carry none;
  a number a later change writes on its own tasks is a sequence inside
  one file that one branch writes, and nothing outside that change
  cites it.
- The numbers the site shows need main's whole history. Where a build
  has only part of it, the step that writes the design pages fails and
  says so, rather than show numbers it cannot derive; the pages workflow
  fetches the whole history.

### The records' form

One record per decision, each with `status: accepted`, `date:
2026-11-01` and `decision-makers: [Chuck Swanberg]`, a `Status: accepted
by the user on 2026-11-01.` line, a `Deciders:` line naming the dates of
the rulings it rests on, and a `Sources:` line citing those rulings by
their words and label and the decisions by number, never by a path to a
place the reader cannot open. "The session" in the paragraphs below is
written out in a record as "the design session of 2026-10-31 on claims,
verdicts and reconciliation", and a decision as "decision 40 of the
design's log". A title has no colon followed by a space, so the
frontmatter stays a plain YAML scalar. A record names the records it
leans on as `record <name>` and the rules as `rule <name>`. The records
cite no clause: they are the decisions, and the statement is drawn from
them.

`names-not-numbers` is written first, in the commit that adds its rule
and names everything else here. The nine others follow, one commit
each, in the order of the section The records below, so the order the
site shows them in is the order they are read in (The numbers the site
shows).

Set aside: one record for the whole design, which a citation could not
narrow to the decision it rests on; records citing the sources by path,
which the user's decision of 2026-11-01 and rule `nothing-commercial`
rule out.

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
`###` headings named for their concern (`### Names and citations`),
never lettered or numbered.

Each clause, global invariant and scenario is a paragraph opening with
its name in bold brackets, `**[a-name-is-held-once]**`, the name two or
more words saying what it holds, chosen when it is written and never
changed; an amended clause keeps its name, and a withdrawn one's name is
never held again. A clause cites one of the statement's items as
`§<name>` and a record as `record <name>`, in one parenthesis at the
point it leans on them: `(§a-name-is-held-once, record
names-not-numbers)`. Nothing in the file is numbered.

The preamble gives the mechanism rule, as the style does, and the rule
on names: every item is known by its name and cited by it; a number the
site shows beside one is derived when the site is built, from the order
the items first landed on main, and is never written in the file; this
departs from the style's section on numbers, which gives a new clause
the next free number, for the reason record `names-not-numbers` gives,
and the rest of the style holds.

Parts A and B name no command, file, format, field, service or tool;
the one thing of the world they name freely is git, which the givens
fix. Real tools appear only in the givens and Part C. The statement's
content is the section The statement below.

### `root:records` and `root:commits`

`check-records.sh`, POSIX sh and awk like the script it replaces, with
no argument reads the files git tracks, from the workspace root; given
a directory, it reads every file under it as the tree, which is how its
cases run. It fails, naming the file and line of each, where:

- a file under `docs/adr` is not `<name>.md` with a name of two or more
  lowercase words joined by hyphens, a name opening with a number among
  them ("a record takes a name, never a number"); its frontmatter lacks
  `title`, `status` or `date`, or holds `number`; its `alias` is not
  four digits, or another record holds it;
- a rule on a page under `docs/architecture` does not open
  `- **[<name>]**` with such a name, another rule on any page holds its
  name, it has no details entry, or another rule holds its legacy id;
- a citation names nothing: outside code spans and fenced blocks, and
  with lines joined so a citation that crosses a line break is read as
  one, `record <name>` (either case of its first letter) names no record
  and `rule <name>` no rule;
- a citation is by number: `record` followed by four digits, or `core.`
  followed by digits, naming the record whose alias or the rule whose
  legacy id it is, and its name to cite, or that none holds it.

It reads every tracked text file but these, each exempt by name with
its reason in the script: `openspec/changes/` (a change's plan names the
identities it changes, before and after); `tests/` (fixtures that break
the rule on purpose); lockfiles and `*.svg` (written by tools). Passing,
it prints how many records, rules and citations it read.

`check-records.sh --messages` reads commit messages from its input
instead, each opening with a line `commit <hash>`, applies the same
reading of citations against the tree's records and rules, and names
the commit of each failure. `root:commits` runs it over `git log
--format='commit %h%n%B' main..HEAD`, the commits the branch holds and
main lacks; with no `main` it fails, naming the ref.

`check-records.test.sh` runs the script over `tests/records/holds/`,
which passes (a record with an alias, two rule pages, a citation of each
kind, one split across a line, and `core.1` and `record 0001` in code
spans); `numbered/` (`docs/adr/0002-amounts-as-floats.md`); `shared/`
(two records with alias `0001`, and one rule name on two pages);
`unresolved/` (`record no-such-record` and `rule no-such-rule`);
`legacy/` (`record 0001` and `core.1` outside code spans, both held as
an alias and a legacy id); and `--messages` over
`tests/records/messages.txt` (two commits, one citing `core.1`). It
asserts each exit status and that each failure names what its fixture
breaks.

The root's `moon.yml` gains `records` (inputs `/**/*`, since any file
may cite) and `commits` (`cache: false`, since it reads git's history
and not the tree), both `check-fast`, run from the workspace root, and
`test` (inputs `/check-records.sh`, `/check-records.test.sh`,
`/tests/**/*`, and the statement check's once it exists), `check-fast`.
`adrs` goes.

Set aside: a check in `crates/boom`, which is boom itself and not a
check of this repository's prose; reading commit messages in `records`,
which would make every run of it depend on git's history and never
replay.

### `root:statement` and the root's tests

`check-statement.sh`, POSIX sh and awk, takes the statement's path
(default `design/requirements.md`) and reads it whole, joining lines. An
item is a line opening `**[<name>]**`, or `- **[<name>]**`: a clause in
Parts A to C (from the `## Part A` heading to the heading after Part C's
last section), an invariant under `## Global invariants`, a scenario
under `## Scenarios the model must satisfy`. It fails, naming each line,
where two items share a name, whatever their kinds; where a line in the
parts, the invariants or the scenarios opens with a number and a full
stop (`12.`, `93a.`: "a clause takes a name"); where a `§<name>`, outside
code spans, names no item; and where a parenthesis holds a clause
number, alone or among citations (`(12)`, `(3, §a-name)`). Record
citations are `root:records`'. Passing, it prints how many clauses,
invariants and scenarios it read.

`check-statement.test.sh` runs it over `tests/statement/holds.md`
(Parts A to C with three clauses, a citation `(§first-clause,` ending a
line and `§second-clause)` opening the next, two invariants, two
scenarios, and a numbered list `1.` under `## What to deliver`, which is
none of the item sections), `shared-name.md` (two clauses named
`same-name`, and a clause and an invariant both `other-name`),
`unresolved.md` (`§no-such-clause`) and `numbered.md` (a clause line
`12. …` in Part A, and `(12)` in another clause), asserting each exit
status and that each failure names what its fixture breaks. The root's
`moon.yml` gains `statement` (inputs `/design/requirements.md`,
`/check-statement.sh`), `check-fast`, and `test` runs both test scripts,
its inputs gaining `/check-statement.sh` and `/check-statement.test.sh`.

### The design pages, computed at build

`site/scripts/design.ts`, run by bun, writes
`site/src/content/docs/design/statement.md` from the statement and
`site/src/content/docs/design/decisions/<name>.md` from each record,
emptying that directory first. `.gitignore` names
`/site/src/content/docs/design/`; nothing commits it (rule
`views-are-computed`'s reasoning holds for these too: written to a
build directory, never read back as input). The site's `moon.yml` gains
`design-pages`, with no layer and `cache: false` like `install`, since
the numbers it writes come from git's history; `typecheck` and `build`
depend on it and take `/design/requirements.md` and `/docs/adr/*.md` as
inputs; `prose` runs `vale --glob='!**/design/**' src/content/docs`.
`package.json`'s `dev` runs the script before `astro dev`.

- **The statement's page**, titled "The statement", with the H1
  dropped. Each item is written as a paragraph opening with an empty
  element whose `id` is its name, then its number as The numbers the
  site shows gives it, in bold (`12` for a clause, `I3` for an
  invariant, `S4` for a scenario) or "new" where main lacks it, then its
  name in a code span, then its words. Each `§<name>` becomes a link to
  `#<name>` reading `§<name>`, and each `record <name>` a link to that
  record's page. An item whose name is the id one of the page's headings
  takes fails the script, naming both, since the two ids would collide.
- **Each record's page**, at `/boom/design/decisions/<name>/`, titled
  with the record's title, its description `<status>, <date>`, its
  `editUrl` the record's file on GitHub, its H1 dropped. It opens with a
  line giving its number ("Decision 7", or "Not yet on main"), how to
  cite it (`record <name>`) and, for a record with an alias, "Formerly
  0001". Its sidebar order is its number, and records main lacks follow
  the rest by name. Each `record <name>` in it, and each link to another
  record's file, becomes a link to that record's page. A record whose
  frontmatter lacks its title, status or date fails the script, naming
  the file.
- **The sidebar** gains a group "Design", after "boom check", holding
  "The statement" and "Decisions", the latter generated from
  `design/decisions`, each label the record's number and title.

Set aside: links to the files on GitHub, where no clause has an anchor
and nothing checks a citation; a content loader composing Starlight's,
whose pages the link checker's pass would have to be shown to reach,
where writing files first is the way sluice's site already works.

### The numbers the site shows

A number on a design page is derived by `design.ts` each time it runs,
from main's first-parent history, and written nowhere else. The
derivation is a pure function, `ordinals`, from a list of landings, each
the names it holds in order, to a number for each name, with git read by
a thin layer around it:

- **Which history.** The default branch is `main`, as moon's
  `vcs.defaultBranch` says; the script reads `refs/heads/main`, else
  `refs/remotes/origin/main`. Where neither exists, or the repository is
  shallow, it fails, naming the ref and saying the whole history is
  needed.
- **Landings.** The commits of `git log --first-parent --reverse main`
  that touch `docs/adr` or `design/requirements.md`, oldest first. A
  landing's own commits are those its merge brings, `<landing>^1..<landing>`,
  in their order; a commit made on main directly is a landing of one.
- **Records.** A record is accepted at a landing when its file there
  says `accepted` or `superseded`. A file of the older layout,
  `NNNN-<slug>.md`, is the record whose alias is `NNNN`. Records take
  numbers from 1 in the order they were first accepted: by landing, and
  within a landing by the first of its own commits that holds them
  accepted, and within a commit by name. A record never accepted on main
  has no number; a superseded one keeps its own.
- **Statement items.** Clauses, invariants and scenarios are numbered
  apart, each from 1 (`12`, `I3`, `S4`), in the order their names first
  appear in the statement on main: by landing, and within a landing in
  reading order. An item main lacks has no number.
- **A name held again.** A name main's history once held and later lost
  is never held again: where a later landing, or the working tree, holds
  it, the script fails, naming it and the landing that withdrew it.

Since a landing only adds to the end of main's history, a later landing
never changes a number an earlier one gave, and two branches landing
side by side change nothing the other was shown. A build on a branch
shows its own new records and items with no number; the numbers they
will show come when they land.

`site/scripts/design.test.ts`, under `bun test`, holds `ordinals` to
that with landings written as data: two branches each adding a record
and landing one after the other; one landing bringing three records in
two commits; a record proposed in one landing and accepted in a later
one; an older-layout file mapped through its alias; a statement whose
second landing adds a clause between two of the first's, numbered 3
while the others keep 1 and 2, its invariant and scenario numbered
apart; and a name withdrawn and held again, which fails. `site:test` runs it beside the diagrams'
check, its inputs gaining `scripts/design.ts` and
`scripts/design.test.ts`.

`.github/workflows/pages.yml` checks out with `fetch-depth: 0`, and
builds on a push that touches `design/**` or `docs/adr/**` as well as
the site.

### How a page cites

A page cites at the end of the sentence, row or item that states what
it cites: `([§a-name-is-held-once](/boom/design/statement/#a-name-is-held-once))`,
several as `([§a](…), [§b](…); [record a-person-accepts](/boom/design/decisions/a-person-accepts/))`.
A rule is plain text, `rule views-are-computed`, which `root:records`
checks. A page that only shows a command's output, the commands page's
examples, cites once per command, in its prose. A page writes a name of
boom's examples, a claim such as `amounts-in-cents`, in a code span, so
no check reads it as a citation of this repository's records. The
roadmap links the first record to its page rather than to GitHub.

### The walkthroughs follow `a-person-accepts`

Record `a-person-accepts` rules out a person accepting a version they
proposed. Every walkthrough where Ada proposes and accepts changes so
that her coding agent proposes, with the agent's session as
provenance, and Ada accepts. `first-claims` shows once what boom says
when she runs `boom accept` on a claim she proposed herself: refused,
since she is its author, naming who may accept it. `importing-adrs`
keeps the import, whose authorship the statement leaves as a question,
and has the agent propose the new decision that follows it.

### Rendering, syncing and importing name each file by its record

`boom render adrs` and `boom sync` write each decision record as
`docs/adr/<name>.md`, its number on render in its heading and its alias,
where it has one, beside it. `boom import adr` records each MADR file as
a decision named by the file's slug with its number dropped
(`0002-amounts-as-floats.md` is `amounts-as-floats`), keeping the number
as the record's alias; `--name <file>=<name>` chooses another, and two
files with one slug are refused, naming both. The first sync after an
import writes each file under its name, in the one pull request sync
keeps. `boom propose` refuses a name the ledger holds, whatever its
record's kind or status, with exit 2 and JSON naming the record that
holds it. `boom check` fails a change whose `Claim:` trailer or marker
cites an alias, naming the name to cite, and, on a tree holding a
ledger, a name two records hold.

### The planner setting lives in the ledger's settings

Each `[repositories.<name>]` entry of the state repository's
`.config/boom.toml` may set `planner`, naming the one planner that takes
that repository's backlog and shows its proposals awaiting a person:
`planner = "millpond"`, or the name another planner asks under.
`boom backlog --planner <name> --json` lists the backlog of the
repositories naming that planner and no other; `boom query --status
proposed --planner <name> --json` lists their proposals. A repository
naming none is planned by no tool; `boom backlog` with no `--planner`
lists every repository's for a person. Both planners call boom; neither
reads the other's settings.

Set aside: millpond's own settings saying which repositories it plans,
which another planner would have to read to stay out, and tools read no
other's files; millpond as the planner wherever none is named, which
would plan a repository nobody chose to have planned and have boom name
a caller in its code.

### A backlog entry's key names its cell and its verdict

Each entry of `boom backlog --json` carries a key, the same on every
host and every run, naming the repository, the claim and the verdict
that put the cell in the backlog: `shop/invoices-final/7c1e0a4`, the
last part the verdict's id, derived from what the verdict records. A
caller takes each entry once. A cell judged again, not satisfied still,
is a new entry with a new key, which millpond's curation may attach to
the item it already sent; a cell judged satisfied is in no entry.

### The millpond loop as a diagram

`site/src/diagrams/millpond.ts`, registered as `millpond` in
`src/diagrams/index.ts`, drawn with `kit.ts` like `fits`, wide and
narrow, CSS keyframes only, one still frame for a reader who asks for
less motion, millpond's, flume's and sluice's accents from the family
list as `fits` takes them. Its one claim: the loop closes in boom. The
stations, in order: `boom judge` judges a claim not satisfied; `boom
backlog --json` lists the entry; millpond takes it in as an item; the
item's send starts work through flume in a place, its commits citing
`Claim: <name>@<version>`; sluice lands it; `boom reconcile` writes the
cell due; `boom judge` judges it satisfied, and the entry leaves the
backlog. Beside the loop, a millpond signal reaches boom as `boom
challenge` and its cells return to be judged. `fits`'s millpond lines
become "boom challenge, boom propose" and "boom backlog --json".
`site/README.md`'s diagrams paragraph names the new one.

## The records

Each paragraph is a record's substance: its name and title, the
question its context asks, the options it weighs, the outcome and its
consequences, and its sources. The record says it in the voice of the
first record and says nothing the paragraph does not.

**`names-not-numbers`, A record is known by a name fixed when it is
proposed, and every number a reader sees is derived on render.** What
identifies a record, and what does a citation name, so that work on two
branches never collides? In another repository of the family, a
constitution interview's branch numbered its records and rules after
main's last; a parallel branch landed records 0002 to 0016 and rules 12
to 18 first, every number on the branch collided, and 22 records and 75
rules were renumbered twice, each citation rewritten. Options: the next
free number, allocated on the branch where the record is written, as
MADR's tools and the family's statements do; a number allocated when
the work lands, the landing rewriting the branch's numbers and
citations; a name chosen by the proposer, held unique by a check at the
merge, every citation naming it, and any number a reader sees derived
on render from the order of acceptance on the default branch; an
identifier no two proposers can share, random or derived from the
record's words. Outcome: the third. A **name** is lowercase words
joined by hyphens, chosen by whoever proposes the record and fixed when
it is proposed; it never depends on what else exists, and it never
changes: an amendment keeps it, a supersession is a new record with a
new name, and a name once held is never held by another record, even
after its record is rejected or superseded. No two records of a ledger
hold one name, whatever their kinds. boom refuses a proposal naming a
name the ledger holds; where two lines of the ledger's history are
joined, a join giving one name to two records fails the check at the
merge, naming both, so the later proposer chooses another name before
anything lands, and nothing landed is renamed. Every citation names
the record, with its version where one is meant: a claim's attachment
and the claim it supersedes, a narrative's inclusion, a landing's
`Claim: <name>@<version>`, a marker, a rule, a statement's clause, a
comment in code. Every other identity boom gives is derived from what
it identifies: a version from a record's words, a verdict's id from
what it records, a backlog entry's key from its repository, claim and
verdict. A number a reader sees, a rendered decision record's 0007, a
place in a list, is a **display ordinal**: for each kind of record, the
order in which records were first accepted in the ledger's history on
its default branch, derived each time it is shown and never written
into a file's name, a citation or a record. A record never accepted
shows none; a superseded one keeps its own. Since an acceptance only
adds to the end of that history, a new record takes the next ordinal of
its kind, and no landing beside another moves an ordinal already shown.
An imported ADR keeps its number as an **alias**, legacy: it resolves
citations written before the import, in old commits and in the imported
bodies ("superseded by 0003"); the rendered record shows it beside the
name; and boom refuses a new citation by alias, naming the name to
cite. The first option falls on the renumbering. The second because a
branch's commits already carry the numbers the landing would replace,
and a gate that rewrites what it lands lands something other than what
it checked. The fourth because a citation of it tells a reader nothing,
and an identifier derived from the words changes with an amendment,
which a name must survive (a version is already that). Consequences:
rendered decision records are files named by their records; an import
keeps each number as an alias, and the first sync names each file by its
record; a clash of names is caught at the merge rather than renamed
after it; boom's own repository follows the same, its records and rules
named and its statement's items named, the numbers they had kept as
legacy aliases and ids (rule `identity-is-a-name`). Where it cannot yet:
commits already on main cite by number and cannot change, and the
legacy ids resolve them; ids from outside the repository are cited as
their places give them; the family's requirements style numbers
clauses, and boom's statement departs from it; a number the site shows
needs main's whole history, and a build without it fails rather than
guess. Sources: the user's direction of 2026-11-01, with the renumbering
as its example.

**`five-kinds-of-record`, The ledger holds claims, narratives,
contexts, relationships and terms.** What does boom record? Options: a
store per form (ADR files, a rules file, a map, a glossary); one ledger
of five kinds of record, every other form a view; one kind, the claim,
with narratives, the map and terms as claims. Outcome: five kinds. A
**claim** has a name, a kind (invariant, decision or assumption), a
statement, at least one scenario saying how one would know it holds,
for a decision a body (context, drivers, options, outcome,
consequences, written when it is proposed), what it attaches to, an
optional check rule, the claim it supersedes, and provenance. A
**narrative** is an explanation in its author's words at the time,
scoped to a context, including claims by name; a person judges it, and
it is never given a verdict. A **context** is a part of the system with
its own model and language: a name, its responsibilities, and its home,
a repository optionally narrowed by paths. A **relationship** names two
contexts, which is upstream, its pattern (customer and supplier,
conformist and the other fixed patterns), and the direction a
dependency may run. A **term** is a word as one context means it. Each
record is known by its name (record `names-not-numbers`) and versions;
a version is the hash of what it says, for a claim its statement,
scenarios and body; what it attaches to and its check rule sit outside
the hash. A claim is written at the destination's grain: it survives a
rewrite of the code, names what someone could observe, and can be
judged from the repository alone; it never names a file, a colour, a
setting or a step of a pipeline, which is a check rule's business. The
one kind not versioned, the verdict, is record
`verdicts-beside-the-claims`'s. The third option falls because a
verdict on a narrative is a category error and a term needs its own
proposal and acceptance. Consequences: an ADR is a decision claim;
rules, decision records, the design book and the map are views (rule
`views-are-computed`); a decision's reasoning is never written after the
fact; an imported MADR file has no scenario, which the statement asks
about. Sources: decision 18 (a claim is any assertion or decision about
the system), decision 26 (reasoning is written when a decision is
made), decision 27 (the design book is a view of narratives and
claims), decision 28 (the map is records), the session's table of
records.

**`status-from-seven-events`, A status is read from seven events, and a
claim changes by amendment or supersession.** How does a record move
from proposed to standing and beyond, and where is its status kept?
Options: a status field, edited; events appended, the status read from
them; a status given by where a file sits. Outcome: seven events,
appended, never changed or removed (rule `status-read-from-events`),
each naming the record and version it acts on, its actor (a person or
an agent) and its provenance (a session, a run, a transcript, a
signal). `proposed`: a new record, or a new version of a standing one;
visible, never checked, never judged. `accepted`: by a person (record
`a-person-accepts`); the version stands, and the earlier one it amends
stays in the history. `rejected`: by a person; the proposed version is
closed. `superseded`: written when a person accepts a claim naming one
it replaces; the old claim no longer stands. `annotated`: a note on one
version; no status changes. `challenged`: names a claim and the signal
that bears on it; it stales the claim's verdicts in every repository,
and an assumption goes back to a person; written by any caller through
boom's command line (record `millpond-both-ways`). `due`: written by
boom when it reconciles a repository (record `judged-on-a-host`); it
names one verdict and stales that cell alone. A standing claim changes
by amendment, a new version of the same name whose cells are judged
again, or by supersession, a new name naming the one it replaces, whose
cells start unjudged as the old one's leave. "After" is order in the
ledger's history, which is one line, and no status depends on a clock;
the same order gives the numbers a reader sees (record
`names-not-numbers`). The edited field falls because edits meet in
conflicts and lose who agreed; the file's place because acceptance
would be a side effect of moving a file and not a person's act.
Consequences: a status is computed every time it is asked; two people
adding events on two branches rarely conflict; the history is the
record of who agreed to what; withdrawing a claim with nothing in its
place, and moving what a claim attaches to, are not among the seven,
and the statement asks how they are recorded. Sources: decision 32
(status is derived from events); ruling Q2; the session's table of
events.

**`verdicts-beside-the-claims`, A verdict is a record beside the
claims, and a cell's status and its two queues are computed.** Where
does a repository's standing against a claim live, and how is it kept
current with nothing running? Options: a status per cell, kept by a
process that runs a state machine for each; verdicts appended as boom
records beside the claims, a cell's status a function of the records,
the queues computed; no verdicts, the gate's checks the only reading.
Outcome: the second. A **cell** is one standing claim and one
repository in its scope (record `one-context-per-repository`), derived
and with no record until judged; it is never a change, a revision or a
piece of work, and work that cites no claim has none. A **verdict**
names the claim, the version judged, the repository, the revision
judged, a value (satisfied, partial, not satisfied, not applicable), the
evidence at that revision, a few words, and who judged it (the session
and its model), and is appended, never edited, in the ledger beside the
claims; its id is derived from what it records. A verdict is a judgment
stored with its inputs, never a computation; a marker naming the claim
at its version is evidence the judge is handed, never the verdict. The
status, in order: the claim does not stand, no cell; the repository
homes nothing it attaches to, out of scope, its verdict reading not
applicable and kept; no verdict, unjudged; the latest verdict judged an
earlier version, stale; a `challenged` event after it, stale; a `due`
event naming it, stale; otherwise current, with its value. A stale cell
lists every reason that holds. The queues: **to judge**, every cell
unjudged or stale; the **backlog**, every current cell judged partial
or not satisfied. Neither is stored. Nothing else moves a verdict: not a
tag, a kind, a term, a narrative, a path inside a home, a check rule or
a check's result; a repository changing does not by itself stale one.
An assumption is about the world, not a repository, and has no cells.
Anyone holding the ledger computes the same status, since the function
reads boom's records alone. The first option falls on the status
ruling: nothing runs between commands, and a stored status breaks rule
`status-read-from-events`; the third because the gate asks another
question (record `the-gate-blocks-regressions`), and without verdicts
there is no backlog. Consequences: `boom judge` takes the queue to judge
and `boom backlog` lists the backlog; a planner reads the backlog and
nothing else (records `judged-on-a-host`, `millpond-both-ways`).
Sources: decision 39; the status ruling; the session's status function
and queues.

**`judged-on-a-host`, boom reconciles and judges each repository at a
commit, on a host, with no other service.** Where and when are verdicts
judged and kept current, and who judges? Options: inside a planner's
run, judging and planning together; boom on a host where the
repository is checked out, reconciling and judging, a planner reading
the judged backlog; in a change's own place before it lands. Outcome:
the second, at shared heads only. `boom reconcile`, for one repository
at its shared head, writes a `due` event for a current verdict whose
cited evidence is gone, or whose claim a landing since cites (a
`Claim:` trailer or a new marker), and nothing else. `boom judge` hands
a model session, for each cell to judge, closed inputs: the claim at
its standing version with its scenarios and body, the narrative that
includes it, the cell's previous verdict and its evidence, whether the
shared head carries a marker naming the claim and at which version, and
the repository at its shared head, read-only; nothing else reaches it.
It writes the one verdict per cell the session returns, checked against
the verdict's form; anything else is refused and the cell stays to
judge. Never a verdict for a proposed claim, never one from a person (a
person answers an unclear check, record `the-gate-blocks-regressions`,
and never asserts that work is done), never one on a revision that has
not landed. Both commands ask the status first, so either run again
with nothing changed writes nothing, and any trigger may run them: by
hand, a hook after each merge, a schedule, a caller's command; a
trigger makes it sooner, a schedule makes a quiet repository converge.
A repository first homed is judged once, as a baseline. The ledger
stays complete with no other service. The planner's run falls because
the ledger would be complete only while something plans, and a planner
holds no code to judge; the change's place because a verdict written
there stands for code that never lands when the change is dropped.
Consequences: a landing that cites a claim makes its cell due, so work
closes its own loop (record `millpond-both-ways`); which model judges
is a question the statement keeps. Sources: decision 40; rulings Q1 and
Q3.

**`the-gate-blocks-regressions`, boom check reads a change in three
layers, and the gate blocks regressions, never the backlog.** What does
the merge gate ask of a change, and from what? Options: whether the
repository satisfies every claim in scope after the change; whether the
change moves the repository away from a standing claim it touches, in
three layers, cheapest first; a model reviewing every claim for every
change. Outcome: the second. **Scope**: the changed files, mapped to
the contexts homed there, keep the standing claims attached to them and
to the whole system. **Rules**: a claim's check rule passes or fails
with no judgment; the map gives one rule for nothing, a new dependency
between contexts that no relationship allows fails; so does a marker or
a trailer naming no standing claim, or naming one by an alias (record
`names-not-numbers`). **Prose**: a reviewer model gets the diff and the
claims still to settle, and nothing else, and returns holds, violated
or unclear for each, with a few words. A claim the repository does not
yet satisfy is the backlog's, and a change that leaves it alone holds
it. The check reads its claims from the slice committed in the
repository (Q4): the standing claims in scope, with their versions,
check rules and the evidence paths of their current verdicts, so
everything it reads is in the change's own tree and a gate that caches
by content replays an unchanged check; a claim is checked there once
its sync lands. Violated blocks the merge: the agent fixes the code, or
proposes an amendment or a claim to replace it, which waits on a person
(record `a-person-accepts`). Unclear fails as unclear until a person
answers for that change, `boom check --answer` (Q5); the answer is that
change's alone and never a verdict. A check is not a verdict: one that
holds writes none, and one that fails moves none, its change never
having landed. boom calls its result a check and keeps "verdict" for
the ledger; in a gate's reading of all a change's checks, `boom check`
is one row, pass or fail. A prose claim checked often is a candidate
for a rule, which changes how it is checked and never its version. The
first option falls because every merge would wait on every claim nobody
has built; the third on cost and on certainty where a rule can say it.
Sources: rulings Q4 and Q5; the session's section on the gate; boom's
first sketch of the check, which this record carries.

**`one-context-per-repository`, A claim's scope is derived, and each
repository is one context until someone models the system.** How does a
claim reach the repositories it binds, and must a project model its
system first? Options: a claim names its repositories or paths; a claim
attaches to a context, a relationship or the whole system, its scope
the repositories homing them, each repository one context until a map
is drawn; a map before any claim. Outcome: the second. Scope is
derived, never chosen: a context gives its home; a relationship both
ends' homes, each repository its own cell; the system every repository
the map homes anything in. Only three things move a cell through scope:
a home moves, an attachment moves, or the claim's version moves. A home
may name paths in its repository; `boom check` reads them to match a
diff to claims (record `the-gate-blocks-regressions`), and scope never
does, so moving a path stales nothing. Until someone draws the map,
each repository is one context, homed in itself, and a new claim
attaches to the context of the repository it was written for. Drawing
the map changes no verdict: a claim keeps its repository's context
until someone attaches it elsewhere, and that context lasts while any
claim is attached to it. Contexts and relationships are records,
proposed and accepted like claims. No second map of what is built is
kept (Q6): the accepted records are the target, and what is built is
read from the verdicts. The first option falls because a claim would
stale whenever code moves; the third because a person on one repository
would have to model a system before writing one claim. Sources:
decision 28; ruling Q6; the session's section on the map.

**`a-person-accepts`, A person whose role allows it accepts, never an
agent and never the record's author.** Who may make a proposed version
stand? Options: anyone who runs `boom accept`; any person, never an
agent; a person whose role carries the permission, never an agent,
never the version's author. Outcome: the third. Roles, and who holds
each, are the ledger's settings, set by its keeper. An accept from an
agent, from the actor of the version's `proposed` event, or from a
person whose role does not carry the permission is refused, naming why,
and records nothing; the accept that supersedes a claim is held to the
same. Rejecting is a person's, never an agent's. An agent proposes,
amends and supersedes by proposal, and never gets a change through on
its own (record `the-gate-blocks-regressions`). The first option falls
because an agent could rewrite what the project claims to get its own
work through; the second because an author's own accept is one person's
word, not an agreement. Consequences: a person working alone accepts
what an agent proposed, and a version they proposed waits on another
person; `boom mcp`, which serves an agent, offers no accept and no
reject; how boom tells a person's act from an agent's, and how it reads
a role, the statement asks. Sources: decision 41.

**`millpond-both-ways`, millpond and boom work both ways, and millpond
takes in a backlog only where no other planner plans it.** How does
what millpond hears reach boom's claims, and how does boom's backlog
reach work? Options: one way, millpond proposing and challenging and
the backlog left to a person; both ways, millpond taking in every
repository's backlog; both ways, millpond taking in a repository's
backlog only where no other planner plans it, the repository's
configuration saying which. Outcome: the third, the user's ruling of
2026-11-01 (request `req_ee1f3b7c`). (a) A millpond signal that bears on
a claim reaches boom as a `challenged` event, `boom challenge <name>
--signal <key>`, or as a proposed claim, `boom propose` with the signal
as its provenance; the challenged claim's cells are judged again
(record `verdicts-beside-the-claims`), and an assumption goes to a
person. (b) millpond takes `boom backlog --json` in as items, each entry
once by its key; an item's send starts the work through flume, the
work's commits cite the claim by name at its version, the landing makes
the cell due (record `judged-on-a-host`), and `boom judge` judges it
again. The loop runs claim, judged not satisfied, item, work, landed,
judged satisfied, and the entry leaves the backlog. (c) Where a
repository's configuration names another planner, that planner reads
the backlog itself and millpond takes none of it in, so nothing is
planned twice. Tools never read each other's files: each calls the
other's command line. boom never plans, starts or lands work, and a
planner never writes a verdict. The first option falls because a
backlog nobody takes in waits on a person to notice it; the second
because a repository another planner plans would be planned twice.
Consequences: the `planner` setting per repository; `boom backlog
--planner`; a challenge naming the same signal twice adds nothing.
Sources: the ruling of 2026-11-01; decision 40 (a planner reads the
judged backlog); the design session of 2026-10-31, which brings a
challenge into boom as an event because no tool reads another's files.

**`proposals-as-signals`, A proposal awaiting a person shows as a
millpond signal where no other planner is connected.** Where does a
person see what waits on them? Options: only by asking boom, `boom
query --status proposed`; as millpond signals for every repository; as
millpond signals where no other planner is connected to the repository,
the planner that is connected showing them where one is. Outcome: the
third (decision 42). millpond lists the versions awaiting a person in
the repositories whose configuration names it, each once by its
record's name and version, as a signal; where the configuration names
another planner, that planner shows them and millpond carries none. The
same setting decides this and record `millpond-both-ways`'s (c).
Showing a proposal anywhere accepts nothing: only a person's accept does
(record `a-person-accepts`), and a version accepted or rejected leaves
the listing. The first option falls because a proposal nobody is shown
waits unseen; the second because a person would be asked twice.
Sources: decision 42; the ruling of 2026-11-01.

## The statement

What each part holds, in substance; each line is one clause or a few,
written to the style's rules (one invariant each, present tense, the
strong words, checkable, no mechanism in Parts A and B), each opening
with its name and citing the record behind it. A clause's name is
chosen as it is written, two to five words saying what it holds; the
names below are the ones the section on names takes, and the rest are
the writer's.

**Preamble and purpose.** A statement of what boom must always do and
never do; the three parts; the mechanism rule; the rule on names (The
statement's shape). boom keeps a ledger of what a project claims about
itself, judges each repository against the claims that stand, computes
every view and queue from the ledger, and reads each change against the
claims it touches; it never plans, starts or lands work.

**Actors**, may decide and may not: **a person** (accepts and rejects
where their role allows, answers an unclear check, names what they
propose; may not accept their own proposal or assert a verdict); **an
agent** (proposes, naming what it proposes, annotates, proposes
amendments and supersessions, runs the check; may not accept or
reject); **the judge**, a model session boom starts (a cell's value,
evidence and words from its closed inputs; may not read beyond them or
judge a proposed claim); **the reviewer**, a model session the check
starts (holds, violated or unclear for a claim against a change; may
not write a verdict); **a planner** (what work a backlog entry becomes,
and when; may not write a verdict or take a repository's backlog its
settings do not name it for); **a caller**, any tool or person running
boom's commands (what to ask and hand in; may not write the ledger but
through boom); **the keeper** (the repositories a ledger describes, each
one's planner, the roles and who holds them); **boom** (what its
commands compute and the events and verdicts they append; may not
accept, choose or change a record's name, decide a verdict for the
judge, plan, start or land work, or read another tool's files).

**Vocabulary**: ledger, record, name, alias, display ordinal, version,
event, actor, provenance, claim, kind, invariant, decision, assumption,
statement (a claim's), scenario, body, narrative, context, home,
relationship, term, map, attachment, scope, standing, proposed,
amendment, supersession, signal, challenge, cell, verdict, value,
evidence, revision, shared head, landing, citation, status (a record's
and a cell's), stale, to judge, backlog, reconcile, judge, change,
check, layer, check rule, outcome, answer, slice, view, role, keeper,
planner, entry, key. Terms by meaning; a model may add, never redefine.

**Part A, Names and citations** (record `names-not-numbers`):
`a-name-is-chosen-once`, every record has a name its proposer chooses
when proposing it, and the name never changes, an amendment keeping it;
`a-name-is-held-once`, no two records of a ledger hold one name,
whatever their kinds, and a name once held is never held by another
record, even after its record is rejected or superseded;
`a-clash-is-refused-before-it-lands`, an append, or a join of two lines
of a ledger's history, that would give one name to two records is
refused before it lands, naming both, and boom never renames a record;
`no-identity-depends-on-others`, no identity boom gives, a name, a
version, a verdict's or an entry's, depends on what else the ledger
holds; `citations-name-records`, every citation of a record names it,
with its version where one is meant, and none names a record by a
number; `numbers-are-derived-when-shown`, a number shown with a record
is derived when shown, from the order records of its kind were first
accepted in the ledger's history, and is part of no record, name or
citation; `no-acceptance-moves-a-number`, a record accepted after
another is shown a later number, so no acceptance, wherever it was
made, changes a number already shown, and a record never accepted is
shown none; `an-alias-is-legacy`, a record brought in from an earlier
numbering keeps that number as its alias, an alias resolves only text
written before the record was brought in, and boom refuses one anywhere
else, naming the record's name.

**Part A, Records and versions** (record `five-kinds-of-record`): a
version is fixed by what a record says alone, so the same text has the
same version and new text a new one; a claim's version covers its
statement, scenarios and body and never what it attaches to or its
check rule; every claim has exactly one kind and at least one scenario;
a decision's reasoning is the reasoning as proposed, and nothing added
later is reasoning; a narrative includes a claim by name, never by
copy, and is never given a verdict; a term belongs to exactly one
context; a relationship names its two contexts, the upstream one and
the direction a dependency may run.

**Part A, Events and standing** (record `status-from-seven-events`): a
status is read from events alone and stored nowhere; an event is
appended and never changed or removed; every event names one version of
one record, its actor and its provenance; order is the ledger's history,
one line, and no status depends on a clock; a proposed version is
visible and never checked or judged; a version stands only by an accept
(Acceptance); an accepted amendment stands in place of the version it
amends, which stays in the history; the accept of a claim that names one
it replaces ends that one's standing in the same act; an annotation
changes no status; a challenge names one claim and one signal, and a
second naming the same pair adds nothing; a due event names exactly one
verdict, and no verdict is due twice.

**Part A, Acceptance** (record `a-person-accepts`): only a person
accepts, never an agent; never the version's author; only a person whose
role carries the permission, roles being the keeper's settings; an
accept that breaks any of these is refused, says why and records
nothing; only a person rejects; showing a proposal anywhere accepts
nothing (record `proposals-as-signals`).

**Part A, Scope and the map** (record `one-context-per-repository`): a
claim never names its repositories, and its scope is the homes of what
it attaches to; a context's scope is its home, a relationship's both
ends' homes, the system's every repository the map homes anything in;
only a home moving, an attachment moving or a version moving changes a
claim's cells; a path inside a home never moves scope; until a map is
drawn, each repository is one context homed in itself and a claim
proposed for it attaches to it; drawing the map changes no verdict; no
map of what is built is kept beside the target, and what is built is
read from verdicts.

**Part A, Cells and verdicts** (record `verdicts-beside-the-claims`): a
cell is exactly one standing claim and one repository in its scope,
never a change, a revision or a piece of work; a cell has no record
until judged; a verdict is appended and never edited, and names the
claim, the version judged, the repository, the revision judged, its
value, its evidence at that revision, its words and its judge; its value
is exactly one of satisfied, partial, not satisfied and not applicable;
a verdict is a judgment and never computed, the mechanical facts being
inputs; an assumption has no cells; verdicts are kept in the ledger
beside the claims.

**Part A, Status and the queues** (record `verdicts-beside-the-claims`):
a cell's status is a function of the ledger alone, so any holder of the
ledger computes the same; the order of that function, one clause per
step as the record gives it; a stale cell lists every reason that holds;
the queue to judge and the backlog are computed, never stored; what
never moves a verdict; a status changes only by a record appended, and
nothing runs between commands.

**Part A, Reconciling and judging** (record `judged-on-a-host`):
verdicts are judged at shared heads only; a landing citing a claim
makes its cell due; a reconcile writes a due event only for a current
verdict whose evidence is gone at the shared head or whose claim a
landing since cites; a judge writes a verdict only for a cell to judge;
either, repeated with nothing changed, writes nothing; the judge's
inputs are closed, and nothing else reaches it; an output not of a
verdict's form is refused and the cell stays to judge; never a verdict
for a proposed claim, from a person, or on a revision that has not
landed; a repository first homed is judged once as a baseline;
reconciling and judging need no other service.

**Part A, The check** (record `the-gate-blocks-regressions`): it asks
whether a change moves the repository away from a standing claim the
change touches, and never whether the repository satisfies a claim; a
claim the change leaves alone holds; the layers run cheapest first, a
claim an earlier layer settles reaches no later one, and only the
claims still to settle reach the reviewer; it reads only what is in the
change's own tree, so a claim is checked once its sync lands; every
claim it reads has one outcome, holds, violated or unclear, with words;
unclear fails until a person answers for that change, and the answer is
never a verdict; a check writes no verdict; a violated claim is changed
only by a proposal a person accepts; a check rule changes how a claim
is checked, never its version; a change citing a record by an alias, or
a name no record holds, fails.

**Part A, Views** (records `five-kinds-of-record`,
`status-from-seven-events`, `names-not-numbers`): every view is computed
from the ledger, and the same ledger gives the same view; no view is
read back as input; rendering arranges and never reasons; every part of
the design book names the record it shows, so a note on it reaches the
record; a view shows each record under its name, and any number it
shows is a display ordinal.

**Part A, What boom never does** (records `judged-on-a-host`,
`millpond-both-ways`): it never plans, starts or lands work, and never
changes work that cites a claim; it gives any caller the standing
version of each claim, so work citing an earlier one can be told; it
reads no other tool's files, and every exchange with another tool is a
command and its output; it writes into a repository it describes only
by proposing a change there, which lands as that repository's other
work does.

**Part B, one section per seam**, each its operations in abstract names
and the guarantee: **the ledger's store** (append; read in order; join
two lines; appends never lost or reordered, two appenders both land and
neither overwrites, an append bringing a name the store holds is
refused, and a join giving one name to two records is refused); **the
repository** (its shared head; what a revision holds; the landings since
a revision, with what each cites; a change against a base; what is read
at a revision is that revision's); **the judge** (judge, from closed
inputs, to one verdict or none; it reads nothing else); **the reviewer**
(review, a change and the claims to settle, to an outcome and words per
claim; an answer not of that form reads unclear); **the actor** (who
acted, a person or an agent, and the roles they hold; an agent is never
taken for a person); **the planner** (backlog, for a planner, every
entry of each repository whose settings name it, each with its key, the
claim by name at its standing version, its scenarios, the verdict's
value, evidence and words and why the cell came back; proposals, for a
planner, every version awaiting a person in those repositories; a key is
the same on every host and run and names one cell and one verdict, and
no repository's entries are given to two planners) (records
`millpond-both-ways`, `proposals-as-signals`); **signals** (challenge, a
claim by name and a signal; propose, a record, its name and its
provenance; standing, whether a named claim stands; the same signal
challenging the same claim twice adds nothing) (record
`millpond-both-ways`); **the gate** (check, a candidate's tree to pass
or fail; everything it reads is in that tree) (record
`the-gate-blocks-regressions`); **binding the contracts** (each seam
bound by naming profiles of Part C; a ledger binds one store and one
actor profile; any number of planners and signal sources; one
conformance suite holds every profile).

**Part C, the profiles.** "Parts A and B do not change to admit a
profile." **The store in git**: a state repository of its own, in git,
recfiles, boom committing directly, a rejected push rebased and its
append checked again, refused where its name is now held; where the
ledger's repository takes changes by merge, `boom check` on the
candidate fails a name two records hold; the order of the history is
the first-parent history of the default branch, and within one landing
the order of its own commits; its layout is a question. **The
repository in git**: the shared head is the default branch at the
remote; a landing cites a claim by a `Claim: <name>@<version>` trailer;
a marker is the same words in an OpenSpec requirement, a test, a doc
comment or a spec file. **The judge and the reviewer**: a model session
boom starts with the closed inputs, the model a setting (a question).
**The actor**: not bound yet; the questions ask how. **The gate in
sluice**: sluice running `boom check --base main` as a moon task in a
layer the branch requires, with `.boom/slice.rec` an input, so moon
replays an unchanged check; in sluice's verdict, one row. **The planner
in millpond**: `planner = "millpond"`, reading `boom backlog --planner
millpond --json` through an adapter, each entry an offer under its key,
curation making items with briefs that cite the claim by name at its
version, a send to a place started through flume, the work landed by
sluice, sluice's hook after each merge running `boom reconcile`, and
`boom judge` closing the loop; and another planner, named by its own
name, which reads the same command for its repositories. **Signals from
millpond and flume**: millpond's challenge move running `boom challenge
<name> --signal <key>`; a signal asserting something new running `boom
propose` with a name and the signal's key as provenance; millpond's
"does this claim stand" read with `boom query <name> --json`; flume
proposing from a run with the run as provenance and asking `boom query
<paths> --json` what applies before work. **The surfaces**: the command
line, every command with `--json`; `boom mcp`, with no accept and no
reject; the Claude Code plugin; the herdr plugin. **The renderer**:
mdBook, run by command where installed, none of its code embedded;
decision records as MADR files named `<name>.md`, each showing its
display ordinal and, where it has one, its alias; `boom import adr`
keeping each file's number as its record's alias.

**Global invariants**, each a named item citing its clauses: a status,
whether a record's or a cell's, is read from the ledger and never
stored; every view and queue is computed; no agent accepts and no author
accepts their own; the gate blocks only a change that moves away from a
standing claim; reconciling and judging, repeated on unchanged inputs,
write nothing; every holder of the ledger computes the same status for
every cell; boom and another tool exchange only commands and their
output; no repository's backlog is planned by two planners; no landing
renames a record or changes a number already shown.

**Non-goals**: planning, ordering, starting or landing work; approving
work; running checks other than its own, or deciding which a repository
requires; hosting anything or serving a network; rendering a book
itself; keeping a map of what is built; judging a claim from anything
but a repository at its shared head; choosing a record's name for its
proposer.

**Givens.** Constraints: repositories are git repositories; boom is free
and open source under MIT, written in Rust, its binary `boom`; each tool
of the family keeps its own state in git and reaches another only by its
command line. The rest of the stack, displaceable with a reason: the
ledger as recfiles in a state repository of its own, written by boom
directly; moon and sluice for the gate; mdBook for the book; millpond
and flume as the family's inbox and launcher; Claude Code and herdr for
agents.

**Questions**, marked as not requirements: how the ledger is laid out
on disk; how boom tells a person's act from an agent's and reads a role
(the actor's binding); how a claim is withdrawn with nothing in its
place, and how an attachment moves, by events of their own or by a
proposal; whether the map needs parts finer than a context, elements
inside one and the links between them; what a check rule looks like;
which model judges and which reviews, and how a project names them;
where an answer to an unclear check is kept, so the gate's next run of
the same change reads it; what keeps a change from editing the slice it
is checked against; an imported decision's scenario, and whose accept
an import records; where the ledger lives for a project of one
repository; what a proposal whose scope spans repositories naming
different planners shows where; the name.

**Scenarios**, each a named item, as the operator lives them, each
citing the clauses it exercises: a person alone on one repository, with
no map, has an agent propose an invariant, accepts it, and after `boom
judge` sees it in the backlog; an agent's accept is refused; a person's
accept of their own proposal is refused; a change that leaves a claim
nobody has built alone passes the gate; a change that breaks a standing
claim is blocked, the agent proposes an amendment, and the gate holds
the old version until a person accepts; an unclear check fails until a
person answers it; forty commits that leave a verdict's evidence alone
judge nothing again; a landing that removes a verdict's evidence makes
the cell due, and the next judge judges it; an amendment makes every
cell of the claim stale, a supersession retires the old cells and starts
the new claim's unjudged; drawing a map changes no verdict; a millpond
signal challenges a claim and its cells are judged again, and a
challenged assumption goes to a person; the millpond loop from not
satisfied to satisfied; a repository whose configuration names another
planner has none of its backlog taken in by millpond; a proposal waits
as a millpond signal where millpond is the planner, and is not one where
another planner is; reconcile and judge run twice with nothing changed
write nothing; two branches each propose a decision and land one after
the other, neither is renamed, the one accepted first shows the earlier
number, and every number shown before either landed is unchanged; two
branches each propose a record named `money-has-currency`, the first
lands, the second is refused at the merge naming both, its proposer
names theirs `amounts-carry-currency` on the branch and it lands, and
nothing landed was renamed; a project imports ADRs 0001 to 0004, a
commit from before the import citing ADR 0002 resolves to
`amounts-as-floats`, and a new change citing 0002 is refused, naming
`amounts-as-floats`.

**What to deliver**: a model of boom satisfying Parts A and B: its
records and events, the status function, the queues, the commands and
their output; a conformance suite whose scenarios cite the clauses they
exercise by name; the profiles of Part C, each passing it.

## The site

Each page, what changes on it and what it cites (How a page cites).
Every claim on a page about the design links the clause or record behind
it, by the names the statement and the records hold.

- **index.mdx**: "Where it stands" says the repository holds its rules,
  its statement and its decision records, linked, and a binary that
  prints its version; the cards' "Status read from events" and "Checked
  at the merge" cite their records; "Who it is for" says agents propose
  and a person whose role allows it accepts.
- **how-its-used.mdx**: the records table cites record
  `five-kinds-of-record` and its clauses; a new section, "Names, not
  numbers", says a record is known by a name its proposer chooses, cited
  by it, and that a number shown is derived, citing record
  `names-not-numbers` and its clauses; the events table gains
  `challenged` and `due` as decided, each row citing its clause, and the
  closing sentence on later events names only those the statement still
  asks about; the accepted row and a new paragraph say who may accept
  (record `a-person-accepts`); amend and supersede cite record
  `status-from-seven-events`; "Verdicts" becomes cells, verdicts and the
  two queues, citing record `verdicts-beside-the-claims` and its
  clauses, and where they are judged, citing record `judged-on-a-host`;
  "Contexts" cites record `one-context-per-repository`; the views
  table's decision records are MADR files named by their records,
  numbered on render in the order accepted.
- **configuration.md**: `[repositories.shop]` gains `planner =
  "millpond"`, commented with records `millpond-both-ways` and
  `proposals-as-signals`; a `[roles]` sketch says who may accept (record
  `a-person-accepts`); `events.rec`'s `%type` gains `challenged` and
  `due`, with one example of each; the verdict example gains its id and
  its judge; the layout sketch shows `docs/adr/` holding each decision
  under its name; the caution keeps the format a sketch.
- **commands.md**: `boom judge`, `boom reconcile`, `boom backlog`, `boom
  challenge` and `boom annotate` join the table with their own sections,
  arguments, an example and its JSON, each citing its record and
  clauses; `boom backlog --json` shows entries with their keys and
  `--planner`; `boom query` gains `--planner`; `boom propose` shows the
  refusal of a name the ledger holds, as JSON with exit 2; `boom accept`
  says who may accept, citing record `a-person-accepts`, and shows the
  refusal of an author's accept as JSON with exit 2; `boom render adrs`
  and `boom sync` write `docs/adr/money-has-currency.md`; `boom import
  adr` prints each record's name and alias, its JSON giving `name` and
  `alias` and no `number`, and names `--name`; `boom mcp` lists no
  accept and no reject; "Also in the design" goes, as nothing in it is
  left sketched less far. Each section cites record `names-not-numbers`
  where it names a record.
- **check.mdx**: each section cites record `the-gate-blocks-regressions`
  and its clauses: the three layers, what it reads, regressions not the
  backlog, the outcomes table, a check is not a verdict, and the two
  meanings of verdict; the rules layer names a citation by alias and a
  name two records hold among what fails, citing record
  `names-not-numbers`.
- **integrations.mdx**: millpond's section becomes both ways: (a) a
  signal to `boom challenge` or `boom propose`, (b) the backlog taken in
  as items, the loop drawn as the `millpond` diagram with a caption
  naming each station, (c) another planner reading the backlog itself,
  the `planner` setting deciding, and proposals as signals where
  millpond is the planner; each cites records `millpond-both-ways` and
  `proposals-as-signals` and the clauses of the planner's and signals'
  profiles. `boom mcp`'s note on accept cites record `a-person-accepts`.
  sluice and flume cite the clauses of their profiles.
- **roadmap.md**: "Settled" lists each record, linked to its page, with
  the clauses that state it, and keeps the rules
  `status-read-from-events` and `views-are-computed` by name; "Leaning"
  and "Proposed" keep only what no record decided (writing to git
  directly, rendering, sync's pull request, import, the plugins,
  signing); "Open questions" become the statement's questions, each
  linked, with the build order kept as the roadmap's own.
- **guides/first-claims.md**: the walkthroughs' decision; Ada's agent
  proposes, Ada accepts; the refusal of her own accept shown once;
  `boom render adrs` writes `amounts-in-cents.md` and `boom sync` names
  `docs/adr/amounts-in-cents.md`; cites records
  `one-context-per-repository`, `a-person-accepts` and
  `names-not-numbers`.
- **guides/an-agent-proposes.md**: the last paragraph's open question
  becomes the rule, citing record `a-person-accepts`.
- **guides/a-change-breaks-a-claim.md**: cites record
  `the-gate-blocks-regressions` for the block and the replacement that
  waits, record `a-person-accepts` for who accepts it.
- **guides/importing-adrs.md**: the import prints each record's name and
  alias; the table's number row says the number is kept as the record's
  alias, legacy, resolving citations written before the import and shown
  on the rendered record, while a new citation names the record; the
  "superseded by 0003" row resolves through the alias; the first sync
  writes each file under its name; the new decision after the import is
  the agent's proposal and Ada's accept, written to
  `docs/adr/money-has-currency.md`, never numbered after the last; the
  scenario and the import's authorship link the statement's questions;
  cites record `names-not-numbers`.
- **hero.ts**: the decision records gain `money-has-currency.md`, in the
  drawing and its description; `docs/diagrams/boom.svg` is drawn again
  from it.
- **README.md**: the table gains `design/requirements.md`, the
  statement, and says the records in `docs/adr/` are each named by its
  name; the "What it does" line on acceptance follows record
  `a-person-accepts`.

## Risks / Trade-offs

- [A name chosen on one branch is chosen on another] → `root:records`
  and `root:statement` fail at the merge's candidate, naming both, and
  the branch landing second renames its own record or item before it
  lands; nothing landed is renamed. Two records of one name are two
  files of one path, which git refuses to merge unless they are the
  same.
- [A reader cites the number the site shows] → each page says how to
  cite what it shows, by name; `root:records` and `root:statement` fail
  a citation by number; and the number shown never moves, so a reader
  who wrote it down finds the same thing later.
- [The numbers shown need git's history] → `design-pages` fails on a
  shallow or missing history rather than guess, and the pages workflow
  fetches the whole history. A local build that replays `site:build`
  after main moved may show an older number until the design pages
  change; nothing cites a number, and the published build runs fresh.
- [The citation check reads an ordinary phrase as a citation] → a name
  here is two or more hyphened words, a citation names its kind
  (`record`, `rule`, `§`), and code spans are quoted, not read; a page
  writes boom's example claims in code spans.
- [A word of the private sources reaches the repository] → the records
  and the statement are written from this design alone, which carries
  only the open part, and the review reads every new file against rule
  `nothing-commercial`. No denylist is added: one in the tree would name
  what it forbids.
- [A clause states more than a record decided] → each clause cites its
  record; what no record decides goes to the questions, never a clause.
- [Record `a-person-accepts` leaves a person working alone unable to
  accept what they write themselves] → the record says so among its
  consequences, and the walkthrough shows the agent proposing.
- [Generated pages under `src/content/docs` could be read by a check as
  written ones] → git ignores them, `site:prose` leaves them out by
  glob, and `root:records` reads tracked files only; `design-pages`
  empties its directory before it writes.

## Open Questions

None for this change. The design's own open questions are the
statement's questions.
