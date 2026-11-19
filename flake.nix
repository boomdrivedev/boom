# boom as a nix package, for `nix profile install github:boomdrivedev/boom/v<x.y.z>`,
# and boom's Claude Code plugin from the same tree at the same version, for a
# host's configuration and Boomdrive's set to take from one tag (record
# 0010, design 6; blueprints principle 17; boomdrive record 0023). The
# development shell is devenv's (devenv.nix); everything else of the
# flake's is under nix/, the moon project `nix`. Rust is the version
# .prototools names, so the package is built by the compiler the checks ran
# with. The release's binaries are cargo's, built by the release workflow to
# run on a host with no Nix, and not this package, which links against the
# store (design 4). The version is the tree's, Cargo.toml's workspace
# version: 0.0.0 on main, and the release's in the release commit its tag
# names (set-version.sh). A build is given nothing: `nix build
# github:boomdrivedev/boom/v<x.y.z>` says that release's version (core.8).
# boom writes no file on a host, so the flake gives no module: a host's
# configuration installs the package.
{
  description = "boom: a ledger of what a project has claimed about itself";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
      ...
    }:
    let
      # The systems the family releases for: nixpkgs refuses x86_64-darwin
      # since 26.11.
      systems = [
        "aarch64-darwin"
        "aarch64-linux"
        "x86_64-linux"
      ];
      forAll =
        f:
        nixpkgs.lib.genAttrs systems (
          system:
          f (
            import nixpkgs {
              inherit system;
              overlays = [ rust-overlay.overlays.default ];
            }
          )
        );
      rustVersion = (builtins.fromTOML (builtins.readFile ./.prototools)).rust;
      inherit ((builtins.fromTOML (builtins.readFile ./Cargo.toml)).workspace.package) version;
    in
    {
      packages = forAll (
        pkgs:
        let
          boom = pkgs.callPackage ./nix/packages/boom.nix {
            inherit version;
            rust = pkgs.rust-bin.stable.${rustVersion}.minimal;
            src = self;
            lockFile = ./Cargo.lock;
          };
        in
        {
          inherit boom;
          default = boom;
          boom-claude-plugin = pkgs.callPackage ./nix/packages/claude-plugin.nix {
            inherit boom;
            src = ./plugins/claude-code;
          };
        }
      );
    };
}
