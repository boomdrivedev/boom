# Verify: boom-serves-its-commands-to-agents-over-mcp

Verified on 2026-11-18 at `d82f568`, on
`change/boom-serves-its-commands-to-agents-over-mcp`. The branch was cut
from `bolt/boom-holds-the-rules-agents-read`, whose head, `526f843`, is
an ancestor of `d82f568`.

**Verdict: the work holds, so the change is archived.** The whole
checks pass. OpenSpec reads all 9 tasks as done, and the change passes
`openspec validate --strict`. Named tests prove all 25 scenarios across
the spec's 6 requirements.

## The whole checks

Run once, as `CLAUDE.md` names them: `sluice exec main`, the gate's
checks (`#check-fast`, `.config/sluice.toml`), in sluice's sandbox, for
what the head affects compared with main (`429dcbc`).

| Task | Outcome |
|---|---|
| `boom:format` | passed |
| `boom:lint` | passed |
| `boom:typecheck` | passed |
| `boom:test` | passed: 232 tests, 0 failed (38 unit; `attachments` 14, `claims` 20, `command_line` 12, `import` 32, `log` 9, `mcp` 23, `persons` 19, `query` 18, `search` 30, `signatures` 6, `store` 11) |
| `root:format` | passed: treefmt, 83 files, 0 changed |
| `root:specs` | passed: 9 specs, 0 failed |
| `root:changes` | passed: 1 open change, valid under `openspec validate --strict` and archiving to specs that pass it |

`passed: 7 tasks`, exit 0. sluice recorded the 7 results under their
keys for `d82f568`. The site's tasks, `root:toolchain` and
`root:changes-cases` didn't run, because the head changes nothing they
read.

moon logged warnings without failing. It couldn't write its output
archives to `.moon/cache/outputs` from the sandbox. It was given folders
and files under `target/debug/incremental` as inputs that it couldn't
hash. It found no network, so it skipped installing dependencies. None
of these comes from this change.

## OpenSpec

- `openspec instructions apply --change
  boom-serves-its-commands-to-agents-over-mcp --json` gives `state`
  `all_done`: all 9 tasks are done, none open.
- `openspec validate boom-serves-its-commands-to-agents-over-mcp
  --strict` says `Change 'boom-serves-its-commands-to-agents-over-mcp'
  is valid`.

## The tasks

Each task's named tests are in the files the task names. All 27 of them
pass in `boom:test`: 4 unit tests in `src/mcp.rs` and 23 in
`tests/mcp.rs`. The other conditions hold too:

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
  `cli::run(words, &NoKeys, env)` with the words design 4 gives: each
  option as `--<long>=<value>`, then `--json`, the store, `--` and the
  positional values. `Place` gains `call`, and also `call_in` for the
  working-folder scenario.
- **2.3:** `checked` reads absent `arguments` as `{}`. It refuses a
  non-object, a name the schema lacks, and a value of another type, each
  with `boom-refused/1` before anything runs.
- **2.4:** `search` is in `SERVED` after `query` and in `READ_ONLY`.
  `the_tools_listed` names it, `a_tools_arguments_are_its_commands`
  asserts `words` an array of strings, required, beside `repository`,
  `all` and `limit`, and holds `search` read-only.
  `a_search_through_its_tool` passes.
- **3.1:** `initialize` gives `INSTRUCTIONS`, design 7's words.
  `boom-propose/1` gains `accept`, and the words gain the `a person
  accepts it with:` line while the version is proposed.
- **4.1:** the README says what `boom mcp` serves and what it leaves to
  a person. It shows the `.mcp.json` from design 8 and says to name
  `--store` where the harness's environment differs. Its table's row
  for `crates/boom` names `mcp`. `CLAUDE.md` says what `boom mcp`
  serves and still names what isn't built.
- **4.2:** the record is
  `docs/adr/0010-boom-mcp-serves-an-agent-the-commands-it-may-run-and-none-a-person-must-sign.md`,
  `number: 10`, naming `search` among its tools. `src/mcp.rs`,
  `tests/mcp.rs`, `README.md` and `CLAUDE.md` cite it as record 0010.
  Every remaining citation of record 0009 is claim-search's. The README
  and `CLAUDE.md` name `search` among `boom mcp`'s tools.

`devenv shell -- vale README.md`, the check both 4.1 and 4.2 name,
reports 0 errors and 0 warnings. It makes 1 suggestion,
`Boom.SentenceLength` at line 68, on claim-search's sentence, which this
change doesn't touch.

No task adds a dependency: `Cargo.toml`, `crates/boom/Cargo.toml` and
`Cargo.lock` are unchanged since `526f843`. The tests make their keys
when they run, and write their own `.config/boom.toml` and MADR files
from strings in their code (core.3, core.5).

## The specs against the work

Each scenario is proved by the test named for it in
`crates/boom/tests/mcp.rs`, or by a unit test in `crates/boom/src/mcp.rs`
marked `unit`. The integration tests drive `boom::mcp::serve` in the
test's own process, through `Place`, on a temporary store of their own.

### agent-tools

**boom mcp speaks the protocol on standard input and output**

| Scenario | Test |
|---|---|
| Initialized | `initialized`: protocol version, `tools`, `serverInfo` `boom` and the crate's version; nothing for the notification; `{}` for `ping` |
| One answer before the next request | `one_answer_before_the_next_request`: the binary answers each request before the next is written, and exits 0 |
| A line that is not JSON | `a_line_that_is_not_json`: `-32700` with a null id, then `ping` answered |
| A method it does not serve | `a_method_it_does_not_serve`: `-32601` naming `resources/list` |
| Asked for one JSON object | `asked_for_one_json_object`: exit 2, one `boom-refused/1` line saying it answers in the protocol's messages |

**Each tool is one of boom's commands**

| Scenario | Test |
|---|---|
| The tools listed | `the_tools_listed`: exactly the seven tools, each with a description and an object schema, `additionalProperties` false; `a_tools_arguments_are_its_commands` holds `show`, `log`, `query` and `search` alone read-only |
| A tool's arguments are its command's | `a_tools_arguments_are_its_commands`: each tool's properties and required arguments as the scenario names them, `kind`'s `enum`, and `search`'s `words` an array of strings, required |
| Each command served or absent | `each_command_served_or_absent` and `each_argument_of_a_served_command_is_its_tools` (unit): every leaf of `definition()`, `search` among them, is in exactly one of `SERVED` and `ABSENT`, and every argument but `key` and the global ones is its tool's |

**A tool runs its command**

| Scenario | Test |
|---|---|
| A proposal through its tool | `a_proposal_through_its_tool`: not an error, `boom-propose/1` naming the version and `["shop"]`, one `proposed` entry by provenance `run-1` with no signature |
| The command's own answer | `the_commands_own_answer`: the result's text equals `boom query --repository shop --json`'s output on the same store |
| A search through its tool | `a_search_through_its_tool`: not an error, the result's text equals `boom search currency --repository shop --json`'s output on the same store, a `boom-search/1` object naming the standing version |
| A refusal is an error | `a_refusal_is_an_error`: an error, `boom-refused/1` saying an agent's proposal names its provenance, the log unchanged |
| Not run is an error | `not_run_is_an_error`: an error, `boom-not-run/1` saying no store is there, and no store made |
| Words beginning with a hyphen | `words_beginning_with_a_hyphen`, and `the_words_for_a_call` (unit) for the `=` form and the `--`: the statement and scenario shown as written |
| The working folder's repository | `the_working_folders_repository`: `query` with no `repository`, served from a folder declaring `shop`, answers for `shop` |

**An argument a tool does not take is refused**

| Scenario | Test |
|---|---|
| A store given | `a_store_given`: an error naming `store`, and the other file never made |
| A value of another type | `a_value_of_another_type`: an error naming `scenarios`, the log unchanged |
| A misspelled switch | `a_misspelled_switch`: an error naming `dryrun`, the log unchanged; the same call without it imports |

**No tool does what only a person may do**

| Scenario | Test |
|---|---|
| No person's act listed | `no_persons_act_listed`: no `init`, `register`, `accept`, `reject` or `mcp`, and no tool takes `key` |
| An accept asked of the server | `an_accept_asked_of_the_server`: `-32602` naming `accept`, the version still proposed |
| A key given to propose | `a_key_given_to_propose`: an error naming `key`, the tests' `SoftKeys` signing nothing, the log unchanged |
| The server holds no key | `the_server_holds_no_key` (unit): `NoKeys` refuses every path saying `boom mcp` signs nothing, and an `init` run with it refuses before a store is made |

**What waits on a person is said**

| Scenario | Test |
|---|---|
| The instructions | `the_instructions`: they name `boom accept` and `boom reject` and say `in their own terminal` |
| A proposal's accept | `a_proposals_accept`: `accept` by `boom propose --json` and by the tool, and the line in the command's words |
| A version that stands | `a_version_that_stands`: `status` `standing`, `accept` null, and no line in the words |

The change modifies no other capability. `command-line`'s `boom-propose/1`
scenario names the claim, its version and its status, and `accept` adds
a field within the major without changing those. `claim-search`'s
`boom search` is served as its command answers, its spec unchanged.
