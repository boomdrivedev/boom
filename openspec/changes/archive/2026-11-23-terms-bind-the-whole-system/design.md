## Context

A term is a claim of the kind `term`: a word, its meaning, and the old
words that it replaces, which only the log keeps. Today a term binds one
repository, and only one (`a-term-binds-one-repository@52e0b523d9a9`).
The state keys a claim by its name and, for a term, its repository. A
claim of another kind has the empty repository in its key, and binds
the whole system when its entry has no attachment.

On 2026-11-23 the user ruled that a term may bind the whole system, and
that `boom terms check` checks a term of the whole system in every
repository that Boom registers. The word "chain" stays allowed.

This change keeps these claims:

- `names-use-the-domains-terms@343c9fac7fc5`: a tool names things with
  the terms of its domain. A term of the whole system gives one word to
  every tool of the drive.
- `status-read-from-events@97d0559deafa` and
  `views-computed-from-the-ledger@36a056c1145b`: the term that stands
  for a repository is computed from the log, and nothing stores it.
- `no-task-reads-markdown@a25dda6425d7`: the check reads a file for the
  old words of the terms and for nothing else.
- `the-gate-validates-specs-strictly@d8e057526e31` and
  `open-changes-archive-cleanly@de24a2987568`: this change passes
  `openspec validate --strict` and archives alone.

## Goals / Non-Goals

**Goals:**

- A person accepts a term of the whole system, and each repository gets
  it.
- The check names an old word of a term of the whole system in a file of
  any repository.
- One rule says which term stands where one word has two.

**Non-Goals:**

- The terms `lineage` and `link` of the whole system. They are proposed
  after the release of this change.
- A term that binds a context, or two repositories.
- A refusal of one meaning of a word. The check matches words. Thus a
  term that refuses only one meaning of a word, such as "chain" for a
  hop of the lineage, does not list that word as an old word.

## Decisions

### 1. A term of the whole system has no attachment

A term of the whole system is proposed with `--system`. Its entry holds
no attachment, as a claim of another kind of the whole system does. The
state keys it by its word and the empty repository. Its fingerprint
holds no repository, so the content of a term of the whole system is
the content of a claim of another kind with the same fields.

A term with no binding named still needs one. In a folder of a
repository, it binds that repository. In a folder of no repository,
`boom propose term` refuses it and says to name `--repository` or
`--system`. A claim of another kind binds the whole system there; a term
does not, because a word of the whole system is a larger act than a word
of one repository.

Alternative: a field in the attachment that says "the whole system".
This changes the form of the log, and a claim of another kind already
binds the whole system with no attachment.

### 2. An amendment keeps its binding

An amendment of a term binds what its base binds. An amendment that
names another binding is refused: a term of `boom` does not amend the
term of the whole system, and a term of the whole system does not amend
a term of `boom`. A word moves from one repository to the whole system
as a new term of the whole system, and a person rejects the term of the
repository, or lets it stand over the term of the whole system.

### 3. Which term stands where one word has two

One word can have a term of the whole system and a term of a
repository. Both can stand. These are the options:

| Option | In the repository of its own term | Elsewhere |
|---|---|---|
| A. The repository's term stands there | the term of the repository: its meaning, its old words | the term of the whole system |
| B. The whole system's term stands | the term of the whole system; a term of a repository with its word is refused | the term of the whole system |
| C. Both stand | both meanings, and the old words of both | the term of the whole system |

**Recommendation: A.** A repository is a bounded part of the system,
and a word in it can mean something of its own. The accepted decision
already lets one word have a term in each of two repositories, each
with its meaning. `names-use-the-domains-terms` breaks on a term of the
domain used in another meaning "with no accepted term that gives that
meaning": a term of the repository is that accepted term. Under A, a
repository gives each word one meaning, as now.

B gives the drive one meaning for each word. But a term of the whole
system would then refuse a term that a repository already holds, or
wait on its rejection, and a repository loses the way to say what a
word means in it.

C gives an agent two meanings for one word in one repository, and the
check stops the old words of a meaning that the repository does not
use.

Under A, `boom query`, `boom search` with `--repository`, the hooks and
`boom terms check` drop the term of the whole system for a repository
where a term of that repository with the same word stands. `boom show`
and `boom search --all` give both. The judge computes this from the
state at each read, and stores nothing.

### 4. The check names "the whole system"

The check reads the accepted terms of the whole system with those of the
repository of each file, less each term of the whole system whose word
the repository's own term has (decision 3). A file is checked against
the terms of the whole system wherever the check finds its repository:
named, set by the launcher, or matched by a registered remote. A file of
no repository found is skipped or refused, as now.

In words, a found word of a term of the whole system says "in the whole
system" where a term of a repository says "in boom". In JSON, its
`repository` is `null`.

### 5. No transition

The store holds no term of the whole system, because boom refused one.
The terms that exist keep their keys, their fingerprints and their
verdicts.

## Risks / Trade-offs

- A term of the whole system binds repositories that do not know it
  yet. The gate of each repository checks only the files that a change
  touches, so an old word already in a file fails no gate until a change
  touches the file.
- Under decision 3, a term of a repository with the word of a term of
  the whole system hides the term of the whole system there. A person
  sees both in `boom show` before accepting either.
