## 1. The kind

- [x] 1.1 Add `Kind::Term` to `claim.rs`, with its rules: a statement, an optional body, and one repository
- [x] 1.2 Add `replaces` to the proposal entry in `entry.rs`. Keep each word one time, in lowercase and sorted. Omit an empty list from the entry and from the content of the fingerprint
- [x] 1.3 Refuse `replaces` on another kind, an empty word, a word of two lines, and the word of the term
- [x] 1.4 Refuse a first term version with no repository, two or more, or the whole system
- [x] 1.5 Put the repository of a term into the content of its fingerprint. Keep a term in the views under its word and its repository, so one word can have a term in each repository
- [x] 1.6 Refuse a term with the name of a claim of another kind, and a claim of another kind with the word of a term
- [x] 1.7 Test that each version of a store from before terms keeps its fingerprint
- [x] 1.8 Say "accepted", not "stands", in each refusal of `judge.rs`

## 2. The commands

- [x] 2.1 `boom propose term` takes `--replaces`, any number of times
- [x] 2.2 `boom show` gives `replaces` in words and in `boom-show/2`. For a word with a term in two repositories, `boom show <word>` gives each, and `--repository` chooses one
- [x] 2.3 `boom query` gives the terms after the assumptions, and never `replaces`
- [x] 2.4 `boom search` gives terms as claims, and does not search `replaces`
- [x] 2.5 The MCP tool `propose` takes the kind `term` and `replaces`. Name `terms check` as absent

## 3. The check

- [x] 3.1 Add `boom terms check <file>...` with `--repository`, `--if-declared`, `--hook`, `--store` and `--json`
- [x] 3.2 Find the repository of each file from its folder, as `repository.rs` finds it from the working folder
- [x] 3.3 Read the accepted terms of each repository from the store, and write nothing, not even the views: replay the log in memory where the views are behind, and read the store as a file that does not change where SQLite cannot share its memory file
- [x] 3.4 The matcher: split names in code, fold case and accents, take stems with the tokenizer of `boom search`, and match whole words and words in order
- [x] 3.5 Exit 2 with `boom-refused/1` and `refused.found` when the check finds an old word. Exit 0 with `boom-terms-check/1` when it finds none
- [x] 3.6 With `--hook`, read one `PreToolUse` event, and check the text that `Write`, `Edit` or `MultiEdit` writes

## 4. The plugin

- [x] 4.1 Add the `PreToolUse` hook to `plugins/claude-code/hooks/hooks.json`
- [x] 4.2 Add the tests of the plugin. Boom runs as the hook says. An old word stops a write. A repository that declares nothing gets silence

## 5. The gate task

- [x] 5.1 Read the marker `boom: allow <old word>` in a file and in the text of a hook event
- [x] 5.2 Add the task `root:terms` to `moon.yml`: it runs `boom terms check` on each file that the change adds or modifies, except under `openspec/changes/archive/`. It reads the store, outside the tree, so it never replays, and it names the entry of the store's log that it read
- [x] 5.3 Test the marker, a change that adds an old word, a file that the change does not touch, and an archived change
- [x] 5.4 Land this change only after the user accepts `no-task-reads-markdown@a25dda6425d7`

## 6. The pages and the gate

- [x] 6.1 Write the term kind and the check into `README.md` and `CLAUDE.md`, in Plain
- [x] 6.2 Run `boom:format`, `boom:lint`, `boom:typecheck`, `boom:test`, the checks of `claude-code`, `root:specs` and `root:changes` through `sluice exec main`
