---
number: 3
title: The site's packages are installed with no network, from bun's cache as flume's packages.nix builds it
status: accepted
date: 2026-11-15
decision-makers:
- the chore booms-tasks-read-no-markdown's code stage
---

# The site's packages are installed with no network, from bun's cache as flume's packages.nix builds it

Status: accepted. The decision is the code stage's own, on the family
rule of CLAUDE.md. Where two tools do one thing, they have one design
with one owner, and the others call or cite it. flume owns this design,
and boomdrivedev/boomdrive already calls it. The decision is cheap to
change.

Deciders: the code stage of the chore `booms-tasks-read-no-markdown`.
sluice returned the chore with: "[sluice] #1 failed on
bolt/booms-checks-read-no-markdown: site:install, site:build, site:test,
site:typecheck."

Sources:

- sluice's #1 on `bolt/booms-checks-read-no-markdown`, and its
  `site:install` log at
  `boom.git/sluice/.logs/bolt/booms-checks-read-no-markdown/c1/site:install.log`
  on deneb: `error: bun is unable to write files to tempdir: EPERM`.
- The `[executor]` of `.config/sluice.toml`: `kind = "sandbox"`,
  `network = "localhost"`.
- sluice's `crates/sluice/src/profiles/checks.rs` on its main. Its
  sandbox lets a run write its checkout, its scratch, the declared
  caches of the host and the cache of the task runner. The sandbox lets
  a run write nothing else of the user's home.
- boomdrivedev/flume's record 0059, and its `site/packages.nix` at
  v0.24.0.
- boomdrivedev/boomdrive's `devenv.nix` and `site/moon.yml` on its main.
- bun 1.4.2, as `devenv.nix` pins it.
- Rule core.6.

## Context and problem statement

`site:install` ran `bun install --frozen-lockfile` in the sandbox of the
gate. To choose a temporary folder, bun moves a file from `TMPDIR` into
its cache. Its cache is the user's `~/.bun/install/cache`, and the
sandbox refuses writes there. bun stopped at that point, and the three
tasks that need the packages failed with it.

Past that point, another limit applies. The network of the sandbox
reaches only localhost. Suppose that bun had continued. Then a package
that was not in a cache that bun could read would have failed on the
name lookup of the registry. flume's `site:install` failed that way on 2026-11-08.
The tasks of the site ran in the gate, and none replayed, because this
chore changed `devenv.nix`, one of their inputs.

After the packages installed, `site:build` failed under `sluice exec`
with `ENOENT` when it renamed `.astro/content-assets.mjs.tmp`. The
cause: `site:typecheck` and `site:build` both sync Astro's content into
`site/.astro/`, and moon ran them at the same time.

## Considered options

1. **flume's function, called at a release that devenv.yaml pins.** The
   input `flume` of `devenv.yaml`, with `flake: false`, is at v0.24.0,
   and `devenv.lock` locks it. `devenv.nix` imports its
   `site/packages.nix` with boom's `bun.lock` and `package.json`. The
   shell builds them outside the sandbox into bun's cache, and gives the
   path of the cache as `BOOM_SITE_PACKAGES`. `site/scripts/install.sh`
   copies the cache to a scratch folder, and runs
   `bun install --frozen-lockfile --offline` from the copy. boomdrive
   does the same through the input of its flake.
2. **A copy of flume's `site/packages.nix` in `site/`.** This gives two
   owners of one design. The two copies drift as each owner fixes its
   copy alone.
3. **bun's cache in the scratch of the run, `BUN_INSTALL_CACHE_DIR`.**
   The cache is empty at the start in each checkout, and the network
   reaches no registry.
4. **`[executor] network = true`.** Each check of the tree could reach
   any address. The gate would fail each time npm's registry did not
   answer, as flume's record 0059 weighs it.
5. **`~/.bun/install/cache` declared as one of the caches of the
   host.** The configuration of the host is not a file of this
   repository. A cache changes how fast a check runs, and never whether
   it passes. An empty cache would fail the gate, as the registry does.

For `.astro/`:

1. `site:typecheck` and `site:build` share moon's mutex `astro`, as
   boomdrive's `check` and `build` do.
2. `site:build` depends on `site:typecheck`. A failed type check would
   then hide whether the site builds.

## Decision outcome

Option 1, and the mutex.

- **The network stays shut.** `.config/sluice.toml` does not change.
- **This repository keeps no hash.** flume's function fetches each
  package by the integrity that `bun.lock` gives it. The pin is the
  revision of the input in `devenv.lock`, and `devenv update flume`
  moves it.
- **The installer is boom's own.** `site/scripts/install.sh` is
  boomdrive's and flume's, and it reads `BOOM_SITE_PACKAGES`. It refuses
  to run where the shell did not set that variable.

### Consequences

- Good: `site:install` in the gate reads nothing of the user's home and
  reaches no registry. On deneb, under `sluice exec main`, it took 6 to
  11 seconds. `site:typecheck`, `site:test` and `site:build` passed
  after it.
- Good: the `moon run site:build` of the Pages workflow installs the
  same way. Its shell builds the cache on Linux.
- Bad: a release of flume that changes the arguments of the function
  fails the evaluation of this shell, in the change that moves the pin.
  A move of bun's pin can change how bun names the folders of its cache.
  `site:install` then fails, and names a package that is not in the
  cache.
- Bad: each install copies the cache, about 300 MB.
- Neutral: the type check and the build never run at the same time.
