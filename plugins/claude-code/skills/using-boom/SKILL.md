---
name: using-boom
description: Read and cite the claims that Boom keeps for a repository, and know which acts are a person's. Use in a repository that Boom serves, when the session starts with a "Boom:" line, before a design or a change that a rule, a decision or a term can bind, when asked what Boom holds about a subject, or before any `boom` command. Load proposing-claims to propose, and writing-narratives to explain a design.
---

# Using Boom

Boom keeps a ledger of what a project claims about itself. A claim is an
invariant, a decision, an assumption, a term, a context, a narrative or
an outline. Each claim has one name for life and a version for each
change of its words. An agent proposes a version. A person accepts it
or rejects it with a security key. Only an accepted version binds work.

Boom computes each answer from its log. No file of the repository holds
the truth of a claim, and you never edit a view to change a claim.

## What your session already has

A session in a repository that Boom serves starts with one line, such as
`Boom: 77 claims for boom, at entry 720`. Under that line, the session
holds:

- each rule of the repository in full: its name, its version and its
  statement;
- the decisions, the assumptions and the contexts by name;
- a menu of the commands below.

Before your first edit of a file in a context, Boom stops the edit and
gives the rules of that context with their bodies. Read them, then make
the edit again. Boom also stops a write that holds an old word of an
accepted term. Use the term's word, or write `boom: allow <old word>` on
a line that names the old word on purpose.

## How to read the claims

Run each command from the repository. Add `--json` for one JSON object.

| To find | Run |
|---|---|
| Every claim that stands for this repository | `boom query` |
| The claims for one file, with those of its contexts | `boom query --path <repository>:<path>` |
| The claims whose words hold some words | `boom search <words>` |
| The same, in every repository | `boom search --all <words>` |
| The versions that wait, with their author and age | `boom search --proposed --all` |
| A claim in full: its versions, edges and body | `boom show <name>` |
| One version | `boom show <name>@<version>` |
| The claims that rest on a claim | `boom show <name>` (under `source of:`) |
| The old words of the accepted terms in files | `boom terms check <file>...` |
| The log, entry by entry | `boom log` |

`boom search` gives only the claims that stand, in the words that a
person accepted. A proposed version is not in its answer. To see a
proposed version, run `boom show <name>`.

`boom page <name>` serves the page of a narrative on `127.0.0.1` for a
person to read. `boom page` with no name serves the design book. These
pages are for the person. You read a narrative with `boom show`.

Read before you design. Before you write a design, a change or a
claim, run `boom search` for its subject in the repository and in
`--all`. A rule that you do not read still binds the work.

## How to cite a claim

Cite a claim by its name, or by `<name>@<version>` where the version
matters: `status-read-from-events@97d0559deafa`. Run `boom show` on each
name before you cite it. Never cite a claim by a number, a rule id, a
URL or a file path. A source that is not in Boom waits until someone
imports it.

A design or a commit that relies on a rule cites the rule. A design that
departs from a rule names the rule and says why.

## The acts that are a person's

You never run these. Each one is a person's act, signed with a key or
written in the person's name:

- `boom accept`: a person accepts versions, a narrative with its
  claims, a cause with its revision, or each batch that waits.
- `boom reject`: a person rejects one or more versions, with why.
- `boom note`: a person writes a note on a paragraph of a narrative.
- `boom init`, `boom register`, `boom repo add` and `boom repo set`:
  an owner sets up the store, its persons and its repositories.
- any command with `--key`.

When a version waits on a person, tell the person its name and its
version. When Boom finds no store or no repository, tell the person what
Boom said, and stop. Never make a store or a key yourself.

`boom mcp` serves the same commands that you may run, as tools: propose,
show, log, query, search and the imports. It serves no person's act.

## What you may write

You may propose versions. Load the skill `proposing-claims` before you
propose an invariant, a decision, an assumption, a term or a context.
Load it too before you run `boom supersede`. Run it only for a
duplicate, or for a claim that another claim replaces, and never to get
your own change through. When your work breaks an accepted claim, tell
the person.
Load `writing-narratives` before you write a narrative or an outline, or
before you propose many claims for one design.
