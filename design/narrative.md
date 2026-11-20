# Boom, end to end

The first draft of Boom's design book: how Boom works from a signal to a
landed change and back, written for you to evaluate before any of it is
locked into decision records and rules.

Every part carries one of these marks:

- **[built]**: in the `boom` binary on main today.
- **[designed 11-01]**: the design of 2026-11-01, from the constitution
  interview (`interview.md`), the site's pages and the unlanded change
  `boom-is-designed-in-its-own-records`. Never built.
- **[designed 11-15]**: the planning and storage session
  (`design/research/2026-11-15-planning-and-boom-storage/`) and records
  0005 to 0011.
- **[decided 11-20]**: your decisions of today, as the orchestrator
  relayed them.
- **[proposed]**: mine, in this draft, for you to rule on.
- **[open]**: nobody has settled it.

Where a later record or decision dropped part of the 11-01 design
without your ruling on it, this draft says so and keeps the 11-01 part,
marked **[kept, needs your ruling]**.

## 1. What Boom is

A project says things about itself: what must always be true, what was
decided and why, what it assumes about the world, where one part ends
and the next begins, what a word means here. Boom keeps those sayings,
**claims**, in one log per drive, records who proposed each and who
accepted it, and computes everything else from that log: the rules an
agent reads, the decision records, the design book, the context map,
and whether each claim is still true of the code.

Boom never plans, routes, launches or lands work [designed 11-01:
`never-plans-starts-or-lands`; Boomdrive context pack, decided]. It
states what holds and finds where it does not. Planning, the inbox, the
places and the gate are other tools'.

## 2. Your flow, as Boom

Your picture has three durable parts and a per-change flow. Each piece
is something Boom keeps or computes:

```
+---------------------------- DURABLE: the drive's log ----------------------------+
|                                                                                  |
|  CONSTITUTION                ARCHITECTURE DOCS             ADR LOG               |
|  = standing invariant        = views rendered from the     = decision claims,    |
|    claims (and terms) for      ledger by context: rules,     each version kept,  |
|    a scope: "rules for a       the context's chapter of      superseded never    |
|    scope", a view            the design book, the map      edited (core.1)      |
|                                                                                  |
|  constitution skill          "loaded by footprint"         "read on deviation"   |
|  = an elaboration that       = boom query for the          = boom show / search  |
|    proposes invariants;        contexts a change's paths     on the decision a   |
|    amend mode = amendments     fall in                       gate cites          |
+-------------------+--------------------------+------------------------+----------+
                    | injected into every      | loaded by              ^
                    | OpenSpec artifact        | footprint              | a deviation
                    v                          v                        | becomes a
+----------------------------- PER CHANGE: OpenSpec -----------------------------+ |
| proposal --> design --> specs + tasks --> construction --> REVIEW GATE --> archive|
|                                                            = boom check         | |
|                                                     holds: lands  |  deviates ---+-+
+----------------------------------------------------------------------------------+
   other decision sources (spikes, incidents, reviews, your thinking)
   = signals into millpond --> elaboration --> proposed claims --> your signed accept
```

| In your picture | In Boom | Mark |
|---|---|---|
| Constitution: invariants, falsifiable, injected everywhere | the standing invariant claims of a scope, rendered as "rules for a scope"; falsifiable because each claim names how one would know it holds | claims [built]; the view [designed 11-01] |
| Constitution skill: interviews you, writes terse rules, re-runnable in amend mode | an elaboration (sounding) whose kind proposes invariant claims; amend mode proposes amendments, each naming the version it amends | amendments [built]; the skill on Boom [proposed] |
| Architecture docs: current state, core / domain / infra / CI | the views of the ledger for one context: its rules, its decisions in force, its chapter of the design book and its place on the map; "core" is the claims bound to the whole system | [designed 11-01] |
| Loaded by change footprint | the paths a change touches name the contexts they fall in, and `boom query` gives those contexts' claims | query by repository [built]; by context [decided 11-20] |
| ADR log: immutable, one decision per record, superseded never edited | decision claims: a version is never changed, an amendment is a new version, a supersession a new claim naming the one it replaces; MADR records rendered from them, numbered at first acceptance | versions [built]; MADR rendering and numbers [designed 11-01] |
| Injected into every artifact via `config.yaml` | the change's agent gets the constitution and its footprint's rules from Boom; how they reach `config.yaml` is question Q1 below | [open] |
| Review gate: conforms, or is the deviation justified? | `boom check`: does this change move the code away from a claim that stands? holds, violated or unclear | [designed 11-01] |
| Deviation becomes a new ADR | a deviation is a proposed decision claim, which only your signed accept makes stand | proposals and accepts [built]; from the gate [designed 11-01] |
| Amendment lane back into the constitution | an accepted decision that changes an invariant is proposed as that invariant's amendment or supersession | [designed 11-01] |
| Decisions flow in freely from spikes, incidents, reviews | anyone, agent or person, proposes; millpond's signals become proposals through elaboration | proposing [built]; from signals [designed 11-01, 11-15] |
| ADR numbered by the ADR CLI | Boom gives the number when a decision is first accepted, never on a branch, so two branches never collide | [designed 11-01: `names-not-numbers`] |

The rest of this draft explains each piece.

## 3. The two loops

[decided 11-20] Boom's work happens in the outer loop. Each unit of a
bolt becomes an OpenSpec change in the inner loop.

```
OUTER LOOP (claims and planning)
  signal ----> elaboration ----> claim, accepted by you ----> planning ----> bolt
  millpond     sounding          signed, in the drive's log   impact walk    covers claim@v4
     ^                                                                         |
     |                                                                         v unit
     | a failing claim, or a claim                              INNER LOOP (OpenSpec)
     | construction finds wrong                                 change: brief names claim@v4
     |                                                          build in a flume place
     |                                                          gate: boom check
     +---------- landing record: unit, revision, tests ------- landed by sluice
```

- **Claims come first.** [decided 11-20] You accept claims in the outer
  loop, before construction: elaboration drafts them, you sign them.
- **Planning makes a bolt that covers `claim@version`**, and each unit
  becomes an OpenSpec change whose brief names `claim@version`.
  [decided 11-20] Planning belongs to the control plane or crew's
  planner, never Boom [designed 11-01, 11-15]; Boom answers the impact
  walk's questions from the log. Planning is outside the MVP.
- **A change never edits a claim.** [decided 11-20] If construction finds
  a claim wrong or impossible, it raises a signal back to planning.
- **A landing writes a record** in the drive's log naming the unit, the
  revision and the tests that check each claim it covers. [decided
  11-20] Boom's check status reads those records.
- **A bolt planned against an older version shows stale** when the claim
  moves on, and planning asks: keep, amend, split or cancel. [decided
  11-20; designed 11-15]

Where this meets the review gate: in your picture a deviation at the gate
becomes a new ADR. Today's decision says a change never edits a claim.
Both hold if the gate's deviation is a *proposal*, not an edit: the
change's agent proposes a decision or an amendment, it waits on you, and
the change waits with it or goes back to planning as a signal. The 11-01
design said the same: "Violated blocks the merge: the agent fixes the
code, or proposes an amendment or a claim to replace it, which waits on
a person." [designed 11-01: `what-blocks`]

## 4. Where everything lives: the drive's log

```
 adapters ---+
 agents  ----+--> millpond: the drive's log --------> boom: claims, status, views
 you, on  ---+    (one SQLite file per host)          planning: impact walk, bolts
 the rail ✍       append-only, hash-chained           the board: what waits on you
                  views kept in step, one transaction
                  attached: the Drive object numbers
                  entries; each host keeps a replica
                  and an outbox
```

- **Millpond keeps the drive's log, and Boom keeps no store of its
  own.** [decided 11-20] Contexts, claims, signed accepts, landing
  records and evidence are entries in that log. Your answers on the rail
  are signed entries in the same log.
- **The log is SQLite on every host**: append-only, each entry carrying
  the hash of the one before, the views per kind of entry updated in
  the same transaction. [decided 11-20; designed 11-15; record 0005]
  Boom's built store is the starting point: one SQLite file, a chained
  log, entries judged on every read, long text kept once by its
  fingerprint [built: record-store, record-log].
- **Attached to the cloud, the Drive object orders entries**; each host
  keeps a full replica and an outbox of proposals waiting to be ordered;
  the runner carries them up and down. [decided 11-20; designed 11-15,
  record 0005] No git branch.
- **Every tool appends one entry form**: sequence, type, id, revision,
  author, previous hash, payload, and a signature on what a person signs.
  [decided 11-20: weir's change A1]
- **A repository holds no Boom file.** Boom finds a repository's drive
  from its git remote and the host's Boomdrive configuration, which
  already lists each drive's repositories. [decided 11-20] This retires
  `.config/boom.toml` [built, record 0008].
- **Each event carries the events its writer had read**, so two hosts'
  appends give one status in either order before the Drive object
  numbers them. [designed 11-01: `events-carry-their-order`; kept,
  needs your ruling: record 0005's ordering by the Drive object covers
  the attached case, not entries made offline on two hosts]
- **What changes by the second stays out**: a bolt's live status, a
  lease, an agent's progress. [designed 11-15, record 0005 point 7]

What the 11-01 design said here and today's decisions replaced, by your
ruling: the ledger as recfiles on a shared git branch, written by the
family's protocol (replaced by record 0005, which you accepted); a
repository's `.config/boom.toml` naming its ledger (replaced by today's
decision 3).

## 5. The records

[designed 11-01: `five-records`] The ledger holds five kinds of record,
each named once for life in one name space, each version the hash of
what it says.

| Record | Holds | Mark |
|---|---|---|
| **claim** | a name, a kind, a one-line statement, its scenarios, a body, what it binds, provenance | built for invariant, decision, assumption |
| **context** | a part of the system with its own model and language: its purpose and the paths it owns, in any repository of the drive | [decided 11-20: a context is a claim]; [designed 11-01] |
| **relationship** | two contexts, which is upstream, the pattern, the interface, the direction a dependency may run | [designed 11-01]; a claim, as today's page draws it [decided 11-20] |
| **term** | a word as one context means it | [designed 11-01] |
| **narrative** | an explanation in its author's words at the time, scoped to a context, including claims by name; judged by a person, never checked | [designed 11-01]; long text stored once by fingerprint [built] |

Today's decision makes a context a claim, and the page draws
relationships as claims too. The 11-01 design kept them as their own
kinds because a narrative cannot be true or false of the code. This
draft follows today's decision for contexts and relationships, and
keeps the narrative as a record of its own: it carries the design and
its reasons, stored once and pointed to by version, and never has a
check status. [proposed]

### Claim kinds

| Kind | Says | Mark |
|---|---|---|
| context | a part of the system and the paths it owns | [decided 11-20] |
| relationship | how one context uses another, through what interface | [designed 11-01] |
| term | a word in one context's language | [designed 11-01] |
| invariant | always or never, inside a context: the constitution | [built] |
| decision | the mechanism chosen, the options, the outcome: the ADR | [built] |
| assumption | a fact about the world that the project relies on; no repository can show it holds, so it has no check status and goes back to you when a signal challenges it | [built]; [designed 11-01] |
| behaviour | when X and Y hold, Z results | [proposed, from the 11-20 talk] |
| model | what must hold under every ordering, failure and timing | [proposed, from the 11-20 talk] |
| limit | a measured bound: CPU, processes, latency | [proposed, from the 11-20 talk] |

### A claim's body

- Today a claim has a statement, at least one scenario, and an optional
  body; the version is the hash of all three. [built]
- In the 11-20 talk each kind of claim gets one body agents read and one
  checker: Allium for invariants and behaviour, TLA+ for models, MADR
  for decisions. You sign the English statement; agents read the formal
  body alone. [decided in the 11-20 talk] **Allium and TLA+ are outside
  the MVP.** [decided 11-20]
- Whether a claim keeps its scenarios, or they stay in OpenSpec while a
  claim keeps its statement, its contexts and the names of the tests that
  check it. [open, with you]

### Status: accepted, and checked

A claim has two statuses, read from the log, never stored [core.1]:

| Accepted | Checked |
|---|---|
| proposed, standing, replaced, rejected, history [built]; to be named proposed, accepted, rejected, superseded, deprecated [open, with you] | **verified**: every test that checks it passed; **failing**: one failed; **unverified**: nothing checks it; **stale**: its tests passed for an older version than the one accepted now [decided 11-20] |

```
            boom propose               boom accept ✍
 (none) -----------------> proposed -----------------> standing / accepted
                              |                            |   \
                  boom reject ✍                 amendment  |    \ supersession
                              v                 accepted   v     v accepted
                          rejected                     replaced  superseded

 checked, for a standing claim, read from landing records and test results:
   unverified --(a landing names its tests, all pass)--> verified
   verified   --(a named test fails)------------------> failing --> a signal
   verified   --(the claim is amended)----------------> stale
```

- **A failing claim appends a signal** to millpond, once per break, and
  planning finds the bolts it touches. [decided 11-20]
- **The 11-01 design's verdicts.** It judged each claim against each
  repository at its shared head (`boom judge`), the "cell", with a
  value (satisfied, partial, not satisfied, not applicable), kept
  verdicts beside the claims, recomputed a cell only when the claim's
  version moved, its evidence left the head or a signal challenged it
  (`boom reconcile`, `boom challenge`), and listed the cells not yet
  satisfied as the backlog. [designed 11-01] Today's check status covers
  the claims a test checks. The judge still answers for the claims no
  test can check, a decision or a prose invariant, by a model whose
  verdict is evidence like a test's. [kept, needs your ruling]

## 6. Contexts and the map

- **A context owns paths in any repository of the drive**, and one claim
  can bind several contexts, so a claim spans repositories. [decided
  11-20] `boom propose context sluice.reader "..." --paths
  sluice:crates/sluice/src/overview.rs`.
- **Until someone draws the map, each repository is one context**, and a
  claim written for a repository binds it; drawing contexts later moves
  no claim until someone proposes the move. [designed 11-01:
  `one-context-per-repository`] Today's built attachment to a repository
  becomes binding that repository's context. [proposed]
- **The map is drawn from context and relationship claims** and drives
  three things: the impact walk, the fixed opening of every planning
  brief (the constitution, then the map), and the check that code
  crosses between contexts only through a relationship's interface.
  [decided in the 11-20 talk; designed 11-01: "the map's free rule";
  11-15 research file 03]
- **Between drives**, a drive publishes a slice of its claims as its
  published language; another imports it pinned by hash and declares the
  relationship. [designed in the strategy pack]
- **Terms are checked without reading a Markdown file**: when a claim
  or narrative is proposed in a context, its words are checked against
  that context's terms, read from the ledger; at the gate, a new name in
  code that collides with another context's term is flagged. Neither
  reads a page of the tree. [proposed; principle 16, core.5]

## 7. The views

[designed 11-01; core.2] Every view is computed from the log each time
it is asked for, and never read back as the truth.

| View | Built from | In your picture | Mark |
|---|---|---|---|
| **rules for a scope** | the standing invariants, decisions and terms bound to a context or the whole system | the constitution | `boom query` by repository [built]; by context [decided 11-20] |
| **decision records** | decision claims, in MADR, `NNNN-<name>.md`, numbered at first acceptance | the ADR log | [designed 11-01] |
| **design book** | narratives woven with the claims each includes, a chapter per context, every paragraph naming its record so a note on it reaches the record | the architecture docs' "why" | [designed 11-01]; outside the MVP [decided 11-20] |
| **context map** | context and relationship claims | the architecture docs' shape | [designed 11-01] |
| **check status** | landing records and test results | the gate's evidence | [decided 11-20] |

`boom render rules|adrs|book|map` writes a view to a folder and never
into the ledger. [designed 11-01]

### How a view reaches a repository

The 11-01 design had `boom sync` write into each repository what its
agents and people read there: the rules for its scope, its decisions as
MADR files, and the slice `boom check` reads, all on one pull request
kept open on the branch `boom/sync`, which the repository's own gate
lands. [designed 11-01: `sync-by-one-pull-request`] Record 0005 retired
the plan's git ledger and with it the slice, and record 0008 had agents
ask `boom query` instead of reading files. You never ruled on sync
itself. [kept, needs your ruling] Today's decision that a repository
holds no Boom file bears on it: it rules out the declaration file, and
it may or may not rule out rendered rules and decision records in the
tree. Your picture injects the constitution through `openspec/config.yaml`.
That is question Q1.

## 8. A change, from brief to landing

```
 brief names claim@v4 --> agent session starts: boom query gives the constitution
                          and the claims of the contexts its paths fall in
                          (Claude Code plugin's session hook) [built, by repository]
      |
      v
 before each edit: the claims bound to the file's context [designed 11-01: hook-before-each-edit]
      |
      v
 OpenSpec proposal, design, specs, tasks: each artifact carries the constitution
 and the footprint's rules; the design cites the rules it relies on and names any
 it departs from (as this repository's openspec/config.yaml already asks)
      |
      v
 gate: boom check --base main, one moon task in a layer the branch requires
   1 scope:  changed paths --> contexts --> standing claims, plus the system's
   2 rules:  each claim's checker passes or fails, no judgment; the map's free
             rule fails a dependency no relationship allows
   3 prose:  a reviewer model reads the diff against the claims still to settle
   --> holds | violated | unclear        [designed 11-01: three layers]
      |
      +-- holds: sluice lands it; a landing record names the unit, the revision
      |          and the tests that check each claim --> check status
      +-- violated: fix the code, or propose a decision (the deviation) that
      |          waits on your signed accept; the change waits or goes back to
      |          planning as a signal
      +-- unclear: fails until you answer for this change alone, signed; the
                 answer is never a verdict [designed 11-01: answers-signed-in-the-tree]
```

- **The gate blocks regressions, never the backlog.** A claim the change
  leaves alone holds, so a change is never held up by a claim nobody has
  built yet. [designed 11-01: `check-asks-direction`]
- **The reviewer and the judge are an agent program's command with its
  exact model**, named by an owner in the drive's settings; Boom holds no
  model credential; an unreachable reviewer is "not run", never a pass.
  [designed 11-01; core.3]
- **What the check reads at a merge.** The 11-01 design read the claims
  from the slice in the change's own tree, so a cached check replays and
  no change can choose its own claims. With no file in the tree, the
  check reads the host's replica of the drive's log at a head the change
  names, that head an input of the check. [proposed; record 0005 left it
  open]
- **The 11-01 answer to an unclear check** lives in the change's tree,
  under `.boom/answers/`, signed, void once the change moves. Today's
  rule of no Boom file in a repository may move it into the drive's log,
  keyed by the change's digest. [kept, needs your ruling]

## 9. Who may do what

- **An agent proposes; only a person accepts or rejects**, never the
  proposal's author, and every person's act is signed with a security
  key, the person present. [built: records 0006, 0011; designed 11-01]
- **`boom mcp` serves an agent every command it may run and none a
  person must sign**; an answer that waits on you says how you accept.
  [built: record 0011]
- **One signature can accept many versions**, so an import of a hundred
  records is one touch. [built: record 0009]
- **Owners register persons and keys**, the first owner self-registered;
  the drive's settings, its reviewer and judge among them, change only
  by an owner's signed act. [built for persons; designed 11-01 for
  settings]
- **Your answers on the rail and your accepts are the same kind of
  thing**: signed entries in the drive's log. [decided 11-20]

## 10. Signals both ways

- **In**: a millpond signal that bears on a claim challenges it, and its
  status is checked again; an assumption goes back to you. A signal that
  asserts something new is proposed as a claim with the signal as its
  provenance. [designed 11-01: `millpond-in-both-directions`]
- **Out**: a failing claim appends a signal [decided 11-20]; Boom's gaps,
  a claim nothing checks, a failing check, an upstream claim that
  changed, are offers at millpond's intake [designed in the update
  ladder, 2026-11-11]; a proposal waiting on you shows where you look:
  the rail when a drive is attached, millpond otherwise [strategy
  decision 42].
- **millpond asks whether a claim stands** (its statement B.10, 100) and
  cites claims in its moves; Boom answers. [designed in millpond's
  statement]

## 11. The MVP

[decided 11-20, as the orchestrator relayed it, with today's addition]

On one host with no cloud:

- the drive's log holds captures, signals, contexts, claims, signed
  accepts, landing records and evidence;
- Boom shows each claim accepted, and verified, failing, unverified or
  stale;
- a failing claim appends a signal;
- `boom query` gives an agent the claims for the contexts it works in;
- **the ADR and constitution system** [added today]: decision claims
  rendered as MADR records, the constitution as the rules for a scope,
  both reaching each OpenSpec artifact, the gate as `boom check`, a
  deviation as a proposed decision. [proposed: which of render, sync and
  check are in the MVP is question Q2]

Proven on sluice's record 0093: `reads-come-from-the-host` shows failing
on deneb, then verified once sluice keeps to its host.

Outside the MVP: the Drive object and sync, Allium, TLA+, the design
book, the planning walk.

## 12. The order

```
A1  weir      the drive log's entry form, which every tool appends
       |
A2  millpond  the drive's log, local: one SQLite file per host, chained,
       |      views in step; Boom's built store is its starting point;
       |      the inbox leaves its git branch
       v
A3  boom      claims, contexts and signed accepts as entries of that log;
              landing records and test results to check status; a failing
              claim's signal; query by context; the ADR and constitution
              views; the gate
       |
       v
    boom moves onto itself (section 13)
```

Each is an OpenSpec change in its own repository, by its own agent.
[decided 11-20: process-model Track A]

## 13. Boom moves onto itself

boom keeps its own design as files today: eleven decision records in
`docs/adr/` (two of them numbered 0009, and one, `the-rename-to-boomdrive`,
with no number), and its constitution in `docs/architecture/core.md`
(core.1 to core.8), which `CLAUDE.md` loads into every session. Once A3
lands:

1. **Its contexts are proposed**: to start, `boom` as one context
   owning the repository, as the 11-01 default has it. [designed 11-01]
2. **`boom import adr docs/adr`** proposes each record in its file's own
   words, with the names and scenarios an agent drafts, superseded ones
   as history; **`boom import rules docs/architecture/core.md`** proposes
   core.1 to core.8 as invariants, each citing the records it rests on.
   Both are built today against boom's own store [built: record 0007]
   and move onto the drive's log with A3. The two records numbered 0009
   keep their names; the legacy-number rule refuses a second 0009, so
   one is given the next number at acceptance. [designed 11-01:
   `legacy-numbers-kept-count-goes-on`]
3. **You accept the import with one signature.** [built: record 0009]
4. **One change then removes the pages from the repository** and drops
   the `@docs/architecture/core.md` line from `CLAUDE.md`, landing only
   once every session in the repository starts with the claims that
   stand for it and an agent finds a record by its words. [built: record
   0009's conditions]
5. **From then on, boom's own decisions are proposed in Boom**: a design
   proposes a decision, you sign it, and the rules an agent reads come
   from `boom query`. This document is the first narrative Boom keeps.

## 14. What a person sees, as boom stands after the MVP

```console
$ boom status --context sluice.reader
claim                                   kind        accepted   checked
reads-come-from-the-host                invariant   v3         failing   reader::reads_the_host_document ✗
one-host-per-person                     invariant   v2         verified  host::one_host_for_two_starts ✓
overview-answers-every-repository       behaviour   v1         unverified
```

```console
$ boom query          # in a sluice worktree, at session start
sluice.reader: 3 claims stand (drive boomdrive, entry 4822)
invariant reads-come-from-the-host@3
  Every read of the queues is answered by the host process. Nothing else asks GitHub.
...
```

## 15. Questions for you

- **Q1. How the constitution and the footprint's rules reach an
  OpenSpec artifact.** (a) The agent's session gets them from
  `boom query` at its start and before each edit, and
  `openspec/config.yaml` carries only a rule telling each artifact to
  cite them [built, record 0008]. (b) `boom sync` renders them into each
  repository's `openspec/config.yaml` and decision records into
  `docs/adr/`, on one pull request the gate lands [designed 11-01], as
  your picture draws it. Recommended: (a) for agents, so nothing in the
  tree can be edited by hand, with `boom render` for people who want the
  pages.
- **Q2. Which of the 11-01 machinery joins the MVP** with the ADR and
  constitution system: `boom render rules|adrs`, `boom check`'s three
  layers, the judge for claims no test checks, `boom sync`.
  Recommended: render and check's first two layers; the judge and sync
  later.
- **Q3. Relationship and term as claim kinds, the narrative as its own
  record.** Recommended: yes, as section 5 has it.
- **Q4. The acceptance names**: proposed, accepted, rejected,
  superseded, deprecated, in place of the built proposed, standing,
  replaced, rejected, history. [open, with you]
- **Q5. Whether a claim keeps its scenarios**, or they stay in OpenSpec.
  [open, with you]
- **Q6. The judge.** Keep the 11-01 verdicts for claims no test can
  check, a model's verdict being evidence like a test's?
- **Q7. Who appends the landing record**: flume when it settles a landed
  place (recommended), or a boom command run by hand for the MVP.
- **Q8. Ordering offline.** Keep the 11-01 rule that each entry names
  what its writer had read, so two hosts working without the cloud give
  one status in either order?
