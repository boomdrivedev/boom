# Design

## Context

See proposal.md for why. The repository holds one decision record,
`docs/adr/0001-…`, in the form `adrs.toml` sets (MADR, the bare
variant): frontmatter giving `number`, `title`, `status`, `date` and
`decision-makers`; an H1 repeating the title; `Status:`, `Deciders:` and
`Sources:` lines; then `## Context and problem statement`,
`## Considered options`, `## Decision outcome` and `### Consequences`.
`check-adrs.sh` (`root:adrs`) holds every record to a number of its own.

The site (`site/`, Astro with Starlight) has nine pages and four
walkthroughs that sketch boom; its build checks every internal link,
anchors included: `starlight-links-validator` 0.27.0 counts any element
with an `id` as a target, not headings alone. Its diagrams are TypeScript
under `site/src/diagrams/`, drawn at build time and checked by
`site:test` (`scripts/diagrams.ts --check`). Its prose is checked by
Vale (`site:prose`, `vale src/content/docs`).

The design this change writes down comes from the design session of
2026-10-31 on claims, verdicts and reconciliation, its rulings (on how a
cell keeps its status, and Q1 to Q6 below), and decisions 39 to 42 of
the design's log, accepted on 2026-11-01, with the user's ruling of
2026-11-01 on millpond (request `req_ee1f3b7c`). Those sources are kept
outside this repository and carry more than the open design. This
design carries only the open part, so the code stage writes from this
file alone (core.4).

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

And the decisions of 2026-11-01, in open terms:

| decision | what it decides |
|---|---|
| 39 | verdicts are boom records, kept beside the claims |
| 40 | boom reconciles per repository: for one repository at a commit it reports which claims in scope have no current verdict, an agent on a host judges them with `boom judge`, it works with no other service, and a planner reads the judged backlog |
| 41 | a person whose role carries the permission accepts a record; never an agent, never the record's author |
| 42 | a proposal awaiting a person shows as a millpond signal where no other planner is connected |

Rules this design relies on: core.1 and core.2, which the records
restate for boom itself; core.4, which bounds every word written here;
core.5, for the new shell scripts (shellcheck and shfmt own `*.sh`), the
fixtures (`*.md`, excluded as prose) and the generator (Biome owns
`site/*.ts`); and core.6, since every tool the new tasks run (sh, awk,
bun, Vale) is one the tree already pins. No record here changes a rule
`core.md` gives. The rules the records imply, among them that an agent
never accepts and that a verdict is appended, are drawn into `core.md`
in a separate step, as the user decided on 2026-11-01, so this change
amends no rule.

## Goals / Non-Goals

**Goals:**

- Every decision of boom's design that the user has made is a record a
  reader can open here, and every invariant it implies is a numbered
  clause a later change can cite.
- A site page states nothing about the design that it cannot link to a
  clause or a record, and the build proves each link.
- Nothing commercial reaches the repository: no pricing, tier, customer
  or hosted service (core.4). Where the sources go past the open design,
  this design says what boom offers any caller, its commands and their
  JSON, and stops.

**Non-Goals:**

- `docs/architecture/core.md`: drawn from these records afterwards.
- A record for anything the user has not decided: signing, sync's
  rolling pull request, rendering and import stay proposed on the
  roadmap, and the statement describes them only where a decided record
  leans on them.
- Vale over the records and the statement: they are read by people and
  reviewed; the gate's prose check stays on the site's written pages.
- Any change in millpond, flume or sluice.

## Decisions

### D1. Nine records in record 0001's form

One record per decision of the brief, numbered 0002 to 0010 in the
order below, each with `status: accepted`, `date: 2026-11-01` and
`decision-makers: [Chuck Swanberg]`, a `Status: accepted by the user on
2026-11-01.` line, a `Deciders:` line naming the dates of the rulings it
rests on, and a `Sources:` line citing those rulings by their words and
label and the decisions by number, never by a path to a place the
reader cannot open. "The session" in the paragraphs below is written
out in a record as "the design session of 2026-10-31 on claims, verdicts
and reconciliation", and a decision as "decision 40 of the design's
log". A title has no colon followed by a space, so the
frontmatter stays a plain YAML scalar, and the filename is the title in
lower case with hyphens, as 0001's is. A record names the records it
leans on as `record 0004`. The records cite no clause: they are the
decisions, and the statement is drawn from them.

Set aside: one record for the whole design, which a citation could not
narrow to the decision it rests on; records citing the sources by path,
which the user's decision of 2026-11-01 and core.4 rule out.

### D2. The product's design is the statement, not OpenSpec specs

`openspec/specs` holds what this repository's checks prove, and boom is
not built. The design of the product is the statement,
`design/requirements.md`, which the family's requirements style writes
for a model not yet designed. A later change that builds a part of boom
writes that part's specs, citing the clauses they satisfy. This change's
own specs, `design-statement` and `design-pages`, are the checks that
hold the statement and the site to their citations.

### D3. The statement in sluice's and millpond's shape

`design/requirements.md`, titled `# boom — requirements`, in the order
the style sets and sluice's statement follows: a preamble giving the
mechanism rule, then `## 1. Purpose`, `## 2. Actors`,
`## 3. Vocabulary`, `## Part A. What must hold of boom`,
`## Part B. Contracts across the seams`, `## Part C. Profiles`,
`## 7. Global invariants`, `## 8. Non-goals`,
`## 9. Environment givens`, `## 10. Questions the model must answer`,
`## 11. Scenarios the model must satisfy` and `## 12. What to deliver`.
Parts A and B name no command, file, format, field, service or tool;
the one thing of the world they name freely is git, which section 9
fixes. Real tools appear only in section 9 and Part C. Clauses are
numbered in one sequence across the parts, from 1 in reading order in
this first draft; from then on a number never moves and a withdrawn
number is never reused. A clause cites earlier clauses by bare number
in parentheses and the records it rests on as `(record 0004)`, both in
one parenthesis where both apply: `(12, record 0004)`.

The statement's content is the section "The statement" below.

### D4. `root:statement` and its cases

`check-statement.sh`, POSIX sh and awk like `check-adrs.sh`, takes the
statement's path (default `design/requirements.md`) and reads it whole,
joining lines, so a citation that crosses a line break is read as one.
A clause is a line between the `## Part A` heading and the next `## `
heading that is not a part's, opening with a number, an optional lower
case letter and `. `; an invariant is a list item opening `**I<n>.**`;
a scenario one opening `**S<n>.**`. A citation is a parenthesis holding
only items separated by `, `, each a clause number (`12`, `93a`) or
`record` and four digits. It fails, naming the number and each line,
where two items of a kind share a number, and where a citation names a
clause the file lacks or a record `docs/adr` lacks; otherwise it prints
the counts. Rule ids such as `core.1` are not citations it reads.

`check-statement.test.sh` runs it over `tests/statement/holds.md`,
`shared-number.md` and `unresolved.md`, asserting each exit status and
that the error names the number the fixture breaks. The root's
`moon.yml` gains `statement` (inputs `/design/requirements.md`,
`/docs/adr/*.md`, `/check-statement.sh`) and `test` (inputs
`/check-statement.sh`, `/check-statement.test.sh`, `/tests/statement/*`,
`/docs/adr/*.md`), both `check-fast`, run from the workspace root.

Set aside: a binary in `crates/boom`, which is boom itself and not a
check of its docs; a TypeScript check in the site, since the statement
is the root's file and holds for readers of the repository whether or
not the site is built.

### D5. The design pages, computed at build

`site/scripts/design.ts`, run by bun, writes
`site/src/content/docs/design/statement.md` from the statement and
`site/src/content/docs/design/decisions/<number>.md` from each record,
emptying that directory first. `.gitignore` names
`/site/src/content/docs/design/`; nothing commits it (core.2's rule for
boom's views holds for these too: written to a build directory, never
read back as input). The site's `moon.yml` gains `design-pages`, with no
layer and `cache: false` like `install`; `typecheck` and `build` depend
on it and take `/design/requirements.md` and `/docs/adr/*.md` as
inputs; `prose` runs `vale --glob='!**/design/**' src/content/docs`.
`package.json`'s `dev` runs the script before `astro dev`.

- **The statement's page**, titled "The statement", with the H1
  dropped. A Markdown ordered list renumbers its items from its first,
  and clause numbers are identifiers, not an order, so the script writes
  each clause as a paragraph opening with an empty element whose `id` is
  the clause's number and the number in bold, its continuation lines
  joined; invariants and scenarios likewise, with `I1` and `S1` as ids.
  Each citation `(N, …)` in the page links each number to `#N`, and each
  `record NNNN` to that record's page.
- **Each record's page**, titled `NNNN: <title>`, its description
  `<status>, <date>`, its sidebar order its number, its `editUrl` the
  record's file on GitHub, its H1 dropped and each `record NNNN` and each
  link to another record's file rewritten to that record's page. A
  record whose frontmatter lacks its number, title, status or date fails
  the script, naming the file.
- **The sidebar** gains a group "Design", after "boom check", holding
  "The statement" and "Decisions", the latter generated from
  `design/decisions`.

Set aside: links to the files on GitHub, where no clause has an anchor
and nothing checks a citation; a content loader composing Starlight's,
whose pages the link checker's pass would have to be shown to reach,
where writing files first is the way sluice's site already works.

### D6. How a page cites

A page cites at the end of the sentence, row or item that states what
it cites: `([clause 12](/boom/design/statement/#12))`, several as
`(clauses [31](…), [32](…); [record 0004](/boom/design/decisions/0004/))`,
an invariant as `[I3](/boom/design/statement/#I3)`. A rule stays plain
text, `core.1`. A page that only shows a command's output, the
commands page's examples, cites once per command, in its prose. The
roadmap links record 0001 to its page rather than to GitHub.

### D7. The walkthroughs follow record 0008

Record 0008 rules out a person accepting a version they proposed. Every
walkthrough where Ada proposes and accepts changes so that her coding
agent proposes, with the agent's session as provenance, and Ada accepts.
`first-claims` shows once what boom says when she runs `boom accept` on
a claim she proposed herself: refused, since she is its author, naming
who may accept it. `importing-adrs` keeps the import, whose authorship
the statement leaves as a question, and has the agent propose the new
decision that follows it.

### D8. The planner setting lives in the ledger's settings

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

### D9. A backlog entry's key names its cell and its verdict

Each entry of `boom backlog --json` carries a key, the same on every
host and every run, naming the repository, the claim and the verdict
that put the cell in the backlog: `shop/invoices-final/7c1e0a4`, the
last part the verdict's id. A caller takes each entry once. A cell
judged again, not satisfied still, is a new entry with a new key, which
millpond's curation may attach to the item it already sent; a cell
judged satisfied is in no entry.

### D10. The millpond loop as a diagram

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

Each paragraph is a record's substance: its title, the question its
context asks, the options it weighs, the outcome and its consequences,
and its sources. The record says it in the voice of record 0001 and
says nothing the paragraph does not.

**0002, The ledger holds claims, narratives, contexts, relationships and
terms.** What does boom record? Options: a store per form (ADR files, a
rules file, a map, a glossary); one ledger of five kinds of record, every
other form a view; one kind, the claim, with narratives, the map and
terms as claims. Outcome: five kinds. A **claim** has a name, a kind
(invariant, decision or assumption), a statement, at least one scenario
saying how one would know it holds, for a decision a body (context,
drivers, options, outcome, consequences, written when it is proposed),
what it attaches to, an optional check rule, the claim it supersedes,
and provenance. A **narrative** is an explanation in its author's words
at the time, scoped to a context, including claims by name; a person
judges it, and it is never given a verdict. A **context** is a part of
the system with its own model and language: a name, its
responsibilities, and its home, a repository optionally narrowed by
paths. A **relationship** names two contexts, which is upstream, its
pattern (customer and supplier, conformist and the other fixed
patterns), and the direction a dependency may run. A **term** is a word
as one context means it. Each record is a name that stays and versions;
a version is the hash of what it says, for a claim its statement,
scenarios and body; what it attaches to and its check rule sit outside
the hash. A claim is written at the destination's grain: it survives a
rewrite of the code, names what someone could observe, and can be
judged from the repository alone; it never names a file, a colour, a
setting or a step of a pipeline, which is a check rule's business. The
one kind not versioned, the verdict, is record 0004's. The third option
falls because a verdict on a narrative is a category error and a term
needs its own proposal and acceptance. Consequences: an ADR is a
decision claim; rules, decision records, the design book and the map
are views (core.2); a decision's reasoning is never written after the
fact; an imported MADR file has no scenario, which the statement asks
about. Sources: decision 18 (a claim is any assertion or decision about
the system), decision 26 (reasoning is written when a decision is
made), decision 27 (the design book is a view of narratives and claims),
decision 28 (the map is records), the session's table of records.

**0003, A status is read from seven events, and a claim changes by
amendment or supersession.** How does a record move from proposed to
standing and beyond, and where is its status kept? Options: a status
field, edited; events appended, the status read from them; a status
given by where a file sits. Outcome: seven events, appended, never
changed or removed (core.1), each naming the record and version it acts
on, its actor (a person or an agent) and its provenance (a session, a
run, a transcript, a signal). `proposed`: a new record, or a new version
of a standing one; visible, never checked, never judged. `accepted`: by
a person (record 0008); the version stands, and the earlier one it
amends stays in the history. `rejected`: by a person; the proposed
version is closed. `superseded`: written when a person accepts a claim
naming one it replaces; the old claim no longer stands. `annotated`: a
note on one version; no status changes. `challenged`: names a claim and
the signal that bears on it; it stales the claim's verdicts in every
repository, and an assumption goes back to a person; written by any
caller through boom's command line (record 0009). `due`: written by boom
when it reconciles a repository (record 0005); it names one verdict and
stales that cell alone. A standing claim changes by amendment, a new
version of the same name whose cells are judged again, or by
supersession, a new name naming the one it replaces, whose cells start
unjudged as the old one's leave. "After" is order in the ledger's
history, which is one line, and no status depends on a clock. The edited
field falls because edits meet in conflicts and lose who agreed; the
file's place because acceptance would be a side effect of moving a file
and not a person's act. Consequences: a status is computed every time
it is asked; two people adding events on two branches rarely conflict;
the history is the record of who agreed to what; withdrawing a claim
with nothing in its place, and moving what a claim attaches to, are not
among the seven, and the statement asks how they are recorded. Sources:
decision 32 (status is derived from events); ruling Q2; the session's
table of events.

**0004, A verdict is a record beside the claims, and a cell's status and
its two queues are computed.** Where does a repository's standing
against a claim live, and how is it kept current with nothing running?
Options: a status per cell, kept by a process that runs a state machine
for each; verdicts appended as boom records beside the claims, a cell's
status a function of the records, the queues computed; no verdicts, the
gate's checks the only reading. Outcome: the second. A **cell** is one
standing claim and one repository in its scope (record 0007), derived
and with no record until judged; it is never a change, a revision or a
piece of work, and work that cites no claim has none. A **verdict**
names the claim, the version judged, the repository, the revision
judged, a value (satisfied, partial, not satisfied, not applicable), the
evidence at that revision, a few words, and who judged it (the session
and its model), and is appended, never edited, in the ledger beside the
claims. A verdict is a judgment stored with its inputs, never a
computation; a marker naming the claim at its version is evidence the
judge is handed, never the verdict. The status, in order: the claim does
not stand, no cell; the repository homes nothing it attaches to, out of
scope, its verdict reading not applicable and kept; no verdict,
unjudged; the latest verdict judged an earlier version, stale; a
`challenged` event after it, stale; a `due` event naming it, stale;
otherwise current, with its value. A stale cell lists every reason that
holds. The queues: **to judge**, every cell unjudged or stale; the
**backlog**, every current cell judged partial or not satisfied. Neither
is stored. Nothing else moves a verdict: not a tag, a kind, a term, a
narrative, a path inside a home, a check rule or a check's result; a
repository changing does not by itself stale one. An assumption is
about the world, not a repository, and has no cells. Anyone holding the
ledger computes the same status, since the function reads boom's
records alone. The first option falls on the status ruling: nothing
runs between commands, and a stored status is core.1's rule broken; the
third because the gate asks another question (record 0006), and without
verdicts there is no backlog. Consequences: `boom judge` takes the queue
to judge and `boom backlog` lists the backlog; a planner reads the
backlog and nothing else (records 0005, 0009). Sources: decision 39; the
status ruling; the session's status function and queues.

**0005, boom reconciles and judges each repository at a commit, on a
host, with no other service.** Where and when are verdicts judged and
kept current, and who judges? Options: inside a planner's run, judging
and planning together; boom on a host where the repository is checked
out, reconciling and judging, a planner reading the judged backlog; in a
change's own place before it lands. Outcome: the second, at shared heads
only. `boom reconcile`, for one repository at its shared head, writes a
`due` event for a current verdict whose cited evidence is gone, or
whose claim a landing since cites (a `Claim:` trailer or a new marker),
and nothing else. `boom judge` hands a model session, for each cell to
judge, closed inputs: the claim at its standing version with its
scenarios and body, the narrative that includes it, the cell's previous
verdict and its evidence, whether the shared head carries a marker
naming the claim and at which version, and the repository at its shared
head, read-only; nothing else reaches it. It writes the one verdict per
cell the session returns, checked against the verdict's form; anything
else is refused and the cell stays to judge. Never a verdict for a
proposed claim, never one from a person (a person answers an unclear
check, record 0006, and never asserts that work is done), never one on
a revision that has not landed. Both commands ask the status first, so
either run again with nothing changed writes nothing, and any trigger
may run them: by hand, a hook after each merge, a schedule, a caller's
command; a trigger makes it sooner, a schedule makes a quiet repository
converge. A repository first homed is judged once, as a baseline. The
ledger stays complete with no other service. The planner's run falls
because the ledger would be complete only while something plans, and a
planner holds no code to judge; the change's place because a verdict
written there stands for code that never lands when the change is
dropped. Consequences: a landing that cites a claim makes its cell due,
so work closes its own loop (record 0009); which model judges is a
question the statement keeps. Sources: decision 40; rulings Q1 and Q3.

**0006, boom check reads a change in three layers, and the gate blocks
regressions, never the backlog.** What does the merge gate ask of a
change, and from what? Options: whether the repository satisfies every
claim in scope after the change; whether the change moves the
repository away from a standing claim it touches, in three layers,
cheapest first; a model reviewing every claim for every change.
Outcome: the second. **Scope**: the changed files, mapped to the
contexts homed there, keep the standing claims attached to them and to
the whole system. **Rules**: a claim's check rule passes or fails with
no judgment; the map gives one rule for nothing, a new dependency
between contexts that no relationship allows fails; so does a marker
naming no standing claim. **Prose**: a reviewer model gets the diff and
the claims still to settle, and nothing else, and returns holds,
violated or unclear for each, with a few words. A claim the repository
does not yet satisfy is the backlog's, and a change that leaves it alone
holds it. The check reads its claims from the slice committed in the
repository (Q4): the standing claims in scope, with their versions,
check rules and the evidence paths of their current verdicts, so
everything it reads is in the change's own tree and a gate that caches
by content replays an unchanged check; a claim is checked there once its
sync lands. Violated blocks the merge: the agent fixes the code, or
proposes an amendment or a claim to replace it, which waits on a person
(record 0008). Unclear fails as unclear until a person answers for that
change, `boom check --answer` (Q5); the answer is that change's alone
and never a verdict. A check is not a verdict: one that holds writes
none, and one that fails moves none, its change never having landed.
boom calls its result a check and keeps "verdict" for the ledger; in a
gate's reading of all a change's checks, `boom check` is one row, pass
or fail. A prose claim checked often is a candidate for a rule, which
changes how it is checked and never its version. The first option falls
because every merge would wait on every claim nobody has built; the
third on cost and on certainty where a rule can say it. Sources: rulings
Q4 and Q5; the session's section on the gate; boom's first sketch of
the check, which this record carries.

**0007, A claim's scope is derived, and each repository is one context
until someone models the system.** How does a claim reach the
repositories it binds, and must a project model its system first?
Options: a claim names its repositories or paths; a claim attaches to a
context, a relationship or the whole system, its scope the repositories
homing them, each repository one context until a map is drawn; a map
before any claim. Outcome: the second. Scope is derived, never chosen: a
context gives its home; a relationship both ends' homes, each repository
its own cell; the system every repository the map homes anything in.
Only three things move a cell through scope: a home moves, an
attachment moves, or the claim's version moves. A home may name paths in
its repository; `boom check` reads them to match a diff to claims
(record 0006), and scope never does, so moving a path stales nothing.
Until someone draws the map, each repository is one context, homed in
itself, and a new claim attaches to the context of the repository it was
written for. Drawing the map changes no verdict: a claim keeps its
repository's context until someone attaches it elsewhere, and that
context lasts while any claim is attached to it. Contexts and
relationships are records, proposed and accepted like claims. No second
map of what is built is kept (Q6): the accepted records are the target,
and what is built is read from the verdicts. The first option falls
because a claim would stale whenever code moves; the third because a
person on one repository would have to model a system before writing
one claim. Sources: decision 28; ruling Q6; the session's section on the
map.

**0008, A person whose role allows it accepts, never an agent and never
the record's author.** Who may make a proposed version stand? Options:
anyone who runs `boom accept`; any person, never an agent; a person
whose role carries the permission, never an agent, never the version's
author. Outcome: the third. Roles, and who holds each, are the ledger's
settings, set by its keeper. An accept from an agent, from the actor of
the version's `proposed` event, or from a person whose role does not
carry the permission is refused, naming why, and records nothing; the
accept that supersedes a claim is held to the same. Rejecting is a
person's, never an agent's. An agent proposes, amends and supersedes by
proposal, and never gets a change through on its own (record 0006). The
first option falls because an agent could rewrite what the project
claims to get its own work through; the second because an author's own
accept is one person's word, not an agreement. Consequences: a person
working alone accepts what an agent proposed, and a version they
proposed waits on another person; `boom mcp`, which serves an agent,
offers no accept and no reject; how boom tells a person's act from an
agent's, and how it reads a role, the statement asks. Sources: decision
41.

**0009, millpond and boom work both ways, and millpond takes in a
backlog only where no other planner plans it.** How does what millpond
hears reach boom's claims, and how does boom's backlog reach work?
Options: one way, millpond proposing and challenging and the backlog
left to a person; both ways, millpond taking in every repository's
backlog; both ways, millpond taking in a repository's backlog only where
no other planner plans it, the repository's configuration saying which.
Outcome: the third, the user's ruling of 2026-11-01 (request
`req_ee1f3b7c`). (a) A millpond signal that bears on a claim reaches
boom as a `challenged` event, `boom challenge <name> --signal <key>`, or
as a proposed claim, `boom propose` with the signal as its provenance;
the challenged claim's cells are judged again (record 0004), and an
assumption goes to a person. (b) millpond takes `boom backlog --json` in
as items, each entry once by its key; an item's send starts the work
through flume, the work's commits cite the claim at its version, the
landing makes the cell due (record 0005), and `boom judge` judges it
again. The loop runs claim, judged not satisfied, item, work, landed,
judged satisfied, and the entry leaves the backlog. (c) Where a
repository's configuration names another planner, that planner reads the
backlog itself and millpond takes none of it in, so nothing is planned
twice. Tools never read each other's files: each calls the other's
command line. boom never plans, starts or lands work, and a planner
never writes a verdict. The first option falls because a backlog nobody
takes in waits on a person to notice it; the second because a
repository another planner plans would be planned twice. Consequences:
the `planner` setting per repository; `boom backlog --planner`; a
challenge naming the same signal twice adds nothing. Sources: the
ruling of 2026-11-01; decision 40 (a planner reads the judged backlog);
the design session of 2026-10-31, which brings a challenge into boom as
an event because no tool reads another's files.

**0010, A proposal awaiting a person shows as a millpond signal where no
other planner is connected.** Where does a person see what waits on
them? Options: only by asking boom, `boom query --status proposed`; as
millpond signals for every repository; as millpond signals where no
other planner is connected to the repository, the planner that is
connected showing them where one is. Outcome: the third (decision 42).
millpond lists the versions awaiting a person in the repositories whose
configuration names it, each once by its record and version, as a
signal; where the configuration names another planner, that planner
shows them and millpond carries none. The same setting decides this and
record 0009's (c). Showing a proposal anywhere accepts nothing: only a
person's accept does (record 0008), and a version accepted or rejected
leaves the listing. The first option falls because a proposal nobody is
shown waits unseen; the second because a person would be asked twice.
Sources: decision 42; the ruling of 2026-11-01.

## The statement

What each part holds, in substance; each line is one clause or a few,
written to the style's rules (one invariant each, present tense, the
strong words, checkable, no mechanism in Parts A and B) and citing the
record behind it.

**Preamble and purpose.** A statement of what boom must always do and
never do; the three parts; the mechanism rule (D3). boom keeps a ledger
of what a project claims about itself, judges each repository against
the claims that stand, computes every view and queue from the ledger,
and reads each change against the claims it touches; it never plans,
starts or lands work.

**Actors**, may decide and may not: **a person** (accepts and rejects
where their role allows, answers an unclear check; may not accept their
own proposal or assert a verdict); **an agent** (proposes, annotates,
proposes amendments and supersessions, runs the check; may not accept
or reject); **the judge**, a model session boom starts (a cell's value,
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
accept, decide a verdict for the judge, plan, start or land work, or
read another tool's files).

**Vocabulary**: ledger, record, name, version, event, actor, provenance,
claim, kind, invariant, decision, assumption, statement (a claim's),
scenario, body, narrative, context, home, relationship, term, map,
attachment, scope, standing, proposed, amendment, supersession, signal,
challenge, cell, verdict, value, evidence, revision, shared head,
landing, citation, status (a record's and a cell's), stale, to judge,
backlog, reconcile, judge, change, check, layer, check rule, outcome,
answer, slice, view, role, keeper, planner, entry, key. Terms by
meaning; a model may add, never redefine.

**A.1 Records and versions** (record 0002): a name never changes and no
two records of a kind share one; a version is fixed by what a record
says alone, so the same text has the same version and new text a new
one; a claim's version covers its statement, scenarios and body and
never what it attaches to or its check rule; every claim has exactly one
kind and at least one scenario; a decision's reasoning is the reasoning
as proposed, and nothing added later is reasoning; a narrative includes
a claim by name, never by copy, and is never given a verdict; a term
belongs to exactly one context; a relationship names its two contexts,
the upstream one and the direction a dependency may run.

**A.2 Events and standing** (record 0003): a status is read from events
alone and stored nowhere; an event is appended and never changed or
removed; every event names one version of one record, its actor and its
provenance; order is the ledger's history, one line, and no status
depends on a clock; a proposed version is visible and never checked or
judged; a version stands only by an accept (A.3); an accepted amendment
stands in place of the version it amends, which stays in the history;
the accept of a claim that names one it replaces ends that one's
standing in the same act; an annotation changes no status; a challenge
names one claim and one signal, and a second naming the same pair adds
nothing; a due event names exactly one verdict, and no verdict is due
twice.

**A.3 Acceptance** (record 0008): only a person accepts, never an
agent; never the version's author; only a person whose role carries the
permission, roles being the keeper's settings; an accept that breaks
any of these is refused, says why and records nothing; only a person
rejects; showing a proposal anywhere accepts nothing (record 0010).

**A.4 Scope and the map** (record 0007): a claim never names its
repositories, and its scope is the homes of what it attaches to; a
context's scope is its home, a relationship's both ends' homes, the
system's every repository the map homes anything in; only a home
moving, an attachment moving or a version moving changes a claim's
cells; a path inside a home never moves scope; until a map is drawn,
each repository is one context homed in itself and a claim proposed for
it attaches to it; drawing the map changes no verdict; no map of what is
built is kept beside the target, and what is built is read from
verdicts.

**A.5 Cells and verdicts** (record 0004): a cell is exactly one standing
claim and one repository in its scope, never a change, a revision or a
piece of work; a cell has no record until judged; a verdict is appended
and never edited, and names the claim, the version judged, the
repository, the revision judged, its value, its evidence at that
revision, its words and its judge; its value is exactly one of
satisfied, partial, not satisfied and not applicable; a verdict is a
judgment and never computed, the mechanical facts being inputs; an
assumption has no cells; verdicts are kept in the ledger beside the
claims.

**A.6 Status and the queues** (record 0004): a cell's status is a
function of the ledger alone, so any holder of the ledger computes the
same; the order of that function, one clause per step as record 0004
gives it; a stale cell lists every reason that holds; the queue to judge
and the backlog are computed, never stored; what never moves a verdict;
a status changes only by a record appended, and nothing runs between
commands.

**A.7 Reconciling and judging** (record 0005): verdicts are judged at
shared heads only; a landing citing a claim makes its cell due; a
reconcile writes a due event only for a current verdict whose evidence
is gone at the shared head or whose claim a landing since cites; a judge
writes a verdict only for a cell to judge; either, repeated with nothing
changed, writes nothing; the judge's inputs are closed, and nothing else
reaches it; an output not of a verdict's form is refused and the cell
stays to judge; never a verdict for a proposed claim, from a person, or
on a revision that has not landed; a repository first homed is judged
once as a baseline; reconciling and judging need no other service.

**A.8 The check** (record 0006): it asks whether a change moves the
repository away from a standing claim the change touches, and never
whether the repository satisfies a claim; a claim the change leaves
alone holds; the layers run cheapest first, a claim an earlier layer
settles reaches no later one, and only the claims still to settle reach
the reviewer; it reads only what is in the change's own tree, so a claim
is checked once its sync lands; every claim it reads has one outcome,
holds, violated or unclear, with words; unclear fails until a person
answers for that change, and the answer is never a verdict; a check
writes no verdict; a violated claim is changed only by a proposal a
person accepts; a check rule changes how a claim is checked, never its
version.

**A.9 Views** (records 0002, 0003): every view is computed from the
ledger, and the same ledger gives the same view; no view is read back as
input; rendering arranges and never reasons; every part of the design
book names the record it shows, so a note on it reaches the record.

**A.10 What boom never does** (records 0005, 0009): it never plans,
starts or lands work, and never changes work that cites a claim; it
gives any caller the standing version of each claim, so work citing an
earlier one can be told; it reads no other tool's files, and every
exchange with another tool is a command and its output; it writes into a
repository it describes only by proposing a change there, which lands as
that repository's other work does.

**Part B, one section per seam**, each its operations in abstract names
and the guarantee: **B.1 the ledger's store** (append; read in order;
appends never lost or reordered, two appenders both land and neither
overwrites); **B.2 the repository** (its shared head; what a revision
holds; the landings since a revision, with what each cites; a change
against a base; what is read at a revision is that revision's); **B.3
the judge** (judge, from closed inputs, to one verdict or none; it reads
nothing else); **B.4 the reviewer** (review, a change and the claims to
settle, to an outcome and words per claim; an answer not of that form
reads unclear); **B.5 the actor** (who acted, a person or an agent, and
the roles they hold; an agent is never taken for a person); **B.6 the
planner** (backlog, for a planner, every entry of each repository whose
settings name it, each with its key, the claim at its standing version,
its scenarios, the verdict's value, evidence and words and why the cell
came back; proposals, for a planner, every version awaiting a person in
those repositories; a key is the same on every host and run and names
one cell and one verdict, and no repository's entries are given to two
planners) (records 0009, 0010); **B.7 signals** (challenge, a claim and
a signal; propose, a record and its provenance; standing, whether a
named claim stands; the same signal challenging the same claim twice
adds nothing) (record 0009); **B.8 the gate** (check, a candidate's tree
to pass or fail; everything it reads is in that tree) (record 0006);
**B.9 binding the contracts** (each seam bound by naming profiles of
Part C; a ledger binds one store and one actor profile; any number of
planners and signal sources; one conformance suite holds every profile).

**Part C, the profiles.** "Parts A and B do not change to admit a
profile." **C.1 the store**: a state repository of its own, in git,
recfiles, boom committing directly, a rejected push rebased and tried
again; its layout is a question. **C.2 the repository**: git; the shared
head is the default branch at the remote; a landing cites a claim by a
`Claim: <name>@<version>` trailer; a marker is the same words in an
OpenSpec requirement, a test, a doc comment or a spec file. **C.3 the
judge and the reviewer**: a model session boom starts with the closed
inputs, the model a setting (a question). **C.4 the actor**: not bound
yet; section 10 asks how. **C.5 the gate**: sluice running `boom check
--base main` as a moon task in a layer the branch requires, with
`.boom/slice.rec` an input, so moon replays an unchanged check; in
sluice's verdict, one row. **C.6 the planner**: millpond, `planner =
"millpond"`, reading `boom backlog --planner millpond --json` through an
adapter, each entry an offer under its key, curation making items with
briefs that cite the claim at its version, a send to a place started
through flume, the work landed by sluice, sluice's hook after each merge
running `boom reconcile`, and `boom judge` closing the loop; and another
planner, named by its own name, which reads the same command for its
repositories. **C.7 signals**: millpond's challenge move running `boom
challenge <name> --signal <key>`; a signal asserting something new
running `boom propose` with the signal's key as provenance; millpond's
"does this claim stand" read with `boom query <name> --json`; flume
proposing from a run with the run as provenance and asking `boom query
<paths> --json` what applies before work. **C.8 the surfaces**: the
command line, every command with `--json`; `boom mcp`, with no accept
and no reject; the Claude Code plugin; the herdr plugin. **C.9 the
renderer**: mdBook, run by command where installed, none of its code
embedded.

**Global invariants**, I1 onward, each citing its clauses: a status,
whether a record's or a cell's, is read from the ledger and never
stored; every view and queue is computed; no agent accepts and no author
accepts their own; the gate blocks only a change that moves away from a
standing claim; reconciling and judging, repeated on unchanged inputs,
write nothing; every holder of the ledger computes the same status for
every cell; boom and another tool exchange only commands and their
output; no repository's backlog is planned by two planners.

**Non-goals**: planning, ordering, starting or landing work; approving
work; running checks other than its own, or deciding which a repository
requires; hosting anything or serving a network; rendering a book itself;
keeping a map of what is built; judging a claim from anything but a
repository at its shared head.

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
(B.5's binding); how a claim is withdrawn with nothing in its place, and
how an attachment moves, by events of their own or by a proposal; whether
the map needs parts finer than a context, elements inside one and the
links between them; what a check rule looks like; which model judges and
which reviews, and how a project names them; where an answer to an
unclear check is kept, so the gate's next run of the same change reads
it; what keeps a change from editing the slice it is checked against; an
imported decision's scenario, and whose accept an import records; where
the ledger lives for a project of one repository; what a proposal whose
scope spans repositories naming different planners shows where; the
name.

**Scenarios**, S1 onward, as the operator lives them, each naming the
clauses it exercises: a person alone on one repository, with no map, has
an agent propose an invariant, accepts it, and after `boom judge` sees
it in the backlog; an agent's accept is refused; a person's accept of
their own proposal is refused; a change that leaves a claim nobody has
built alone passes the gate; a change that breaks a standing claim is
blocked, the agent proposes an amendment, and the gate holds the old
version until a person accepts; an unclear check fails until a person
answers it; forty commits that leave a verdict's evidence alone judge
nothing again; a landing that removes a verdict's evidence makes the
cell due, and the next judge judges it; an amendment makes every cell
of the claim stale, a supersession retires the old cells and starts the
new claim's unjudged; drawing a map changes no verdict; a millpond
signal challenges a claim and its cells are judged again, and a
challenged assumption goes to a person; the millpond loop from not
satisfied to satisfied; a repository whose configuration names another
planner has none of its backlog taken in by millpond; a proposal waits
as a millpond signal where millpond is the planner, and is not one where
another planner is; reconcile and judge run twice with nothing changed
write nothing.

**What to deliver**: a model of boom satisfying Parts A and B: its
records and events, the status function, the queues, the commands and
their output; a conformance suite whose scenarios cite the clauses they
exercise; the profiles of Part C, each passing it.

## The site

Each page, what changes on it and what it cites (D6). Every claim on a
page about the design links the clause or record behind it, the clause
numbers being those the statement holds once written.

- **index.mdx**: "Where it stands" says the repository holds its rules,
  its statement and its decision records, 0001 to 0010, linked, and a
  binary that prints its version; the cards' "Status read from events"
  and "Checked at the merge" cite their records; "Who it is for" says
  agents propose and a person whose role allows it accepts.
- **how-its-used.mdx**: the records table cites record 0002 and A.1's
  clauses; the events table gains `challenged` and `due` as decided,
  each row citing its clause, and the closing sentence on later events
  names only those the statement still asks about; the accepted row and
  a new paragraph say who may accept (record 0008, A.3); amend and
  supersede cite record 0003; "Verdicts" becomes cells, verdicts and the
  two queues, citing record 0004 and A.5, A.6, and where they are judged,
  citing record 0005 and A.7; "Contexts" cites record 0007 and A.4.
- **configuration.md**: `[repositories.shop]` gains `planner =
  "millpond"`, commented with records 0009 and 0010; a `[roles]` sketch
  says who may accept (record 0008); `events.rec`'s `%type` gains
  `challenged` and `due`, with one example of each; the verdict example
  gains its id and its judge; the caution keeps the format a sketch.
- **commands.md**: `boom judge`, `boom reconcile`, `boom backlog`,
  `boom challenge` and `boom annotate` join the table with their own
  sections, arguments, an example and its JSON, each citing its record
  and clauses; `boom backlog --json` shows entries with their keys (D9)
  and `--planner`; `boom query` gains `--planner`; `boom accept` says
  who may accept, citing record 0008, and shows the refusal of an
  author's accept as JSON with exit 2; `boom mcp` lists no accept and no
  reject; "Also in the design" goes, as nothing in it is left sketched
  less far.
- **check.mdx**: each section cites record 0006 and its A.8 clause: the
  three layers, what it reads, regressions not the backlog, the outcomes
  table, a check is not a verdict, and the two meanings of verdict.
- **integrations.mdx**: millpond's section becomes both ways: (a) a
  signal to `boom challenge` or `boom propose`, (b) the backlog taken in
  as items, the loop drawn as the `millpond` diagram (D10) with a
  caption naming each station, (c) another planner reading the backlog
  itself, the `planner` setting deciding, and proposals as signals where
  millpond is the planner; each cites records 0009, 0010 and C.6, C.7.
  `boom mcp`'s note on accept cites record 0008. sluice and flume cite
  C.5 and C.7.
- **roadmap.md**: "Settled" lists each record, 0001 to 0010, linked,
  with the clauses that state it, and keeps core.1 and core.2; "Leaning"
  and "Proposed" keep only what no record decided (writing to git
  directly, rendering, sync's pull request, import, the plugins,
  signing); "Open questions" become the statement's section 10, each
  linked, with the build order kept as the roadmap's own.
- **guides/first-claims.md**: D7; Ada's agent proposes, Ada accepts; the
  refusal of her own accept shown once; cites records 0007 and 0008.
- **guides/an-agent-proposes.md**: the last paragraph's open question
  becomes the rule, citing record 0008.
- **guides/a-change-breaks-a-claim.md**: cites record 0006 for the
  block and the replacement that waits, record 0008 for who accepts it.
- **guides/importing-adrs.md**: the new decision after the import is
  the agent's proposal and Ada's accept (D7); the scenario and the
  import's authorship link the statement's questions.
- **README.md**: the table gains `design/requirements.md`, the
  statement; the "What it does" line on acceptance follows record 0008.

## Risks / Trade-offs

- [The site cites clause numbers the statement assigns] → numbers never
  move once written (D3), and `site:build` fails on a citation that
  reaches no anchor.
- [A word of the private sources reaches the repository] → the records
  and the statement are written from this design alone, which carries
  only the open part, and the review reads every new file against
  core.4. No denylist is added: one in the tree would name what it
  forbids.
- [A clause states more than a record decided] → each clause cites its
  record; what no record decides goes to section 10 as a question, never
  a clause.
- [The anchors the generator writes break Markdown it does not expect]
  → the statement keeps the style's plain forms (numbered lines,
  invariant and scenario list items), and `site:build` proves every
  cited anchor.
- [Record 0008 leaves a person working alone unable to accept what they
  write themselves] → the record says so among its consequences, and the
  walkthrough shows the agent proposing.
- [Generated pages under `src/content/docs` could be read by a check as
  written ones] → git ignores them and `site:prose` leaves them out by
  glob; `design-pages` empties its directory before it writes.

## Open Questions

None for this change. The design's own open questions are the
statement's section 10.
