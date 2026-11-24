## Context

A claim has a version for each change. A version that `--amends` a
version names it as its base. Today Boom accepts a version only when its
base is the accepted version of its claim, and a first version only when
no version of its claim is accepted. Thus each version that amends a
draft waits for ever. On 2026-11-23 Boom refused the user's accept of
`new-design-goes-into-boom-not-blueprints@bd68243ee18f`, which amends the
draft `a43ac547d4e6`, and 12 claims of another session.

On 2026-11-23 the user ruled: "Replace: the new draft closes the old
one." The person sees only the newest draft of a claim, and an accept of
it works.

This change keeps these claims:

- `status-read-from-events@97d0559deafa` and
  `events-are-appended-never-changed@886b3f05e89f`: the entry that
  proposes the new draft is the event that supersedes the old draft. No
  entry changes, and no field holds a status.
- `the-status-words@3b98631a1d53`: the five status words stay. The
  decision names "replaced" as a status word that breaks it.
- `views-computed-from-the-ledger@36a056c1145b`: each status comes from
  a replay of the log, and the views take a new form so that each store
  builds them again.
- `the-gate-validates-specs-strictly@d8e057526e31` and
  `open-changes-archive-cleanly@de24a2987568`: this change passes
  `openspec validate --strict` and archives alone (core.7).

## Goals / Non-Goals

**Goals:**

- A person accepts the newest draft of a claim, whatever drafts come
  before it.
- An old draft cannot be accepted, and no listing of the drafts that
  wait gives it.
- Two amendments of one accepted version stay a conflict.

**Non-Goals:**

- A merge of two drafts into one text.
- A change to the log's format, or to the fingerprint of a version.
- Code that moves a store from the old rule to the new one. The rule
  reads the log as it is.

## Decisions

### 1. The base that an accept checks

The base that an accept checks is the nearest version in the line of
bases of the version that a person accepted. The status of that version
now can be `accepted` or `superseded`. Where the line holds no such
version, the accept checks no version. Boom accepts the version when
that base is the accepted version of the claim, or when both are none.
The walk goes past a draft, a rejected version and a version that came
in as history, because no person accepted it to stand.

Thus, with `v1` accepted, `v2` amending `v1` and `v3` amending `v2`, an
accept of `v3` checks `v1`. With `v2` rejected and `v3` amending `v2`,
an accept of `v3` checks `v1` too.

The options:

1. The walk to the nearest version that a person accepted (chosen). It
   reads the log, and the content of the new draft names its own base.
2. A base field that the new draft copies from the old one. The base is
   part of the fingerprint, and `boom show` would lose the line of
   drafts.
3. A refusal of an amendment of a draft. This is the fault that the
   user's ruling removes.

### 2. The old draft is `superseded`

When an amendment of a proposed version counts, the old draft becomes
`superseded`. Its proposal entry stays as it is. The new draft is the
event that supersedes it. No accept and no reject of the old draft
counts: Boom refuses each one, names the newer draft, and appends
nothing. A reject of the newer draft leaves the old draft superseded,
because an edit of an email that is not sent does not come back when
the new text is thrown away.

The options:

1. `superseded` (chosen). A later version takes the place of the draft,
   which is what the word says now for an accepted version.
2. A new word, `replaced`. `the-status-words@3b98631a1d53` names
   "replaced" as a word that breaks it, so this needs an amendment of
   that decision, and each reader of a status learns a sixth word.
3. `rejected`. No person rejected the draft, and a reject carries a
   person and a reason.

### 3. A draft has one line

Boom refuses an amendment of a draft that a newer draft superseded, and
names the newer draft. Thus one draft at most waits in each line, and
the person sees the newest one. Two amendments of one accepted version,
proposed apart, stay two drafts. When the person accepts one, Boom
refuses the other, and its agent proposes it again as an amendment of
the new accepted version. Several first versions of a claim with no
accepted version still wait together. An amendment of a rejected version
stays allowed, and the rejected version stays rejected.

The options:

1. A refusal that names the newer draft (chosen). An agent that read the
   claim before another agent's draft came reads it again.
2. A second draft beside the newer one. The person would see two drafts
   of one line.

### 4. What each command gives for an old draft

- `boom show <name>` gives the newest version first, as now. The old
  draft is under its base in the tree, with the status `superseded`,
  and the newer draft is under it. The JSON gives its `status` as
  `superseded`, with no `accepted_by`.
- `boom query`, `boom search` and the hook at session start give the
  versions that stand. A draft never stands, so they do not change.
- `boom page` gives the newest version of a narrative where no version
  is named, as now. The page of an old draft gives no line to accept
  it, because that line comes only for a proposed version.

### 5. Notes on a narrative's old draft

A new version of a narrative answers each open note of the narrative,
as now. This includes the notes on the draft that it supersedes. An
answer that an old draft gives stays the answer of its note: the note
stays answered while no person rejects that draft, and a superseded
draft cannot be rejected. Thus the person reads each answer on the page
of the newest draft, under "Notes on the other versions".

The options:

1. The rule of notes as it is (chosen). It needs no new code.
2. A note that opens again when a newer draft supersedes the draft that
   answers it. Each new draft would have to answer the same notes again.

### 6. The log's history

Boom computes each status from the log, so the drafts that came before
this change read under the new rule. The views' form goes from 8 to 9,
and each store builds its views again from its log when a command opens
it.

A copy of today's store, at entry 600, holds 39 drafts that a later
draft amends. Main's boom already refuses entries 338 and 352, two
versions of the narrative `decisions-that-check-themselves` that bind
two repositories, under the rule that a narrative binds one level. Thus
38 of the drafts count, and under the new rule each of the 38 is
`superseded`. No person accepted or rejected one of them, and no
proposal amends one of them a second time. Thus each entry that counts
under main's boom still counts. On two copies of the store, `boom
verify` of main's boom and of this change's boom build the same
judgments and the same edges. Their versions differ in the 38 drafts
alone, and their `views_head` in its form alone. The store holds no
note, so the notes change nothing.

`new-design-goes-into-boom-not-blueprints@bd68243ee18f` amends the draft
`a43ac547d4e6`, and its accept checks no version. The first version
`550046331eaa` was accepted after it, so Boom still refuses its accept,
and names `550046331eaa`. This is the conflict of two drafts of one
base that the ruling keeps.

## Risks / Trade-offs

- An agent that proposes a draft while another agent's draft waits on
  the same line gets a refusal, and proposes again on the newer draft.
- A person who accepts a list of drafts by version, with one of them
  superseded since the list was made, gets a refusal of the whole list,
  which names the newer draft.
