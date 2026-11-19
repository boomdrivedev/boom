# nix:test: what the flake gives (record 0010, design 6, 7): each system's
# boom at the tree's version, Cargo.toml's [workspace.package] and nothing
# else; its plugin at that same version, the tree's plugins/claude-code;
# and the three systems alone, with no module.
t:
let
  inherit (t) lib outputs systems;
  workspace = (builtins.fromTOML (builtins.readFile ../../Cargo.toml)).workspace.package;

  # Each system the flake gives, with its packages.
  given = lib.filter (system: outputs.packages ? ${system}) systems;
  package = system: name: outputs.packages.${system}.${name} or { };
in
{
  "The package's version is the tree's" = lib.concatMap (
    system:
    let
      version = (package system "boom").version or "none";
    in
    t.expect (version == workspace.version)
      "packages.${system}.boom names ${version}, not Cargo.toml's workspace version ${workspace.version}"
  ) given;

  "The plugin is at boom's version" = lib.concatMap (
    system:
    let
      plugin = (package system "boom-claude-plugin").version or "none";
      boom = (package system "boom").version or "none";
    in
    t.expect (
      plugin == boom
    ) "packages.${system}.boom-claude-plugin names ${plugin}, not its boom's ${boom}"
  ) given;

  "The plugin is the tree's" = lib.concatMap (
    system:
    let
      src = (package system "boom-claude-plugin").src or null;
    in
    t.expect (src == ../../plugins/claude-code)
      "packages.${system}.boom-claude-plugin's source is ${toString src}, not the tree's plugins/claude-code"
  ) given;

  "Three systems and no module" =
    t.expect (
      builtins.attrNames outputs.packages == lib.sort lib.lessThan systems
    ) "packages gives ${toString (builtins.attrNames outputs.packages)}, not ${toString systems}"
    ++ t.expect (!(outputs ? darwinModules)) "the flake gives darwinModules"
    ++ t.expect (!(outputs ? nixosModules)) "the flake gives nixosModules";
}
