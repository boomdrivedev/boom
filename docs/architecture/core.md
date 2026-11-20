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

- **core.1** Rules out: a status field on a claim. Rules out: an edit or a deletion of an event that changes a status. Rules out: a supersession that rewrites the claim that it replaces. Not ruled out: a new event that supersedes, rejects or annotates. Source: record 0001.
- **core.2** Rules out: a rendered constitution, design book or context map that a commit holds and that a program then reads again as input. Not ruled out: a view that a program writes to a build directory, or publishes. Source: record 0001.
- **core.3** Rules out: a credential in a tracked file, a fixture or a commit message. Rules out: a credential that a program prints in its output. Not ruled out: the name of the variable or the file from which a program reads a credential. Source: record 0001.
- **core.4** Rules out: pricing, a tier, a customer or a hosted plan that code, docs, the site or a commit names. Source: record 0001.
- **core.5** Rules out: a file type that treefmt does not format and does not exclude. Rules out: an exclude with no comment that says why. Rules out: a crate, a plugin under `plugins/`, the flake or the site with no project of its own. Rules out: one task that bundles several checks. It also forbids a task, test or workflow that reads a Markdown file of the tree for its prose, its numbers or its form. Examples of such a file are a README, a site page, a spec and a record. Until record 0002, `root:adrs` read the numbers of the records, and `site:prose` and the Pages workflow ran Vale over the pages. Not ruled out: OpenSpec's validation of the specs and the open changes. That validation is the change tool's own reading of its format (principle 16 as amended 2026-11-15). Not ruled out: `root:format`, whose treefmt excludes `*.md` and `*.mdx` by name. Not ruled out: the build and the type check of the site, which are Astro's own reading of its content. Not ruled out: the diagram test of the site, which reads `docs/diagrams/boom.svg`. Not ruled out: Claude Code's own validation of the plugin and the marketplace, `claude-code:typecheck` and `root:marketplace`. Their advice on a README changes no verdict (record 0010). Not ruled out: Vale when a writer runs it from the shell as `prose`, which no task or workflow runs (record 0002). Source: record 0001, and flume's core.21, the pattern that this repository copies. For Markdown, also boomdrivedev/blueprints principle 16 and record 0002.
- **core.6** Rules out: a tool that a task finds outside its pin. Rules out: a pinned toolchain without a component that a task runs. Rules out: two files that pin one tool, when no check fails if the two pins differ. Source: flume's core.22.
- **core.7** Rules out: an open change that `openspec archive` refuses. An example is a change that adds a requirement that its spec already holds with other words. Another example is a change that modifies a requirement that its spec does not hold. Rules out: an open change that archives into a spec that `root:specs` fails. An example is a new capability with the placeholder Purpose that the archive writes. Another example is a MODIFIED requirement past OpenSpec's 500 characters. Rules out: a judgment of a change with the other open changes applied. Rules out: a check that writes the specs or the changes of the tree. Rules out: a change to the check that lets either kind of change pass, and `root:changes-cases` fails such a change to the check. Not ruled out: a change whose tasks are not all checked. Not ruled out: a spec that the change leaves as it was, which only `root:specs` checks. Not ruled out: two open changes that conflict: the gate finds the conflict on the branch that lands second. Not ruled out: the cases of the check, `root:changes-cases`, to write changes into scratch trees of their own. Those changes are data that only OpenSpec reads (record 0004 as amended 2026-11-16). Source: boomdrivedev/blueprints principle 16 as amended on 2026-11-15. The amendment came from the user's answer on their card: "OpenSpec's own validation stays; the rule is about checks of what pages say". Also record 0004 (as amended 2026-11-16), and Boomdrive's core.61, the pattern that this repository copies. Also flume's record 0073 and core.59, whose cases this repository copies.
- **core.8** Rules out: a file on main that carries boom's version and gives a version other than `0.0.0`. Rules out: a version in the marketplace. Rules out: a release commit on main, and a branch that a release pushes. Rules out: a build that takes its version from an environment variable, an argument or `--impure`. Rules out: a release binary that Nix builds, or that runs only where Nix is. Rules out: a plugin that is built or versioned apart from boom's flake and its tree. Not ruled out: the release commit off main that only its tag names. Not ruled out: a plugin that is installed from the marketplace at the tag of a release, beside the binary of that release, outside Nix. Not ruled out: timber to publish boom, through a change of this rule that retires the release commit (timber record 0011). Source: record 0010, boomdrivedev/blueprints principles 14 and 17, and boomdrivedev/boomdrive record 0023. Also sluice's record 0111 and core.55, the pattern that this repository copies.
