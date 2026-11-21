## 1. The layers

- [ ] 1.1 Add `hook.rs` with three functions: the session layer of a repository, the edit layer of a context, and the menu. Each takes the claims that the store gives, and gives text. `boom sync` (turn 2b) calls the same functions later. Proved by `devenv shell -- moon run boom:test`
- [ ] 1.2 The session layer: the first line, each invariant then each term as one line, a count and the names of the decisions, the assumptions and the contexts, then the menu. No scenario, no body, and no line for a kind with no claim. Proved by `boom:test`
- [ ] 1.3 The edit layer: the first line, why the edit stopped, the context with its globs, each rule with its body as `boom show` gives it, then the decisions and the assumptions by name. Proved by `boom:test`

## 2. The commands

- [ ] 2.1 Add `boom hook session-start` and `boom hook pre-tool-use` to `cli.rs`. Each reads one event on standard input, takes `--store`, refuses `--json`, and writes one JSON object of Claude Code's hook output, or nothing. Proved by `boom:test`
- [ ] 2.2 `boom hook session-start` finds the repository from the `cwd` of the event, as `boom query --if-declared` finds it, and gives `systemMessage` and `additionalContext`. Proved by `boom:test`
- [ ] 2.3 `boom hook pre-tool-use` runs the terms check of `terms.rs` on the written text, then finds the contexts of the file with `context::falls_in`, from the path in its worktree. It gives one stop with the old words, the rules, or both, and `systemMessage`. Proved by `boom:test`
- [ ] 2.4 Each hook reads the store as `boom terms check` reads it, and writes nothing into it. Each is silent where boom finds no store or no repository. Delivery off silences the rules and not the terms check. A store that cannot be read gives exit 3. Proved by `boom:test`, with the bytes of the store file compared before and after each run

## 3. The record of a session

- [ ] 3.1 Keep the record in `$XDG_STATE_HOME/boom/sessions/<session_id>/`, else `~/.local/state/boom/sessions/<session_id>/`, one file for each context, made with `create_new`. Refuse a `session_id` with a character other than a letter, a digit, `-` or `_`. Proved by `boom:test`
- [ ] 3.2 Delete the record on `source` `clear` or `compact`, keep it on `resume`, and delete each record that no hook changed in 30 days. Proved by `boom:test`
- [ ] 3.3 Test two `pre-tool-use` hooks of one session that run at one time in one new context: one gives the rules. Proved by `boom:test`

## 4. The plugin

- [ ] 4.1 `hooks/hooks.json`: `SessionStart` runs `boom hook session-start`, and one `PreToolUse` hook for `Write|Edit|MultiEdit` runs `boom hook pre-tool-use`. Write the descriptions of the hooks and of the plugin in Plain. Proved by `devenv shell -- moon run claude-code:format claude-code:lint claude-code:typecheck`
- [ ] 4.2 `tests/session.sh` and `tests/write.sh`: the hooks run as Claude Code runs them, with a recording `boom` and with the boom the tree builds, for each scenario of `claude-code-plugin`. Add a test of the first edit in a context, and of the edit made again. Proved by `claude-code:test`
- [ ] 4.3 Test a stop whose reason passes 10,000 characters on the pinned Claude Code, and write what the agent gets into the design. Where Claude Code cuts the reason, give the rules without bodies above the cap. Proved by `claude-code:test`

## 5. The pages and the gate

- [ ] 5.1 Write the layers, `boom hook` and the record of a session into `README.md` and `CLAUDE.md`, in Plain. Proved by `devenv shell -- moon run root:format root:terms`
- [ ] 5.2 Run `boom terms check` on each file that the change writes, before each commit
- [ ] 5.3 Run `boom:format`, `boom:lint`, `boom:typecheck`, `boom:test`, `claude-code:test`, `root:specs`, `root:changes` and `root:terms` through `sluice exec main -- moon run <task>`, then `sluice exec main` once the work is committed
