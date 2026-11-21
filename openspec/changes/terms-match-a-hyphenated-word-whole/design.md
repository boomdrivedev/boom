## Context

`boom terms check` finds the old words of the accepted terms of a
repository (`a-term-binds-one-repository@52e0b523d9a9`). It matches a
word as `boom search` matches one, with the FTS5 tokenizer
`porter unicode61`. Before the tokenizer reads a line, boom puts a mark
at each change from lowercase to uppercase. The tokenizer splits at
each mark that is not a letter or a digit. Thus the old word
`came-from` is the phrase "came" "from", and the plain words "came <!-- boom: allow came-from -->
from" match it.

The user ruled on 2026-11-21 that the check stops only the name
`came-from`, and that the plain phrase stays free. <!-- boom: allow came-from -->

This change keeps these claims:

- `names-use-the-domains-terms@343c9fac7fc5`: a tool names things with
  the terms of its domain. Its body names "came-from" as a word that an <!-- boom: allow came-from -->
  agent made up, and names the plain words of a domain as free.
- `a-term-binds-one-repository@52e0b523d9a9`: the check reads the old
  words from the log, and no file holds a list of them.
- `no-task-reads-markdown@a25dda6425d7`: the check reads a file for the
  old words of the terms and for nothing else.
- `the-gate-validates-specs-strictly@d8e057526e31` and
  `open-changes-archive-cleanly@de24a2987568`: the change passes
  `openspec validate --strict` and archives alone.

## Goals / Non-Goals

**Goals:**

- An old word that holds a hyphen matches each form of its name in
  code.
- The plain words of such an old word never match.

**Non-Goals:**

- The term `origin`. The orchestrator asks for it after this change
  lands.
- A change to an old word with no hyphen.

## Decisions

### 1. The joins that count

A hyphen in an old word says that the old word is a name. In code, a
name joins its words in one of these forms:

| Form | Example | Where it is used |
|---|---|---|
| kebab case | `came-from` | flags, claim names, file names | <!-- boom: allow came-from -->
| snake case | `came_from`, `CAME_FROM` | fields of Rust and JSON, constants | <!-- boom: allow came-from -->
| camel case | `cameFrom` | fields of TypeScript | <!-- boom: allow came-from -->
| Pascal case | `CameFrom` | types | <!-- boom: allow came-from -->

Each two words of the match are joined by one mark: `-`, `_`, or a
change from lowercase to uppercase. The check folds case as before, so
`CAME_FROM` and `Came-From` match. A mix of joins in one name, such as <!-- boom: allow came-from -->
`a-b_c`, matches too. The check does not require one join for the
whole name.

These do not count:

- A space, as in "came from". This is the plain phrase that the user
  kept free.
- A dot, as in `x.came.from`. In code, a dot joins a path of two
  names, not the words of one name.
- Two marks or more, as in `came - from` or `came--from`.
- No mark, as in `camefrom`. That is one other word.

A match can be part of a longer name, as `came_from_id`. An old word <!-- boom: allow came-from -->
with no hyphen matches in the same way now, as `keeper_id`. <!-- boom: allow keeper -->

### 2. FTS5 finds the match, and boom checks its joins

FTS5 still finds each match of the phrase, and `highlight` marks its
text on the line. For an old word that holds a hyphen, boom then reads
the marks between the words of each match. Boom drops each match where
a mark between two words is not one join.

The mark that boom puts at a change of case is the control character
U+0003, not a space. FTS5 reads it as a mark between two words, as it
read the space. Thus boom tells `CameFrom` from `Came From`. Boom <!-- boom: allow came-from -->
removes U+0003 from the text that `written` gives, so `written` is the
text of the line.

Alternative: a tokenizer of boom's own that keeps the joins. This
needs a custom FTS5 tokenizer in Rust. The tokenizer would also change
how `boom search` reads, and the check must match as `boom search`
matches.

Alternative: a regular expression for each hyphenated old word. This
gives a second way to fold case, accents and stems, apart from the way
of `boom search`.

### 3. No transition

The check has no flag for the old match. The store holds one term,
`owner`, and its old word holds no hyphen. Thus no verdict of the gate
changes.

## Risks / Trade-offs

- A hyphen at the end of a line, as "came-" and then "from" on the next
  line, does not match. The check reads one line at a time, as before.
- An old word that holds a hyphen and is also plain English, such as
  `e-mail`, no longer matches "e mail". A term that must stop the plain
  words gives them as an old word with a space.
