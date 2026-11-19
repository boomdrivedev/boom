# boom's Claude Code plugin: plugins/claude-code of the flake's own tree,
# copied whole, at the boom package's version, so a host installs boom and
# its plugin from one tag at one version (record 0010, design 6;
# blueprints principle 17). Its hook calls `boom` from PATH, the boom of the
# same release beside it.
{
  lib,
  stdenvNoCC,
  boom,
  src,
}:
stdenvNoCC.mkDerivation {
  pname = "boom-claude-plugin";
  inherit (boom) version;
  inherit src;
  dontUnpack = true;
  dontConfigure = true;
  dontBuild = true;
  installPhase = ''
    runHook preInstall
    cp -R "$src" "$out"
    chmod -R u+w "$out"
    runHook postInstall
  '';
  meta = {
    description = "boom's Claude Code plugin: every session in a declared repository starts with its claims";
    homepage = "https://github.com/boomdrivedev/boom";
    license = lib.licenses.mit;
  };
}
