# Design

## Context

`crates/boom` keeps a store's claims in a log in one SQLite file, and
one judge (`src/judge.rs`) replays the log into a `State`: each claim
with its versions, each version's `proposal` (name, kind, statement,
scenarios, body fingerprint), its `repositories`, its status and, for
an imported record, its `imported` part with `legacy` (`0003`, `core.1`)
and `section` (a rule's `###` heading). `boom query` (`src/cli.rs`)
replays the whole log inside `Store::read` and answers from that state
alone, never from the view tables (record 0008, point 2), giving every
claim that stands for a repository named or declared in
`.config/boom.toml` (`src/repository.rs`). A body is a blob in the
`blobs` table, read by `store::blob`, which refuses bytes that no
longer hash to their fingerprint. The bundled SQLite that `rusqlite`
builds (`libsqlite3-sys` 0.40.0, locked in `Cargo.lock`) is compiled
with `SQLITE_ENABLE_FTS5`.

The research left the similarity net open: whether to embed at all at
first, and if so in a derived local index (`06-open-questions.md`). Its
packets keep long text out, by pointer (`03-agent-context-packets.md`).
The interview's `caches-derived-only` lets boom keep an index beside
the ledger, or none, so long as deleting it changes no answer, and
reserves `boom query`'s arguments for paths (`flume-asks-what-applies`:
`boom query <paths> --json`).

The rules this design relies on: core.1 (a version stands by its
events, read from the replay), core.2 (the index a search ranks from is
computed for that answer and never stored), core.3 (the tests' keys
made when they run), core.5 (no test reads a Markdown file of the tree:
each page a test imports it writes into its own folder from a string in
its code) and core.6 (no new tool or crate: FTS5 is in the SQLite the
tree already pins). It reaches no control plane. Record 0009 records
the choices below that are this change's own.

## Goals / Non-Goals

**Goals:**

- An agent finds, by a few words, the standing claims about a subject,
  in its repository or every repository, the best matches first, each
  short enough for its context, with a pointer to the long text.
- The answer is computed from the log for each search, as `boom query`'s
  is, so nothing kept beside the log can change what an agent finds.
- An agent's words never fail as syntax.

**Non-Goals:**

- Search by meaning: embeddings, a model or a vector index.
- An index kept in the store or under `$XDG_CACHE_HOME/boom/`.
- Versions proposed, rejected, replaced or history, and filters by kind
  or status.
- Changing `boom query`'s answer, which its tests keep as it is.
- `boom mcp`, the Claude Code plugin's skill that would call this
  command, and the site's pages.

## Decisions

### 1. A command of its own

`Command::Search` takes `words`, one or more, positional and required,
`--repository <name>`, `--all`, which clap makes conflict with it, and
`--limit <n>`, a `u64` from 1, ten by default, beside the global
`--store` and `--json`. A clap refusal is exit 2 as for every command.
Not taken: `boom query <words>`, since `boom query`'s positional
arguments are kept for paths (`flume-asks-what-applies`), and a word
and a path cannot be told apart. A word that begins with `-` is read by
clap as a flag; an agent writes `--` before its words, as for any
command.

### 2. The words

The command first drops each word holding no character for which
`char::is_alphanumeric` is true, and refuses where none is left, exit
2, "boom search needs a word holding a letter or a digit", before it
reads a store or a declaration. Each word left is written into the
FTS5 query as a string, `"` + the word with each `"` doubled + `"`, and
the strings are joined by spaces, which FTS5 reads as AND: every word
must be found. Inside a string FTS5 reads no operator, so `OR`, `NOT`,
`NEAR`, `*`, `^`, `:`, `-` and parentheses are words or separators, and
a word holding punctuation, `core.2`, is the phrase of its parts,
`core 2`. Not taken: passing the words to `MATCH` as they are, which
fails on an agent's stray quote or bracket, and any word matching
(OR), which ranks well but floods an agent's context with weak matches;
an agent that finds nothing searches again with fewer words.

### 3. Searched in memory, from the replay

A new module, `src/search.rs`, holds the search, apart from the command
line. `cli::search` finds the scope (decision 4), opens the store,
which checks the chain as every command does, and inside `Store::read`
takes the log's rows and replays them with `judge::replay` into a state
of its own, as `boom query` does. For each standing version in scope it
reads the body, where it has one, with `store::blob`, whose refusal of
an altered body stops the search, exit 3, naming the fingerprint; a
body that is not UTF-8 is searched as no text.

`search::search` then opens a SQLite connection in memory,
`Connection::open_in_memory`, and fills one table:

```sql
CREATE VIRTUAL TABLE found USING fts5(
    name, legacy, statement, section, scenarios, body,
    tokenize = 'porter unicode61 remove_diacritics 2'
);
```

one row per claim in scope, its `rowid` the claim's place in the order
of decision 5, its scenarios joined by newlines, `legacy` and `section`
from the version's `imported`, empty where it has none. `unicode61`
splits a name's hyphens and folds case and accents, and `porter` takes
each word to its stem, so `Amounts` finds `amount` and `currencies`
finds `currency`. The connection is dropped with the answer: nothing is
written to the store, to `$XDG_CACHE_HOME` or anywhere else, so there
is no index to edit or to fall behind the head (`caches-derived-only`;
core.2), and a view edited by hand changes nothing found (record 0008,
point 2).

Not taken: an FTS5 table kept in the store among the views, which a
program editing the store could change to hide a claim from every
search, and which only a whole scan would find; an index under
`$XDG_CACHE_HOME/boom/`, which `caches-derived-only` allows and which a
later change can add, checked against the head, without changing an
answer; matching substrings in Rust, with no stems and no ranking;
embeddings (decision 7).

### 4. The scope

With `--all`, every standing claim. Otherwise the repository is
`repository::find(named, cwd)`, as `boom query` finds it, and the claims
are those whose standing version's `repositories` is empty or holds it.
With neither a name, a declaration nor `--all`, the command refuses,
exit 2, "boom search searches a repository: name it with --repository,
or in .config/boom.toml, or search every repository with --all"; a
declaration it cannot read is refused naming the file, as for
`boom query`. One helper in `judge.rs`, `State::standing_for(repository:
Option<&str>)`, gives each claim with its standing version, `None`
giving every one, in the order invariants, decisions, assumptions, each
kind by name; `boom query` takes its claims from it with the
repository, and its answer stays byte for byte as it is.

### 5. Ranked, then limited

```sql
SELECT rowid, snippet(found, 5, '', '', '…', 16)
FROM found WHERE found MATCH ?1
ORDER BY bm25(found, 10.0, 10.0, 5.0, 5.0, 2.0, 1.0), rowid
```

FTS5's `bm25` with a weight per column, in the table's order: a name or
a legacy id 10, a statement or a section 5, scenarios 2, a body 1, so
`boom search core.2` gives the rule `core.2` before a record citing it.
Lower is better, and claims that match equally fall back to `rowid`,
the order of decision 4's helper, so the same log gives the same order.
The command keeps every row for `total` and gives the first `--limit`.
No score is given: it would be a float in the answer, and the order is
what an agent needs.

The excerpt is FTS5's `snippet` of the body, column 5: the sixteen
tokens of the body where the words found fall thickest, `…` where the
body was cut, or its first sixteen where no word falls in it. Its runs
of whitespace become one space, so it is one line in words and in JSON.
A claim with no body text has no excerpt, `null`.

### 6. The answer

`boom-search/1`:

```json
{"format": "boom-search/1",
 "words": ["rounding"],
 "repository": "shop",
 "head": {"number": 9, "hash": "<64 hex>"},
 "total": 1,
 "claims": [{"name": "amounts-in-cents", "kind": "decision",
             "version": "9b04e7aa1c3f", "statement": "Amounts are kept in integer cents.",
             "scenarios": ["Given an amount, then it is a whole number of cents."],
             "body": "<64 hex>", "excerpt": "… so no rounding ever loses a cent between …",
             "legacy": "0003", "repositories": ["shop"],
             "accepted": {"by": "ada", "entry": 8}}]}
```

`words` are the words searched, as given, less those dropped;
`repository` is `null` with `--all`. Each claim holds `boom query`'s
fields, as it gives them, and `excerpt` and `legacy`.

In words, to standard output, standard error left empty:

```text
1 claim matches "rounding" for shop, at entry 9 of the store's log.
decision amounts-in-cents@9b04e7aa1c3f, for shop: Amounts are kept in integer cents.
  excerpt: … so no rounding ever loses a cent between …
  body: boom show amounts-in-cents@9b04e7aa1c3f
```

The first line reads `No claim matches`, `1 claim matches` or `<n>
claims match`; `across every repository` in place of `for <name>` with
`--all`; and, where `total` passes the limit, `; the first <limit>
follow` before its period. A claim's line names what it binds, its
repositories joined by `, `, or `the whole system`. Its scenarios are
in the JSON alone, to keep the words short; the body is a pointer,
never its text, as the research's packets have it.

### 7. Words, not meaning

This change searches words and nothing else, as its brief asks ("by
searching their words"), on the machine and offline (record 0005,
consequences). Not taken: the similarity net, embeddings over each
claim with a local vector index such as sqlite-vec. It needs a model
to embed with, pinned on every host (core.6), an index that is either
kept, and so edited or stale, or rebuilt at a model's cost on each
search, and answers that change with the model. A later change can add
it beside words, as one more way to find candidates, without changing
`boom-search/1`'s major.

### 8. The tests

`crates/boom/tests/search.rs` holds the claim-search scenarios, each
test named for its scenario and run through `boom::cli::run` in a
temporary folder with the tests' `SoftKeys` and `Place`. Each claim a
test proposes takes a scenario and a body written in the test so that
only the fields a scenario names hold the words it searches, since the
tests' `SCENARIO` holds `amount` and `currency`. A test that imports
writes its folder or page from strings in its code with
`Place::folder`, and its drafts file with `Place::file`. A test that
edits the views or the blobs opens the file with `rusqlite` itself, as
`query.rs` does. `src/search.rs` holds unit tests of the words and the
query string.

## Risks / Trade-offs

- [`porter` stems English alone] → a word in another language is found
  in its own form, its case and accents aside; the words are the
  person's, and most of a family's claims are in English.
- [An agent's words miss a claim worded otherwise] → it searches again,
  or reads `boom query`'s whole answer for its repository; the
  similarity net is a later change (decision 7).
- [A search replays the whole log and reads each body in scope] →
  linear in the log and in the bodies, small while a store holds
  decisions; a kept index under `$XDG_CACHE_HOME/boom/`, checked against
  the head, can come later without changing an answer.
- [A later `rusqlite` built without FTS5] → `search.rs`'s tests fail at
  the gate before it lands; at run time the command could not run, exit
  3.
- [An excerpt gives a body's opening where the words were in its name
  or statement] → that opening is still the record's own words; the
  pointer gives the whole text.
- [`--all` gives every repository's claims to an agent working in one]
  → the agent asks for them by name, and each claim says what it binds.
