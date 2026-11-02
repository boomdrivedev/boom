# Proposal

## Why

boom's design was settled in the design session of 2026-10-31 on
claims, verdicts and reconciliation, with its rulings, in decisions 39
to 42 of the design's log, accepted on 2026-11-01, and in the user's
ruling of 2026-11-01 on millpond. The same day the user decided that
boom's open design lives here, in its own repository, as its own
records, and not as a pointer to a place a reader cannot open.

On 2026-11-01 the user also went through boom's constitution interview,
whose notes are `interview.md` on main. It settles how anything here is
known: in another repository of the family, a parallel branch landed
records 0002 to 0016 and rules core.12 to core.18 under the numbers an
interview's branch had taken, and 22 records and 75 rules were
renumbered twice, each citation rewritten. A well-designed boom makes
that impossible, for any project and for itself: a record is known by a
name fixed when it is proposed, and a number is given only from the
order of acceptance on a line never rewritten. The interview settles
much else besides: where the ledger lives and how it is written, how a
person's act is proven and who owns a ledger, what the check runs and
what it does when it cannot, the command line's exits and JSON, the
surfaces an agent meets, how `boom sync` hands a repository its files,
how an import is accepted, and what stays out of this repository. Its
section "What the change must take in" lists what this change carries.

The same day the user took three items of the awesome-lists research of
2026-11-01 into this design (request `req_ee5dd28f`): a rule under the
ledger, that each event carries an ordering key of its own, so the
appends two hosts make to one record, merged in either order, derive the
same status, as radicle-cob orders a collaborative object's changes; a
rule that no surface an agent calls accepts a record, `boom mcp`
offering reading and proposing and an accept only as a link a person
opens; and two design notes, `boom mcp` built on rmcp, the Rust MCP SDK,
and boom's rebase-and-retry push run by the host's own `git`, gitoxide
marking push and rebase not done.

None of it is in the repository: it holds one decision record and a site
whose pages sketch a design they cannot cite, and contradict it. The
first walkthrough has Ada accept the claims she proposed herself, which
decision 41 rules out; the roadmap lists as open what the interview
settles; the millpond section has one direction where the ruling has
two; the configuration page keeps the ledger's settings, the
repositories it describes, in a file any commit can edit.

## What Changes

- Names, not numbers, in this repository (record `names-not-numbers`,
  rule `core.names-not-numbers`): a decision record is
  `docs/adr/<name>.md`, its frontmatter naming it and holding no
  number; a new rule's id is `core.<name>`; each clause, invariant,
  question and scenario of the statement opens with its name and is
  anchored by it; a citation names what it cites. Record 0001 keeps its
  file and core.1 to core.6 their ids, as legacy, cited only as legacy.
  `root:adrs` holds records and rules to names no two share and every
  citation of one to something that exists, and `root:statement` does
  the same for the statement's items; nothing holds a number unique.
  `adrs.toml`, whose tool takes the next number on a branch, goes.
  `CLAUDE.md` sends a rule to `core.md` with a name.
- `docs/architecture/core.md` gains, each in the commit of the record
  behind it: `core.names-not-numbers`; `core.events-carry-their-order`,
  under the ledger; `core.no-commercial-strategy`, rewording core.4,
  which stays its legacy alias; `core.private-sources-by-words`;
  `core.callers-pin-boom`; `core.plugins-in-this-repository`; and
  `core.no-agent-surface-accepts`.
- Eighteen decision records, each accepted by the user and citing the
  rulings, decisions, interview topics and research it rests on:
  `names-not-numbers`, `no-commercial-strategy-here`, `five-records`,
  `seven-events`, `the-ledger-on-a-shared-branch`,
  `events-carry-their-order`, `cells-and-verdicts`,
  `one-context-per-repository`,
  `judged-on-a-host`, `the-gate-blocks-regressions`,
  `a-person-accepts`, `a-persons-act-is-signed`,
  `the-command-line-contract`, `the-surfaces`,
  `sync-by-one-pull-request`, `millpond-in-both-directions`,
  `proposals-as-signals` and `import-proposes-for-a-person`.
- A statement of invariants, `design/requirements.md`, in the family's
  requirements style, its sections named and its items each opening
  with a name, every requirement the interview names that is boom's own
  a clause of Part A under that name, Parts B and C carrying the
  interview's seams and profiles, the design notes `mcp-on-rmcp` and
  `writes-through-the-hosts-git` named clauses of Part C that the
  givens' stack cites, and its questions the nine still open,
  `open-signature-format`, `open-sync-without-pull-requests`,
  `open-after-landing-step` and `open-accept-link-opener` among them.
  Where the style gives a new clause the next free number, the
  statement says it departs and why.
- The site shows the statement and each record as pages computed at
  build and never committed, each item anchored by its name and showing
  a number derived from the order it landed on main ("Record 0002",
  "Clause 12"). Every page that describes the design cites its clauses
  and records by links the build checks, and each is brought in line
  with them: how it's used, configuration, commands, `boom check`, the
  walkthroughs, integrations, the roadmap and the start page. The
  integrations page draws the millpond loop as a new diagram.

Out of scope:

- Drawing the rest of the constitution from these records, the rules
  they imply (an agent never accepts, a verdict is appended), which is
  the separate step the user decided on.
- Building any of boom: the crate still prints its version.
- The family's requirements style and other repositories' rules, which
  keep their numbering until they change it themselves.

Depends on, outside this change (named in the design, built by each
tool's own change):

- flume: a launch field naming the claims a place's work serves, and
  `Claim: <name>@<version>` on every commit the place lands
  (`claims-travel-in-the-launch`).
- millpond: its send filling that field from a backlog entry, its
  adapter for `boom backlog --json`, and its challenge move calling
  `boom challenge`.
- sluice: a step that runs a repository's command after each landing,
  or a host's watcher doing it (`open-after-landing-step`).

## Capabilities

### New Capabilities

- `record-identity`: each decision record and each rule of this
  repository is known by a name no other holds, numbered ones only as
  the legacy the check names, and every citation of one in the tree
  resolves, checked by `root:adrs` in the fast layer.
- `design-statement`: the statement of invariants holds together, each
  clause, invariant, question and scenario known by a name no other
  item holds, nothing in it numbered, and every citation in it naming
  an item, checked by `root:statement` in the fast layer.
- `design-pages`: the site shows the statement and the decision records
  as pages computed from their files, each reached by its name, the
  numbers it shows derived from the order things landed on main, every
  citation of the design a link the build checks, and the integrations
  page drawing the millpond loop.

### Modified Capabilities

None.

## Impact

- New prose: eighteen records, the statement and seven rules. Each is
  reviewed by a person; no check judges what a record decides, and
  review holds `core.no-commercial-strategy` and
  `core.private-sources-by-words`, as the user decided.
- The root's `adrs` task reads every tracked file and holds names, not
  numbers; the root gains `statement` and `test`, all in `check-fast`,
  which sluice requires on `main`. `adrs.toml` goes.
- The citations already in the tree stay as written: core.1 to core.6
  and record 0001 are legacy ids and resolve. core.md's two citations
  of flume's rules move into code spans, so no check reads them as this
  repository's.
- The site gains a step that writes the design pages before its type
  check and build, reading main's history for the numbers it shows; its
  tests gain that step's cases; the pages workflow fetches the whole
  history and builds when a record or the statement changes.
- Every page that sketched what the interview settles changes to the
  settled answer: the ledger on a shared branch under `boom/`, its
  settings signed by an owner, each event carrying its own order, a
  person's accept signed with them present and offered to them as a
  link, exits 0 to 3, `boom sync` opening one ready pull request, `boom
  import adr` proposing for a person.

## Touches

```
docs/adr/names-not-numbers.md
docs/adr/no-commercial-strategy-here.md
docs/adr/five-records.md
docs/adr/seven-events.md
docs/adr/the-ledger-on-a-shared-branch.md
docs/adr/events-carry-their-order.md
docs/adr/cells-and-verdicts.md
docs/adr/one-context-per-repository.md
docs/adr/judged-on-a-host.md
docs/adr/the-gate-blocks-regressions.md
docs/adr/a-person-accepts.md
docs/adr/a-persons-act-is-signed.md
docs/adr/the-command-line-contract.md
docs/adr/the-surfaces.md
docs/adr/sync-by-one-pull-request.md
docs/adr/millpond-in-both-directions.md
docs/adr/proposals-as-signals.md
docs/adr/import-proposes-for-a-person.md
docs/architecture/core.md
design/requirements.md
adrs.toml
check-adrs.sh
check-adrs.test.sh
tests/adrs/
check-statement.sh
check-statement.test.sh
tests/statement/
moon.yml
.moon/workspace.yml
CLAUDE.md
openspec/config.yaml
.github/workflows/pages.yml
.gitignore
README.md
site/moon.yml
site/package.json
site/astro.config.mjs
site/README.md
site/scripts/design.ts
site/scripts/design.test.ts
site/src/diagrams/millpond.ts
site/src/diagrams/index.ts
site/src/diagrams/fits.ts
site/src/content/docs/index.mdx
site/src/content/docs/how-its-used.mdx
site/src/content/docs/configuration.md
site/src/content/docs/commands.md
site/src/content/docs/check.mdx
site/src/content/docs/integrations.mdx
site/src/content/docs/roadmap.md
site/src/content/docs/guides/first-claims.md
site/src/content/docs/guides/an-agent-proposes.md
site/src/content/docs/guides/a-change-breaks-a-claim.md
site/src/content/docs/guides/importing-adrs.md
styles/config/vocabularies/Boom/accept.txt
```
