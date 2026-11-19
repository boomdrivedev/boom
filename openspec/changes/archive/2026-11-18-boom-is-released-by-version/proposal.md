# Proposal

## Why

boom has no release: its tree names `0.0.0`, nothing tags it, and no
flake or archive gives it, so a machine can only build whatever commit
it checks out, and an agent's session gets its claims only where a
repository's own settings were written by hand to run `boom query`.
Record 0009 lets a repository move into boom only once boom is on the
machine at a release its callers pin (`callers-pin-boom`) and every
agent session there starts with the claims that stand for it. A tool's
plugins come with its command line from one flake at one revision
(boomdrivedev/blueprints principle 17), and Boomdrive's flake takes each
tool's own package and module from that tool's flake at a release tag
(boomdrivedev/boomdrive record 0023). This change gives boom named
releases, each carrying boom and its Claude Code plugin at one version.

## What Changes

- Every file that carries boom's version names `0.0.0` on main:
  `Cargo.toml`'s workspace version, `Cargo.lock`'s entry for boom and
  the plugin's manifest. `set-version.sh` writes a release's version
  into each, all or none, and lists them; `root:versions` holds them
  equal and the writer to its word.
- Each landing on main in which git-cliff finds a new version, by the
  Conventional Commits since the newest tag, is released: a workflow
  builds each platform's binary by cargo from the pushed commit with the
  version written, runs it on a runner with no Nix, then commits those
  files as a release commit off main, tags it `v<x.y.z>`, pushes the tag
  alone and publishes the binaries and notes as the tag's GitHub
  release. Nothing is pushed to main. `root:release` holds the workflow
  to these steps.
- boom gains a flake: for each system it releases, `boom`, built from
  the tree with the tree's version, and `boom-claude-plugin`, the
  plugin's folder at that same version. A machine installs a release by
  its tag, `github:boomdrivedev/boom/v<x.y.z>`. A moon project `nix`
  lints and evaluates it offline.
- boom gains its Claude Code plugin, `boom`, in `plugins/claude-code`,
  a moon project of its own, listed in a marketplace at the tree's root:
  its `SessionStart` hook runs `boom query --if-declared`, so every
  session in a repository that declares itself starts with the claims
  that stand for it, and a session anywhere else is told nothing.
- `boom query --if-declared` answers nothing and exits 0 where no
  repository is named or declared, and otherwise answers as `boom query`
  does.
- Record 0010 records the choices; core.5 names each plugin and the
  flake among the moon projects, and a new rule, core.8, says how a
  release carries its version.

## Capabilities

### New Capabilities

- `release-version`: what a boom release is: main's tree naming
  `0.0.0`, the files that carry the version and their writer, a
  release's tag naming a release commit off main, which landings are
  released, the binaries built by cargo and run with no Nix, and boom's
  flake giving a release's binary and plugin from its tag at one
  version.
- `claude-code-plugin`: boom's Claude Code plugin: its manifest at
  boom's version, its marketplace, and its session hook giving an agent
  the claims that stand for the repository it works in, silent where
  none is declared.

### Modified Capabilities

- `repository-rules`: `boom query --if-declared`, answering nothing
  where no repository is named or declared, for a harness's hook.

## Impact

- `crates/boom/src/cli.rs`: `boom query --if-declared`; tests in
  `crates/boom/tests/query.rs`.
- New at the root: `flake.nix`, `flake.lock`, `set-version.sh`,
  `check-versions.sh`, `check-release.sh`, `cliff.toml`,
  `.github/workflows/release.yml` and `.claude-plugin/marketplace.json`;
  `moon.yml` gains `root:versions`, `root:release` and
  `root:marketplace`.
- New projects in `.moon/workspace.yml`: `nix` (`nix/`, the flake's
  recipes and offline checks) and `claude-code` (`plugins/claude-code`,
  with `plugins/claude.sh` beside it).
- `devenv.nix` pins Claude Code, nix, statix and jq for the new tasks,
  and `check-toolchain.sh` asserts each; `treefmt.nix` and `biome.json`
  claim the plugin's and the marketplace's JSON.
- `README.md` says how a machine installs a release and the plugin;
  `CLAUDE.md` names the new projects; record 0010, core.5 amended and
  core.8 added.
- Out of this change: Boomdrive's set taking boom's package and plugin
  from boom's flake at a release tag, which is boomdrivedev/boomdrive's
  after boom's first release; the plugin's hook before each edit, its
  skills and `boom mcp`; the herdr plugin; a module of boom's, which
  writes no file on the host; boom published by timber, at which point
  timber's MVP retires the release commit (timber record 0011).
