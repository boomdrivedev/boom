# Proposal

## Why

`boom query` gives an agent every claim that stands for its repository,
but an agent at work asks about a subject: what was decided about
amounts, which rule `core.2` is, whether any repository already settled
how refunds are kept. Today it can only read the whole answer, or every
claim one by one with `boom show`, and it has no way to look across
repositories at all. Record 0005 keeps the decisions and rules that
cross repositories in boom, so an agent needs to find them by their
words, on its machine and offline, and get the few that bear on its
subject, as a person accepted them. The research left open whether to
search by meaning with embeddings (`06-open-questions.md`, the
similarity net); this change searches by words, with no model and no
index kept.

## What Changes

- `boom search <words>...` gives the claims that stand whose words hold
  every word searched, in their name, statement, scenarios, body, and
  an imported record's legacy number or id and its section, a word's
  other forms found with it, case and accents aside.
- It searches the repository `--repository` names or `.config/boom.toml`
  declares, giving the claims that stand for it and the whole system's,
  or with `--all` every claim that stands, whatever it binds; with none
  of these it refuses, saying how to name one.
- It gives only versions that stand, in the words the person accepted,
  replayed from the log for each answer and searched in memory: no view
  table is read and no index is kept, so a view edited by hand changes
  nothing found, and deleting nothing changes an answer.
- Its answer ranks the claims by how well their words match, a word in
  a name or a legacy id above one in a body, gives the first ten or the
  number `--limit` names and how many matched in all, and gives for
  each claim with a body the passage where the words fall, with a
  pointer to the whole text, `boom show <name>@<version>`. In words it
  writes to standard output alone, for an agent's context; with
  `--json` it answers `boom-search/1`.
- A word is always a word: each is searched for as written, so an
  agent's words are never read as a query language and never refused
  as one.

## Capabilities

### New Capabilities

- `claim-search`: `boom search`, the words searched and where, one
  repository or every repository, only what stands as the person
  accepted it, replayed from the log with no index kept, and its ranked
  answer in JSON and in words.

### Modified Capabilities

None. `repository-rules` keeps `boom query`, whose answer this change
leaves as it is, and `command-line` its commands' common rules; the new
command and its format are `claim-search`'s, as `boom query` is
`repository-rules`'.

## Impact

- `crates/boom`: a new module `search` builds an in-memory SQLite FTS5
  table from the judge's replay for each answer and ranks it; `cli`
  gains `boom search`; the claims that stand for a scope are read by one
  helper `boom query` and `boom search` share. No new dependency: the
  bundled SQLite `rusqlite` builds has FTS5. The store's schema stays 1,
  and nothing is written to the store or a cache.
- `README.md` and `CLAUDE.md` say what `boom search` does.
- `docs/adr/0009-an-agent-finds-claims-by-their-words-searched-in-memory-from-the-log.md`
  records this change's choices.
- Out of this change: search by meaning, a kept index, versions
  proposed, replaced or history, filters by kind, `boom mcp` and the
  plugins, and the site's pages, which show what boom will do.
