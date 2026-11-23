## Why

Boom refuses a term that binds the whole system. The accepted decision
`a-term-binds-one-repository@52e0b523d9a9` names such a term as a
break. Thus the terms `lineage` and `link`, which name parts of the
drive's one log, are proposed for `boom` only, and they do not hold in
sluice, flume or the other repositories.

On 2026-11-23 the user ruled: "Yes: amend it so a term may bind the
whole system." `boom terms check` checks a term of the whole system in
every repository that Boom registers. The word "chain" stays allowed,
because boom's tree uses it for the hash chain of the log. A refusal of
one meaning of a word needs a model, not a match of words.

## What Changes

- A term binds one repository or the whole system, and never a context.
  `boom propose term <word> <meaning> --system` proposes a term of the
  whole system. A term with no binding named, in a folder of no
  repository, is still refused.
- An amendment of a term binds what its base binds: one repository, or
  the whole system. A term of one binding never amends a term of
  another.
- One word can have a term of the whole system and a term of a
  repository. In that repository, the term of the repository stands,
  and the term of the whole system does not. Each other repository gets
  the term of the whole system.
- `boom terms check`, the hook and the gate task `root:terms` use the
  accepted terms of the whole system with those of the repository of
  each file. A found word of a term of the whole system names "the whole
  system" in words, and `repository` `null` in JSON.
- `boom show <word>` gives the term of the whole system first, then the
  term of each repository.
- This change proposes no term. The terms `lineage` and `link` of the
  whole system come after the release of this change.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `term-claims`: "A term names a word and its meaning" lets a term bind
  the whole system. A new requirement says which term stands where one
  word has a term of the whole system and a term of a repository.
- `term-check`: "boom terms check names each old word" uses the terms of
  the whole system too, and says how it names one.
- `claim-versions`: "One name space" and "A word shown for each
  repository" hold a term of the whole system.

## Impact

- `crates/boom/src/claim.rs`: the check of a term's binding, and the
  fingerprint of a term of the whole system.
- `crates/boom/src/judge.rs`: the refusal of a term on a context, an
  amendment across bindings, and the term that stands for a repository.
- `crates/boom/src/cli.rs`: `boom propose term` with `--system`, and the
  refusal of a term with no binding.
- `crates/boom/src/terms.rs`: the terms that the check uses, and how it
  names a term of the whole system.
- `crates/boom/tests/terms.rs`: a test for each new scenario.
- The amendment of `a-term-binds-one-repository`, proposed in Boom for a
  person to accept.
- No change to the log's format, the views' tables or the fingerprint
  of a version that exists.
