## Context

Boom has three kinds of claim: invariant, decision and assumption
(`claim.rs`). The user approved terms on 2026-11-20. These
are the sources of the design:

- the page "Terms by Context"
- the narrative, `design/narrative.md`, sections 5 and 6
- the rulings of the user `a-term-binds-one-repository` and
  `claims-cite-claims-by-name`
- the words of the user on 2026-11-01, that a list of words in the
  tree "would name what it forbids"

This change covers four of the places in the design where a term
works. They are the kind itself, load (`boom query`), write (the hook),
and the gate (the files that a change touches). The purge (a later turn) and
`boom sync` are not in this change.

## Goals / Non-Goals

**Goals:**

- A person accepts a word and its meaning as a claim, with the old
  words that it replaces.
- An agent gets only the right words.
- An agent that writes an old word is stopped at the write, and told
  the right word.

**Non-Goals:**

- Contexts. A term binds one repository, as each repository is one
  context until the map comes.
- The purge of old words from existing sources (`boom reconcile
  terms`, a later turn).
- A check of the words of a claim when an agent proposes it.
- A check of a file that no change touches. The purge finds
  old words in those.
- A check of a right word with a wrong meaning. That needs a judge.

## Decisions

### 1. A term is a claim of a fourth kind

The kind `term` goes beside the three kinds in `Kind`. Its name is its
word, in kebab case: `drive-log` is the word "drive log". Its statement
is its meaning. A body is optional, as for an invariant. The version,
the accept, the amendment and the status work as for each other kind.

Alternative: a record of its own, as the design of 2026-11-01 had it.
As a claim, a term shares the log, the accept and the views with the
other kinds.

### 2. `replaces` is a field of the proposal

The proposal entry gets `replaces`, a list of strings. Boom keeps each
word one time, in lowercase, sorted, so the same words give the same
version. The entry omits an empty list, as it omits empty scenarios.
The content of the fingerprint includes `replaces` only when the list
is not empty. So each version in a store now keeps its fingerprint.

`replaces` is in the content because the person signs it. A changed
list is a new version that a person accepts.

### 3. A term binds one repository

A first term version must bind exactly one repository. Boom refuses no
repository, the whole system and two or more. The rule of
`--repository` for a first version stays: with none, boom takes the
repository that the working folder declares. For a term, boom refuses
where none is declared, and does not fall back to the whole system.

### 4. A word can mean one thing in each repository

A term binds one repository, as the other kinds bind theirs. So one word
can have one term in `boom` and another term in `sluice`, with two
meanings. The names of the other kinds stay in one name space for the
store.

- Two terms can have one word when they bind two repositories. A term
  cannot have the word of a claim of another kind, and a claim of
  another kind cannot have the name of a term.
- The content of a term version includes its repository. So two terms
  with one word and one meaning in two repositories have two
  fingerprints, and `<word>@<version>` names one version.
- The views keep a term under its word and its repository. The other
  kinds stay under their names.
- `boom show <word>` gives each term of that word, one for each
  repository. `--repository <name>` gives the term of one repository.
- `boom accept`, `boom reject` and `--amends` name a version, so they
  find one term.

Alternative: one name space for all kinds, so one meaning for each word
in a store until contexts come. That is against the brief: a term binds
one repository, as the other kinds do.

### 5. Where the old words show

| Command | `replaces` |
|---|---|
| `boom show` | given: the person reads what they sign |
| `boom log` | given: the log shows each entry |
| `boom query` | never |
| `boom search` | never given, never searched |
| `boom terms check` | the old word that it found, and its term |
| `boom mcp` | `show` and `log` give it, as their commands do |

An agent loads its words from `boom query`, the session hook and a
brief. None of them gives an old word. An agent that calls `show` reads
the old word as an old word, beside the term that replaces it.

### 6. `boom terms check` reads the store and writes nothing

The check opens the store to read, and writes nothing, not even the
views. The gate runs in a sandbox that lets nothing write beside the
store, and a write hook must not change the store of a session.

- Where the views are at the last entry of the log, the check reads the
  accepted terms from them, as `boom query` reads the accepted versions.
- Where the views are of another form or behind the log, the check
  replays the log in memory.
- Where SQLite cannot share the memory file of the store, as in the
  sandbox, the check reads the store as a file that does not change.
  That holds only while no write waits in the write-ahead log of the
  store. Where a write waits, the check does not run, and exits 3.

The check reads each file as text, or the text of a hook event. A file
that is not text holds no word.

It finds the repository of each file as `boom query` finds the
repository of a session, with the folder of the file as the working
folder (`repository.rs`).

### 7. The matcher

The check splits a line into words, and a name in code into its words
at `_`, `-`, `.` and a change from lowercase to uppercase. It folds
case and accents, and takes each word to its stem with the same
tokenizer as `boom search` (SQLite FTS5, `porter unicode61
remove_diacritics 2`). An old word of two or more words matches its
words in order, on one line. A stem matches a whole word only, so
`gatekeeper` does not match `keeper`.

### 8. An old word found is a refusal

The check exits 2 when it finds an old word, as boom exits 2 when it
refuses. With `--json`, the answer is `boom-refused/1`, and
`refused.found` lists each old word. With no old word, the answer is
`boom-terms-check/1`. Claude Code stops a `PreToolUse` call whose hook
exits 2 and gives its standard error to the agent. So the hook needs
no script of its own.

### 9. The hook checks what the agent writes, not the whole file

A file can hold old words that are older than the term. The purge removes
them. If the hook checked the whole file, each edit of such a file
would stop. So `--hook` checks only the text that the tool writes:
`content` of `Write`, `new_string` of `Edit` and each `new_string` of
`MultiEdit`. It runs before the write, so the old word never reaches
the file.

### 10. The hook stops the write before it happens

The hook runs at `PreToolUse`. When the text holds an old word, Claude
Code stops the write, and the agent reads each old word and the term
that replaces it. The file never holds the old word. A hook at
`PostToolUse` lets the write happen and tells the agent after. Then the
old word is in the file until the agent edits it again.

### 11. The hook checks `Write`, `Edit` and `MultiEdit`, and not `Bash`

A `Bash` call can write a file, for example by a heredoc. The hook does
not read a command to find the text that it writes. The purge
finds an old word that a `Bash` call wrote.

### 12. The gate checks the files that a change touches

A gate task runs `boom terms check` on each file that a change touches,
beside the writer's hook. The hook stops an agent at the write. The gate
stops a change that a writer made in another way, such as a `Bash` call
or an editor. The task finds the files from the difference between the
change and the branch that it lands on. It skips the files that the
change deletes.

The check reads a Markdown file of the tree for the old words of the
accepted terms, and for nothing else. It is the check of the term tool
of its own words. In the same way, OpenSpec's validation is the check
of the change tool of its own format. It is not a check of what a page says. A new
version of `no-task-reads-markdown`, `no-task-reads-markdown@a25dda6425d7`,
names this check under "Does not break it", with a Before and Now. The
user accepts or rejects it, and this change lands only after the user
accepts it.

The gate task names the repository, `--repository boom`, and declares
the last entry of the log of the store as an input, as the user
approved. A task `root:terms-head` runs each time. It writes the entry
that `boom terms check` reads, its number and hash, to
`.boom-terms/head.json`, which git ignores. `root:terms` depends on it
and names that file among its inputs. So moon replays a pass only for
the same files and the same entry. Where there is no store, no term is
accepted: the task says so and passes.

Alternative: the hook alone. The user reviewed that design on
2026-11-21 and asked for the gate.

### 13. A marker allows an old word on purpose

A line that holds `boom: allow <old word>` may name that old word. The
check skips that word on that line. It checks each other word of the
line. The marker can be in a comment of code, or in the text of a
page, such as a note on the history of a word. A reviewer sees the
marker in the diff of the change. The hook reads the marker in the text
that an agent writes, as the gate reads it in a file.

Alternative: a list of the files or the lines that may name an old
word. That list names what it allows, far from the line, and a
reviewer does not see it beside the word.

### 14. History is out of scope

The gate task skips `openspec/changes/archive/`. An archived change is
the history of a change that landed, and it never changes. A record of
history in the log keeps its words, because the check reads files and
not the log.

### 15. No Markdown read by a test

The tests of boom write their files into a scratch folder as data. No
test reads a Markdown file of the tree.

## Risks / Trade-offs

- [A stem matches too much] → The porter stem of a word can match a
  word with another meaning. The check names the line, and the person
  can amend the term.
- [An old word that a page must quote] → The marker of decision 13
  allows it on its line. A term can also explain its old word in its
  body. That body is in the log, not in a file, so no check sees it. A
  file that must quote an old word, such as a record of history, gets
  no exception in this change. The purge decides how a record of history
  keeps its words.
- [MCP agents] → An agent that writes through MCP and not through
  Claude Code gets no hook. `terms check` is absent from MCP, because
  its writer runs it, and not the agent.
- [An older boom] → A boom from before terms reads a term proposal as
  an entry that fails its checks, and counts it for nothing. Each host
  that opens the store needs this release or later. The views change
  their form for terms. When an older boom and this boom take turns on
  one store, each builds the views again in its own form, one full
  replay each time.
- [A write that waits] → In the sandbox of the gate, the check does not
  run while a write waits in the write-ahead log of the store. A boom
  command checkpoints the log when it closes, so the gate runs again.
