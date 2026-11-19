{
  pkgs,
  lib,
  inputs,
  ...
}:

let
  # proto from its own release build: nixpkgs trails it by minor versions or
  # more. Linux takes the static musl build, so no loader needs patching.
  proto =
    let
      version = "0.63.1";
      system = pkgs.stdenv.hostPlatform.system;
      target =
        {
          aarch64-darwin = "aarch64-apple-darwin";
          x86_64-darwin = "x86_64-apple-darwin";
          aarch64-linux = "aarch64-unknown-linux-musl";
          x86_64-linux = "x86_64-unknown-linux-musl";
        }
        .${system} or (throw "proto ${version}: no release build for ${system}");
      hashes = {
        aarch64-apple-darwin = "8bfde4c862381eeb6e82d0858bb6215732cbd1e384e4eae45da8fa08095e0ffd";
        x86_64-apple-darwin = "aaaff78c7125de1b765f00177f35da75ee16fb1363e07632ace27213ff5d893b";
        aarch64-unknown-linux-musl = "111f5b6a38d3979ebafbc095c8d91ef69c99d1f018addb3582179d4ab91b3441";
        x86_64-unknown-linux-musl = "8596b98ccd833d1203e71931ba25e866d3239f9d3859c922a035a7b830f29d17";
      };
    in
    pkgs.stdenvNoCC.mkDerivation {
      pname = "proto";
      inherit version;
      src = pkgs.fetchurl {
        url = "https://github.com/moonrepo/proto/releases/download/v${version}/proto_cli-${target}.tar.xz";
        sha256 = hashes.${target};
      };
      sourceRoot = "proto_cli-${target}";
      installPhase = ''
        runHook preInstall
        install -Dm755 -t $out/bin proto proto-shim
        runHook postInstall
      '';
    };

  # The application toolchain .prototools pins (moon and Rust), on PATH.
  # proto's home is this repository's, in the git common dir so every
  # worktree shares it, and only this repository's pins are read.
  protoHome = ''
    PROTO_HOME="$(${lib.getExe pkgs.git} rev-parse --path-format=absolute --git-common-dir)/proto"
    export PROTO_HOME
  '';
  activate = ''
    ${protoHome}
    eval "$(${proto}/bin/proto activate bash --export --no-bin --config-mode upwards 2>/dev/null)"
  '';

  # The site's packages as site/bun.lock locks them, fetched by this shell
  # outside the gate's sandbox into bun's cache, from which site:install
  # installs with no network (record 0003). flume's function at the release
  # devenv.yaml's input `flume` names, which owns how a site's packages
  # reach a sandboxed gate (flume's record 0059); a release that changes its
  # arguments fails this shell's evaluation in the change that moves it.
  sitePackages = import "${inputs.flume}/site/packages.nix" {
    inherit pkgs lib;
    lock = ./site/bun.lock;
    manifest = ./site/package.json;
  };

  # OpenSpec, which root:specs and root:changes validate with: flume's
  # release, built as nixpkgs builds its older one. Bump the version, then
  # each hash from the mismatch the build reports.
  openspecVersion = "1.14.3";
  openspec = pkgs.openspec.overrideAttrs (
    finalAttrs: _: {
      version = openspecVersion;
      src = pkgs.fetchFromGitHub {
        owner = "Fission-AI";
        repo = "OpenSpec";
        tag = "v${finalAttrs.version}";
        hash = "sha256-tdTX0nkDe/aNZgLckh6/B6BtoSb40w0Y5qzg+d1LjHU=";
      };
      pnpmDeps = pkgs.fetchPnpmDeps {
        inherit (finalAttrs) pname version src;
        pnpm = pkgs.pnpm_10;
        fetcherVersion = 3;
        hash = "sha256-fKm4sBdTDeQfIiE0k301jtRbiuJrBE7YIEBu8RZxuXI=";
      };
    }
  );

  # Vale, for the pages' writers to run on their prose (.vale.ini,
  # styles/Boom); no check runs it (core.5; record 0002). From its own
  # release: nixpkgs trails it. Bump the version and the four hashes from
  # the release's checksums file.
  valeVersion = "3.24.0";
  valeAssets = {
    aarch64-darwin = {
      os = "macOS_arm64";
      sha256 = "87b513f26499f6657c15cf7175ba5b058a7c1ff2bb2d4e07265db2dd43451e44";
    };
    x86_64-darwin = {
      os = "macOS_64-bit";
      sha256 = "eeb39e86f1daac27cc2a77fef3af1824d50559bdc22329fb25fb921c80c79472";
    };
    aarch64-linux = {
      os = "Linux_arm64";
      sha256 = "104cbd349279e50c4cb40adb70adf345d93cc4a3a0191f9e68bdccb97086b582";
    };
    x86_64-linux = {
      os = "Linux_64-bit";
      sha256 = "867534ddb678abca7f214bc4706ead0922bd5252ffe8035fc227013f66c8b214";
    };
  };
  valeAsset = valeAssets.${pkgs.stdenv.hostPlatform.system};
  vale = pkgs.stdenvNoCC.mkDerivation {
    pname = "vale";
    version = valeVersion;
    src = pkgs.fetchurl {
      url = "https://github.com/errata-ai/vale/releases/download/v${valeVersion}/vale_${valeVersion}_${valeAsset.os}.tar.gz";
      inherit (valeAsset) sha256;
    };
    sourceRoot = ".";
    installPhase = "install -Dm755 vale $out/bin/vale";
  };

  # Claude Code, run by the plugin's check (claude-code:typecheck) and the
  # marketplace's (root:marketplace), both through plugins/claude.sh: the
  # release timber's and sluice's plugins are checked with, derived as
  # timber's devenv.nix derives it (boomdrivedev/timber 1adfecc7), so every
  # host checks the plugin against one engine and never the user's own
  # `claude`, which may be a wrapper adding options of its own (core.6).
  # Bump the version and the four checksums from the release's
  # manifest.zst.json, as timber's and sluice's are bumped.
  claudeVersion = "2.1.296";
  claudeAssets = {
    aarch64-darwin = {
      platform = "darwin-arm64";
      sha256 = "26426aa3dcf41b0f554951766c3c583cf243345ca3d32fa590caf84685be5f97";
    };
    x86_64-darwin = {
      platform = "darwin-x64";
      sha256 = "6f1d6f926a303c423eb4815a01764165ed4375054bfa8ef6203e55fc806f50ac";
    };
    aarch64-linux = {
      platform = "linux-arm64";
      sha256 = "23aa91437a06eff80a65bfc32b0e4b5a6dda3e821019c543696c154274e56913";
    };
    x86_64-linux = {
      platform = "linux-x64";
      sha256 = "fac1d78be9446052ac084878c04c2648abb6b22d53279a685fa19ee517d74410";
    };
  };
  claudeAsset = claudeAssets.${pkgs.stdenv.hostPlatform.system};
  claude = pkgs.stdenvNoCC.mkDerivation {
    pname = "claude-code";
    version = claudeVersion;
    src = pkgs.fetchurl {
      url = "https://downloads.claude.ai/claude-code-releases/${claudeVersion}/${claudeAsset.platform}/claude.zst";
      inherit (claudeAsset) sha256;
    };
    dontUnpack = true;
    # The binary carries its program after the runtime; stripping drops it.
    dontStrip = true;
    nativeBuildInputs = [
      pkgs.zstd
      pkgs.makeBinaryWrapper
    ]
    ++ lib.optionals pkgs.stdenv.hostPlatform.isElf [ pkgs.autoPatchelfHook ];
    installPhase = ''
      mkdir -p $out/bin
      unzstd -q $src -o $out/bin/.claude
      chmod 755 $out/bin/.claude
      makeWrapper $out/bin/.claude $out/bin/claude \
        --set DISABLE_AUTOUPDATER 1 \
        --set DISABLE_INSTALLATION_CHECKS 1
    '';
  };

  # jq, which the plugin's test (claude-code:test) reads the hook's command
  # with: nixpkgs' build, pinned by devenv.lock and asserted to be this
  # version, so every host that runs the gate runs the same (core.6).
  jq =
    assert pkgs.jq.version == "1.8.2";
    pkgs.jq;

  # The flake's checks' tools (record 0010, design 7): the nix nix:typecheck
  # and nix:test evaluate with, and the statix nix:lint runs, as timber's
  # are pinned (boomdrivedev/timber 1adfecc7, its record 0035). nixpkgs'
  # builds, pinned by devenv.lock and asserted to be these versions, so
  # every host that runs the gate runs the same (core.6). treefmt.nix's
  # nixfmt and deadnix are from the same nixpkgs. The nix is reached by its
  # path, BOOM_FLAKE_NIX, and is not on PATH, where the host's own nix,
  # which devenv runs on, stays.
  flakeNix =
    assert pkgs.nix.version == "2.34.8";
    pkgs.nix;
  statix =
    assert pkgs.statix.version == "0.5.8-unstable-2026-07-17";
    pkgs.statix;

in
{
  # The system tools are this file's, pinned by devenv.lock; moon and Rust
  # are .prototools', installed by the boom:proto:install task on
  # entering the shell and activated by enterShell. OpenSpec and Vale are
  # the releases pinned above, never the host's (core.6). The site's tools
  # are bun and Biome, which treefmt runs too, and Playwright with the one
  # browser it drives, for the site's screenshots (site/scripts/shots.mjs).
  packages = [
    pkgs.git
    proto
    openspec
    vale
    pkgs.bun
    pkgs.biome
    pkgs.playwright-test
    # The plugin's checks: Claude Code validates the plugin and the
    # marketplace (claude-code:typecheck, root:marketplace), and jq reads
    # the hook's command in the plugin's test (claude-code:test).
    claude
    jq
    # The flake's lint (nix/moon.yml).
    statix
  ];

  # The flake's checks (nix/check.sh): the pinned nix by its path.
  env.BOOM_FLAKE_NIX = "${flakeNix}/bin/nix";

  # The site's packages, given to site:install by their store path
  # (site/scripts/install.sh; record 0003).
  env.BOOM_SITE_PACKAGES = "${sitePackages}";

  env.PLAYWRIGHT_BROWSERS_PATH = "${pkgs.playwright-driver.browsers-chromium}";

  # moon's task archives (.moon/cache/outputs) are a link into the git common
  # dir, so every worktree shares them. The rest of .moon/cache records the
  # checkout's absolute paths and stays its own.
  enterShell = ''
    if _moon_root="$(git rev-parse --show-toplevel 2>/dev/null)" &&
      _moon_common="$(git rev-parse --path-format=absolute --git-common-dir 2>/dev/null)"; then
      _moon_cache="$_moon_root/.moon/cache"
      [ -L "$_moon_cache" ] && rm -f "$_moon_cache"
      mkdir -p "$_moon_common/moon-outputs" "$_moon_cache"
      if [ ! -L "$_moon_cache/outputs" ]; then
        rm -rf "$_moon_cache/outputs"
        ln -s "$_moon_common/moon-outputs" "$_moon_cache/outputs"
      fi
      # moon puts $MOON_HOME/bin on every task's PATH too: this repository's,
      # beside proto's, so no moon of the user's runs in a task.
      MOON_HOME="$_moon_common/moon-home"
      export MOON_HOME
    fi
    unset _moon_root _moon_common _moon_cache
    # A sluice run is handed its share of the host's cores (SLUICE_JOBS):
    # cargo builds and runs tests with that many, so the runs the host's
    # bound lets go at once share its cores rather than each taking them
    # all. A session's own shell, with no SLUICE_JOBS, keeps every core.
    if [ -n "''${SLUICE_JOBS:-}" ]; then
      CARGO_BUILD_JOBS="$SLUICE_JOBS"
      RUST_TEST_THREADS="$SLUICE_JOBS"
      export CARGO_BUILD_JOBS RUST_TEST_THREADS
    fi
    # Only activates: sluice sources this inside its executor, where
    # nothing may be installed.
    ${activate}
  '';

  # The versions .prototools lists, installed on entering the shell, before
  # enterShell activates them and before devenv's own treefmt run formats
  # with the pinned rustfmt. Rust is rustup's first, with the components
  # rust-toolchain.toml names: proto's own install of it leaves them out.
  # A task, not part of enterShell: the script `devenv print-dev-env` prints
  # holds enterShell and none of the tasks, so a check run sourcing it never
  # installs.
  tasks."boom:proto:install" = {
    exec = ''
      cd "$(${lib.getExe pkgs.git} rev-parse --show-toplevel)"
      ${pkgs.rustup}/bin/rustup toolchain install
      ${protoHome}
      ${proto}/bin/proto install --config-mode upwards
    '';
    before = [
      "devenv:enterShell"
      "devenv:treefmt:run"
    ];
  };

  treefmt.enable = true;
  treefmt.config = {
    imports = [ ./treefmt.nix ];
    projectRootFile = "devenv.nix";
  };

  scripts.prose = {
    description = "Vale over the site's pages and the README, for their writers (.vale.ini)";
    exec = ''
      set -euo pipefail
      cd "$(git rev-parse --show-toplevel)"
      vale site/src/content/docs README.md "$@"
    '';
  };
  scripts.site-build = {
    description = "The site, built into site/dist: moon run site:build";
    exec = ''
      set -euo pipefail
      cd "$(git rev-parse --show-toplevel)"
      moon run site:build "$@" </dev/null
    '';
  };
  scripts.site-shots = {
    description = "Screenshots of the built site at phone and desktop widths, in both themes";
    exec = ''
      set -euo pipefail
      cd "$(git rev-parse --show-toplevel)/site"
      bun scripts/shots.mjs "$@"
    '';
  };
}
