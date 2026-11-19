# boom rules

The rules that bind wherever a change is made in this repository,
written so a reviewer can point at a diff and say which rule it breaks.
boom keeps a ledger of what a project has claimed about itself and
computes every view of it. There are no area pages yet; a rule that
binds only one part of the tree moves to an area page when that part
exists.

## Rules

### The ledger

- **[core.1]** A record's status is read from its events, never stored on the record, and an event is appended, never changed or removed.
- **[core.2]** A view, a scope's rules, a decision record, a design book or a context map, is computed from the ledger and never stored as the truth.

### Secrets and records

- **[core.3]** No token, key or credential is in the repository: none is committed, none is a default in code, configuration or tests.
- **[core.4]** Nothing commercial, no pricing, tier, customer or hosted service, appears in this repository.

### The gate

- **[core.5]** Every file the tree tracks is owned by a formatter the gate runs, or is excluded by name with its reason; no task, test, formatter or workflow reads a Markdown file of the tree for its words or its form (boomdrivedev/blueprints principle 16); every crate, every plugin, the flake and the site is a moon project of its own, listed in `.moon/workspace.yml` with its language declared, whose check-fast tasks format, lint, type-check and test it, one task per check.
- **[core.6]** Every tool a gate task runs is the version the tree pins, on every host that runs the gate.
- **[core.7]** The gate holds the specs and every open change to OpenSpec's own `openspec validate --strict`, and each open change to archiving cleanly: `openspec archive` applies its deltas, alone, to a scratch copy of the tree's specs, and every spec it writes passes `openspec validate --strict`. (Record 0004; boomdrivedev/blueprints principle 16 as amended on 2026-11-15.)

### Releases

- **[core.8]** A release's tag names a release commit off main that writes boom's version into every file that carries it; main's tree names `0.0.0`; a build takes its version from its tree alone; and boom's plugins come from boom's flake at boom's version. (Record 0010.)

## Details

- **core.1** Rules out: a status field on a claim; an event edited or deleted to change a status; a supersession that rewrites the claim it replaces. Not ruled out: a new event that supersedes, rejects or annotates. Source: record 0001.
- **core.2** Rules out: a rendered constitution, design book or context map committed and read back as input. Not ruled out: a view written to a build directory or published. Source: record 0001.
- **core.3** Rules out: a credential in a tracked file, a fixture or a commit message; a credential printed in output. Not ruled out: the name of the variable or file a credential is read from. Source: record 0001.
- **core.4** Rules out: pricing, a tier, a customer or a hosted plan named in code, docs, the site or commits. Source: record 0001.
- **core.5** Rules out: a file type treefmt neither formats nor excludes; an exclude with no comment saying why; a crate, a plugin under `plugins/`, the flake or the site with no project of its own; one task bundling several checks. Also rules out: a task, a test or a workflow that reads a Markdown file of the tree, a README, a site page, a spec or a record, for its prose, its numbers or its form, as `root:adrs` read the records' numbers and `site:prose` and the Pages workflow ran Vale over the pages until record 0002. Not ruled out: OpenSpec's validation of the specs and the open changes, the change tool's own reading of its format (principle 16 as amended 2026-11-15); `root:format`, whose treefmt excludes `*.md` and `*.mdx` by name; the site's build and type check, Astro's own reading of its content; the site's diagram test reading `docs/diagrams/boom.svg`; Claude Code's own validation of the plugin and the marketplace, `claude-code:typecheck` and `root:marketplace`, whose advice on a README changes no verdict (record 0010); Vale run by a writer from the shell, `prose`, which no task or workflow runs (record 0002). Source: record 0001; flume's core.21, the pattern this repository copies; boomdrivedev/blueprints principle 16 and record 0002, for Markdown.
- **core.6** Rules out: a tool a task finds outside its pin; a pinned toolchain without a component a task runs; two files pinning one tool with no check that fails when they differ. Source: flume's core.22.
- **core.7** Rules out: an open change that `openspec archive` refuses, such as one adding a requirement its spec already holds with other words or modifying one its spec does not hold; an open change that archives into a spec `root:specs` fails, such as a new capability with archive's placeholder Purpose or a MODIFIED requirement past OpenSpec's 500 characters; a change judged with the other open changes applied; the tree's specs or changes written by the check; a change to the check that lets either through, which `root:changes-cases` fails. Not ruled out: a change whose tasks are not all checked; a spec the change leaves as it was, which `root:specs` alone checks; two open changes that conflict, found by the gate on the branch that lands second; the check's cases, `root:changes-cases`, writing changes into scratch trees of their own as data that OpenSpec alone reads (record 0004 as amended 2026-11-16). Source: boomdrivedev/blueprints principle 16 as amended on 2026-11-15, on the user's answer on their card ("OpenSpec's own validation stays; the rule is about checks of what pages say"); record 0004 (as amended 2026-11-16); Boomdrive's core.61, the pattern this repository copies; flume's record 0073 and core.59, whose cases this repository copies.
- **core.8** Rules out: a file that carries boom's version giving other than `0.0.0` on main, or a version in the marketplace; a release commit committed to main, or a branch pushed by a release; a build given its version by an environment variable, an argument or `--impure`; a release's binary built by Nix, or run only where Nix is; a plugin built or versioned apart from boom's flake and its tree. Not ruled out: the release commit off main that only its tag names; a plugin installed from the marketplace at a release's tag, beside that release's binary, outside Nix; timber publishing boom, which retires the release commit by a change of this rule (timber record 0011). Source: record 0010; boomdrivedev/blueprints principles 14 and 17; boomdrivedev/boomdrive record 0023; sluice's record 0111 and core.55, the pattern this repository copies.
