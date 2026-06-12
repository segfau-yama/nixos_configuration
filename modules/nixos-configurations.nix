{ ... }:
let
  nixosDir = ../nixos;
  dirEntries = builtins.readDir nixosDir;
  hostNames = builtins.filter (name: dirEntries.${name} == "directory") (builtins.attrNames dirEntries);
  flakeParts = builtins.filter builtins.pathExists (
    map (name: nixosDir + "/${name}/flake-parts.nix") hostNames
  );
in
{
  imports = flakeParts;
}
