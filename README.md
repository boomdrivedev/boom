# boom

**Built in part.** The binary keeps a store's claims in a log on the
machine, answers from it, and gives an agent the claims that stand for
the repository it works in. The views and `boom check` are designed,
and the site shows what boom will do.

A ledger of what a project has claimed about itself: its invariants,
its decisions, its assumptions and its terms, each recorded with who proposed it,
who accepted it and what it replaced. Its rules for a scope, its
decision records, its design book and its context map are views
computed from the ledger, and `boom check` reads the work against the
claims that hold.

<p align="center"><img src="docs/diagrams/boom.svg" alt="An agent runs boom propose and a person runs boom accept; each appends an event to the ledger, where each record's status is read from its events. The views, the rules for a scope, the decision records, the design book, the context map and boom check, are computed again from the ledger after each accept, and a superseded claim leaves them." width="900"></p>

- **The site:** https://agentplot.github.io/boom

## What it does

- **One store on the machine.** A store is one SQLite file,
  `boom/boom.sqlite` under `$XDG_DATA_HOME` unless `--store` names
  another. Its log holds the truth: boom appends each entry and never
  edits one, and each entry carries the hash of the one before it, so
  boom finds an entry changed or removed.
- **Claims by name, versions by fingerprint.** A claim, an invariant, a
  decision or an assumption, keeps its name for life and gains a
  version for each change, named by the fingerprint of its words. An
  amendment names the version it amends. boom keeps a long body once,
  under the fingerprint of its bytes.
- **A term is a word and its meaning in one repository.** `boom propose
  term <word> <meaning> --repository <name>` proposes one, and each
  `--replaces <old word>` names an old word that the term replaces. A
  term binds one repository, and only one, so one word can have a term
  in each repository, with a meaning for each. A person signs the old
  words with the term. boom keeps them in the log alone: `boom show`
  gives them to the person who accepts, and `boom query` and `boom
  search` never give them. No list of old words goes into a repository.
- **Each kind needs its own fields.** Every claim needs a statement of
  one line. A decision also needs a body: its context, options, outcome
  and consequences. An invariant's body is optional and says what it
  rules out. An assumption's body is optional and says what would
  challenge it. A new version has no scenarios. An old version keeps
  the scenarios it was written with.
- **An agent proposes; a person accepts.** An agent's proposal names
  its provenance. Only a person accepts or rejects, and signs the entry
  with a security key the store registers to them, the person present
  (record 0006). `ssh-keygen -Y verify` checks that signature with no
  boom.
- **Every answer computed from the log.** boom reads whether a version
  is proposed, accepted, superseded, rejected or deprecated from the log.
  A record that came in as history keeps its file's own status. It judges
  each entry it reads as it judges a write, so an entry written by hand
  that breaks a rule counts for nothing.
- **Views kept in step with the log.** Each write updates the views in
  the same transaction. A command checks the log's last entry against
  the views, applies any entries after them, and answers from the
  views. It reads no older entry, so it does not slow down as the log
  grows. `boom verify` checks the whole chain, replays the whole log,
  and builds the views again, naming any view that differed.
- **A version binds a repository, or the whole system.** A proposal
  names the repositories its version binds with `--repository`, or the
  whole system with `--system`, and the version binds them once a
  person accepts it. Naming neither, an amendment binds what its base
  binds, and a first version binds the repository of the session, else
  the whole system. The attachment is no part of the
  version's fingerprint, and `boom show` gives it before anyone accepts.
- **The repository of a session.** boom takes the one `--repository`
  names. Else it takes the one `BOOM_REPOSITORY` names, which a launcher
  such as flume sets. Else it takes the registered repository whose
  remote matches a remote of the git repository that holds the working
  folder. No file in a repository names it: boom reads no
  `.config/boom.toml`.
- **An owner registers each repository.** `boom repo add <name> --remote
  <url> --key <key>` registers a repository and its remote, as an act
  that the owner signs in the log. `boom repo set` changes its remote or
  its delivery at session start, on or off. `boom repo show` gives each
  one, with the entry that set it.
- **An agent asks boom for its rules.** `boom query` gives every claim
  that stands for a repository, as the person accepted it: its kind,
  name, version and statement, and who accepted it. boom
  answers from views it keeps in step with the log, and writes nothing
  into the repository for anyone to edit (record 0008).
- **An agent finds claims by their words.** `boom search` gives the
  claims that stand whose words hold every word searched. A word may
  fall in a claim's name, statement, scenarios, body, legacy id or
  section, and its other forms are found with it: `currencies` finds
  `currency`. It searches the repository of the session, or every
  repository with `--all`. It
  ranks the claims, a word in a name above one in a body, and gives the
  first ten, or the number `--limit` names. Each comes with the passage
  of its body where the words fall and a pointer to the whole text.
  boom searches the views and keeps no index (record 0009).
- **An old word is stopped where it is written.** `boom terms check
  <file>...` names each old word of an accepted term in each file, with
  its line and the term that replaces it, and exits 2 where it finds
  one. It finds the repository of each file as `boom query` finds the
  repository of a session. It matches an old word in another form, case
  or accent, an old word of two words in their order, and the words of
  a name in code, such as `RefundTotal`, but never an old word inside a
  longer word. A line that holds `boom: allow <old word>` may name that
  old word on purpose. The check reads the store and writes nothing,
  not even the views, so it runs where nothing may write beside the
  store.
- **A repository's records, imported once.** `boom import adr` reads a
  folder of MADR records and `boom import rules` a rules page, and
  proposes each record in its file's own words, with the names the
  agent that runs the import drafts, each attached to the
  repository `--repository` names, so it stands for that repository
  alone. A person accepts them together, one signed entry each. From
  then on boom keeps them, and never takes a file again as the record
  (records 0007 and 0008).
- **An agent's program calls boom's commands as tools.** `boom mcp` is
  an MCP server on standard input and output, one JSON-RPC message a
  line. Its tools are `propose`, `show`, `log`, `query`, `search`,
  `import_adr` and `import_rules`: each takes its command's arguments under their
  names and answers its command's `--json` object, a refusal or a
  command that cannot run marked an error. The server refuses a call
  naming an argument its command does not take. No tool accepts,
  rejects, registers a person or makes a store, no tool takes a `key`,
  and the server holds no key, so no call sets off a person's security
  key (record 0011). Its instructions, and `boom propose`'s answer,
  give the `boom accept` a person runs in their own terminal.

```sh
boom init --person ada --key ~/.ssh/id_ed25519_sk
boom propose invariant money-has-currency "Every amount names its currency." \
  --repository shop --provenance run-1
boom accept money-has-currency@<version> --key ~/.ssh/id_ed25519_sk
boom import adr docs/adr --repository shop --drafts drafts.json --provenance run-1
boom show money-has-currency --json
boom query --repository shop
boom search amounts cents --repository shop
boom log
```

`boom query` writes its words to standard output alone, so a harness
can put them in an agent's context as they are. boom's Claude Code
plugin does it: its `SessionStart` hook runs `boom query --if-declared`,
so every session in a registered repository starts with the claims that
stand for it. Its `PreToolUse` hook runs `boom terms check --hook
--if-declared` on the text that `Write`, `Edit` or `MultiEdit` writes,
so a write that holds an old word is stopped, and the agent reads the
term to write in its place. The text that a `Bash` command writes is not
checked. Its first line names the repository, the number of claims
and the entry. A session anywhere else, or in a repository whose
delivery at session start is off, hears nothing. A repository writes no
hook into its own `.claude/settings.json`. With `--if-declared`, where
boom finds no repository of the session, it answers nothing and exits
0. A name from `BOOM_REPOSITORY` that it cannot read, it still refuses.

A repository's `.mcp.json` names `boom mcp`, and the harness starts it
in the repository, so `query` answers for the repository of the
session and `propose` attaches a first version to it:

```json
{ "mcpServers": { "boom": { "command": "boom", "args": ["mcp"] } } }
```

Each tool runs on the store `boom mcp` starts with, and no call names
another. Where the harness's environment differs from the person's
terminal, its `$XDG_DATA_HOME` or `HOME` naming another store, the
`.mcp.json` names the store: `"args": ["mcp", "--store", "<path>"]`.

Each command answers in words, or with `--json` as one JSON object
that names its format. It exits 0 when it answers, 2 when it refuses
and 3 when it cannot run, and on 2 and 3 it writes nothing.

## Install

boom is released by version. Each landing on main in which the
Conventional Commits since the last release make a new version is
released as `v<x.y.z>`: its tag names a commit that writes that version
into the tree, so boom and its plugin built from the tag say it, given
nothing (record 0010).

With Nix, install a release from its tag. boom's flake gives the plugin
from the same tag as `boom-claude-plugin`, the plugin's folder at boom's
version, for a host's configuration to hand to Claude Code:

```sh
nix profile install github:boomdrivedev/boom/v<x.y.z>
nix build github:boomdrivedev/boom/v<x.y.z>#boom-claude-plugin
```

Without Nix, take `boom-<system>.tar.gz` from the release's GitHub page
for `aarch64-darwin`, `aarch64-linux` or `x86_64-linux`, put `boom` on
your `PATH`, and add the marketplace at the same tag with Claude Code's
own command:

```sh
claude plugin marketplace add https://github.com/boomdrivedev/boom.git#v<x.y.z>
claude plugin install boom@boom
```

The plugin's hook runs the `boom` on `PATH`, so install both from one
tag.

## What it will do

- **Views computed.** boom computes the rules for a scope, the decision
  records in MADR, the design book in mdBook and the context map from
  the ledger. It writes none of them into a
  repository: a repository's agents get its rules from `boom query`
  (records 0005 and 0008).
- **Checked at the merge.** `boom check` reads a change against the
  accepted claims it touches: scope first, then each claim's rule, then
  a reviewer model for the claims in prose.

## This repository

| path | what |
|---|---|
| `crates/boom` | the engine, a library, and `boom`, a thin command line over it: `init`, `register`, `propose`, `accept`, `reject`, `show`, `log`, `verify`, `import`, `query`, `search`, `repo`, `terms check` and `mcp` |
| `plugins/claude-code` | boom's Claude Code plugin, its session hook running `boom query --if-declared` and its write hook running `boom terms check --hook --if-declared`, listed by the marketplace in `.claude-plugin/` |
| `check-terms.sh` | the gate's terms task: `boom terms check` on the files that a change adds or modifies |
| `flake.nix`, `nix/` | boom's flake, giving `boom` and `boom-claude-plugin` at one version, and its offline checks |
| `.github/workflows/release.yml` | the release: a landing with a new version, tagged on a release commit off main, its binaries built by cargo |
| `site/` | the site, Astro with Starlight; its diagrams are drawn at build time (`site/README.md`) |
| `docs/architecture/core.md` | the rules every change keeps |
| `docs/adr/` | the decisions behind them, one MADR record each |
| `docs/diagrams/` | the README's diagram, generated from the site's |

```sh
devenv shell -- cargo run -p boom -- --version
devenv shell -- moon run :#check-fast   # every check in the fast layer
devenv shell -- site-build              # the site, through moon
devenv shell -- prose                   # the prose, through Vale, for its writers
```

## License

MIT
