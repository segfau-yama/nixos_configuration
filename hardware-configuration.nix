{ ... }:
{
  # プラットフォームを明示し、評価時の意図しない不一致を防ぐ。
  nixpkgs.hostPlatform = "x86_64-linux";

  # 必要に応じてラベルは実際のディスク構成に合わせて変更する。
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