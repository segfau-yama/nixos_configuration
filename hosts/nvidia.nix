{ ... }:
{
  imports = [
    ../hardware-configuration.nix
    ../configuration.nix
    ../modules/hardware/nvidia.nix
  ];

  networking.hostName = "nixos-nvidia";
}