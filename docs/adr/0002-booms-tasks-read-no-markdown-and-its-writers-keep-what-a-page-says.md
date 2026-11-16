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

Status: proposed. The decision is the user's, boomdrivedev/blueprints
principle 16 at `3a5c19d`, written 2026-11-11 from their words that day
and amended 2026-11-15 on their answer; the chore's brief names what
goes and what stays. The choices below that neither names are the code
stage's own, made on the rules written and cheap to change.

Deciders: the user. Their words, on sluice's `root:installs`: "I don't
want fucking checks on instructions. That has got to fucking stop. That
is my biggest pet peeve. We do not fucking write code tests against
Markdown files." Their answer of 2026-11-15: "OpenSpec's own validation
stays; the rule is about checks of what pages say". The brief of the
chore `booms-tasks-read-no-markdown`: "boom's tasks read no Markdown
file of its own: root:adrs and site:prose go. OpenSpec's validation,
root:format, the site's build, type check and diagram test stay." The
chore's code stage, on the rest.

Sources: boomdrivedev/blueprints `design/principles.md` 16 at
`3a5c19d`; flume's record 0071 and core.21, and boomdrive's record 0049
and core.7, which take the same rule into their trees; rule core.5.

## Context and problem statement

Principle 16: no check, test or gate task in any kit reads a Markdown
file, a doc, a README, a skill, a site page, an instruction, a spec or a
record; a rule about what a page must say is kept by its writers and
enforced by no task. A tool's own validation of the files it works on is
no check of what a page says. boom's gate read Markdown three ways:

1. `root:adrs`, `check-adrs.sh`, read every record's frontmatter and
   name for its number.
2. `site:prose` ran Vale over the site's pages.
3. The Pages workflow ran `site:prose` before it built the site.

The brief names the first two. The third is the second run by a
workflow, and a reading the rules decide.

## Considered options

For Vale:

1. Keep it in the shell, `prose`, for a writer to run on their own
   pages, and in no task or workflow.
2. Take Vale, `.vale.ini`, `styles/` and `prose` out of the tree.

For the Pages workflow's prose step:

1. It goes, and the site is built and published with nothing reading
   its words.
2. It stays, as a workflow step and no gate task.

For `root:toolchain`'s check of Vale's version:

1. It stays: the shell's Vale is the release `devenv.nix` pins.
2. It goes, since no gate task runs Vale.

## Decision outcome

Vale is option 1, as flume's record 0071 and boomdrive's record 0049
have it. A writer running a tool on their own page keeps what it says,
which principle 16 asks; a task that fails a candidate on it enforces,
which it forbids. Option 2 takes from the writers a tool that cost the
gate nothing, and is cheap to take later.

The prose step is option 1. A workflow that holds the site back for a
page's words is a check reading Markdown, whatever runs it.

The toolchain's check of Vale is option 1, as boomdrive's
`check-toolchain.sh` keeps it. It reads Vale's answer to `--version`,
no page, and holds the writers' Vale to its pin as it holds OpenSpec.

What stays, as the brief says: OpenSpec's validation of the specs and
the open changes, the change tool's own reading of its format;
`root:format`, whose treefmt excludes `*.md` and `*.mdx` and reads none;
the site's build and type check, Astro's own reading of its content; and
the site's diagram test, which reads `docs/diagrams/boom.svg` and no
page.

### Consequences

- `root:adrs`, `check-adrs.sh`, `site:prose` and the Pages workflow's
  prose step are gone. A record's number is its writer's to keep: the
  next free one, read from `docs/adr/` and from the open branches that
  add records. Nothing fails two records sharing a number.
- A page can stray from the house style until a writer or a reviewer
  runs `prose` or reads it.
- The records keep their words: `the-rename-to-boomdrive` still names
  `check-adrs.sh`, as it stood when that record was written.
- core.5 says no task, test, formatter or workflow reads a Markdown file
  of the tree for its words or its form, citing principle 16 and this
  record.
