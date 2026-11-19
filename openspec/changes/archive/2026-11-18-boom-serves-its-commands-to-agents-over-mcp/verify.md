# Verify: boom-serves-its-commands-to-agents-over-mcp

Verified on 2026-11-18 at `5e25ad1`, on
`change/boom-serves-its-commands-to-agents-over-mcp`. The branch was cut
from `bolt/boom-holds-the-rules-agents-read`, whose head, `c32ecaf`, is
an ancestor of `5e25ad1`.

**Verdict: the whole checks pass and the work holds, so the change is
archived.** OpenSpec reads all 12 tasks as done, and the change passes
`openspec validate --strict`. Named tests prove all 25 scenarios across
the spec's 6 requirements, and every citation of records 0010 and 0011
names the record it means.

## The whole checks

Run once, as `CLAUDE.md` names them: `sluice exec main`, the gate's
checks (`#check-fast`, `.config/sluice.toml`), in sluice's sandbox, for
what the head affects compared with main (`429dcbc`).

| Task | Outcome |
|---|---|
| `boom:format` | passed |
| `boom:lint` | passed |
| `boom:typecheck` | passed |
| `boom:test` | passed: 238 tests, 0 failed (38 unit; `attachments` 14, `claims` 20, `command_line` 12, `import` 32, `log` 9, `mcp` 23, `persons` 19, `query` 24, `search` 30, `signatures` 6, `store` 11) |
| `claude-code:format` | passed: Biome, 2 files, no fixes |
| `claude-code:lint` | passed: Biome, 2 files, no fixes |
| `claude-code:typecheck` | passed |
| `claude-code:test` | passed: the session hook passes every case |
| `nix:lint` | passed |
| `nix:typecheck` | passed: every case holds |
| `nix:test` | passed: every case holds |
| `site:format` | passed: Biome, 14 files, no fixes |
| `site:lint` | passed: Biome, 17 files, no fixes |
| `site:typecheck` | passed: 0 errors, 0 warnings, 0 hints |
| `site:test` | passed: 4 diagrams, both layouts |
| `site:build` | passed: 12 pages, all internal links valid |
| `root:format` | passed: treefmt, 105 files, 0 changed |
| `root:toolchain` | passed: each tool at its pin |
| `root:specs` | passed: 11 specs, 0 failed |
| `root:changes` | passed: 1 open change, valid under `openspec validate --strict` and archiving to specs that pass it |
| `root:changes-cases` | passed: every case holds |
| `root:versions` | passed: every file gives 0.0.0, and `set-version.sh` writes each |
| `root:release` | passed |
| `root:marketplace` | passed |

`passed: 24 tasks`, exit 0. sluice recorded the 24 results under their
keys for `5e25ad1`. `site:install` ran as a dependency of the site's
tasks.

moon logged warnings without failing. It couldn't write its output
archives to `.moon/cache/outputs` from the sandbox, and it couldn't hash
folders it was given as inputs under `.devenv` and `.moon/cache`. It
found no network, so it skipped installing dependencies. The site's
build warned of an empty `i18n` collection and no `404` entry. None of
these comes from this change.

## OpenSpec

- `openspec instructions apply --change
  boom-serves-its-commands-to-agents-over-mcp --json` gives `state`
  `all_done`: all 12 tasks are done, none open.
- `openspec validate boom-serves-its-commands-to-agents-over-mcp
  --strict` says `Change 'boom-serves-its-commands-to-agents-over-mcp'
  is valid`.

## The tasks

Each task's named tests are in the files the task names, and all of
them pass in `boom:test`: 4 unit tests in `src/mcp.rs` and 23 in
`tests/mcp.rs`, beside `query`'s `if_declared` cases for 2.5. The other
conditions hold for every task:

- **1.1:** `src/mcp.rs` is `pub mod mcp` in `src/lib.rs` and gives
  `serve(input, output, store, env)`. `Command::Mcp` is taken in
  `cli::run` before the store's path is resolved. With `--json` it
  refuses, exit 2. Otherwise it serves standard input and output and
  answers an empty `Outcome`, exit 0. `Place` gains `mcp` and `mcp_in`.
- **1.2:** `one_answer_before_the_next_request` runs
  `CARGO_BIN_EXE_boom` and reads each answer on a thread within ten
  seconds.
- **2.1:** `cli::definition()` is clap's reading of `Cli`. `SERVED`,
  `ABSENT` and `LEFT_OUT` hold what the task names, and each `ABSENT`
  entry and `key` say why. `tools/list` builds each tool from
  `definition()`.
- **2.2:** `NoKeys`'s `signer` refuses every path. Each call runs
  `cli::run(words, &NoKeys, env)` with the words design 4 gives. `Place`
  gains `call`.
- **2.3:** `checked` reads absent `arguments` as `{}`. It refuses a
  non-object, a name the schema lacks, and a value of another type, each
  with `boom-refused/1` before anything runs.
- **2.4:** `search` is in `SERVED` after `query` and in `READ_ONLY`, and
  its tests pass.
- **2.5:** the agent-tools delta says `query` takes `repository` and
  `if_declared`. `a_tools_arguments_are_its_commands` asserts both,
  neither required, and `if_declared` a boolean. `if_declared` is a
  `SetTrue` switch in `cli.rs`, so the schema reads it from the command
  line's definition, as design 3 serves every argument, and a call's
  words give it as `--if-declared`.
- **3.1:** `initialize` gives `INSTRUCTIONS`, design 7's words.
  `boom-propose/1` gains `accept`, and the words gain the `a person
  accepts it with:` line while the version is proposed.
- **4.1, 4.2:** the README says what `boom mcp` serves, names `search`
  among its tools and leaves the person's acts to the person. It shows
  the `.mcp.json` from design 8 and says to name `--store` where the
  harness's environment differs. Its table's row for `crates/boom`
  names `mcp`. `CLAUDE.md` says what `boom mcp` serves and still names
  what isn't built. Every remaining citation of record 0009 is
  claim-search's.
- **4.3:** the record is
  `docs/adr/0011-boom-mcp-serves-an-agent-the-commands-it-may-run-and-none-a-person-must-sign.md`,
  `number: 11`. Record 0011 is cited only by `README.md` line 85,
  `CLAUDE.md` line 22, `src/mcp.rs`, `tests/mcp.rs` and the change's
  proposal, design and agent-tools delta, each for `boom mcp`. Every
  other citation of record 0010 in the tree is the release's.
- **4.4:** `README.md`'s Install section ends its first paragraph with
  `given nothing (record 0010).` at line 135, as #18 wrote it, naming
  `docs/adr/0010-a-boom-release-is-a-tag-on-a-release-commit-and-its-flake-gives-boom-and-its-plugin-at-one-version.md`.

`devenv shell -- vale README.md CLAUDE.md`, the check 4.3 and 4.4 name,
reports 0 errors and 0 warnings, with 9 suggestions. No task adds a
dependency: `Cargo.toml`, `crates/boom/Cargo.toml` and `Cargo.lock` are
as `c32ecaf` has them. The tests make their keys when they run, and
write their own `.config/boom.toml` and MADR files from strings in their
code (core.3, core.5).

## The specs against the work

Each scenario is proved by the test named for it in
`crates/boom/tests/mcp.rs`, or by a unit test in `crates/boom/src/mcp.rs`
marked `unit`. The integration tests drive `boom::mcp::serve` in the
test's own process, through `Place`, on a temporary store of their own.

### agent-tools

**boom mcp speaks the protocol on standard input and output**

| Scenario | Test |
|---|---|
| Initialized | `initialized` |
| One answer before the next request | `one_answer_before_the_next_request` |
| A line that is not JSON | `a_line_that_is_not_json` |
| A method it does not serve | `a_method_it_does_not_serve` |
| Asked for one JSON object | `asked_for_one_json_object` |

**Each tool is one of boom's commands**

| Scenario | Test |
|---|---|
| The tools listed | `the_tools_listed`, with `a_tools_arguments_are_its_commands` holding `show`, `log`, `query` and `search` alone read-only |
| A tool's arguments are its command's | `a_tools_arguments_are_its_commands`: each tool's properties and required arguments as the scenario names them, `query` taking `repository` and `if_declared` |
| Each command served or absent | `each_command_served_or_absent` and `each_argument_of_a_served_command_is_its_tools` (unit) |

**A tool runs its command**

| Scenario | Test |
|---|---|
| A proposal through its tool | `a_proposal_through_its_tool` |
| The command's own answer | `the_commands_own_answer` |
| A search through its tool | `a_search_through_its_tool` |
| A refusal is an error | `a_refusal_is_an_error` |
| Not run is an error | `not_run_is_an_error` |
| Words beginning with a hyphen | `words_beginning_with_a_hyphen`, and `the_words_for_a_call` (unit) |
| The working folder's repository | `the_working_folders_repository` |

**An argument a tool does not take is refused**

| Scenario | Test |
|---|---|
| A store given | `a_store_given` |
| A value of another type | `a_value_of_another_type` |
| A misspelled switch | `a_misspelled_switch` |

**No tool does what only a person may do**

| Scenario | Test |
|---|---|
| No person's act listed | `no_persons_act_listed` |
| An accept asked of the server | `an_accept_asked_of_the_server` |
| A key given to propose | `a_key_given_to_propose` |
| The server holds no key | `the_server_holds_no_key` (unit) |

**What waits on a person is said**

| Scenario | Test |
|---|---|
| The instructions | `the_instructions` |
| A proposal's accept | `a_proposals_accept` |
| A version that stands | `a_version_that_stands` |

The change modifies no other capability. `accept` adds a field to
`boom-propose/1` within its major, and `boom search` and `boom query`
are served as their commands answer, their specs unchanged.
