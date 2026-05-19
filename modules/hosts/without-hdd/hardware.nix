{ ... }:
{
  flake.modules.nixos.nixos-without-hdd = { ... }: {
    imports = [ ../../../nixos/hardware-configuration.nix ];
  };
}
