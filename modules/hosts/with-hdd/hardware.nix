{ ... }:
{
  flake.modules.nixos.nixos-with-hdd = { ... }: {
    imports = [ ../../../nixos/hardware-configuration.nix ];
  };
}
