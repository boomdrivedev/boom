# Design

## Context

boom's tree names `0.0.0` in `Cargo.toml`'s `[workspace.package]`,
which `crates/boom` takes, and `boom --version` prints clap's
`boom <CARGO_PKG_VERSION>`. Nothing tags a commit, no workflow but the
site's runs on GitHub, and the tree has no flake and no plugin. sluice
is boom's only gate: every push to main is a sluice landing, whose
commit carries `Sluice-Tested: <candidate>` (`.config/sluice.toml`).
The README tells a repository to name `boom query` as a `SessionStart`
hook in its own `.claude/settings.json`.

The family already releases this way, and boom copies its form rather
than inventing one (CLAUDE.md, the family):

- sluice's record 0111, with its amendment of 2026-11-18, Flume's
  release and crew's `code.10`: a release's tag names a release commit
  off main that writes the release's version into every file that
  carries it, so a build of the tag reports it with nothing given
  (blueprints principle 14); its binaries are cargo's, run on a runner
  with no Nix. timber's record 0031 does the same for timber's own
  GitHub release.
- boomdrive record 0023: Boomdrive's flake takes each tool's package
  from the tool's own flake at a release tag; blueprints principle 17:
  that flake gives the tool's plugins from the same revision, so a
  plugin's version is its command line's by construction.
- timber's `nix/` (record 0035, at `boomdrivedev/timber` 1adfecc7) and
  sluice's at b6c30e78: a moon project `nix` that lints the flake with
  statix and evaluates it offline against no store; timber's
  `plugins/claude.sh` and its pinned Claude Code, which check a plugin
  with `claude plugin validate --strict`.

The rules this design relies on: core.3 (the workflow names no
credential but the run's own `github.token`), core.4 (nothing
commercial in the plugin, the notes or the README), core.5 (each new
file type claimed by a formatter; the plugin and the flake moon
projects of their own, one task per check; no check reads Markdown:
`root:release` reads a YAML workflow and `root:versions` TOML, JSON and
a lock), core.6 (Claude Code, nix, statix and jq pinned by
`devenv.nix` and asserted by `check-toolchain.sh`; the release built by
the Rust `.prototools` pins) and core.7 (this change validated as it
would archive). It reaches no control plane. Record 0010 records the
choices below, each this change's own and cheap to change.

## Goals / Non-Goals

**Goals:**

- A release's tag names a tree that says its version, so a cargo build,
  a nix build of the tag and the release's binaries all say it, given
  nothing.
- A machine installs boom and its plugin from one tag, at one version,
  through boom's flake or, without Nix, from the GitHub release and
  Claude Code's marketplace at the tag.
- Every agent session in a repository that declares itself starts with
  the claims that stand for it, and a session elsewhere is not
  interrupted.

**Non-Goals:**

- Boomdrive's set taking boom: an input at boom's tag, its package
  installed and its plugin in the set's marketplace. That is
  boomdrivedev/boomdrive's, after boom's first release.
- The plugin's hook before each edit (`hook-before-each-edit`, which
  needs a claim's paths), its skills and `boom mcp`; the herdr plugin.
- A version that callers refuse by: `callers-pin-boom` is each caller's
  own, by `boom --version` and the format major of boom's JSON.

## Decisions

### 1. A release commit off main, tagged

A release's tag `v<x.y.z>` names a commit whose one parent is the
landing it releases and whose only change writes the release's version
into the files decision 2 lists. main's tree names `0.0.0`, and a build
takes its version from its tree alone: no environment variable, no
`--impure`, no argument. `boom --version` stays clap's, so it says the
tree's version.

Not taken: the tag on the landing commit with the version handed to the
build, which every builder but boom's workflow would have to know
(principle 14); the release commit pushed to main, which only sluice
lands and whose gate never saw it; a binary that says only its revision.
That last is timber's design for a binary timber publishes (timber
record 0011, its statement 24 and C.11, and its MVP, which retires each
tool's release commit once timber publishes it): when timber publishes
boom, this decision goes with it.

### 2. The files that carry the version, and their writer

Three files carry boom's version: `Cargo.toml`'s `[workspace.package]`
`version`; `Cargo.lock`'s entry for `boom`, the package with no
`source`, which `--locked` and the flake's `cargoLock` read; and
`plugins/claude-code/.claude-plugin/plugin.json`'s `version`, which
Claude Code reads. The marketplace names no version for its plugin, so
the plugin's is said once.

`set-version.sh` is sluice's at b6c30e78 for these three files:
`sh set-version.sh <major.minor.patch> [<root>]`, POSIX sh and awk,
refusing a version that is not `x.y.z` before it touches a file, writing
each file by an awk program of its own beside the original, refusing a
file its program wrote nothing in, naming it, and moving the files over
their originals only once all are written, so a refusal leaves the tree
as it was. `sh set-version.sh --files` lists the three from the list the
writer walks.

`root:versions` (`check-versions.sh`, sluice's adapted) fails where a
listed file gives other than `0.0.0`, naming it; where the marketplace
names a version; and where a plugin manifest under `plugins/` is not
listed. On a scratch copy it runs the writer with `98.76.54` and fails
unless each file gives it and differs from the tree's only in the lines
that give the version; and holds the refusals: `0.98` refused, and a
manifest with no `version` refused naming it, each leaving every file as
it was and no scratch file behind.

Not taken: a `VERSION` file, one fact in two places; a version in the
marketplace, as Flume's has, a second home for the plugin's.

### 3. Which landings are released, and the workflow

`.github/workflows/release.yml` runs on each push to main. Its jobs, in
sluice's form:

- `landing` reads the head commit's `Sluice-Tested` trailer by
  `git interpret-trailers --parse`.
- `version`, only where that trailer is present, runs
  `git-cliff --unreleased --bumped-version` from the flake's nixpkgs
  (`nix shell --inputs-from . nixpkgs#git-cliff`) and hands on the tag
  where no such tag exists. `cliff.toml` is sluice's with boom's
  addresses: the Conventional Commits since the newest `v*` tag by
  version, `chore` skipped, `initial_tag` `v0.1.0`.
- `binaries` and `release`, decisions 4 and the rest of this one.

`release` writes git-cliff's `--unreleased` notes, runs `set-version.sh`,
commits the files `--files` lists, by path, as `chore(release): <tag>`
by `github-actions[bot]`, tags that commit, pushes the tag alone with
`git push origin "refs/tags/$TAG"`, and publishes with
`gh release create "$TAG" --verify-tag`, the binaries and the notes
attached. It runs under `concurrency: release` with `contents: write`,
and names no token but `github.token` (core.3). Nothing is committed to
main, no branch is pushed and no tag moves.

Only a landing is released. sluice's gate is the only place boom's
checks run, and a landing's tree passed it; a push no landing made is
built and tested by nothing on GitHub, so it is released with the next
landing, whose range holds it. Not taken: sluice's CI tiers run again on
GitHub before a release, which boom's gate already ran.

### 4. The binaries are cargo's and run with no Nix

`binaries` builds, for each of `x86_64-unknown-linux-musl` on
ubuntu-latest, `aarch64-unknown-linux-musl` on ubuntu-24.04-arm and
`aarch64-apple-darwin` on macos-latest, `cargo build --release --locked
-p boom --target <target>` under the toolchain `rust-toolchain.toml`
names, which rustup installs, after `set-version.sh` writes the
release's version into the pushed commit's tree. Linux's are static
against musl, musl-tools' compiler building the SQLite rusqlite bundles.
The job installs no Nix, and fails unless the built `boom --version`
prints `boom <x.y.z>`, so a binary that loaded anything from a Nix store
fails where it was built. Each is archived as `boom-<system>.tar.gz`.

Not taken: `nix build`, whose binaries load from `/nix/store` and fail
on a host without it, as sluice v0.97.1's did (sluice record 0111's
amendment); x86_64-darwin, which the family releases for no tool.

### 5. `root:release` holds the workflow to its steps

`check-release.sh`, sluice's adapted, reads `release.yml` job by job and
fails, naming what is missing, unless: `version` runs only on a
landing's trailer and reckons with `--unreleased --bumped-version`;
`binaries` names only `-linux-musl` Linux targets, runs `set-version.sh`
before its `cargo build --release --locked -p boom --target`, runs the
built binary's `--version` against the tag, and holds no
`install-nix-action` and no `nix` command; `release` runs
`set-version.sh`, commits `--files` by path as `chore(release):`, tags,
pushes `refs/tags/$TAG` alone and runs `gh release create` with
`--verify-tag`; and the file holds no `--impure`, no `BOOM_VERSION` and
no push of a branch. It reads YAML, never a Markdown file (core.5).
actionlint is not added: yamlfmt formats the file and this check holds
its steps.

### 6. boom's flake

`flake.nix` and `flake.lock` at the root, the rest under `nix/`, as
timber's (record 0035). Inputs: `nixpkgs` (nixos-unstable) and
`rust-overlay` following it, so a flake that takes boom's can make both
follow its own. For each of aarch64-darwin, aarch64-linux and
x86_64-linux:

- `packages.<system>.boom`, also `default`: `nix/packages/boom.nix`,
  `buildRustPackage` by the Rust `.prototools` pins (rust-overlay),
  `src` the flake's own tree, `cargoLock.lockFile` `Cargo.lock`, `-p
  boom`, its version `Cargo.toml`'s workspace version and nothing else,
  `doCheck` false since the tests run in the gate, `mainProgram` `boom`.
- `packages.<system>.boom-claude-plugin`: `nix/packages/claude-plugin.nix`,
  `plugins/claude-code` of the same tree copied whole, its version the
  `boom` package's.

A system the flake gives nothing for fails as nix fails it. The flake
gives no module: boom writes no file on a host (its store is data under
`$XDG_DATA_HOME`, and a repository's `.config/boom.toml` is the
repository's), so a host's configuration installs the package, as
Boomdrive's set will (record 0023). Not taken: a `services.boom` module
that installs the package alone, which a module of no file adds nothing
to; the plugin built by Boomdrive from boom's source, which principle 17
gives to boom's own flake.

### 7. The `nix` project

A moon project `nix` (`nix/moon.yml`, language `nix`):

- `lint`: `statix check flake.nix && statix check nix`.
- `typecheck`: `sh nix/check.sh typecheck`, every system's two packages
  evaluated to their derivations.
- `test`: `sh nix/check.sh test`, the cases in `nix/tests`: each
  system's `boom` package names `Cargo.toml`'s workspace version;
  `boom-claude-plugin`'s version is the `boom` package's; the plugin
  package's source is the tree's `plugins/claude-code`; and the flake
  gives no module and the three systems alone.

`nix/check.sh`, `nix/inputs.nix` and `nix/tests/default.nix` and
`lib.nix` are timber's at 1adfecc7 for boom's names: the pinned nix,
`BOOM_FLAKE_NIX`, evaluates offline and read-only against
`dummy://?store=/boom-check/store`, its configuration and caches a
scratch folder's, reading the flake's inputs where the devenv shell
fetched them, `BOOM_FLAKE_INPUTS` (`nix/inputs.nix`'s `paths`, as JSON).
Each failure is one line, `nix: <case>: <what>`, and every case runs.
`devenv.nix` pins nix 2.34.8, reached by its path and kept off `PATH`,
and statix 0.5.8-unstable-2026-07-17, each asserted to its version, as
timber's are; `check-toolchain.sh` asserts both (core.6). Each task's
inputs are the flake, the lock, `nix/`, devenv's files, `Cargo.toml`,
`Cargo.lock`, `.prototools` and `plugins/claude-code`, with `crates/`
for `typecheck`. The format is `root:format`'s: nixfmt and deadnix read
the `.nix` files, shellcheck and shfmt `nix/check.sh`.

### 8. The plugin

`plugins/claude-code` holds `.claude-plugin/plugin.json`, the plugin
`boom` at the tree's version with boom's description, homepage,
repository and the MIT license, and `hooks/hooks.json`, one
`SessionStart` hook whose command is `boom query --if-declared`. Claude
Code runs it in the session's folder and puts its standard output in
the agent's context: the claims that stand for the repository that
folder declares, in the words decision 10 keeps. Where nothing is
declared it says nothing and exits 0, so a plugin installed for every
session on a machine, as Boomdrive's set installs its tools' plugins,
interrupts none outside a declared repository. A declaration boom
cannot read is still refused, exit 2, its words shown to the person,
who must fix it. The hook calls `boom` by its command line from `PATH`,
the boom of the same release beside it (`plugins-in-this-repository`).

`.claude-plugin/marketplace.json` at the root is the marketplace
`boom`, listing the plugin `boom` with its source
`./plugins/claude-code` and no version. Without Nix a person adds it
from the release's tag with Claude Code's own marketplace command, the
repository's git address with `#v<x.y.z>`, and installs the plugin
from it; the README gives the lines, checked against the pinned Claude
Code's help.

Not taken: a script beside the hook that finds the declaration itself,
which would repeat boom's discovery (`repository-rules`) in shell; `||
true` on the command, which would hide a declaration boom cannot read;
the README's hook in each repository's own settings as the only way,
which a person writes by hand in every repository.

### 9. The `claude-code` project

A moon project `claude-code` (`plugins/claude-code/moon.yml`, language
`bash`, its test being shell):

- `format`: `biome format` of the plugin's JSON, by `biome.json`, whose
  includes gain `plugins/**` and `.claude-plugin/**`.
- `lint`: `biome lint` of the same files.
- `typecheck`: `../claude.sh plugin validate --strict .`, the manifest
  and its hooks read by the pinned Claude Code.
- `test`: `bash tests/session.sh`: it builds boom from the tree with
  `cargo build --locked -p boom`, reads the `SessionStart` command from
  `hooks/hooks.json` with jq, and runs it with `sh -c`, as Claude Code
  runs a hook, in scratch folders under `SLUICE_TMP` or `TMPDIR`, the
  built boom first on `PATH` and `XDG_DATA_HOME` a scratch folder's: in
  a repository that declares nothing and outside any repository it exits
  0 with both streams empty and no store made; in a repository whose
  declaration names `Shop` it exits 2 naming `.config/boom.toml`. The
  claims given for a declared repository are `boom query`'s, proven in
  `crates/boom/tests/query.rs` with the tests' keys, which a shell
  cannot make (record 0006).

`root:marketplace` runs `bash plugins/claude.sh plugin validate --strict
.` on the root's marketplace. `plugins/claude.sh` is timber's at
1adfecc7: the pinned `claude` with a scratch home, the user's
`CLAUDE_*` and `ANTHROPIC_*` unset and no nonessential traffic, so a
check reads none of the user's configuration or credentials (core.3).
`devenv.nix` pins Claude Code 2.1.296 from its release, with timber's
four checksums, and jq 1.8.2 from `devenv.lock`'s nixpkgs, asserted to
that version; `check-toolchain.sh` asserts both versions (core.6). `treefmt.nix` gives Biome the plugin's and the
marketplace's JSON (core.5).

### 10. `boom query --if-declared`

`--if-declared` conflicts with `--repository`. Where the working folder
declares a repository, `boom query --if-declared` answers exactly as
`boom query` does, byte for byte. Where nothing is declared, from inside
a repository or outside one, it opens no store, writes nothing to
standard error, and exits 0: in words it prints nothing, and with
`--json` it prints `boom-query/1` with `repository` and `head` null and
`claims` empty. A declaration it cannot read is refused as without the
flag. Within `boom-query/1` a null `repository` and `head` are new, and
only a caller that passes the new flag can be given them, so no caller
built for the format before it reads one (`json-versioned-additive`).

Not taken: `boom query` answering nothing whenever nothing is declared,
which would make the refusal `repository-rules` keeps for a person
silent; a refusal with a code a hook parses, which every hook would
parse.

### 11. Records and rules

Record 0010 records decisions 1 to 10, each with the path not taken,
numbered after main's record 0009, which landed after this bolt was cut.
core.5 is amended in the same commit to say every crate, every plugin,
the flake and the site is a moon project of its own; core.8 is added:
a release's tag names a release commit off main that writes boom's
version into every file that carries it, main's tree names `0.0.0`, a
build takes its version from its tree alone, and boom's plugins come
from boom's flake at boom's version. README says how a machine installs
a release and the plugin; CLAUDE.md names the new projects.

## Risks / Trade-offs

- [`gh release create` fails after the tag is pushed, and the next
  landing reckons from that tag] → run `gh release create "$TAG"
  --verify-tag` by hand with the run's artifacts, as for sluice, Flume
  and crew.
- [Claude Code's hooks or manifest schema moves] → the plugin is checked
  by the pinned Claude Code, moved with timber's pin.
- [A host installs boom's plugin with a boom of another release on its
  `PATH`] → the flake gives both from one revision; outside it, the
  person installs both from one tag.
- [The offline evaluation, statix and `plugins/claude.sh` are copies of
  timber's and sluice's, and a fix to one is no fix to the other] →
  each file's opening comment names the revision it was taken from.
- [A session in a declared repository on a machine with no store sees
  boom's not-run in Claude Code's verbose output] → the person runs
  `boom init`; the hook never hides a store it cannot read.

## Migration Plan

The bolt's landing on main is boom's first release, `v0.1.0`. No host
holds boom yet, so nothing moves; a repository whose settings name `boom
query` as a hook keeps working, and drops the hook once the plugin is
installed.
