## 1. The settings in the log

- [x] 1.1 Add the act `repository` to `entry.rs`: a name, a remote and the delivery at session start
- [x] 1.2 Judge it in `judge.rs`. It needs the signature of an owner, a kebab-case name, and a remote that no other repository has. A repeat appends nothing
- [x] 1.3 Keep a view row for each repository in `store.rs`, built from the log
- [x] 1.4 Read a remote into one form, with tests for the `https://`, `ssh://` and `git@<host>:` forms

## 2. The repository of a session

- [x] 2.1 Find the repository in `repository.rs` from `--repository`, then `BOOM_REPOSITORY`, then the remotes in the git configuration, for a worktree too, and start no process
- [x] 2.2 Stop reading `.config/boom.toml`
- [x] 2.3 Use the one order in `query`, `search`, `propose` and the tools of `boom mcp`
- [x] 2.4 Make `boom query --if-declared` silent where it finds no repository or the delivery is off, and make no store

## 3. The commands

- [x] 3.1 Add `boom repo add`, `boom repo set` and `boom repo show`, with `boom-repo/1`
- [x] 3.2 Name the `repo` commands as absent in `boom mcp`
- [x] 3.3 Make the first line of `boom query` name the repository, the number of claims and the entry

## 4. The plugin and the proof

- [x] 4.1 Update the tests of the hook for a registered remote, delivery off, and a name from the launcher
- [x] 4.2 Write the repository of a session into `README.md` and `CLAUDE.md`, in Plain
- [ ] 4.3 On deneb, register `sluice` with its remote. Then start a new Claude session in a worktree of sluice, with the plugin of boom and no brief. The hook shows the claims of sluice
- [x] 4.4 Run `boom:format`, `boom:lint`, `boom:typecheck`, `boom:test`, the checks of `claude-code`, `root:specs` and `root:changes` through `sluice exec main`
