# Proposal

## Why

boom's design was settled in a design session on 2026-10-31, with its
rulings, and in four decisions the user accepted on 2026-11-01, numbered
39 to 42 in the design's log. None of it is in this repository: it holds
one decision record, 0001, and a site whose pages sketch a design they
cannot cite. The user decided on 2026-11-01 that boom's open design lives
here, in its own repository, as its own records, and not as a pointer to
a place a reader cannot open.

The site already describes parts of that design and contradicts others.
Its first walkthrough has Ada accept the claims she proposed herself,
which decision 41 rules out; its roadmap lists as open who accepts,
which decision 41 settles; its millpond section has one direction where
the user's ruling of 2026-11-01 has two. A reader cannot tell which page
states a decision and which a guess.

## What Changes

- Nine decision records, `docs/adr/0002` to `0010`, MADR in record
  0001's form, each accepted by the user and citing the rulings and the
  decision numbers it rests on: the five records; the seven events and a
  status read from them; cells, verdicts beside the claims and the two
  computed queues; reconciling and judging per repository at a commit,
  on a host, with no other service; `boom check`'s three layers and a
  gate that blocks regressions, never the backlog; scope and one context
  per repository until someone models the system; who may accept;
  millpond both ways; and proposals awaiting a person as millpond
  signals where no other planner is connected.
- A statement of invariants, `design/requirements.md`, in the family's
  requirements style: numbered clauses in three parts (what must hold of
  boom, its contracts across its seams, and the profiles that bind
  them), global invariants, non-goals, givens, questions, scenarios and
  what to deliver. Its clauses cite the records they rest on.
- A check that the statement holds together, `root:statement`: each
  clause, invariant and scenario has a number no other has, and every
  citation in it names a clause or a record that exists. Its cases run
  as `root:test`.
- The site shows the statement and each record as pages, computed from
  the files at build time and never committed, each clause with an
  anchor named for its number. Every page that describes the design
  cites its clauses and records by links, which the build already checks.
- The site's pages are brought in line with the records: how it's used,
  configuration, commands, `boom check`, the walkthroughs, integrations,
  the roadmap and the start page. The integrations page carries the
  millpond loop, claim to verdict to item to work to landing to verdict,
  as a new diagram.

Out of scope:

- The constitution, `docs/architecture/core.md`: the user decided it is
  drawn from these records afterwards, as a separate step. No rule
  changes here.
- Building any of boom: the crate still prints its version.
- millpond's side of the loop, its adapter for boom's backlog and its
  challenge move calling boom, which is millpond's own work.

## Capabilities

### New Capabilities

- `design-statement`: the statement of invariants holds together, each
  number held by one clause, invariant or scenario, and every citation in
  it naming a clause or a record that exists, checked in the fast layer.
- `design-pages`: the site shows the statement and the decision records
  as pages computed from their files, each clause reachable by its
  number, every citation of the design a link the build checks, and the
  integrations page drawing the millpond loop.

### Modified Capabilities

None.

## Impact

- New prose: nine records and the statement. Each is reviewed by a
  person; no check judges what a record decides.
- The root project gains `statement` and `test`, both in `check-fast`,
  which sluice requires on `main`.
- The site gains a step that writes the design pages before its type
  check and build, two inputs on those tasks, a sidebar group, a
  diagram, and a glob that keeps the generated pages out of its prose
  check.
- Every page that sketched the open question of who accepts, the
  walkthroughs among them, changes to the decided answer: an agent
  proposes and a person accepts, never their own proposal.

## Touches

```
docs/adr/0002-the-ledger-holds-claims-narratives-contexts-relationships-and-terms.md
docs/adr/0003-a-status-is-read-from-seven-events-and-a-claim-changes-by-amendment-or-supersession.md
docs/adr/0004-a-verdict-is-a-record-beside-the-claims-and-a-cells-status-and-its-two-queues-are-computed.md
docs/adr/0005-boom-reconciles-and-judges-each-repository-at-a-commit-on-a-host-with-no-other-service.md
docs/adr/0006-boom-check-reads-a-change-in-three-layers-and-the-gate-blocks-regressions-never-the-backlog.md
docs/adr/0007-a-claims-scope-is-derived-and-each-repository-is-one-context-until-someone-models-the-system.md
docs/adr/0008-a-person-whose-role-allows-it-accepts-never-an-agent-and-never-the-records-author.md
docs/adr/0009-millpond-and-boom-work-both-ways-and-millpond-takes-in-a-backlog-only-where-no-other-planner-plans-it.md
docs/adr/0010-a-proposal-awaiting-a-person-shows-as-a-millpond-signal-where-no-other-planner-is-connected.md
design/requirements.md
check-statement.sh
check-statement.test.sh
tests/statement/holds.md
tests/statement/shared-number.md
tests/statement/unresolved.md
moon.yml
.gitignore
README.md
site/moon.yml
site/package.json
site/astro.config.mjs
site/README.md
site/scripts/design.ts
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
