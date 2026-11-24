## 1. The origin of an elaboration

- [ ] 1.1 Add the origin `elaboration` with its repository and its name to `claim.rs`: its form, its words and its check, and one elaboration at most in a version
- [ ] 1.2 Read `--origin elaboration:<repository>/<name>` in `boom propose`, and give its JSON in `boom show`
- [ ] 1.3 `boom:test` proves the scenarios of "An origin names the elaboration that a version came from", the one appended by hand among them

## 2. A note's elaboration

- [ ] 2.1 Read a note's elaboration from the version it is on, and give it with each note in `boom show`
- [ ] 2.2 `boom query --elaboration <repository>/<name>` gives the open notes of that elaboration, and refuses it with `--repository`, `--path` or `--if-declared`
- [ ] 2.3 `boom:test` proves the scenarios of "A note reaches the elaboration of its version", "A query gives the open notes of an elaboration" and the tool `query` with `elaboration`

## 3. Claims, docs and the gate

- [ ] 3.1 Propose the new versions of `origin-and-source-name-the-links-of-a-claim` and of the term `origin` that name the elaboration as an origin
- [ ] 3.2 Say the origin of an elaboration and the query of its notes in `README.md` and `CLAUDE.md`
- [ ] 3.3 Run `boom:format`, `boom:lint`, `boom:typecheck`, `boom:test`, `root:specs`, `root:changes` and `root:terms` through `sluice exec main`
