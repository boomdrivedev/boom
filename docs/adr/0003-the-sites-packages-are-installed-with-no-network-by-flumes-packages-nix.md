---
number: 3
title: The site's packages are installed with no network, from bun's cache as flume's packages.nix builds it
status: accepted
date: 2026-11-15
decision-makers:
- the chore booms-tasks-read-no-markdown's code stage
---

# The site's packages are installed with no network, from bun's cache as flume's packages.nix builds it

Status: accepted. The code stage's own, on CLAUDE.md's family rule:
where two tools do one thing, one design with one owner the others call
or cite. flume owns this one, and boomdrivedev/boomdrive already calls
it. Cheap to change.

Deciders: the code stage of the chore `booms-tasks-read-no-markdown`,
sent back with: "[sluice] #1 failed on bolt/booms-checks-read-no-markdown:
site:install, site:build, site:test, site:typecheck."

Sources: sluice's #1 on `bolt/booms-checks-read-no-markdown`, its
`site:install` log at
`boom.git/sluice/.logs/bolt/booms-checks-read-no-markdown/c1/site:install.log`
on deneb: `error: bun is unable to write files to tempdir: EPERM`;
`.config/sluice.toml`'s `[executor]`, `kind = "sandbox"`,
`network = "localhost"`; sluice's `crates/sluice/src/profiles/checks.rs`
on its main, whose sandbox lets a run write its checkout, its scratch,
the host's declared caches and the task runner's, and nothing else of
the user's home; boomdrivedev/flume's record 0059 and its
`site/packages.nix` at v0.24.0; boomdrivedev/boomdrive's `devenv.nix` and
`site/moon.yml` on its main; bun 1.4.2, as `devenv.nix` pins it; rule
core.6.

## Context and problem statement

`site:install` ran `bun install --frozen-lockfile` in the gate's
sandbox. bun moves a file from `TMPDIR` into its cache to choose a
temporary folder, and its cache is the user's `~/.bun/install/cache`,
which the sandbox refuses to write; bun stopped there, and the three
tasks that need the packages failed with it. Had it got past, the
sandbox's network reaches localhost alone, so a package missing from a
cache bun could read would have failed on the registry's name lookup, as
flume's `site:install` did on 2026-11-08. The site's tasks ran in the
gate, none replayed, because this chore changed `devenv.nix`, one of
their inputs.

Once the packages installed, `site:build` failed under `sluice exec` on
`ENOENT` renaming `.astro/content-assets.mjs.tmp`: `site:typecheck` and
`site:build` both sync Astro's content into `site/.astro/`, and moon ran
them at once.

## Considered options

1. **flume's function, called at a release devenv.yaml pins.**
   `devenv.yaml`'s input `flume`, `flake: false`, at v0.24.0, locked in
   `devenv.lock`; `devenv.nix` imports its `site/packages.nix` with
   boom's `bun.lock` and `package.json`, which the shell builds outside
   the sandbox into bun's cache, and gives its path as
   `BOOM_SITE_PACKAGES`. `site/scripts/install.sh` copies it to a scratch
   folder and runs `bun install --frozen-lockfile --offline` from the
   copy. boomdrive does the same through its flake's input.
2. **A copy of flume's `site/packages.nix` in `site/`.** Two owners of
   one design, which drift as each is fixed alone.
3. **bun's cache in the run's scratch, `BUN_INSTALL_CACHE_DIR`.** It
   starts empty in every checkout, and the network reaches no registry.
4. **`[executor] network = true`.** Every check of the tree could reach
   any address, and the gate would fail whenever npm's registry did not
   answer, as flume's record 0059 weighs it.
5. **`~/.bun/install/cache` declared as one of the host's caches.** The
   host's configuration is no file of this repository's, and a cache
   changes how fast a check runs, never whether it passes: an empty one
   would fail the gate as the registry does.

For `.astro/`:

1. `site:typecheck` and `site:build` share moon's mutex `astro`, as
   boomdrive's `check` and `build` do.
2. `site:build` depends on `site:typecheck`. A failed type check would
   then hide whether the site builds.

## Decision outcome

Option 1, and the mutex.

- **The network stays shut.** `.config/sluice.toml` is unchanged.
- **No hash is kept here.** flume's function fetches each package by the
  integrity `bun.lock` gives it; the pin is the input's revision in
  `devenv.lock`, which `devenv update flume` moves.
- **The installer is boom's own.** `site/scripts/install.sh` is
  boomdrive's and flume's, reading `BOOM_SITE_PACKAGES`; it refuses to
  run where the shell did not set it.

### Consequences

- Good: the gate's `site:install` reads nothing of the user's home and
  reaches no registry. On deneb under `sluice exec main` it took 6 to 11
  seconds, and `site:typecheck`, `site:test` and `site:build` passed
  after it.
- Good: the Pages workflow's `moon run site:build` installs the same
  way, its shell building the cache on Linux.
- Bad: a release of flume that changes the function's arguments fails
  this shell's evaluation in the change that moves the pin; a move of
  bun's pin can change how bun names its cache's folders, and
  `site:install` then fails naming a package not in the cache.
- Bad: each install copies the cache, about 300 MB.
- Neutral: the type check and the build never run at once.
