{ ... }:
{
  flake.modules.nixos.nixos-amd = { ... }: {
    imports = [ ../../../nixos/hardware-configuration.nix ];
  };
}
