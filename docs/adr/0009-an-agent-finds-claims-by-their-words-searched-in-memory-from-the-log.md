---
number: 9
title: An agent finds claims by their words, searched in memory from the log
status: accepted
date: 2026-11-18
decision-makers:
- the change an-agent-finds-decisions-by-searching-their-words's construct stage
---

# An agent finds claims by their words, searched in memory from the log

Status: accepted. The construct stage's own, on the brief "An agent
finds the decisions, rules and records about a subject by searching
their words, in its own repository or across every repository", and on
records 0005 and 0008. Cheap to change: a search writes nothing, so no
store holds anything of it, and its answer is a new format,
`boom-search/1`, versioned as every command's is.

Deciders: the construct stage of the change
`an-agent-finds-decisions-by-searching-their-words`.

Sources: record 0005, its consequence that agents read decisions and
rules locally, quickly and offline; record 0008, point 2;
`design/research/2026-11-15-planning-and-boom-storage/06-open-questions.md`,
the similarity net, and `03-agent-context-packets.md`, long content by
pointer; `interview.md`, the user's answers of 2026-11-01:
`caches-derived-only`, `flume-asks-what-applies` and
`legacy-number-alias`; core.1, core.2 and core.6.

## Context and problem statement

`boom query` gives an agent every claim that stands for its repository,
replayed from the log (record 0008). An agent at work asks about a
subject instead: what was decided about amounts, which rule a page's
`core.2` is, whether another repository settled how refunds are kept.
Nothing lets it find the few claims that bear on a subject, or look
across repositories at all. The research left open whether to search
by meaning, embedding each claim into a local vector index, or not at
first.

## Considered options

For how claims are found:

1. By their words, in a full-text table SQLite's FTS5 builds in memory
   from the judge's replay for each search, and drops with the answer
   (chosen).
2. By their words, in an FTS5 table kept in the store among the views.
3. By meaning: embeddings of each claim in a local vector index, the
   research's similarity net.
4. By substrings matched in Rust, with no stems and no ranking.

For what a search covers:

5. The claims that stand for the repository named or declared, or with
   `--all` every claim that stands, refusing with none of these (chosen).
6. Every repository, unless one is named.

For the command:

7. `boom search <words>...`, a command of its own (chosen).
8. `boom query <words>...`.

## Decision outcome

1. **Words, in memory, from the log.** `boom search` replays the whole
   log for each answer, as `boom query` does, puts each standing claim
   in scope into an FTS5 table in a SQLite connection held in memory,
   its name, legacy id, statement, section, scenarios and body as
   columns, and drops the connection with the answer. Nothing is kept,
   so deleting nothing changes an answer and no index falls behind the
   head (`caches-derived-only`; core.2), and a view edited by hand
   changes nothing found (record 0008, point 2). FTS5 is in the SQLite
   the tree already pins, so no tool or crate is added (core.6). Not
   taken: an FTS5 table kept in the store (option 2), which a program
   editing the store could change to hide a claim from every search,
   found only by a whole scan; meaning (option 3), which needs a model
   on every host, an index kept or rebuilt at a model's cost, and
   answers that change with the model, and which the brief does not ask
   for; substrings (option 4), which miss a word's other forms and rank
   nothing. A later change can add a kept index under
   `$XDG_CACHE_HOME/boom/`, checked against the head, or the similarity
   net beside words, without changing the format's major.
2. **Every word, as written.** A claim is found when every word
   searched is in it, a word matching its other forms by the Porter
   stemmer, its case and accents aside. Each word is searched as an
   FTS5 string, so no word is read as an operator and an agent's stray
   quote or bracket never fails the search; a word holding punctuation,
   `core.2`, is found as its parts in order, so a legacy id a page still
   cites finds its record (`legacy-number-alias`). Not taken: the words
   as an FTS5 query, which fails on punctuation; any word matching,
   which floods an agent's context with weak matches.
3. **Ranked, short, with a pointer.** FTS5's `bm25` ranks the claims, a
   word in a name or a legacy id weighing most, then a statement or a
   section, then scenarios, then a body; equal matches keep `boom
   query`'s order, kind then name, so the same log gives the same
   answer. A search gives the first ten, or `--limit`, with how many
   matched, and for each claim with a body the sixteen words of it
   where the words searched fall thickest, and the pointer `boom show
   <name>@<version>`, never its whole text, as the research's packets
   keep long content out.
4. **Only what stands.** A search gives the versions that stand, in the
   words the person accepted; never one proposed, rejected, replaced or
   history. Not taken: proposals and history in the answer, which an
   agent would read as what holds.
5. **The repository it works in, or every one.** A search covers the
   claims that stand for the repository `--repository` names or
   `.config/boom.toml` declares, as `boom query` reads it, and the whole
   system's; `--all` covers every claim that stands, each saying what
   it binds. With none of these it refuses, as `boom query` does. Not
   taken: every repository by default (option 6), which gives an agent
   other repositories' claims it did not ask for.
6. **A command of its own.** Not taken: `boom query <words>` (option
   8), since `boom query`'s arguments are kept for paths
   (`flume-asks-what-applies`), and a word and a path cannot be told
   apart.

## Consequences

- Good: an agent finds the claims about a subject on its machine,
  offline, as a person accepted them, with nothing kept that could
  differ from the log.
- Good: a page's legacy citation, `core.2` or `0005`, finds the record
  boom now keeps.
- Bad: a claim worded otherwise than the agent's words is not found;
  the agent searches again or reads `boom query`'s whole answer.
- Bad: the Porter stemmer knows English alone; a word in another
  language is found only in its own form.
- Neutral: a search replays the whole log and reads each body in scope,
  linear in both, which a kept index can later shorten without changing
  an answer.

## Amendment of 2026-11-20 (proposed)

Status: proposed, for the user to accept. The change
`boom-keeps-its-views-in-step` makes it.

This amendment changes point 1 and the last consequence.

`boom search` no longer replays the whole log for each search. It reads
the standing claims from the views. The views are tables that boom keeps
in step with the log, as record 0008's amendment says.

`boom search` still builds its full-text table in memory for each
search. It keeps no index.

A search reads the log's last entry and no older entry. Its cost grows
with the claims in scope. It does not grow with the log.

A view that another program edits changes what `boom search` finds,
until `boom verify` runs. The user accepted this on 2026-11-20.
