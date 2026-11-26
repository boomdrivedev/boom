---
name: writing-narratives
description: Write a narrative for Boom, the explanation of a design that includes its claims at their paragraphs, and the outline of the design book. Use when a design, an elaboration or a research ends with findings, when many claims come from one design, when a person's note on a narrative waits for an answer, or when asked to write, revise or order a narrative. Load proposing-claims too for the claims that it includes.
---

# Writing narratives

Load `using-boom` and `proposing-claims` first if your session does not
have them. A narrative explains a design in its author's words. A person
reads it and accepts it with a key, or writes a note on a paragraph. No
one checks a narrative against the code. The claims that it includes
are what binds work.

## What a narrative is

- A **title** of one line, as its statement.
- A **body** of Markdown, kept by its fingerprint. Boom numbers its
  paragraphs from 1: each heading, paragraph of text, code block, block
  quote, table, and each item of a top-level list.
- The **claims that it includes**, each at the version that it meant,
  with `--includes <name>`. A narrative includes claims of the other
  kinds, never another narrative.
- **One level** that it binds: the whole system (`--system`), one
  repository (`--repository`), or one context (`--context`). Boom
  refuses a narrative that binds two.

A narrative names no book, part or chapter. An outline holds it.

## How to write it

1. Write one narrative for each level of the design. A design across
   two tools is a narrative of the whole system and one for each tool.
2. Open with one paragraph that says what the part is and what the
   narrative explains.
3. Give each subject its own paragraph. A paragraph explains why the
   system does what it does, then names the claims that hold it, by
   name, in parentheses at its end:
   `(`a-starting-session-collects-its-owed-outcomes-first`)`.
4. Include each claim that a paragraph names, with `--includes`.
5. Write in Plain, and say what the system is, never the plan
   (`proposing-claims`). A narrative never holds a list of versions, a
   section of open questions, or a list of work.
6. A diagram is a fenced `mermaid` block. It counts as one paragraph.

## An open question is a proposed decision

A narrative holds no open question. For each question:

1. Propose a decision. Its statement gives the answer that you
   recommend. Its body gives the context, the options and why you
   recommend one.
2. Include the decision in the narrative, and name it in the paragraph
   where the question comes up.

The person accepts the decision with the narrative, or writes a note on
that paragraph to choose another option.

## The batch

A narrative with its claims is one batch. One accept of the narrative
accepts each proposed version that it includes. Boom refuses that
accept while the narrative includes a rejected, withdrawn, superseded
or deprecated version, and names each one.

So, in this order:

1. Propose each claim, or amend the claim that exists.
2. Propose the narrative with `--includes` for each claim. A name stores
   the accepted version, or the newest version when none is accepted.
3. When you propose a new draft of an included claim, propose a new
   version of the narrative that includes the new draft.
4. Tell the person the narrative's `<name>@<version>` and the claims in
   it. The person reads it with `boom page <name>`.

## A note and its answer

A person writes a note on one paragraph of a version. The note stays
open until a later version answers it. Boom refuses a new version of a
narrative that leaves a note open, and names each open note.

1. Run `boom show <narrative>` to read the open notes.
2. Change the body, and the claims that the note is about.
3. Propose the new version with `--amends <version>` and one
   `--answers <note>=<words>` for each open note. The words say in one
   line what the new version does about the note.

## The outline of the design book

An outline is a claim of the kind `outline` that holds the order of a
view. A store has one design book, and one outline holds its order. An
outline binds the whole system.

```console
$ boom propose outline the-design-book "The design book" \
    --view book \
    --narrative what-boomdrive-delivers \
    --narrative the-lineage \
    --narrative sluices-merge-queue \
    --provenance "<your place and session>"
```

Boom computes the book from the log. Part I holds the narratives of the
whole system and of the contexts of two repositories or more. Part II
holds a chapter for each repository, with its contexts. Each part keeps
the order of the outline, and then the narratives that the outline does
not hold, marked. To add a narrative to the book, amend the outline.

## Example

`sluices-plugins` binds the context `sluice-plugins` and includes five
invariants. Run `boom show sluices-plugins`. Its third paragraph:

```text
A session can stop before its outcome arrives. So a session that starts
collects first, by `sluice inbox`, every outcome owed to its readers.
The plugins for Claude Code tell a new session to do so, and the board
runs the inbox when it first reads the overview. Thus no session must
wait for an outcome, and no carrier keeps a list of its own.
(`a-starting-session-collects-its-owed-outcomes-first`)
```

More examples: `what-boomdrive-delivers` for the whole system,
`sluices-merge-queue` for one repository.
