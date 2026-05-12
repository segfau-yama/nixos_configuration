{ ... }:
{
  # Explicit platform helps avoid accidental evaluation mismatches.
  nixpkgs.hostPlatform = "x86_64-linux";

  # Replace labels with your actual disk layout if needed.
  fileSystems."/" = {
    device = "/dev/disk/by-label/nixos";
    fsType = "ext4";
  };

  fileSystems."/boot" = {
    device = "/dev/disk/by-label/boot";
    fsType = "vfat";
  };

  swapDevices = [
    { device = "/dev/disk/by-label/swap"; }
  ];
}