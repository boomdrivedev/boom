## Context

Boom's plugin gives an agent the claims of its repository, at session
start and before the first edit of a file in a context
(`session-context`). It does not teach the agent how to write a claim.
An agent learns that only from the brief of its session.

This change keeps these claims:

- `no-task-reads-markdown@a25dda6425d7`: a skill is a Markdown file. No
  task reads its words or its form. Only Claude Code's own validation of
  the plugin reads a skill, as core.5 allows. The plugin's test checks
  that each skill's file is at its path, and reads none of it.
- `plugins-come-from-the-tools-flake@78305fd0bae0`: the skills are part
  of the plugin, so boom's flake gives them at boom's version (core.8).
- `only-an-accepted-version-binds-work@4f151e9d63ed`: the skills tell an
  agent that a proposed version binds nothing.
- `claims-cite-claims-by-name@99b743d15e5e`: the skills tell an agent to
  cite a claim only as `<name>` or `<name>@<version>`.
- `a-new-draft-replaces-the-one-before@b5d90ffa4df6`,
  `a-narratives-open-question-is-a-proposed-decision-included-at-its-paragraph@5b473b9ae454`
  and `the-accept-of-a-narrative-accepts-the-claims-it-includes@5642b23601f5`:
  the skills teach these as the person accepted them.
- `a-claim-or-narrative-describes-the-system-and-names-no-plan@acd2e8e1efc6`,
  which waits on the person: the skills teach it, and the warning finds
  the words that its body names.
- `proposals-reach-the-person-in-batches-a-narrative-a-ruling-or-a-revision@868a48d5ee39`,
  which waits on the person: the skills teach its three shapes. When the
  person amends it, the skills change with it.
- `boom-mcp-serves-an-agent-the-commands-it-may-run-and-none-a-person-must-sign`:
  the skills name the same acts as the person's.
- `a-term-binds-one-repository@52e0b523d9a9`: the old words of a term
  come from the log, and no file holds them. The plan words are not old
  words of a term (decision 4).

## Goals / Non-Goals

**Goals:**

- An agent in any repository that Boom serves can read the claims,
  propose each kind, write a narrative with its claims, and answer a
  note, from the plugin alone.
- `boom propose` tells an agent when its words name a plan.

**Non-Goals:**

- Skills for a person's acts: `init`, `register`, `repo`, `accept`,
  `reject` and `note`. A person runs them, and `boom --help` gives them.
- A skill for the import of a repository's records and rules. The
  import's own help and its proposals cover it.
- The wiring of the skills into flume's kinds. That is a change in
  flume. Decision 5 says what flume's change needs.
- A refusal of a plan word. The warning never refuses.

## Decisions

### 1. Three skills, by what the agent does

| Skill | The agent | Loaded |
|---|---|---|
| `using-boom` | reads and cites claims | In every session that Boom serves, before a design or a change |
| `proposing-claims` | proposes an invariant, a decision, an assumption, a term or a context, or a new version | Before any `boom propose` |
| `writing-narratives` | explains a design in a narrative with its claims, answers a note, orders the book | When a design, an elaboration or a research ends with findings |

Claude Code loads a skill by its description, and then reads its
`SKILL.md` in full. So each skill holds what one act needs, and no
more. An agent that only reads claims loads `using-boom` alone, which
is short. An agent that proposes loads `proposing-claims`, and reads
`kinds.md` for the kind that it proposes.

The names follow sluice's skills, `using-sluice` and
`setting-up-sluice`: a verb of the act, then the object. In a session
they are `boom:using-boom`, `boom:proposing-claims` and
`boom:writing-narratives`.

Not taken:

- One skill `boom` with a reference file for each subject. Its
  description must cover every act, so Claude Code loads it for each
  question about a claim, and the agent reads the parts that it needs
  only if it follows the links.
- A skill for each kind. Most rules apply to each kind: Plain, the plan
  words, the level, the edges, the drafts and the batches. Seven skills
  would hold seven copies of them.

### 2. What each skill holds

**`using-boom`** (`skills/using-boom/SKILL.md`):

1. What Boom is: a ledger of claims, an agent proposes, a person
   accepts, only an accepted version binds work.
2. What the session already has: the `Boom:` line, the rules in full,
   the other claims by name, and the stop before the first edit in a
   context or a write with an old word.
3. How to read: `boom query`, `query --path`, `search`, `search --all`,
   `show`, `show <name>@<version>`, `terms check`, `log`, and `page` as
   the person's page.
4. How to cite a claim: by name, checked with `boom show`, never by a
   number, an id, a URL or a path.
5. The acts that are a person's: `accept`, `reject`, `note`, `init`,
   `register`, `repo add`, `repo set`, and any `--key`. What to tell the
   person instead.
6. What `boom mcp` serves.
7. Which skill to load to write.

**`proposing-claims`** (`skills/proposing-claims/SKILL.md`, and
`kinds.md`):

1. Before you propose: search, choose the kind, the level, the words,
   the links and the batch.
2. The kinds in one table, with `kinds.md` for each kind in full.
3. Say what the system is, never the plan: the plan words, the order of
   work, the status of work, and a failure named by what happens.
4. Plain: ASD-STE100 as this store uses it, in eight lines.
5. The narrowest level: a context, a repository, the whole system.
6. The links: `--source`, `--changes`, `--origin`, `--provenance`.
7. Drafts: `--amends`, a new draft replaces the one before, amend the
   newest version in the line, never amend a version that an accepted
   version supersedes.
8. Batches: a narrative with its claims, a ruling, a revision. No claim
   waits alone.
9. One whole `boom propose`, and the accept that the agent never runs.

`kinds.md` gives each kind its use, its fields, how to write it, what
breaks it and what does not where the kind has a rule, and an example
from the store: `a-decision-that-changes-a-rule-says-before-and-now`,
`a-narratives-open-question-is-a-proposed-decision-included-at-its-paragraph`,
`origin` and `sluice-plugins`, with two more names for each. The store
holds no accepted assumption, so its example is a form that fits.

**`writing-narratives`** (`skills/writing-narratives/SKILL.md`):

1. What a narrative is: a title, a body with numbered paragraphs, the
   claims that it includes, one level.
2. How to write it: one narrative for each level, one subject for each
   paragraph, the claims named at the end of their paragraph and
   included.
3. An open question is a proposed decision, included at its paragraph.
4. The batch: propose the claims, then the narrative. A new draft of a
   claim needs a new version of the narrative.
5. A note and its answer: `--answers <note>=<words>`.
6. The outline of the design book, and how Boom computes the book.
7. An example: `sluices-plugins`, with two more names.

The first drafts are in the plugin, at the paths above. The person
reads them before the code of the warning is built.

### 3. The warning of a plan word

`boom propose` reads the statement and the body of the version for the
plan words below. For each match, it writes one line on standard error:

```text
warning: "wave" names a plan, in the statement. A claim says what the system is.
warning: "happy path" names a plan, in line 12 of the body. A claim says what the system is.
```

It exits as before, and appends the version as before. With `--json`,
the object of `boom-propose/2` gets the field `plan_words`, a list of
`{"word", "in", "line"}`: `in` is `statement` or `body`, and `line` is
the line of the body, or null. The field is new, so a reader of
`boom-propose/2` that does not know it reads the object as before. The
MCP tool `propose` answers the same object, so an agent's program gets
the same warnings.

The words:

| Word | Why it names a plan |
|---|---|
| `wave` | a part of the work in time |
| `phase` | a part of the work in time |
| `milestone` | a point of the work in time |
| `sprint` | a part of the work in time |
| `epic` | a group of work items |
| `user story` | a work item |
| `roadmap` | the order of work |
| `MVP` | the first part of the work to ship |
| `stretch goal` | work that may not be done |
| `nice to have` | work that may not be done |
| `stopgap` | a form that later work replaces |
| `interim` | a form that later work replaces |
| `for now` | a form that later work replaces |
| `until then` | a form that later work replaces |
| `not built yet` | a status of work |
| `happy path` | names a case by the plan's hope, not by what happens |
| `sad path` | names a case by the plan's hope, not by what happens |
| `unhappy path` | names a case by the plan's hope, not by what happens |
| `TODO` | a status of work |
| `TBD` | a status of work |
| `WIP` | a status of work |
| `FIXME` | a status of work |

The list leaves out words that the domain of the family uses in another
sense: `plan` (an approved plan), `step` (a drive's step), `status` (a
version's status), `backlog`, `draft`, `in progress` (a run in
progress), `not yet`, `first` and `next`. Of the claims that stand on
2026-11-24, `boom search --all` finds no single word of the list, except
`MVP` in two claims: one quotes the user, and one names
timber's plan. The warning's tests count the phrases.

The match is the match of `boom terms check` (`term-check`): another
form, case or accent matches, so `Phases` matches `phase`. A word of two
words matches those words in order. A word inside a longer word, such
as `microwave`, does not match. The marker `boom: allow <word>` is not
read: a warning stops nothing, so it needs no marker.

### 4. The list is in the code

The list is a constant in `crates/boom/src/terms.rs`, beside the
matcher. A change of the list is a change of boom, through an OpenSpec
change and a release.

Not taken:

- Terms in the store. A term gives a word in place of its old words, and
  a plan word has no word in its place. A term also stops a write in the
  hook, and the warning stops nothing.
- A list in each repository's settings. The rule that a claim names no
  plan holds in each repository, so one list serves each store.

### 5. What flume's kinds need

A stage of a kind names the skills that its agent loads at its start,
as `boom:<skill>`. The place has boom's plugin, from boom's flake at
boom's version. The launcher sets `BOOM_REPOSITORY` where the remote
does not name the repository.

| Kind and stage | Skills |
|---|---|
| `change`: `construct` | `using-boom`, `proposing-claims` |
| `change`: `code` | `using-boom` |
| `change`: `verify` | `using-boom`, `proposing-claims` |
| `fix`: each stage | `using-boom` |
| `chore`: each stage | `using-boom` |
| a design, an elaboration or a research: the stage that ends with its findings | `using-boom`, `proposing-claims`, `writing-narratives` |

The `construct` stage proposes the decisions of its change. The
`verify` stage proposes a revision where the code that it verifies
differs from a claim. A design, an elaboration or a research ends with
its findings in Boom (`an-elaboration-ends-with-its-findings-in-boom`),
as a narrative with its claims.

flume's change cites this decision for the names of the skills, and
owns how a kind declares them.

## Risks / Trade-offs

- [A skill and the code differ after a later change of boom] → Each
  change of boom that changes a command, a kind or a rule that a skill
  teaches changes the skill in the same branch. The tasks of such a
  change name the skill.
- [The warning teaches an agent to ignore it] → The list holds only
  words that name a plan in every sense that the store uses. It matches
  two claims that stand.
- [A skill holds a rule that a claim also holds] → A skill names the
  claim that it teaches. When the claim changes, the skill changes with
  it. The claim is the truth, and the skill is how to follow it.
