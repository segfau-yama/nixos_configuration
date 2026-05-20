{ ... }:
{
  flake.modules.nixos.nixos-base = { ... }: {
    imports = [ ../../../nixos/hardware-configuration.nix ];
  };
}
