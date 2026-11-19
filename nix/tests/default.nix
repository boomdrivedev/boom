# The nix project's cases, which nix/check.sh evaluates against no store
# (record 0010, design 7): `typecheck` and `test`, one for each task, read
# the flake's inputs from BOOM_FLAKE_INPUTS, the devenv shell's JSON of
# their store paths. Each case is a list of what fails, empty where it
# holds, so one failing case never stops the rest. A check's result is one
# line for each failure, `nix: <case>: <what>`. timber's
# nix/tests/default.nix at boomdrivedev/timber 1adfecc7, for boom's
# cases.
let
  t = import ./lib.nix { paths = builtins.fromJSON (builtins.getEnv "BOOM_FLAKE_INPUTS"); };
  inherit (t.lib)
    concatLists
    concatMap
    concatStringsSep
    mapAttrsToList
    ;

  # Each file's cases kept apart, so two files' cases of one name both
  # run.
  lines =
    files:
    concatStringsSep "\n" (
      concatMap (
        file: concatLists (mapAttrsToList (case: map (what: "nix: ${case}: ${what}")) (import file t))
      ) files
    );
in
{
  typecheck = lines [ ./typecheck.nix ];
  test = lines [ ./packages.nix ];
}
