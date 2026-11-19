# Verify: boom-is-released-by-version

The work on `change/boom-is-released-by-version` at 732712e meets the
change: the whole checks pass, every task is done, the change passes
OpenSpec's strict validation, and every requirement and scenario of its
three specs is met by the work.

## The whole checks

`sluice exec main`, run once on 732712e as CLAUDE.md names the gate's
checks: 24 tasks passed, recorded under their keys for 732712e. These
are every task tagged `check-fast` in the five projects, with
`site:install` run as their dependency:

| project | tasks |
|---|---|
| `boom` | format, lint, typecheck, test |
| `claude-code` | format, lint, typecheck, test |
| `nix` | lint, typecheck, test |
| `site` | format, lint, typecheck, test, build |
| `root` | format, toolchain, specs, changes, changes-cases, versions, release, marketplace |

## OpenSpec

- `devenv shell -- openspec validate boom-is-released-by-version --strict`:
  `Change 'boom-is-released-by-version' is valid`.
- `devenv shell -- openspec instructions apply --change boom-is-released-by-version --json`:
  `state` `all_done`, all 15 tasks (1.1 to 6.2) `done: true`.
- `root:changes` and `root:specs`, in the gate, also passed: the change
  archives cleanly onto a scratch copy of the specs, and the specs it
  writes pass strict validation (core.7).

## Requirements and scenarios

### release-version

| requirement | scenario | met by |
|---|---|---|
| main's tree names no release | main's files | `root:versions` passes; `Cargo.toml`, `Cargo.lock`'s `boom` and the plugin manifest give `0.0.0` |
| | A file that gives another version | probe on a scratch copy with the manifest at `0.1.0`: `versions: plugins/claude-code/.claude-plugin/plugin.json gives 0.1.0, and the workspace 0.0.0`, exit 1 |
| | A version in the marketplace | probe on a scratch copy with a marketplace version: `versions: .claude-plugin/marketplace.json gives boom the version 0.0.0; ...`, exit 1 |
| One writer writes a release's version, all or none | A version written; Not a version; A file with no version | `check-versions.sh` runs each on a scratch copy (`98.76.54` written with no other line changed, `0.98` refused naming it, a manifest with no `version` refused naming it, each leaving every file as it was and no `.set-version` file), under `root:versions`, which passed |
| A landing with a new version is released | Only a landing | `release.yml`'s `version` job: `if: needs.landing.outputs.tested != ''`, `git-cliff --unreleased --bumped-version`, an existing tag handed on as none; held by `root:release`; `cliff.toml`'s `tag_pattern` `v*` and `initial_tag` `v0.1.0` |
| A release's tag names a release commit off main | The release job | `release` job: `set-version.sh "${TAG#v}"`, `commit -m "chore(release): $TAG" -- "${files[@]}"` from `--files`, `git tag`, `git push origin "refs/tags/$TAG"` its one push, `gh release create "$TAG" --verify-tag`; no other job pushes; held by `root:release` |
| A build takes its version from its tree alone | The flake's package | `nix:test` "The package's version is the tree's", each of the three systems |
| | Nothing handed to the build | `root:release` fails on any `--impure` or `BOOM_VERSION`; the workflow holds neither |
| A release's binaries run with no Nix | The binaries job | `binaries` job: musl Linux targets and `aarch64-apple-darwin`, the writer before `cargo build --release --locked -p boom --target "$TARGET"`, `--version` tested against the tag, no `install-nix-action` and no `nix` command; Rust from `rust-toolchain.toml` (1.98.1, as `.prototools`); held by `root:release` |
| The flake gives a release's boom and plugin at one version | Every system evaluates | `nix:typecheck`: both packages of aarch64-darwin, aarch64-linux and x86_64-linux evaluate to a derivation against no store, and no other system is given |
| | The plugin at boom's version | `nix:test` "The plugin is at boom's version" and "The plugin is the tree's" |

### claude-code-plugin

| requirement | scenario | met by |
|---|---|---|
| The plugin and its marketplace | The plugin validated | `claude-code:typecheck`, `claude plugin validate --strict .` by the pinned Claude Code |
| | The marketplace validated | `root:marketplace`, the same on the root; the marketplace `boom` lists the plugin `boom` by `./plugins/claude-code`, with no version |
| A session starts with its repository's claims | The hook runs boom | `claude-code:test` "The hook runs boom": the command from `hooks/hooks.json` run by `sh -c`, boom called as `query --if-declared` in the repository's folder, its line passed on as it is |
| Silent where no repository is declared | A repository that declares nothing; Outside a repository | `claude-code:test`, with the boom the tree builds: exit 0, both streams empty, nothing under `XDG_DATA_HOME` |
| A declaration boom cannot read is shown | A name not in kebab case | `claude-code:test`: exit 2, standard error naming `.config/boom.toml` |

### repository-rules

| requirement | scenario | met by |
|---|---|---|
| Nothing where nothing is declared, when asked | Declared | `if_declared_answers_as_without_it`, in words and `--json`, from the repository and a folder within it |
| | Nothing declared | `if_declared_with_nothing_declared_answers_nothing`, no store made |
| | Nothing declared, outside a repository | `if_declared_outside_a_repository_answers_nothing` |
| | Nothing declared, answered as JSON | `if_declared_with_nothing_declared_answers_null_json` |
| | A declaration it cannot read | `if_declared_refuses_a_declaration_it_cannot_read`, the log unchanged |
| | Named as well | `if_declared_with_a_repository_is_refused`, clap's `conflicts_with` |

All in `crates/boom/tests/query.rs`, under `boom:test`, which passed.

## Records and rules

Record 0010 is in `docs/adr/`; `core.md` amends core.5 to name every
plugin and the flake among the moon projects and adds core.8, citing
record 0010. The README says how a machine installs a release and the
plugin, and CLAUDE.md names the projects `claude-code` and `nix` and
says boom is released by version.

## Observations

These do not block the archive:

- `.moon/workspace.yml`'s opening comment still lists the check-fast
  layer as the crate's, the site's and the root's format, toolchain,
  specs and changes, without the `claude-code` and `nix` projects or
  `root:versions`, `root:release` and `root:marketplace`. The projects
  and their tags are right; only the comment lags.
- Record 0009 is main's and is not on this branch, which was cut from
  `bolt/boom-holds-the-rules-agents-read` before it landed; record
  0010 is numbered after it, as design 11 says.
