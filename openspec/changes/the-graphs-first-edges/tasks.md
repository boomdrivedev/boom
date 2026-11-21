## 1. The edges

- [ ] 1.1 Add `source`, `changes` and `origin` to the proposal in `claim.rs` and `entry.rs`. Omit an empty list from the entry and from the content of the fingerprint
- [ ] 1.2 Resolve `--source <name>` and `--changes <name>` to the accepted version, else the newest, when boom judges the proposal
- [ ] 1.3 Refuse an edge to a claim or a version that the views do not hold, and an edge of any other form
- [ ] 1.4 Refuse `changes` on a version that is not a decision
- [ ] 1.5 Keep an artifact as a blob under its fingerprint, with the name of its file. Check the form of `git:<repository>@<commit>:<path>`: a commit of 40 hexadecimal characters, and a relative path with no `..`
- [ ] 1.6 Test that each version of a store from before the edges keeps its fingerprint, terms included

## 2. The views

- [ ] 2.1 Keep a view row for each edge, built from the log, and drop it with the views

## 3. The commands

- [ ] 3.1 `boom propose` takes `--source`, `--changes` and `--origin`, each any number of times
- [ ] 3.2 `boom show <name>@<version>` gives each edge, its version and statement, and marks an edge out of date. It gives `Before:` and `Now:` for a change
- [ ] 3.3 `boom show <name>` gives the other versions as a tree by base, and gives `source_of` and `changed_by`
- [ ] 3.4 `boom show --artifact <fingerprint>` writes the bytes of an artifact, and `boom show` refuses both a target and an artifact, or neither
- [ ] 3.5 The MCP tool `propose` takes `source`, `changes` and `origin`, and `show` takes `artifact`

## 4. The pages and the gate

- [ ] 4.1 Write the edges into `README.md` and `CLAUDE.md`, in Plain
- [ ] 4.2 Test on boom and on sluice: a link to a missing claim is refused, and a claim walks back to its mockup
- [ ] 4.3 Run `boom:format`, `boom:lint`, `boom:typecheck`, `boom:test`, `root:specs`, `root:changes` and `root:terms` through `sluice exec main`
