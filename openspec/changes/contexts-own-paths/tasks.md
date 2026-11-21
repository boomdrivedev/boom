## 1. The kind

- [x] 1.1 Add `Kind::Context` to `claim.rs`: a statement, an optional body, and one glob at least. Proved by `devenv shell -- moon run boom:test`
- [x] 1.2 Add `paths` to the proposal in `entry.rs`, each glob one time and sorted. Put the globs of a context into the content of its fingerprint. Omit an empty list from the entry and from the content. Proved by `boom:test`
- [x] 1.3 Check a glob: `<repository>:<pattern>`, a repository in kebab case, a relative pattern with no `/` at its start, no `..` part and no `\`, and a pattern that `globset` compiles with `literal_separator`. Add `globset` at a pinned version to `Cargo.toml` and `Cargo.lock`. Proved by `boom:test` and `boom:lint`
- [x] 1.4 Write the repositories of the globs of a context as its attachment. Refuse `--repository`, `--system` and `--context` on a context. Proved by `boom:test`
- [x] 1.5 Test that each version of a store from before contexts keeps its fingerprint and its attachment, terms and edges included. Proved by `boom:test`

## 2. The binding

- [x] 2.1 Add `contexts` to `Attach` in `entry.rs`, as `<name>@<version>`, sorted by name, each one time. Proved by `boom:test`
- [x] 2.2 Resolve `--context <name>` to the accepted version of the context, else its newest version. Refuse a name that is not a context, a version that the context does not have, and `--context` on a term. Proved by `boom:test`
- [x] 2.3 Compare two attachments by their repositories and the names of their contexts. Proved by `boom:test`
- [x] 2.4 Give an amendment that names nothing the repositories of its base, and the contexts of its base at their accepted versions now. Proved by `boom:test`
- [x] 2.5 Judge the contexts of an attachment on every read in `judge.rs`, with the reason in `boom log`. Proved by `boom:test`

## 3. The views

- [x] 3.1 Keep a view row for each glob of a context version and each binding of a version, built from the log, and drop them with the views. Proved by `boom:test`, with `boom verify`

## 4. The commands

- [x] 4.1 `boom propose` takes `--path` and `--context`, each any number of times. Proved by `boom:test`
- [x] 4.2 One function in the library gives the contexts that `<repository>:<path>` falls in, from the globs of the accepted contexts. `boom query`, `boom search` and later turns call it. Proved by `boom:test`
- [x] 4.3 `boom query --path` gives the claims for each path, with `paths` and `contexts` in `boom-query/2` and one line for each path in words. It refuses paths of two repositories, `--repository` and `--if-declared`. Without `--path`, it gives no claim that binds only contexts. Proved by `boom:test`
- [x] 4.4 `boom search --repository` searches the claims bound to the accepted contexts of the repository. Proved by `boom:test`
- [x] 4.5 `boom show` gives the globs of a context, each binding with the version that it meant and a mark where it is out of date, and the claims bound to a context under `bound:`. Proved by `boom:test`
- [x] 4.6 The MCP tool `propose` takes the kind `context`, `paths` and `contexts`. The tool `query` takes `paths`. Proved by `boom:test`

## 5. The pages and the gate

- [x] 5.1 Write contexts and `boom query --path` into `README.md` and `CLAUDE.md`, in Plain. Proved by `devenv shell -- moon run root:format root:terms`
- [x] 5.2 Test on a store with the contexts of boom and of sluice: a file in a context, a file in no context, a glob that does not cross a folder, and a context with a new glob. Proved by `boom:test`
- [x] 5.3 Run `boom:format`, `boom:lint`, `boom:typecheck`, `boom:test`, `root:specs`, `root:changes` and `root:terms` through `sluice exec main -- moon run <task>`
