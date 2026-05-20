{ lib, ... }:
{
  # lib.mkDefault (優先度 1000) を使うことで、flake の mkNixosWithEnv が
  # 通常優先度 (100) で上書きできる。nixos-generate-config も同じ形式を生成する。
  nixpkgs.hostPlatform = lib.mkDefault "x86_64-linux";

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
