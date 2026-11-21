## 1. The check

- [x] 1.1 Write U+0003, not a space, at each change of case in a line. Remove it from `written`. `boom:test` proves that `written` gives `DriveLog`
- [x] 1.2 For an old word that holds a hyphen, drop each match where a mark between two words is not one `-`, `_` or change of case. `boom:test` proves the scenario "A hyphenated old word in a name in code"
- [x] 1.3 `boom:test` proves the scenarios "The plain words of a hyphenated old word" and "Both on one line"

## 2. The pages and the gate

- [x] 2.1 Write the match of an old word with a hyphen into `README.md`, in Plain
- [x] 2.2 Run `boom:format`, `boom:lint`, `boom:typecheck`, `boom:test`, `root:specs`, `root:changes` and `root:terms` through `sluice exec main`
