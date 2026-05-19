{ ... }:
{
  flake.modules.nixos.nixos-nvidia = { ... }: {
    imports = [ ../../../nixos/hardware-configuration.nix ];
  };
}
