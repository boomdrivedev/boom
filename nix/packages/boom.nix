# boom's binary, built by cargo from the flake's own tree with the Rust
# .prototools pins (record 0010, design 6). Its version is the one the
# flake reads from Cargo.toml's [workspace.package] and nothing else, so a
# build of a release's tag says the release's version and a build of main
# says 0.0.0 (core.8).
{
  lib,
  makeRustPlatform,
  rust,
  src,
  lockFile,
  version,
}:
let
  platform = makeRustPlatform {
    cargo = rust;
    rustc = rust;
  };
in
platform.buildRustPackage {
  pname = "boom";
  inherit version src;
  cargoLock = { inherit lockFile; };
  cargoBuildFlags = [
    "-p"
    "boom"
  ];
  # The tests run in the devenv shell, through moon, in the gate.
  doCheck = false;
  meta = {
    description = "A ledger of what a project has claimed about itself";
    homepage = "https://github.com/boomdrivedev/boom";
    license = lib.licenses.mit;
    mainProgram = "boom";
  };
}
