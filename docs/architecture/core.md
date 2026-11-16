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

- **[core.5]** Every file the tree tracks is owned by a formatter the gate runs, or is excluded by name with its reason; no task, test, formatter or workflow reads a Markdown file of the tree for its words or its form (boomdrivedev/blueprints principle 16); every crate, and the site, is a moon project of its own, listed in `.moon/workspace.yml` with its language declared, whose check-fast tasks format, lint, type-check and test it, one task per check.
- **[core.6]** Every tool a gate task runs is the version the tree pins, on every host that runs the gate.

## Details

- **core.1** Rules out: a status field on a claim; an event edited or deleted to change a status; a supersession that rewrites the claim it replaces. Not ruled out: a new event that supersedes, rejects or annotates. Source: record 0001.
- **core.2** Rules out: a rendered constitution, design book or context map committed and read back as input. Not ruled out: a view written to a build directory or published. Source: record 0001.
- **core.3** Rules out: a credential in a tracked file, a fixture or a commit message; a credential printed in output. Not ruled out: the name of the variable or file a credential is read from. Source: record 0001.
- **core.4** Rules out: pricing, a tier, a customer or a hosted plan named in code, docs, the site or commits. Source: record 0001.
- **core.5** Rules out: a file type treefmt neither formats nor excludes; an exclude with no comment saying why; a crate or the site with no project of its own; one task bundling several checks. Also rules out: a task, a test or a workflow that reads a Markdown file of the tree, a README, a site page, a spec or a record, for its prose, its numbers or its form, as `root:adrs` read the records' numbers and `site:prose` and the Pages workflow ran Vale over the pages until record 0002. Not ruled out: OpenSpec's validation of the specs and the open changes, the change tool's own reading of its format (principle 16 as amended 2026-11-15); `root:format`, whose treefmt excludes `*.md` and `*.mdx` by name; the site's build and type check, Astro's own reading of its content; the site's diagram test reading `docs/diagrams/boom.svg`; Vale run by a writer from the shell, `prose`, which no task or workflow runs (record 0002). Source: record 0001; flume's core.21, the pattern this repository copies; boomdrivedev/blueprints principle 16 and record 0002, for Markdown.
- **core.6** Rules out: a tool a task finds outside its pin; a pinned toolchain without a component a task runs; two files pinning one tool with no check that fails when they differ. Source: flume's core.22.
