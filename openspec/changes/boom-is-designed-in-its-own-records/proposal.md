# Proposal

## Why

boom's design was settled in a design session on 2026-10-31, with its
rulings, and in four decisions the user accepted on 2026-11-01, numbered
39 to 42 in the design's log. None of it is in this repository: it holds
one decision record and a site whose pages sketch a design they cannot
cite. The user decided on 2026-11-01 that boom's open design lives here,
in its own repository, as its own records, and not as a pointer to a
place a reader cannot open.

On the same day the user decided how anything here is known. In another
repository of the family, a constitution interview's branch numbered its
records and rules after the last ones on main. A parallel branch landed
records 0002 to 0016 and rules 12 to 18 first, every number on the
interview's branch collided, and 22 records and 75 rules were renumbered
twice, each citation rewritten. Any number allocated on a branch does
that, and boom's own sketch allocates them: its rendered decision
records are numbered after the last, its import numbers new records
after the last, and the family's statements give a new clause the next
free number. boom keeps the ledger of what a project claims; it must
make that renumbering impossible, for any project and for itself.

The site also contradicts the design in places. Its first walkthrough
has Ada accept the claims she proposed herself, which decision 41 rules
out; its roadmap lists as open who accepts, which decision 41 settles;
its millpond section has one direction where the user's ruling of
2026-11-01 has two. A reader cannot tell which page states a decision
and which a guess.

## What Changes

- A record, `names-not-numbers`: a record is known by a name, lowercase
  words joined by hyphens, chosen by its proposer and fixed when it is
  proposed; no two records hold one name, and the check fails at the
  merge when two branches bring the same one; every citation names it;
  any number a reader sees is a display ordinal derived on render from
  the order of acceptance on the default branch, never written into a
  file's name, a citation or a record; an imported ADR keeps its number
  as an alias, cited only as legacy.
- A rule in `docs/architecture/core.md`, `identity-is-a-name`, holding
  it, with what it rules out (the renumbering above) and its source,
  landing in the commit of its record. boom's six rules take names, each
  keeping its old id as a legacy id; the first record takes the name
  `an-open-repository-of-its-own`, keeping its number as an alias; every
  citation of a rule or a record in the tree names it; `CLAUDE.md` and
  `openspec/config.yaml` say so.
- Nine more decision records, each accepted by the user and citing the
  rulings and the decisions it rests on: `five-kinds-of-record`,
  `status-from-seven-events`, `verdicts-beside-the-claims`,
  `judged-on-a-host`, `the-gate-blocks-regressions`,
  `one-context-per-repository`, `a-person-accepts`,
  `millpond-both-ways` and `proposals-as-signals`.
- A statement of invariants, `design/requirements.md`, in the family's
  requirements style, its clauses, invariants and scenarios each known
  by a name and cited by it, its sections named and not numbered. Where
  the style gives a new clause the next free number, the statement says
  it departs from the style and why.
- Checks: `root:records` (every record and rule has a name of its own,
  and every citation of one in the tree names it), `root:commits` (the
  branch's commits cite by name too), `root:statement` (each item of the
  statement has a name of its own, and every citation in it names one)
  and `root:test` (their cases). `root:adrs`, which held numbers apart,
  goes.
- The site shows the statement and each record as pages, computed from
  the files at build time and never committed, each anchored and
  addressed by its name, the numbers it shows derived from main's
  history. Every page that describes the design cites its clauses and
  records by links the build checks.
- The site's pages are brought in line with the records: how it's used,
  configuration, commands, `boom check`, the walkthroughs, integrations,
  the roadmap and the start page. Rendering, syncing and importing write
  each decision record as a file named by its name. The integrations
  page carries the millpond loop, claim to verdict to item to work to
  landing to verdict, as a new diagram.

Out of scope:

- Drawing the constitution from these records: the rules the records
  imply, among them that an agent never accepts and that a verdict is
  appended, are drawn into `core.md` afterwards, as a separate step the
  user decided on 2026-11-01. This change adds one rule and names the
  six that stand, changing none of what they hold.
- Building any of boom: the crate still prints its version.
- millpond's side of the loop, its adapter for boom's backlog and its
  challenge move calling boom, which is millpond's own work.
- The family's requirements style and other repositories' rules, which
  keep their numbering until they change it themselves.

## Capabilities

### New Capabilities

- `record-identity`: each decision record and each rule of this
  repository is known by a name of its own, every citation of one in
  the tree and in the branch's commits names it, and a citation by a
  number fails, all checked in the fast layer.
- `design-statement`: the statement of invariants holds together, each
  clause, invariant and scenario known by a name no other holds, and
  every citation in it naming one, checked in the fast layer.
- `design-pages`: the site shows the statement and the decision records
  as pages computed from their files, each reachable by its name, the
  numbers it shows derived from the order things landed on main, every
  citation of the design a link the build checks, and the integrations
  page drawing the millpond loop.

### Modified Capabilities

None.

## Impact

- New prose: ten records, the statement and one rule. Each is reviewed
  by a person; no check judges what a record decides.
- The root project loses `adrs` and gains `records`, `commits`,
  `statement` and `test`, all in `check-fast`, which sluice requires on
  `main`. `adrs.toml` goes: the tool it configures writes a new record
  numbered after the last.
- Every citation of a rule or a record in the tree is rewritten to its
  name, once: the configuration files' comments, the scripts, `CLAUDE.md`,
  `openspec/config.yaml`, the first record, the site's pages and the
  README. Commits already on main keep the numbers they cite, which the
  legacy ids and the alias resolve.
- The site gains a step that writes the design pages before its type
  check and build, reading main's history for the numbers it shows; its
  tests gain that step's cases. The pages workflow fetches the whole
  history and builds when a record or the statement changes.
- Every page that sketched the open question of who accepts, the
  walkthroughs among them, changes to the decided answer: an agent
  proposes and a person accepts, never their own proposal.

## Touches

```
docs/adr/0001-boom-is-an-open-source-repository-of-its-own-and-its-binary-is-boom.md
docs/adr/an-open-repository-of-its-own.md
docs/adr/names-not-numbers.md
docs/adr/five-kinds-of-record.md
docs/adr/status-from-seven-events.md
docs/adr/verdicts-beside-the-claims.md
docs/adr/judged-on-a-host.md
docs/adr/the-gate-blocks-regressions.md
docs/adr/one-context-per-repository.md
docs/adr/a-person-accepts.md
docs/adr/millpond-both-ways.md
docs/adr/proposals-as-signals.md
docs/architecture/core.md
design/requirements.md
adrs.toml
check-adrs.sh
check-records.sh
check-records.test.sh
tests/records/
check-statement.sh
check-statement.test.sh
tests/statement/
moon.yml
.moon/workspace.yml
CLAUDE.md
openspec/config.yaml
Cargo.toml
crates/boom/moon.yml
devenv.nix
treefmt.nix
rust-toolchain.toml
check-toolchain.sh
.github/workflows/pages.yml
.gitignore
README.md
docs/diagrams/boom.svg
site/moon.yml
site/package.json
site/astro.config.mjs
site/README.md
site/scripts/design.ts
site/scripts/design.test.ts
site/src/diagrams/millpond.ts
site/src/diagrams/index.ts
site/src/diagrams/fits.ts
site/src/diagrams/hero.ts
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
