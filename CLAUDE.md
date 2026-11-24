# boom

A ledger of what a project has claimed about itself, invariants,
decisions, assumptions and terms, with every view of it (rules for a scope,
decision records, a design book, a context map) computed from the
ledger, and `boom check` reading the work against the claims that hold.
The crate keeps a store's claims in a log in one SQLite file on the
machine: an agent proposes, a person accepts or rejects with a security
key, and every answer is computed from the log. It imports a
repository's MADR records and rules page once, proposing each in its
file's own words for a person to accept together, attached to that
repository. A version binds the repositories its proposal names, or the
whole system, once a person accepts it, and `boom query` gives an agent
every claim that stands for its repository, named, set by its launcher
in `BOOM_REPOSITORY`, or matched by its git remote against the
repositories an owner registers with `boom repo add`, from views kept in
step with the log, which `boom verify` checks against a full replay, and
`boom search` finds the claims that stand by their words, in a
repository or every repository.
A version that amends a draft supersedes that draft, so a person
accepts the newest draft of a claim, on the nearest version in its line
of bases that a person accepted.
A version links to the claims that it rests on with `--source`, to
the claims that a decision changes with `--changes`, and to the file or
commit that it came from with `--origin`, a file that boom keeps by its
fingerprint or `git:<repository>@<commit>:<path>`, and to the
elaboration that made it with `--origin elaboration:<repository>/<name>`.
Each edge to a claim stores the version that it meant, and `boom show`
marks it out of date, gives the versions by base and the claims that
rest on a claim, and writes a kept file with `--artifact`.
A context owns path globs, each `<repository>:<pattern>`, and a version
binds a context with `--context`, which stores the version of the
context that it meant; `boom query --path <repository>:<path>` gives
the claims for a file, those of the whole system, of its repository and
of each context whose accepted globs match it, and `boom show` marks a
binding out of date. Until someone draws a context, each repository is
one context.
The Claude Code plugin gives an agent's session its claims in layers.
`boom hook session-start` gives the rules of the repository of the
session in full, the decisions, the assumptions and the contexts by
name, and a menu. `boom hook pre-tool-use` stops the first edit of a
file in a context in a session, gives the rules of that context with
their bodies, and the agent makes the edit again. Each hook shows the
person one line. Boom keeps the contexts that a session got in
`$XDG_STATE_HOME/boom/sessions/<session_id>/`, outside each repository
and the store.
A term is a word and its meaning in one repository, with the old words
that it replaces, which only the log keeps: `boom query` gives the term
and never its old words. `boom terms check` names each old word of an
accepted term in a file, reading the store and writing nothing;
`boom hook pre-tool-use` runs the same check on what an agent writes,
and the gate's `root:terms` on the files that a change adds or modifies. A line
that names an old word on purpose holds `boom: allow <old word>`.
`boom mcp` serves an agent's program the commands it may run,
`propose`, `show`, `log`, `query`, `search` and the imports, as MCP
tools, and none a person must sign: no tool accepts, rejects or
registers, and the server holds no key (record 0011). boom is released
by version: each landing on main with a new version is tagged
`v<x.y.z>` on a release commit off main that writes the version, and
boom's flake gives boom and its plugin from that tag at one version
(record 0010, core.8).
A narrative is a claim of the kind `narrative`: a title and a body of
Markdown, kept by its fingerprint, that includes claims with
`--includes`, each at the version it meant. A person accepts it with
the key, and no one checks it; no hook gives one. `boom page <name>`
serves its page on `127.0.0.1` under a token, computed from the log for
each request, where a person reads it and writes a note on a paragraph.
`boom note` writes the same note: an entry with the person's name and
no signature. A note stays open until a later version answers it with
`--answers <note>=<words>`, and boom refuses a version that leaves one
open. A note's elaboration is the one its version names as its origin,
and `boom query --elaboration <repository>/<name>` gives the open notes
of that elaboration. No tool of `boom mcp` writes a note or serves a page.
A narrative binds one level: the whole system, one repository or one
context. An outline is a claim of the kind `outline` that holds the
order of a view, `--view book`, with `--narrative`, each at the version
it meant; it binds the whole system, a person accepts it with the key,
and a store has one book. Boom computes the design book from the log:
Part I the narratives of the whole system and the contexts of two
repositories or more, Part II a chapter for each repository with its
contexts, each in the outline's order, then the narratives that it does
not hold, marked. `boom show <outline>` gives the book, `boom page` with
no name serves it, and `boom page --view <outline>` serves the view of
an outline. The other views, a rules page, decision records and a
context map, and `boom check` are not built yet.

## The family

**The family.** This tool works with the others of the Boomdrive
family: sluice (the merge queue), flume (places and their agents),
Boomdrive (the runner), timber, weir and boom. Each tool's statement is
in agentplot/blueprints, `design/<tool>/requirements.md`. Before
designing or fixing a behavior here, look at how the related tools do
the same thing, in their code and their statements: readiness,
configuration, credentials, retries, what a place is. Where two tools do
one thing, reconcile them into one design with one owner the others call
or cite, and say so in the design. Never fix one tool alone when the
cause is that the tools differ.

**Specs.** `openspec/specs/` changes only through an OpenSpec change: a
proposal, a delta spec, and tasks, validated with
`openspec validate --strict` and archived with `openspec archive`, so
the spec reconciles. This holds for a fix or a chore too. Where the
work changes a requirement, it carries that change in the same branch.
Never edit a spec file directly.

## Rules

The rules are in `docs/architecture/`: `core.md` binds wherever a change
is made, and an area page, when one exists, binds the paths its
frontmatter names. The decisions behind them are `docs/adr/`, one MADR
record each. A design or a commit that relies on a rule cites its id;
one that departs from a rule says which and why. A rule is sent to
`core.md` or its area page with an id, the next free number in its area,
amended in the same commit as the decision that changes it.

## Checks

Every tool runs through the shell, one command at a time:
`devenv shell -- <command>`. moon runs the checks, tagged by layer
(`.moon/workspace.yml`); `.config/sluice.toml` says which layers each
branch requires:

| layer | what | who requires it |
|---|---|---|
| `check-fast` | the crate's format (rustfmt), lint (clippy -D warnings), type check and tests; the plugin's format and lint (Biome), validation (Claude Code), its session and `PreToolUse` hook tests, and its test of a long stop on the pinned Claude Code; the flake's lint (statix), evaluation and cases; the site's format and lint (Biome), type check (astro check), diagrams and build; the root's format (treefmt), toolchain, terms (the old words of the accepted terms in the files a change touches), specs, open changes, each validated as it would archive, and that check's cases, versions, release workflow and marketplace | main |

Run suites through sluice, so the agents on this host take turns under
its bound: once your work is committed, `sluice exec main` for the
gate's checks, and while iterating `sluice exec main -- moon run
<project>:<task>`. The projects are those `.moon/workspace.yml` lists:
`boom` (`crates/boom`), `claude-code` (`plugins/claude-code`), `nix`
(`nix/`, with `flake.nix` at the root), `site` (`site/`, its own
`README.md`) and `root`. A quick, narrow call, `cargo check` or one
unit test, is fine to run directly. `devenv shell -- moon run :#check-fast` runs the fast
layer. `devenv shell -- prose` runs Vale over the pages and the README
for their writers; no task or workflow runs it, and none reads a
Markdown file for its words (core.5; record 0002).

## Commits

Stage by pathspec and commit the paths you wrote: `git add <paths>`, then
`git commit -m "<subject>" -- <paths>`. Never `-a`, never `add .`.
Conventional Commits subjects, scope the part of the tree touched.

@docs/architecture/core.md
