{ ... }:
{
  imports = [
    ../hardware-configuration.nix
    ../configuration.nix
    ../modules/hardware/amd.nix
  ];

  networking.hostName = "nixos-amd";
}