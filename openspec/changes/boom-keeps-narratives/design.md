## Context

Boom keeps five kinds of claim: invariant, decision, assumption, term
and context (`claim-versions`). A claim's body is a blob, kept one time
by the fingerprint of its bytes (`record-store`). A version stores its
edges `amends`, `source`, `changes` and `origin`, each edge to a claim
with the version that it meant (`claim-edges`). A person accepts or
rejects with a security key. An agent only proposes (`persons-acts`).

The design of 2026-11-01 (`interview.md`, `five-records`) made the
narrative a record of its own: "an explanation in its author's words at
the time, scoped to a context, including claims by name; judged by a
person, never checked". Its design book wove narratives with the claims
that each includes, "every paragraph naming its record so a note on it
reaches the record" (`design/narrative.md`, section 7). On 2026-11-20
the design book went outside the MVP. Nothing built a narrative, so
each design agent writes a Markdown file in its repository.

The user ruled on 2026-11-23: "Before turn 6, make narratives live in
Boom, so turn 6's design is the first one Boom holds. Yes: a narrative
kind and a page to read and annotate it first." A note on the page goes
back as a signal. For turn 6, the user also ruled:

- The ledger records every act of the person on the rail, from intent
  to plan, and a bolt's hold and release.
- The person's security key signs only the acts that bind later work:
  the accept of a claim, and the approval of a plan. Boom records each
  other act with the person who made it.
- A landing is never the act of a person.

This design keeps these claims of the store:

- `views-computed-from-the-ledger`: the page is a view. Boom computes it
  from the log for each request, and no file or commit holds it.
- `status-read-from-events`: the status of a narrative and of a note
  come from the entries of the log.
- `events-are-appended-never-changed`: a note and its answer are new
  entries. No entry changes.
- `a-link-remembers-the-version-it-meant`: `includes` stores the version
  of the claim that it meant, and a note names the version and the
  paragraph that it meant.
- `claims-cite-claims-by-name`: `includes` names claims of the store
  only.
- `boom-keeps-its-records-in-a-database-on-each-machine-and-the-drive-orders-them-when-attached`:
  its point 3 names the narrative as long text, kept by fingerprint. Its
  point 5 says that Boom holds the narratives.
- `boom-mcp-serves-an-agent-the-commands-it-may-run-and-none-a-person-must-sign`:
  a note is the act of a person, so no tool writes one.
- `a-persons-act-in-boom-is-an-ssh-signature-by-a-security-key`: the
  change proposes its amendment for the note, by the user's ruling of
  2026-11-23. Decision 7 says how.
- `names-use-the-domains-terms`: `narrative`, `paragraph`, `note`,
  `page`, `include` and `answer` are the words of the design of
  2026-11-01 and of Markdown. `serve` and `127.0.0.1` are the words of
  HTTP.
- `no-credential-in-the-repository`: the page's token is random for
  each run of `boom page`, and no file holds it.
- `nothing-commercial`: the page runs on the person's machine. This
  change names no service of another host.
- `no-task-reads-markdown`: boom reads the Markdown of a body from its
  store, never a file of the tree. A person or an agent gives the body
  of the first narrative one time, by a command. No task reads it.
- `each-part-is-a-moon-project` and `one-task-per-check`: the page is
  Rust code in the crate `boom`. It adds no file of a new type and no
  part of the repository, so it adds no moon project.

## Goals / Non-Goals

**Goals:**

- A design lives in Boom: a narrative is proposed, read, noted,
  amended and accepted in Boom, and no repository holds its file.
- A person reads a narrative on a page, with the claims that it
  includes, and writes a note on a paragraph.
- An agent finds the open notes, and the next version answers each.
- `design/narrative.md` becomes the first narrative, and then leaves
  the repository.

**Non-Goals:**

- The design book: the narratives of a context woven into chapters.
  The page gives one narrative at a time.
- Notes on a claim of another kind. A note is on a paragraph of a
  narrative only.
- Images in a body, served from the artifacts of a version's origin.
  A later change can serve them on the page.
- The terms check of a narrative's body when an agent proposes it.
- A note sent to an agent. Session start gives no narrative and no
  open note. Only the agent that revises a narrative needs its open
  notes, and each other session pays for the line in context. In
  this change, the agent that revises a narrative reads the open notes
  with `boom show <name>`, the tool `show`, or the refusal of
  `boom propose ... --amends`. In turn 6, a note is a signal on the
  rail, and the rail starts the work that revises the narrative.
- The edit layer of `boom hook pre-tool-use`. It does not give the
  narratives of a context.
- `boom render`, `boom sync` and `boom check`.

## Decisions

### 1. A narrative is a claim of the kind `narrative` (decided 11-23)

`boom propose narrative <name> "<title>" --body <file>` proposes a
narrative. A version holds these:

- **The name**, in kebab case, in the one name space of the store.
- **The title**, as the statement: one line.
- **The body**, required: Markdown text in UTF-8, kept as a blob by its
  fingerprint. The title is not a heading of the body.
- **What it binds**: `--repository`, `--context` or `--system`, as for
  each other claim (`claim-attachments`).
- **The claims that it includes**, `--includes` (decision 2).
- **Its sources and origins**, `--source` and `--origin`, as for each
  other claim. `--changes` stays a decision's alone.
- **The answers to the open notes**, `--answers` (decision 8).

`--amends <version>` proposes the next version. Its fingerprint is the
fingerprint of all of the above.

The narrative is in the one name space and the one log. Versions,
amendments, origins, attachments, the accept of many versions with one
signature, `boom show`, `boom search` and `boom verify` work for a
narrative with no new code. The store calls each of its records a
claim. A narrative is a claim of the kind `narrative`, and no one
checks it (decision 3).

### 2. `includes`: the claims of a narrative, at the version it meant

`--includes <name>` and `--includes <name>@<version>` name a claim that
the narrative includes. Boom resolves each as it resolves `--source`:
`<name>` stores the accepted version, or the newest version where none
is accepted. Boom refuses a name that no claim has, a version that its
claim does not have, a narrative, and `--includes` on a claim of
another kind. The edge is part of the version's fingerprint.

The page places an included claim after the first paragraph whose
words name it, as the terms check matches a hyphenated word whole. It
gives the other included claims after the body, under "Claims it
includes". The page marks an included version out of date when its
claim has another accepted version, as `boom show` marks an edge.

Options:

1. Edges on the command line, and the page places each claim by its
   name in the text (recommended). The edge is the truth, and the body
   stays plain Markdown.
2. A mark in the body, such as `{{include <name>}}`, that boom reads at
   the propose. The place is exact, but the body is no longer plain
   Markdown, and a reader of the blob sees the marks.
3. Boom finds the names of the claims in the body and stores an edge
   for each. An agent cannot then name a claim in passing without an
   edge.

### 3. A person accepts a narrative with the key, and no one checks it (decided 11-23)

A person accepts or rejects a version of a narrative with
`boom accept` or `boom reject` and a security key, as for each claim.
`boom accept` takes a narrative with the claims that it includes, so
one touch of the key accepts a design. Status comes from the log:
`proposed`, `accepted`, `superseded` or `rejected`. `boom search` finds
an accepted narrative by its words, as it finds each claim.
`boom query` gives it with the fingerprint of its body.

The key signs the accept of a claim, by the user's ruling of
2026-11-23. An accepted narrative binds later work: agents and the
planner read it as the design.

Boom never checks a narrative. No hook gives a narrative, and session
start gives no line for one. A later check status gives a narrative
none.

### 4. The paragraphs of a body

Boom reads a body with `pulldown-cmark` (CommonMark, with tables) and
numbers its paragraphs from 1, in order. A paragraph is one block at
the top of the body: a heading, a paragraph of text, a code block, a
block quote or a table. Each item of a list at the top of the body is a
paragraph too. A thematic break is no paragraph. The fingerprint of a
paragraph is the first 12 hexadecimal characters of the SHA-256 of its
bytes in the body.

A version never changes, so its paragraph numbers never change.
`boom show <name>@<version> --json` gives `paragraphs`: for each, its
number, its fingerprint and its kind. A note names a paragraph by its
number and its fingerprint.

The page escapes each raw HTML block and each inline HTML of a body as
text. A body cannot put a script or a form on the page.

### 5. `boom page`: the page, on the person's machine

`boom page [<name>[@<version>]]` serves the page and opens it:

1. Boom binds `127.0.0.1` on `--port <port>`, or on a free port.
2. Boom makes a token of 128 random bits from the random source of the
   operating system. Each path of the page starts with the token. No
   file holds the token.
3. Boom prints one line, `Boom: page of <name>@<version> at <URL>`, and
   then opens the URL with `open` on macOS or `xdg-open` on Linux,
   unless `--no-open`.
4. Boom serves until it is stopped, or until `--idle <minutes>` pass
   with no request (60 by default). It then exits 0.

With no name, the first path gives the list of the narratives of the
store, by repository, each with its status and its open notes. With
`<name>`, it gives the newest version, as `boom show` does. With
`<name>@<version>`, it gives that version.

Boom computes each response from the views at the last entry of the
log, as each command does (`record-store`, "A command answers from the
views"). A note that another person or command writes shows at the next
request.

Boom refuses each request with no token, with a `Host` other than
`127.0.0.1:<port>`, and each `POST` with an `Origin` other than the
page's own. Each response has the header
`Content-Security-Policy: default-src 'none'; style-src 'unsafe-inline'; form-action 'self'`.
The page holds its style, loads nothing from another host, and runs no
script. A note is a plain HTML form.

**From a session.** An agent runs `boom page <name> --no-open` in the
background of its harness and gives the person the URL of the line that
boom prints. In Claude Code, the person clicks the URL in the pane. A
person can also type `! boom page <name>` in the session.

Options:

1. A server on `127.0.0.1` with a token, plain HTML forms and no script
   (recommended). The person reads and writes notes in a browser, and
   boom stays one binary with no file of a new type.
2. A page written to a folder, with notes written by `boom note` alone.
   The person copies a paragraph number into the terminal for each
   note.
3. A view in the terminal. It shows no tables or code with their form,
   and it gives no place to write a long note.
4. A page with a script and a style sheet as files of the crate. The
   page can do more, but each file of a new type needs a formatter and
   a lint (`each-tracked-file-has-a-formatter`), and the script needs a
   moon project of its own (`each-part-is-a-moon-project`).

### 6. What the page gives

```
Boom · narratives of boom                                    entry 262
──────────────────────────────────────────────────────────────────────
Boom, end to end
narrative boom-end-to-end@3f2a9c01b7de · proposed by an agent
(boom-method, design of turn 5) · amends 9e1d44a0c2b3 · binds boom
versions: 3f2a9c01b7de proposed · 9e1d44a0c2b3 accepted by chuck
──────────────────────────────────────────────────────────────────────
¶1  The first draft of Boom's design book: how Boom works      [note]
    from a signal to a landed change and back ...
¶2  1. What Boom is                                             [note]
¶3  A project says things about itself ... and computes        [note]
    everything else from that log ...
    ┌ invariant status-read-from-events@97d0559deafa · accepted ──────┐
    │ A record never holds its status, and boom reads the status      │
    │ from the events of the record.                    ▸ details     │
    └─────────────────────────────────────────────────────────────────┘
    ┌ note 5b1c0e7a92f4 · chuck · 2026-11-23 14:02 · open ────────────┐
    │ Say what a drive is before the first time you use the word.     │
    └─────────────────────────────────────────────────────────────────┘
¶4  ...                                                         [note]
──────────────────────────────────────────────────────────────────────
Claims it includes that no paragraph names
  decision origin-and-source-name-the-links-of-a-claim@4c1e09d2aa71
    out of date: accepted now 7c2e5f10b9a4
Notes on 9e1d44a0c2b3 and their answers
  note 0d7a33e1c5f8 on ¶12 (chuck): "Where does a note go?"
    answered: "Section 10 now says it: an entry of the log, ..."
──────────────────────────────────────────────────────────────────────
To accept, in your terminal:
  boom accept boom-end-to-end@3f2a9c01b7de --key <key>
```

- **The head** gives the title, `<name>@<version>`, the status, the
  proposer and the provenance, the base, what it binds, and each
  version with its status, as a link.
- **Each paragraph** has its number as an anchor, `#p3`, and a
  `[note]` control. The control opens a form under the paragraph, a
  `<details>` element with a text box and a button. The form sends the
  note, and the page then shows itself again at that paragraph.
- **An included claim** is a box after its paragraph: the kind,
  `<name>@<version>`, the status, the statement, and its body in a
  `<details>` element.
- **The notes** of a version are boxes at their paragraphs, each with
  its name, its person, its time, and `open` or the answer.
- **The end** gives the claims that no paragraph names, the notes of
  the earlier versions with their answers, and the command line of the
  accept. The page holds no key and does no act of a key.

### 7. A note: the act of a person, with no key

A note is an entry with the act `noted`. It holds the narrative's name,
the version, the paragraph number, the paragraph's fingerprint and the
words of the note. Its actor is the person, by name, and it has no
signature. The name of a note is the first 12 hexadecimal characters
of the SHA-256 of the narrative's name, the version, the paragraph, the
words and the person. The same note again adds nothing.

`boom note <name>@<version> --paragraph <n> "<words>"` writes a note
from the terminal. The page writes the same entry. Each takes the
person from `--person <name>`. Where `--person` is not given and the
store registers one person, the person is that person. Where the store
registers more than one person, `boom note` refuses, and the page shows
no `[note]` control and says how to start it with `--person`.

Boom refuses a note, and appends nothing, in each of these cases:

- The person is not registered.
- The version is not a version of a narrative.
- The paragraph does not exist, or its fingerprint is not the
  fingerprint of that paragraph.
- The words are empty.
- The command gives `--key` or `--provenance`.

The judge counts an unsigned entry of a person for the act `noted`
only. An unsigned `accepted` or `rejected` still counts for nothing.

The change proposes the amendment
`a-persons-act-in-boom-is-an-ssh-signature-by-a-security-key@560f29e4bbe7`:
a person's key signs the acts that bind later work, by the user's
ruling of 2026-11-23. A note binds nothing: the next version
answers it, and a person accepts that version with the key. The change
also proposes
`boom-keeps-its-records-in-a-database-on-each-machine-and-the-drive-orders-them-when-attached@2ecf312dc459`,
whose point on a person's signature says the same.
Boom does not prove who wrote a note. The person's name in the entry is
the name that the command or the page gave. An agent that runs a shell
can write a note in a person's name. `boom mcp` serves no note tool, and
the instructions of `boom mcp` say that only a person writes a note.

Options:

1. An unsigned entry with the person's name (recommended), as the
   ruling says.
2. A note outside the log, in a file of the page. A note is then no
   act of the log, and an agent on another machine never sees it.

### 8. A new version answers each open note (decided 11-23)

A note is open until a version of its narrative answers it and a
person does not reject that version. A note is the signal of this
change: it stays in the log, and each reader of the narrative sees it
open.

`--answers <note>=<words>` names a note and the answer of the new
version, in one line. The answers are part of the version's content and
fingerprint. Boom refuses, and appends nothing, in each of these cases:

- A new version of a narrative leaves a note of that narrative open.
  The refusal names each open note.
- The answer names a note that does not exist, a note of another
  narrative, or a note that another version answers and that no person
  rejected.

An agent finds the open notes in these places:

- `boom show <name>`, and the tool `show`, which give each open note
  with its paragraph and its words.
- The refusal of `boom propose ... --amends`.

No note is lost, and the person reads why each paragraph changed.

### 9. `boom mcp`

The tool `propose` of `boom mcp` takes the kind `narrative`, and
`includes` and `answers` as arrays of strings. `boom mcp` names `note`
and `page` as absent: a note is the act of a person, and `page` serves
until it is stopped, which no tool call can do.

### 10. The first narrative, and the file that then leaves

After this change lands and boom is released with it:

1. An agent proposes `design/narrative.md` from main as the narrative
   `boom-end-to-end`, title "Boom, end to end". The body is the file
   with its first heading taken out. The origin is
   `git:boom@<commit of main>:design/narrative.md`. The narrative binds
   `boom`. It includes the claims of the store that its text names. The
   file's other citations, such as `core.1` or "record 0005", stay as
   its author wrote them at the time, because a narrative is in its
   author's words.
2. The person reads it with `boom page boom-end-to-end`, writes notes,
   and accepts it with the key.
3. One change then removes `design/narrative.md` from the repository.
   It changes no spec, so it is a chore with no OpenSpec change.
4. The held branch `design/a-change-is-a-record` does not land its
   edit of the file. Turn 6's design agent proposes the design as a
   narrative of its own, or as an amendment of `boom-end-to-end`.
   This is the first design that Boom holds.

`interview.md` and `design/research/` stay. Each can become a
narrative by the same steps, in a later change.

Options for the body:

1. `boom propose narrative` with `--body` and `--origin` (recommended).
   No new command, and the origin keeps the file's commit.
2. `boom import narrative <file>`, which takes the title from the
   first heading. One more command for a one-time step.

### 11. Two dependencies, and no new moon project

`pulldown-cmark` reads a body's Markdown and writes its HTML, with its
default features off and `html` on. `tiny_http` serves the page on one
thread, with no async runtime, as `boom mcp` reads one request at a
time. `Cargo.lock` pins each (`tools-at-their-pinned-versions`). The
HTML and its style are Rust code in `page.rs`. `boom:test` checks the
HTML that a function gives, and it checks the server through a request
on `127.0.0.1`.

## Risks / Trade-offs

- [A note in a person's name, written by an agent] → Boom does not prove
  who wrote a note, by the ruling. `boom mcp` serves no note tool. A
  note binds nothing, and the accept of the next version keeps its
  key.
- [Another program on the machine reads the page] → The page binds
  `127.0.0.1`, and each path needs the token that boom prints to the
  person. The `Host` and `Origin` checks refuse a page of another site
  that sends a form to the port.
- [A body with HTML] → The page escapes it, and the policy of the
  response refuses each script.
- [An older boom on another machine] → It counts each `narrative`
  version and each `noted` entry for nothing, as it does for each kind
  that it does not know. Each machine of a store updates boom to read
  them.
- [The paragraph numbers of a long list] → Each item of a list at the
  top is one paragraph. A nested list is part of its item.
- [Many open notes] → Each new version answers each of them. A version
  that answers 30 notes has 30 `--answers`.
