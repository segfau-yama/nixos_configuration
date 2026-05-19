{ ... }:
{
  # hdd: HDD を /mnt/hdd にマウントし、/nix をバインドマウントする構成。
  # HDD にラベルを付ける方法: sudo e2label /dev/sdX1 "nixstore-hdd"
  # ラベル確認: lsblk -o NAME,LABEL,FSTYPE
  flake.modules.nixos.hdd = { ... }: {
    fileSystems."/mnt/hdd" = {
      device = "/dev/disk/by-label/nixstore-hdd";
      fsType = "ext4";
      options = [
        "defaults"
        "nofail"                        # HDD 未接続時もシステム起動を継続
        "noatime"                       # 読み取り性能向上 (HDD 向け)
        "x-systemd.device-timeout=5s"  # 検出タイムアウトを短縮
      ];
    };

    fileSystems."/nix" = {
      device = "/mnt/hdd/nix";
      fsType = "none";
      options = [
        "bind"
        "nofail"
        "x-systemd.requires-mounts-for=/mnt/hdd"  # /mnt/hdd マウント後に実行
      ];
    };
  };
}
