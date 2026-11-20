---
number: 2
title: boom's tasks read no Markdown, and its writers keep what a page says
status: proposed
date: 2026-11-15
decision-makers:
- Chuck Swanberg
- the chore booms-tasks-read-no-markdown's code stage
---

# boom's tasks read no Markdown, and its writers keep what a page says

Status: proposed. The decision is the user's: boomdrivedev/blueprints
principle 16 at `3a5c19d`. The principle was written on 2026-11-11 from
the words of the user on that day, and amended on 2026-11-15 on their
answer. The brief of the chore names what goes and what stays. The
choices below that neither names are the code stage's own. The code
stage made them on the written rules, and they are cheap to change.

Deciders: the user. Their words, on sluice's `root:installs`: "I don't
want fucking checks on instructions. That has got to fucking stop. That
is my biggest pet peeve. We do not fucking write code tests against
Markdown files." Their answer of 2026-11-15: "OpenSpec's own validation
stays; the rule is about checks of what pages say". The brief of the
chore `booms-tasks-read-no-markdown`: "boom's tasks read no Markdown
file of its own: root:adrs and site:prose go. OpenSpec's validation,
root:format, the site's build, type check and diagram test stay." The
code stage of the chore, for the rest.

Sources:

- boomdrivedev/blueprints `design/principles.md` 16 at `3a5c19d`.
- flume's record 0071 and core.21, and boomdrive's record 0049 and
  core.7. These take the same rule into their trees.
- Rule core.5.

## Context and problem statement

Principle 16 says this. No check, test or gate task in any kit reads a
Markdown file. Such a file is a doc, a README, a skill, a site page, an
instruction, a spec or a record. The writers of a page keep a rule about what the page
must say, and no task enforces it. A tool's own validation of the files
that it works on is not a check of what a page says. boom's gate read
Markdown in three ways:

1. `root:adrs`, `check-adrs.sh`, read the frontmatter and the name of
   each record for its number.
2. `site:prose` ran Vale over the pages of the site.
3. The Pages workflow ran `site:prose` before it built the site.

The brief names the first two. The third is the second, run by a
workflow. The rules decide that reading.

## Considered options

For Vale:

1. Keep it in the shell, as `prose`, for a writer to run on their own
   pages. No task or workflow runs it.
2. Remove Vale, `.vale.ini`, `styles/` and `prose` from the tree.

For the prose step of the Pages workflow:

1. Remove it. The workflow builds and publishes the site, and nothing
   reads its words.
2. Keep it, as a workflow step and not as a gate task.

For the check of the version of Vale in `root:toolchain`:

1. Keep it: the Vale of the shell is the release that `devenv.nix` pins.
2. Remove it, because no gate task runs Vale.

## Decision outcome

For Vale, option 1, as flume's record 0071 and boomdrive's record 0049
have it. A writer who runs a tool on their own page keeps what the page
says, and principle 16 asks for that. A task that fails a candidate on
the page enforces what it says, and principle 16 forbids that. Option 2
removes from the writers a tool that cost the gate nothing. Option 2 is
also cheap to do later.

For the prose step, option 1. A workflow that blocks the site for the
words of a page is a check that reads Markdown, whatever runs it.

For the check of Vale in the toolchain, option 1, as boomdrive's
`check-toolchain.sh` keeps it. The check reads the answer of Vale to
`--version`, and no page. It holds the Vale of the writers to its pin,
as it holds OpenSpec.

These stay, as the brief says:

- OpenSpec's validation of the specs and the open changes, which is the
  change tool's own reading of its format.
- `root:format`, whose treefmt excludes `*.md` and `*.mdx` and reads
  none.
- The build and the type check of the site, which are Astro's own
  reading of its content.
- The diagram test of the site, which reads `docs/diagrams/boom.svg` and
  no page.

### Consequences

- `root:adrs`, `check-adrs.sh`, `site:prose` and the prose step of the
  Pages workflow are gone. The writer of a record keeps its number: the
  next free number, read from `docs/adr/` and from the open branches
  that add records. Nothing fails two records that share a number.
- A page can stray from the house style until a writer or a reviewer
  runs `prose` or reads it.
- The records keep their words. `the-rename-to-boomdrive` still names
  `check-adrs.sh`, as the script stood when that record was written.
- core.5 says that no task, test, formatter or workflow reads a Markdown
  file of the tree for its words or its form. core.5 cites principle 16
  and this record.
