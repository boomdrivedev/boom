No task adds a part of the repository. The page is Rust code of the
crate `boom`, so this change adds no moon project. Each task names the
moon task that proves it, run as `devenv shell -- moon run
<project>:<task>`. Each scenario of the delta specs is one test, named
by its scenario, in the test file that its task names.

## 1. The kind `narrative`

- [x] 1.1 Add `Kind::Narrative` to `claim.rs`. A narrative needs a body, and its body is UTF-8 text. `boom propose narrative` refuses `--changes`. Tests in `crates/boom/tests/narratives.rs` for the scenarios of `narratives`, "A narrative has a title and a body", and of `claim-versions`, "A claim and its name". Proved by `boom:test`
- [x] 1.2 Add the edge `includes` to `Proposal`, written only where there is one, and resolve each `--includes` as `--source` is resolved. Refuse it on a claim that is not a narrative, and refuse an include of a narrative. Tests in `narratives.rs` for "A narrative includes claims at the version it meant", and in `crates/boom/tests/edges.rs` for the new scenario of "A version stores its edges". Proved by `boom:test`
- [x] 1.3 Add `answers` to `Proposal`, written only where there is one, and add `includes` and `answers` to the content of a version. Tests in `crates/boom/tests/claims.rs` for "One more include" and "A version from before narratives": a store that this tree's main wrote gives each version the fingerprint that its entry names. Proved by `boom:test`
- [x] 1.4 Test the accept of two invariants and a narrative with one signature, and a session start that gives no line for the narrative. Tests in `narratives.rs` and `crates/boom/tests/hooks.rs` for "A narrative is judged by a person and never checked". Proved by `boom:test`

## 2. The paragraphs of a body

- [x] 2.1 Add `pulldown-cmark` to `crates/boom/Cargo.toml` at one version, with its default features off and `html` on. `Cargo.lock` pins it. Proved by `boom:typecheck` and `nix:typecheck`
- [x] 2.2 Add `narrative.rs` with one function that gives the paragraphs of a body: the number, the kind, the bytes and the fingerprint of each block at the top of the body and each item of a list at the top. Tests in `narratives.rs` for "The paragraphs of a body" and "boom show gives the paragraphs of a version". Proved by `boom:test`

## 3. Notes

- [x] 3.1 Add the act `noted` to `entry.rs`, with the narrative's name, the version, the paragraph, its fingerprint and the words. The judge counts an unsigned entry of a person for `noted` only, and only for a person that the store registered before it. Tests in `crates/boom/tests/notes.rs` for "Only a note counts with no signature", in `crates/boom/tests/persons.rs` for "A note with no signature", and in `crates/boom/tests/log.rs` for "A note is an entry". Proved by `boom:test`
- [x] 3.2 Add `boom note <name>@<version> --paragraph <n> "<words>"` to `cli.rs`, with `--person`, `--store` and `--json`, answering `boom-note/1`. It refuses `--key` and `--provenance`. Tests in `notes.rs` for "A person writes a note on a paragraph", "A note names what it meant" and "A note's name", and in `crates/boom/tests/command_line.rs` for "A note answered as JSON". Proved by `boom:test`
- [x] 3.3 Read open notes from the log: a note is open until a version of its narrative answers it and no person rejects that version. `boom propose --answers <note>=<words>` answers a note. Boom refuses a new version of a narrative that leaves a note of the narrative open, and names each open note. Tests in `notes.rs` for "A note is open until a version answers it". Proved by `boom:test`
- [x] 3.4 `boom show` gives a narrative's title, body, includes with their mark out of date, and its notes. Tests in `narratives.rs` for "boom show gives a narrative". Proved by `boom:test`

## 4. The page

- [x] 4.1 Add `tiny_http` to `crates/boom/Cargo.toml` at one version. `Cargo.lock` pins it. Proved by `boom:typecheck` and `nix:typecheck`
- [x] 4.2 Add `page.rs`: functions that give the HTML of the page of a version and of the list of narratives, from the views. The HTML holds its style and no script, and gives each raw HTML of a body as text. Tests in `crates/boom/tests/page.rs` for "What the page gives", "The notes on the page", "The included claims on the page", "The list of narratives" and "The page runs nothing from a body or another host". Proved by `boom:test`
- [x] 4.3 Add `boom page [<name>[@<version>]]` to `cli.rs`, with `--port`, `--no-open`, `--idle`, `--person` and `--store`. It binds `127.0.0.1`, makes the token from the random source of the operating system, prints its line, and opens the URL with `open` or `xdg-open`. Each response is computed from the views at the last entry of the log, and has the policy header. Tests in `page.rs` that start `boom page --no-open` and send requests over a `TcpStream` on `127.0.0.1`, for "boom page serves the page on the machine", "Only the browser of the person reads the page", "boom page stops" and "The page is computed from the log for each request". Proved by `boom:test`
- [x] 4.4 A `POST` of the form of a paragraph appends the entry of `boom note` for the person of the page, and answers 303 to `#p<n>`. Refuse a `POST` whose `Origin` is not the page's own. Tests in `page.rs` for "A note written on the page". Proved by `boom:test`
- [x] 4.5 Add `boom note` and `boom page` to the commands of the command line. Tests in `command_line.rs` for "The commands". Proved by `boom:test`

## 5. `boom mcp`

- [x] 5.1 `mcp.rs`: the tool `propose` takes the kind `narrative`, `includes` and `answers`. `note` and `page` are named as absent. The instructions say that only a person writes a note. Tests in `crates/boom/tests/mcp.rs` for "Each tool is one of boom's commands" and "No tool writes a note". Proved by `boom:test`

## 6. The pages and the gate

- [x] 6.1 Write the narrative, `boom note` and `boom page` into `README.md` and `CLAUDE.md`, in Plain. Proved by `devenv shell -- moon run root:format root:terms`
- [x] 6.2 Run `boom terms check` on each file that the change writes, before each commit
- [ ] 6.3 Run `boom:format`, `boom:lint`, `boom:typecheck`, `boom:test`, `nix:typecheck`, `root:specs`, `root:changes` and `root:terms` through `sluice exec main -- moon run <task>`, then `sluice exec main` once the work is committed
