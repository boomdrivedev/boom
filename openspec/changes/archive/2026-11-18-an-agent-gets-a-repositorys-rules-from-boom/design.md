# Design

## Context

`crates/boom` keeps a store's claims in a log in one SQLite file: an
agent proposes a version, a person accepts or rejects it with a
signature by their security key (record 0006), and every answer is
judged from the log by one judge (`src/judge.rs`), whose replay builds
the view tables `Store::read` answers from. Its commands are `init`,
`register`, `propose`, `accept`, `reject`, `show`, `log` and `import`
(`src/cli.rs`). Nothing says which repository a claim binds, and no
command gives the claims that stand for one.

An agent in a repository of this family reads its rules today from
pages its writers keep by hand: this repository's `CLAUDE.md` pulls in
`docs/architecture/core.md`, and the records are `docs/adr/`. Record
0005 retired the slice `boom sync` was to commit into each repository,
and left open how work reads the rules it must keep. The interview's
requirements that still stand say how the family asks: flume asks
`boom query` before work and puts what applies in the agent's task
(`flume-asks-what-applies`); a Claude Code plugin's hook gives the
agent the claims in scope before it edits (`hook-before-each-edit`),
the plugin a moon project of its own in `plugins/`
(`plugins-in-this-repository`); a claim's attachment moves only by a
person's accept and never changes its version
(`retire-attach-and-rules-need-a-person`); and a claim reaches a
repository through what it attaches to (`scope-crosses-by-homes`).
The research's agent packets keep long text out and give it by pointer,
`boom show <name>@<version>` (`03-agent-context-packets.md`).

The rules this design relies on: core.1 (an attachment, like a status,
read from the log), core.2 (the answer computed from the log each time,
and nothing written into the repository), core.3 (the tests' keys made
when they run), core.5 (no test reads a Markdown file of the tree: the
tests write each `.config/boom.toml` they read into their own folder)
and core.6 (the new dependency locked). It reaches no control plane.
Record 0008 records the choices below that are this change's own.

## Goals / Non-Goals

**Goals:**

- A version binds the repositories its proposal names, or the whole
  system, and binds them only once a person accepts it.
- `boom query` gives an agent, for the repository it works in, every
  claim standing there, as the person accepted it, computed from the log
  and written nowhere in the repository.
- Its words can be an agent's context as they are; its JSON is
  `boom-query/1`, for flume and a plugin to call.

**Non-Goals:**

- Paths, contexts and relationships: `boom query <paths>` and a claim
  attached to a context homed in a repository (`scope-crosses-by-homes`),
  which come with the context map.
- Moving the attachment of a version proposed or standing, and retiring
  a claim (`retire-attach-and-rules-need-a-person`).
- The Claude Code plugin and its hook, the herdr plugin and `boom mcp`,
  each a change of its own calling this command; flume putting the
  answer in an agent's task, flume's own change.
- `boom render` (a rules page, MADR records, a design book), `boom
  check`, and filters on `boom query` such as `--kind` or `--status`.
- Moving this repository's own `docs/architecture/` and `docs/adr/` into
  a store: it needs an import and a person's accept, acts no task does.
- A store shared between machines (record 0005, point 4).

## Decisions

### 1. The attachment in the entry

`entry.rs`'s proposed part gains `attach`, written after `claim` and
left out where it is `None`: `{"repositories": [<name>...]}`, an object
so a context can join it later as one more field. A proposal with no
attachment is written byte for byte as before, so entries stay
`boom-entry/1` and every entry written before reads as it did. The
signed part is read back refusing fields it does not know, as now.
`claim.rs` is untouched: a version's fingerprint is its content's, and
the attachment is no part of it (record 0008, point 3).

### 2. The judge holds the attachment

The judge's `Version` gains `repositories`, empty for the whole system,
from the entry that proposed it. Its rules for a proposal gain, after
the ones every proposal keeps: an `attach` holds at least one
repository, each kebab case by `claim`'s rule for names, in ascending
order with none twice. Where the version is one the claim has, the same
attachment is a repeat, adding nothing, as a repeat is now; another is
refused, "<name>@<version> is attached to <repositories>; a version's
attachment does not move". A repeat or a refusal is found before the
key signs (design 6 of the store's change), and a hand-written entry
that breaks a rule counts for nothing, `boom log` saying why.

### 3. What `boom propose` attaches

`Propose` gains `--repository <name>`, any number of times, and
`--system`, which clap makes conflict. The command sorts the names and
writes each once, and otherwise attaches, in this order (record 0008,
point 4): to the whole system with `--system`; for an amendment, to
what its base is attached to, read from the state the judge built; for
a first version, to the repository the working folder declares
(decision 4); else to the whole system. A declaration that cannot be
read is refused, naming the file, rather than binding the whole system.
A name not in kebab case is the judge's refusal, exit 2. The answer,
`boom-propose/1`, gains `repositories`, a field added within its major.
`boom show` gives `repositories` for each version, read from the
proposing entry, which the `versions` view names, so the store's schema
stays 1.

### 4. A repository declared

A new module, `src/repository.rs`, finds a repository's name. With
`--repository`, the name given, refused unless kebab case. Otherwise it
finds the nearest folder holding `.git`, a file or a folder, from the
working folder up; none found, it reads nothing. Then, from the working
folder up to that one, the nearest `.config/boom.toml` is read with the
`toml` crate into a struct that refuses unknown keys, holding
`repository`, kebab case; a file it cannot parse, or one holding
anything else, is refused, exit 2, naming the file and what it could
not read. Not finding a `.git` keeps a host's own `~/.config` from being
read as a repository's.

`cli::Env` gains `cwd`, the working folder: `src/main.rs` fills it from
the process, and the tests' `Place` with its own folder, which holds no
`.git`, so a test reads only the declarations it writes.

### 5. `boom query`

`Command::Query` takes `--repository <name>` beside the global
`--store` and `--json`. It finds the repository (decision 4), opens the
store, which checks the chain as every command does, and inside
`Store::read`, which rebuilds stale views as for every command, takes
the log's entries and replays them with `judge::replay` into a state of
its own. It answers from that state alone and reads nothing from the
view tables (record 0008, point 2): each claim with a standing version
whose `repositories` is empty or holds the repository, in the order
invariant, decision, assumption, each kind by name. It appends nothing.

`boom-query/1`:

```json
{"format": "boom-query/1",
 "repository": "shop",
 "head": {"number": 7, "hash": "<64 hex>"},
 "claims": [{"name": "money-has-currency", "kind": "invariant",
             "version": "3f9a1c2e4b5d", "statement": "Every amount names its currency.",
             "scenarios": ["Given an amount, then it names its currency."],
             "body": null, "repositories": ["shop"],
             "accepted": {"by": "ada", "entry": 5}}]}
```

In words, to standard output, standard error left empty:

```text
2 claims stand for shop, at entry 7 of the store's log.
invariant money-has-currency@3f9a1c2e4b5d: Every amount names its currency.
  - Given an amount, then it names its currency.
decision amounts-in-cents@9b04e7aa1c3f: Amounts are kept in integer cents.
  - Given an amount, then it is a whole number of cents.
  body: boom show amounts-in-cents@9b04e7aa1c3f
```

and `No claim stands for shop, at entry 7 of the store's log.` where
none does. A body is a pointer, never its text, as the research's
packets have it. A harness takes the words as they are, a Claude Code
`SessionStart` hook putting a command's standard output in the agent's
context; `README.md` shows the hook a repository names in its agent
settings.

### 6. The tests

`crates/boom/tests/attachments.rs` holds the claim-attachments
scenarios and `crates/boom/tests/query.rs` the repository-rules ones,
each test named for its scenario and run through `boom::cli::run` in a
temporary folder with the tests' `SoftKeys`. `Place` gains a way to
make a repository folder, a `.git` folder and, where the test gives it,
a `.config/boom.toml` from a string in the test's code, and to run a
command with that folder as `cwd`. A test that edits the views or
appends to the log opens the file with `rusqlite` itself, appending a
chained row with `entry`'s hash, as `log.rs` does.

### 7. An import's proposals attached to its repository

The change `boom-imports-a-repositorys-records-and-rules` landed in the
bolt before this one: it adds `imported` after a proposal's `claim`,
`history` to a version's content, and a required `--repository` to
`boom import` (record 0007). This change, landing second, merges the
bolt and reconciles them. The proposed part's fields are `claim`,
`imported`, `attach`, each after the one before and only where present,
so a proposal with neither is written as before. An import's proposals
carry `attach` naming the repository its `--repository` names, so a
repository's imported records and rules stand for it alone, and the
import's answer and `boom show` give that attachment before a person
accepts them. A `history` version never stands, so `boom query` never
gives it.

The judge holds an imported proposal to the import's rules first and
then to every proposal's, design 2's among them, so a record that came
in before as this version is a repeat only under the same attachment.
The judge does not
require an imported proposal's `attach` to name its `imported`
repository: the import writes it, the person reads `repositories`
before accepting, and an agent may propose for any attachment anyway
(record 0008, point 6).

## Risks / Trade-offs

- [No harness runs `boom query` on its own until the plugin and flume's
  launch do] → `README.md` shows the hook a repository names; both
  callers are changes of their own, calling one command whose JSON is
  versioned.
- [`.config/boom.toml` is in the tree a change edits, so a change could
  name another repository for its agent's context] → the context is
  advice to the agent, not the gate; `boom check`, a later change, reads
  nothing of the tree it trusts.
- [An agent proposing in a declared repository binds it there when it
  meant every repository] → the narrow failure: the person reads
  `repositories` in `boom show` before accepting, and `--system` says
  the whole system outright.
- [A version of the whole system reaches every repository's agents] →
  the person accepting it reads `repositories` as `[]`.
- [A query replays the whole log, verifying each person's signature] →
  linear in the log and small while a store holds decisions; a verified
  checkpoint can come later without changing an entry.
- [A program that rewrites the log and its chain from some entry on,
  dropping an accept, is not found on one machine] → as before: a
  forged accept still needs the person's key, and a drive holding every
  entry (record 0005, point 4) finds the rewrite.
- [The gate reads crates from cargo's registry with no network] → the
  task that adds `toml` fetches it outside the sandbox, as the store's
  change did.
