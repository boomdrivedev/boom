# boom

**Built in part.** The binary keeps a store's claims in a log on the
machine, answers from it, and gives an agent the claims that stand for
the repository it works in. The views and `boom check` are designed,
and the site shows what boom will do.

A ledger of what a project has claimed about itself: its invariants,
its decisions and its assumptions, each recorded with who proposed it,
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
- **An agent proposes; a person accepts.** An agent's proposal names
  its provenance. Only a person accepts or rejects, and signs the entry
  with a security key the store registers to them, the person present
  (record 0006). `ssh-keygen -Y verify` checks that signature with no
  boom.
- **Every answer computed from the log.** boom reads whether a version
  is proposed, standing, replaced, rejected or history from the log. It judges
  each entry it reads as it judges a write, so an entry written by hand
  that breaks a rule counts for nothing.
- **A version binds a repository, or the whole system.** A proposal
  names the repositories its version binds with `--repository`, or the
  whole system with `--system`, and the version binds them once a
  person accepts it. Naming neither, an amendment binds what its base
  binds, and a first version binds the repository its working folder
  declares, else the whole system. The attachment is no part of the
  version's fingerprint, and `boom show` gives it before anyone accepts.
- **A repository declares itself.** Its `.config/boom.toml` holds one
  key, `repository`, a kebab-case name. boom reads the nearest one from
  the working folder up to the folder holding `.git`, and outside a
  repository reads none.
- **An agent asks boom for its rules.** `boom query` gives every claim
  that stands for a repository, as the person accepted it: its kind,
  name, version, statement and scenarios, and who accepted it. boom
  replays the log for each answer and reads no view, so a view edited
  by hand changes nothing an agent gets, and it writes nothing into the
  repository for anyone to edit (record 0008).
- **An agent finds claims by their words.** `boom search` gives the
  claims that stand whose words hold every word searched. A word may
  fall in a claim's name, statement, scenarios, body, legacy id or
  section, and its other forms are found with it: `currencies` finds
  `currency`. It searches the repository `--repository` names or
  `.config/boom.toml` declares, or every repository with `--all`. It
  ranks the claims, a word in a name above one in a body, and gives the
  first ten, or the number `--limit` names. Each comes with the passage
  of its body where the words fall and a pointer to the whole text.
  boom replays the log for each search and keeps no index, so nothing
  kept beside the log changes what an agent finds (record 0009).
- **A repository's records, imported once.** `boom import adr` reads a
  folder of MADR records and `boom import rules` a rules page, and
  proposes each record in its file's own words, with the names and
  scenarios the agent that runs the import drafts, each attached to the
  repository `--repository` names, so it stands for that repository
  alone. A person accepts them together, one signed entry each. From
  then on boom keeps them, and never takes a file again as the record
  (records 0007 and 0008).

```sh
boom init --person ada --key ~/.ssh/id_ed25519_sk
boom propose invariant money-has-currency "Every amount names its currency." \
  --scenario "Given an amount, then it names its currency." \
  --repository shop --provenance run-1
boom accept money-has-currency@<version> --key ~/.ssh/id_ed25519_sk
boom import adr docs/adr --repository shop --drafts drafts.json --provenance run-1
boom show money-has-currency --json
boom query --repository shop
boom search amounts cents --repository shop
boom log
```

`boom query` writes its words to standard output alone, so a harness
can put them in an agent's context as they are. In Claude Code, a
repository's `.claude/settings.json` names it as a `SessionStart` hook,
and the agent starts each session with the claims that stand for the
repository its `.config/boom.toml` declares:

```json
{
  "hooks": {
    "SessionStart": [
      { "hooks": [{ "type": "command", "command": "boom query" }] }
    ]
  }
}
```

Each command answers in words, or with `--json` as one JSON object
that names its format. It exits 0 when it answers, 2 when it refuses
and 3 when it cannot run, and on 2 and 3 it writes nothing.

## What it will do

- **Views computed.** boom computes the rules for a scope, the decision
  records in MADR, the design book in mdBook and the context map from
  the ledger. It writes none of them into a
  repository: a repository's agents get its rules from `boom query`
  (records 0005 and 0008).
- **Checked at the merge.** `boom check` reads a change against the
  standing claims it touches: scope first, then each claim's rule, then
  a reviewer model for the claims in prose.

## This repository

| path | what |
|---|---|
| `crates/boom` | the engine, a library, and `boom`, a thin command line over it: `init`, `register`, `propose`, `accept`, `reject`, `show`, `log`, `import`, `query` and `search` |
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
