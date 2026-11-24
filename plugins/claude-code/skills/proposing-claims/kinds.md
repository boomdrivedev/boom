# The kinds of a claim

Each kind is fixed by the first version of its claim. A name is kebab
case: lowercase letters and digits, in words that single hyphens join.
The name says the claim in short, such as `a-new-draft-replaces-the-one-before`.
A statement is one line. Run `boom show <name>` on an example to see it
in full.

## invariant

**For:** something that holds of the work at every moment, which a
reviewer can check against a change.

**Fields:** a statement. A body is optional, and a rule needs one.

**How to write it:**

- The statement says what holds, in the present tense. A reviewer can
  point at a diff and say that it breaks the statement.
- The body explains the rule in a paragraph or two, then lists:
  - `Breaks the rule:` each kind of change that breaks it;
  - `Does not break it:` each change near the line that does not;
  - `Source:` the claims that it rests on, by name.
- The lines of a list have no blank line between them.
- A rule names another rule by its words in running text, and by its
  name only in `Source:`.

**Example:** `a-decision-that-changes-a-rule-says-before-and-now`

```text
A decision that changes a rule says Before and Now in plain words.

Breaks the rule:
- a decision that says only that it changes a rule, by the id of the rule or the number of a clause
- a Before or a Now that a person cannot read without the rule open beside it

Does not break it:
- a Before that says in plain words what the rule allowed, and a Now that says what holds after the decision
- a decision that changes no rule and has no Before and Now
```

More examples: `a-claim-counts-only-when-its-signer-may-make-it`,
`status-read-from-events`.

## decision

**For:** a choice that the system follows, with the reasons written
when the choice was made.

**Fields:** a statement and a body. Boom refuses a decision with no
body.

**How to write it:**

- The statement gives the choice, not the question.
- The body is a MADR record, in these sections:
  - `## Context and problem statement`: what forced the choice, and the
    person's words with their date.
  - `## Considered options`: each option, numbered, the chosen one
    marked.
  - `## Decision outcome`: what holds now, in numbered points. Each
    option not taken says why.
  - `## Consequences`: `Good:` and `Bad:` lines.
- A decision that changes a rule links it with `--changes <rule>`, and
  says Before and Now in plain words.
- An open question of a design is a decision too. Its statement gives
  the recommended answer, and its body gives the options.

**Example:** `a-narratives-open-question-is-a-proposed-decision-included-at-its-paragraph`

```text
A narrative holds no open question: its author proposes each one as a
decision claim with its options and its recommendation, included at the
paragraph where the question comes up

## Context and problem statement
... On 2026-11-23 the user ruled: "A design's open question is a
proposed decision claim, included at its paragraph."

## Considered options
1. Each open question is a proposed decision claim, included at the
   paragraph where it comes up (chosen).
2. A section of open questions at the end of the narrative, answered
   in notes.

## Decision outcome
1. **A decision at its paragraph.** ...
2. **The person answers with the key or a note.** ...
   - Not taken: a section of open questions (option 2). Its answers
     stay in notes, which bind nothing.

## Consequences
- Good: each answer is a claim, with versions and links ...
- Bad: a narrative with many questions proposes many claims.
```

More examples: `a-new-draft-replaces-the-one-before`,
`each-hook-shows-the-person-one-line`.

## assumption

**For:** something about the world, not the code, that the system takes
as true and does not check: a fact about a user, a vendor, a load or a
law. No repository shows that it holds, so no check judges it. A person
reads it again when something challenges it.

**Fields:** a statement. A body is optional.

**How to write it:**

- The statement says what is taken as true, in the present tense.
- The body says why it is taken as true, and what changes when it stops
  being true.
- An assumption about the code is an invariant. An assumption about a
  plan, such as a date of delivery, is not a claim.

**Example:** the store holds no accepted assumption yet. A form that
fits:

```text
Every order is paid in one currency.

The shop sells in one country, and its payment provider settles in that
country's currency. When the shop sells in a second country, each amount
needs its currency, and the decision `money-has-currency` changes.
```

## term

**For:** a word and its meaning in one repository, with the old words
that it replaces. Boom stops a write that holds an old word.

**Fields:** the word is the name. The meaning is the statement.
`--replaces <old word>`, any number of times. A body is optional.

**How to write it:**

- The name is the word, in kebab case.
- The statement is the meaning, in one sentence that starts with "A" or
  "The".
- Name each old word that an agent or a person used for the same thing.
- The body says which plain words stay free, then lists what breaks it,
  what does not, and its source.
- A term binds one repository. A term with `--system` binds every
  repository, and the term of a repository stands over it.
- `boom query` gives the term and never its old words. Only the log
  keeps them.

**Example:** `origin`, for `boom`, replacing `came-from` <!-- boom: allow came-from -->

```text
The file or the commit that a version of a claim came from.

The old word is the name `came-from` in each of its joined forms ... <!-- boom: allow came-from -->
The plain words "came from" stay free.
```

More examples: `owner`, `signer`.

## context

**For:** a part of the system with its own model and words, which owns
path globs in one or more repositories. A claim that binds a context
binds only the files of its globs. Until someone draws a context, each
repository is one context.

**Fields:** a statement and one or more `--path <repository>:<glob>`.
The glob is relative to the root of the repository.

**How to write it:**

- The name is the part, such as `sluice-forge`.
- The statement says what the part is and what it owns, in one line.
- Each glob names the code, the specs and the changes of the part.
- Draw a context only where its rules differ from the rest of the
  repository.

**Example:** `sluice-plugins`

```text
The plugins of sluice for Claude Code and for herdr, each one a client of `sluice host`.
path: sluice:.claude-plugin/**
path: sluice:plugins/**
path: sluice:openspec/specs/plugin-*/**
path: sluice:openspec/changes/*/specs/plugin-*/**
```

More examples: `sluice-forge`, `sluice-host-process`.

## narrative and outline

Load the skill `writing-narratives`.
