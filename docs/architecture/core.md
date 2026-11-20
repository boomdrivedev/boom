# boom rules

These rules bind each change in this repository. A reviewer can point at
a diff and say which rule it breaks. boom keeps a ledger of what a
project claims about itself, and computes each view of it. There are no
area pages yet. A rule that binds only one part of the tree moves to an
area page when that part exists.

## Rules

### The ledger

- **[core.1]** A record never holds its status: boom reads the status from the events of the record. boom appends an event, and never changes or removes it.
- **[core.2]** boom computes each view from the ledger: the rules of a scope, a decision record, a design book or a context map. Nothing stores a view as the truth.

### Secrets and records

- **[core.3]** The repository holds no token, key or credential. No commit holds one, and no default in code, configuration or tests is one.
- **[core.4]** This repository holds nothing commercial: no pricing, tier, customer or hosted service.

### The gate

- **[core.5]** A formatter that the gate runs owns each file that the tree tracks, or the formatter excludes the file by name, with its reason. No task, test, formatter or workflow reads a Markdown file of the tree for its words or its form (boomdrivedev/blueprints principle 16). Each crate, each plugin, the flake and the site is a moon project of its own. `.moon/workspace.yml` lists each such project and declares its language. The check-fast tasks of the project format, lint, type-check and test it, with one task for each check.
- **[core.6]** On each host that runs the gate, each tool that a gate task runs is the version that the tree pins.
- **[core.7]** The gate holds the specs and each open change to OpenSpec's own `openspec validate --strict`. The gate also holds each open change to a clean archive. For this, `openspec archive` applies the deltas of that change alone to a scratch copy of the tree's specs. Each spec that the archive writes passes `openspec validate --strict`. (Record 0004, and boomdrivedev/blueprints principle 16 as amended on 2026-11-15.)

### Releases

- **[core.8]** The tag of a release names a release commit off main. That commit writes boom's version into each file that carries the version. The tree of main names `0.0.0`. A build takes its version from its tree alone. boom's plugins come from boom's flake, at boom's version. (Record 0010.)

## Details

Each rule below gives its sentence, then what breaks it, what does not
break it, and its source. The lines of a rule have no blank line
between them, so that `boom import rules` reads them all as the body of
the rule.

- **core.1** A record never holds its status.
  Breaks the rule:
  - a status field on a claim
  - an edit or a deletion of an event that changes a status
  - a supersession that rewrites the claim that it replaces
  Does not break it:
  - a new event that supersedes, rejects or annotates
  Source: record 0001.
- **core.2** boom computes each view from the ledger.
  Breaks the rule:
  - a rendered constitution, design book or context map that a commit holds and that a program then reads again as input
  Does not break it:
  - a view that a program writes to a build directory
  - a view that a program publishes
  Source: record 0001.
- **core.3** The repository holds no token, key or credential.
  Breaks the rule:
  - a credential in a tracked file, a fixture or a commit message
  - a credential that a program prints in its output
  Does not break it:
  - the name of the variable or the file from which a program reads a credential
  Source: record 0001.
- **core.4** This repository holds nothing commercial.
  Breaks the rule:
  - pricing, a tier, a customer or a hosted plan that code, docs, the site or a commit names
  Source: record 0001.
- **core.5** A formatter that the gate runs owns each file that the tree tracks, or the formatter excludes the file by name, with its reason.
  Breaks the rule:
  - a file type that treefmt does not format and does not exclude
  - an exclude with no comment that says why
  - a crate, a plugin under `plugins/`, the flake or the site with no project of its own
  - one task that bundles several checks
  - a task, test or workflow that reads a Markdown file of the tree for its prose, its numbers or its form, such as a README, a site page, a spec or a record
  - `root:adrs`, which read the numbers of the records until record 0002
  - `site:prose` and the Pages workflow, which ran Vale over the pages until record 0002
  Does not break it:
  - OpenSpec's validation of the specs and the open changes, the change tool's own reading of its format (principle 16 as amended 2026-11-15)
  - `root:format`, whose treefmt excludes `*.md` and `*.mdx` by name
  - the build and the type check of the site, Astro's own reading of its content
  - the diagram test of the site, which reads `docs/diagrams/boom.svg`
  - Claude Code's own validation of the plugin and the marketplace, `claude-code:typecheck` and `root:marketplace`, whose advice on a README changes no verdict (record 0010)
  - Vale when a writer runs it from the shell as `prose`, which no task or workflow runs (record 0002)
  Source: record 0001, and flume's core.21, the pattern that this repository copies. For Markdown, also boomdrivedev/blueprints principle 16 and record 0002.
- **core.6** On each host that runs the gate, each tool that a gate task runs is the version that the tree pins.
  Breaks the rule:
  - a tool that a task finds outside its pin
  - a pinned toolchain without a component that a task runs
  - two files that pin one tool, when no check fails if the two pins differ
  Source: flume's core.22.
- **core.7** The gate holds the specs and each open change to OpenSpec's own `openspec validate --strict`.
  Breaks the rule:
  - an open change that `openspec archive` refuses, such as a change that adds a requirement that its spec already holds with other words
  - an open change that modifies a requirement that its spec does not hold
  - an open change that archives into a spec that `root:specs` fails, such as a new capability with the placeholder Purpose that the archive writes
  - a MODIFIED requirement past OpenSpec's 500 characters
  - a judgment of a change with the other open changes applied
  - a check that writes the specs or the changes of the tree
  - a change to the check that lets either kind of change pass, which `root:changes-cases` fails
  Does not break it:
  - a change whose tasks are not all checked
  - a spec that the change leaves as it was, which only `root:specs` checks
  - two open changes that conflict, which the gate finds on the branch that lands second
  - the cases of the check, `root:changes-cases`, which write changes into scratch trees of their own as data that only OpenSpec reads (record 0004 as amended 2026-11-16)
  Source: boomdrivedev/blueprints principle 16 as amended on 2026-11-15, from the user's answer on their card: "OpenSpec's own validation stays; the rule is about checks of what pages say". Also record 0004 (as amended 2026-11-16), Boomdrive's core.61, the pattern that this repository copies, and flume's record 0073 and core.59, whose cases this repository copies.
- **core.8** The tag of a release names a release commit off main.
  Breaks the rule:
  - a file on main that carries boom's version and gives a version other than `0.0.0`
  - a version in the marketplace
  - a release commit on main
  - a branch that a release pushes
  - a build that takes its version from an environment variable, an argument or `--impure`
  - a release binary that Nix builds, or that runs only where Nix is
  - a plugin that is built or versioned apart from boom's flake and its tree
  Does not break it:
  - the release commit off main that only its tag names
  - a plugin that is installed from the marketplace at the tag of a release, beside the binary of that release, outside Nix
  - timber to publish boom, through a change of this rule that retires the release commit (timber record 0011)
  Source: record 0010, boomdrivedev/blueprints principles 14 and 17, and boomdrivedev/boomdrive record 0023. Also sluice's record 0111 and core.55, the pattern that this repository copies.
