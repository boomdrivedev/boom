## 1. The skills

- [x] 1.1 Write `using-boom`, `proposing-claims` with `kinds.md`, and `writing-narratives` under `plugins/claude-code/skills/`, in Plain
- [x] 1.2 Add `skills/**/*` to the inputs of `claude-code:typecheck` and `claude-code:test`. `claude-code:typecheck` proves the scenario "The skills validated"
- [x] 1.3 Add `tests/skills.sh`, which lists the files of `skills/` and reads none of them, to `claude-code:test`. `claude-code:test` proves the scenario "Each skill at its path"
- [x] 1.4 Name the skills in the descriptions of `plugins/claude-code/.claude-plugin/plugin.json` and `.claude-plugin/marketplace.json`. `claude-code:typecheck` and `root:marketplace` prove that both pass

## 2. The warning

- [x] 2.1 Add the list of plan words to `crates/boom/src/terms.rs`, matched by `terms::find`. `boom:test` proves the scenarios "Another form", "A plan word inside a longer word" and "A marker on the line"
- [x] 2.2 Write each warning on standard error in `propose`, and `plan_words` in its JSON. `boom:test` proves the scenarios "A plan word in the statement", "A plan word in the body", "The warnings as JSON", "No plan word" and "The same version again"
- [x] 2.3 `boom:test` proves the scenario "The MCP tool warns too"

## 3. The pages and the gate

- [x] 3.1 Write the skills and the warning into `README.md` and `CLAUDE.md`, in Plain
- [x] 3.2 Run `boom:format`, `boom:lint`, `boom:typecheck`, `boom:test`, `claude-code:format`, `claude-code:lint`, `claude-code:typecheck`, `claude-code:test`, `root:specs`, `root:changes`, `root:terms` and `root:marketplace` through `sluice exec main`
