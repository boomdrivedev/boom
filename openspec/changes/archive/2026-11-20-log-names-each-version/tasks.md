## 1. The log

- [x] 1.1 Add `Act::target`, the name and version that an act names
- [x] 1.2 `boom log` gives `<name>@<version>` after the act on each line, and `target` in `boom-log/1`

## 2. The show

- [x] 2.1 `boom show <number>` reads the entry with that number and shows the version that it names. It refuses a number that the log does not hold, and an entry that names no version
- [x] 2.2 `boom show <name>` gives the newest version with its statement, what it binds and its body, then the other versions, newest first. `boom-show/2` gets `newest`

## 3. The tests and the gate

- [x] 3.1 Tests: each line names its version, an entry number shows its version, and a name shows its newest version in full
- [x] 3.2 Run `boom:format`, `boom:lint`, `boom:typecheck`, `boom:test`, `root:specs` and `root:changes` through `sluice exec main`
