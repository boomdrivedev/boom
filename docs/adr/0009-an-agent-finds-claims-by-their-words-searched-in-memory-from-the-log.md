---
number: 9
title: An agent finds claims by their words, searched in memory from the log
status: accepted
date: 2026-11-18
decision-makers:
- the change an-agent-finds-decisions-by-searching-their-words's construct stage
---

# An agent finds claims by their words, searched in memory from the log

Status: accepted. This record is the decision of the construct stage. It
rests on records 0005 and 0008, and on this brief:

> An agent finds the decisions, rules and records about a subject by
> searching their words, in its own repository or across every
> repository

It is cheap to
change. A search writes nothing, so no store holds anything of it. Its
answer is a new format, `boom-search/1`, with a version as the answer of
each command has.

Deciders: the construct stage of the change
`an-agent-finds-decisions-by-searching-their-words`.

Sources:

- Record 0005, its consequence that agents read decisions and rules
  locally, quickly and offline.
- Record 0008, point 2.
- `design/research/2026-11-15-planning-and-boom-storage/06-open-questions.md`,
  the similarity net, and `03-agent-context-packets.md`, long content by
  pointer.
- `interview.md`, the answers of the user of 2026-11-01:
  `caches-derived-only`, `flume-asks-what-applies` and
  `legacy-number-alias`.
- core.1, core.2 and core.6.

## Context and problem statement

`boom query` gives an agent each claim that stands for its repository,
replayed from the log (record 0008). But an agent at work asks about a
subject. Examples:

- What the project decided about amounts.
- Which rule the `core.2` of a page is.
- Whether another repository decided how it keeps refunds.

Nothing lets the agent find the few claims about a subject. Nothing lets
it look across repositories. The research did not decide whether to
search by meaning at first. A search by meaning puts an embedding of
each claim into a local vector index.

## Considered options

For how an agent finds claims:

1. By their words, in a full-text table. SQLite's FTS5 builds the table
   in memory from the replay of the judge, for each search. The table
   goes away with the answer (chosen).
2. By their words, in an FTS5 table that the store keeps among the
   views.
3. By meaning: embeddings of each claim in a local vector index. This is
   the similarity net of the research.
4. By substrings that Rust matches, with no stems and no ranks.

For what a search covers:

5. The claims that stand for the repository named or declared. Or, with
   `--all`, each claim that stands. With none of these, the search
   refuses (chosen).
6. Each repository, unless the agent names one.

For the command:

7. `boom search <words>...`, a command of its own (chosen).
8. `boom query <words>...`.

## Decision outcome

1. **Words, in memory, from the log.** For each answer, `boom search`
   replays the whole log, as `boom query` does. It puts each standing
   claim in scope into an FTS5 table, in a SQLite connection held in
   memory. The columns are the name, legacy id, statement, section,
   scenarios and body. The search drops the connection with the answer.

   Boom keeps nothing. Thus a deletion changes no answer, and no index
   falls behind the head (`caches-derived-only`, core.2). A view edited
   by hand changes nothing that a search finds (record 0008, point 2).
   FTS5 is in the SQLite that the tree already pins, so the search adds
   no tool or crate (core.6).

   Not taken:
   - An FTS5 table kept in the store (option 2). A program that edits
     the store can change that table to hide a claim from each search.
     Only a whole scan finds that change.
   - Meaning (option 3). It needs a model on each host. It needs an
     index kept, or rebuilt at the cost of a model. Its answers change
     with the model. The brief does not ask for it.
   - Substrings (option 4). They miss the other forms of a word and rank
     nothing.

   A later change can add a kept index under `$XDG_CACHE_HOME/boom/`,
   checked against the head. Or it can add the similarity net beside
   words. Neither changes the major of the format.
2. **Each word, as written.** A search finds a claim when each word
   searched is in it. A word matches its other forms by the Porter
   stemmer, with no difference for case or accents. The search sends
   each word as an FTS5 string. Thus FTS5 reads no word as an operator,
   and a stray quote or bracket of an agent never makes the search fail.
   A word that holds punctuation, such as `core.2`, matches as its parts
   in order. Thus a legacy id that a page still cites finds its record
   (`legacy-number-alias`).

   Not taken:
   - The words as an FTS5 query. That query fails on punctuation.
   - A match on any word. That fills the context of an agent with weak
     matches.
3. **Ranked, short, with a pointer.** The `bm25` of FTS5 ranks the
   claims. A word in a name or a legacy id weighs most. Then a statement
   or a section. Then scenarios. Then a body. Equal matches keep the
   order of `boom query`, kind then name, so the same log gives the same
   answer.

   A search gives the first ten, or `--limit`, with the number that
   matched. For each claim with a body, it gives the sixteen words of
   the body where the words searched are thickest. It gives the pointer
   `boom show <name>@<version>`, and never the whole text. The packets
   of the research also keep long content out.
4. **Only what stands.** A search gives the versions that stand, in the
   words that the person accepted. It never gives a version that is
   proposed, rejected, replaced or history.

   Not taken: proposals and history in the answer. An agent reads them
   as what holds.
5. **The repository it works in, or each repository.** A search covers
   the claims that stand for the repository that `--repository` names or
   that `.config/boom.toml` declares, as `boom query` reads it. It also
   covers the claims of the whole system. `--all` covers each claim that
   stands, and each claim says what it binds. With none of these, the
   search refuses, as `boom query` does.

   Not taken: each repository by default (option 6). That gives an agent
   the claims of other repositories, which it did not ask for.
6. **A command of its own.** Not taken: `boom query <words>` (option 8).
   The arguments of `boom query` are kept for paths
   (`flume-asks-what-applies`). A word and a path look the same.

## Consequences

- Good: an agent finds the claims about a subject on its machine,
  offline, as a person accepted them. Boom keeps nothing that can differ
  from the log.
- Good: a legacy citation of a page, `core.2` or `0005`, finds the record
  that boom now keeps.
- Bad: a search does not find a claim with other words than the words of
  the agent. The agent searches again, or reads the whole answer of
  `boom query`.
- Bad: the Porter stemmer knows only English. A search finds a word of
  another language only in its own form.
- Neutral: a search replays the whole log and reads each body in scope.
  Its cost is linear in both. A kept index can make it shorter later,
  with no change to an answer.

## Amendment of 2026-11-20 (proposed)

Status: proposed, for the user to accept. The change
`boom-keeps-its-views-in-step` makes it.

This amendment changes point 1 and the last consequence.

`boom search` no longer replays the whole log for each search. It reads
the standing claims from the views. The views are tables that boom keeps
in step with the log, as the amendment of record 0008 says.

`boom search` still builds its full-text table in memory for each
search. It keeps no index.

A search reads the last entry of the log and no older entry. Its cost
grows with the claims in scope. It does not grow with the log.

A view that another program edits changes what `boom search` finds,
until `boom verify` runs. The user accepted this on 2026-11-20.
