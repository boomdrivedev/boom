---
name: proposing-claims
description: Propose a claim to Boom that a person can accept as written. Use before any `boom propose` or the MCP tool `propose`, when a person makes a ruling about the system, when a design finds a rule, a decision, an assumption, a term or a context, or when a claim needs a new version. Covers each kind, writing in Plain, what a claim never says, the level a claim binds, its edges, drafts and batches.
---

# Proposing claims

Load `using-boom` first if your session does not have it. A claim says
what the system is and does, and why. A person reads it, and accepts it
or rejects it with a key. Write each claim so that the person can accept
it as it is.

## Before you propose

1. Run `boom search <subject>` and `boom search --all <subject>`. When a
   claim on the subject exists, amend it. Do not propose a second claim.
2. Choose the kind. See [the kinds](kinds.md) for what each kind is for,
   its fields, and examples from a store.
3. Choose the narrowest level that the claim binds (below).
4. Write the statement and the body in Plain (below).
5. Find the claims that it rests on, and where it came from (below).
6. Put it in a batch with the others of its cause (below).

## The kinds, in short

| Kind | It says | Fields |
|---|---|---|
| `invariant` | What holds of the work at every moment | A statement. A body with what breaks it, what does not, and its source |
| `decision` | A choice made, and why | A statement and a body: context, options, outcome, consequences |
| `assumption` | Something taken as true about the world, and not checked | A statement. A body with why it is taken as true |
| `term` | A word and its meaning in one repository, with the old words it replaces | A word as its name, the meaning as its statement, `--replaces` |
| `context` | A part of the system that owns path globs | A statement and `--path <repository>:<glob>` |
| `narrative` | An explanation of a design in its author's words | Load `writing-narratives` |
| `outline` | The order of the narratives in a view | Load `writing-narratives` |

## Say what the system is, never the plan

A claim stays true for as long as it stands. A plan is true for one
moment. So a claim or a narrative never says:

- a wave, a phase, a milestone, a sprint, an epic or a roadmap item;
- an MVP, a first cut, a stopgap, an interim form or a stretch goal;
- the order of work: "first", "then", "later", "after this lands";
- a status of work: TODO, TBD, WIP, FIXME, "for now", "until then",
  "not built yet";
- "the happy path" or "the sad path". Name a failure by what happens:
  "When the store is locked, boom waits 5 seconds and exits 2".

A claim says what the system does when the claim holds. When work is
not done, the claim is still true of the system that the person
accepts. The gap between the claim and the code is the work, and the
plan holds the work, not Boom.

`boom propose` warns when a statement or a body holds a plan word, and
names the word and its line. With `--json`, and through the MCP tool
`propose`, the answer gives each one in `plan_words`. Read the warning.
Change the words, or keep a word on purpose, such as a quote of a
person. A line that holds `boom: allow <word>` gives no warning for that
word.

## Write in Plain

Plain is ASD-STE100, Simplified Technical English, as this store uses
it:

- One sentence says one thing. A sentence has 20 words or fewer for an
  instruction, and 25 or fewer for a description.
- Use the active voice and the present tense: "boom refuses the
  version", not "the version will be refused".
- Use one word for one meaning, and the same word each time. Use the
  terms of the repository (`boom query` gives them).
- Use a verb for an action, not a noun made from a verb: "boom checks
  the chain", not "boom performs a check of the chain".
- Write numbers and names exactly: `5 seconds`, `exit 2`, `boom show`.
- A paragraph has one subject, and six sentences or fewer.
- No "should", "may want to", "ideally", "simply" or "just".
- No word that a reader must guess: "it", "this" and "that" name a thing
  in the same sentence or the sentence before.

## Bind the narrowest level

A version binds one of these:

| Level | Flag | When |
|---|---|---|
| A context | `--context <name>` | The claim holds only for the files of that context |
| A repository | `--repository <name>`, any number of times | The claim holds for each named repository |
| The whole system | `--system` | The claim holds for every repository |

Choose the narrowest level where the claim is true. A claim about one
tool binds that tool's repository, not the whole system. With no flag, a
first version binds the repository of the session, and an amendment
binds what its base binds.

## Link the claim

| Flag | Links to | Use |
|---|---|---|
| `--source <claim>` | A claim that this one rests on | Each claim that the body cites |
| `--changes <claim>` | A claim that a decision changes | A decision that changes a rule. Its body says Before and Now in plain words |
| `--origin <file>` | A file, or `git:<repository>@<commit>:<path>` | The exploration, mockup or session output that the claim came from |
| `--cause <claim>` | The claim whose change forced this version | Each version of a revision |
| `--provenance <text>` | Your session | Your place, your session id, and the person's words with their date |

A name stores the accepted version of a claim, or its newest version
when none is accepted. Name `<name>@<version>` to link another version.

The provenance carries the person's words: `the user, 2026-11-24: "<the
words>"`. A ruling's statement is the person's approved sentence.

## Drafts

- A new version of an existing claim amends a version:
  `--amends <version>`. Never propose a second first version of a claim.
- A new draft replaces the draft before it. When you amend a proposed
  version, the old draft becomes `superseded`, and the person sees only
  the newest draft.
- Amend the newest version in the line. Boom refuses an amendment of a
  draft that a newer draft superseded, and names the newer draft.
- Never amend a version that another person's accepted version
  supersedes. Run `boom show <name>`, and amend the accepted version or
  the newest draft on it.
- The same content gives the same version. Boom appends nothing for it.
- Withdraw your own proposal when it is wrong, a duplicate or no longer
  needed, and you have no better version:
  `boom withdraw <name>@<version> --reason <why> --provenance <session>`.
  Boom withdraws only a version that an agent proposed and that waits.
  When you have a better version, amend the draft.

## Supersede an accepted claim

`boom supersede` proposes a version that takes an accepted claim out of
force once a person accepts it. Run it in two cases only:

- **A duplicate.** Two accepted claims say the same thing. Supersede
  the one to retire with the one to keep. The new version of the claim
  to keep copies its words:
  `boom supersede <claim to retire> <claim to keep> --reason <why> --provenance <session>`.
- **A claim that another replaces.** An accepted claim no longer holds,
  and a new claim says what holds now. Give the new claim its name and
  its statement:
  `boom supersede <old claim> <new claim> "<statement>" --reason <why> --provenance <session>`.

Never supersede a claim to get your own change through. When your work
breaks an accepted claim, tell the person which claim and why, and let
the person decide. A version that supersedes waits on a person like
any other, and the old claim holds until the person accepts it. Only an
invariant, a decision or an assumption is superseded. To change the
words of a claim, amend it with `--amends`.

## Batches

The person signs a batch once. Boom computes the batches from the
links of the log, so the links that you give make the batch:

1. **A narrative with its claims.** The narrative includes each claim
   with `--includes`. One accept of the narrative accepts each proposed
   version that it includes.
2. **A revision with its cause.** The new versions that one change
   forces, such as a new rule. Each one names the claim that forced it
   with `--cause <claim>`. One accept of the cause accepts each proposed
   version that names it.
3. **A ruling.** Any other proposed version, alone. Draft it before
   you ask the person, with the person's words in its provenance.

No claim waits alone without a reason. A claim that is in no narrative
and has no cause goes into the narrative of its level, unless it is a
ruling. When you end your work, tell the person each batch: its
narrative, cause or ruling, and the command `boom accept --waiting`
that lists what waits. The person reads the batches on `boom page`,
under "Waiting on you".

## The command

```console
$ boom propose invariant refund-within-payment \
    "A refund never exceeds what the order was paid." \
    --body refund-within-payment.md \
    --repository shop \
    --source money-has-currency \
    --provenance "shop place change--refunds, session 1f2e: the user, 2026-11-24"
```

The answer names the version and its status, and the command that a
person runs to accept it. Do not run that command.
