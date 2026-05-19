{ inputs, ... }:
{
  # nixos-with-hdd: HDD を /mnt/hdd にマウントするホスト構成。
  # 共通デスクトップ層は system-desktop (Inheritance Aspect) から継承する。
  flake.modules.nixos.nixos-with-hdd = { ... }: {
    imports = with inputs.self.modules.nixos; [
      system-desktop  # system-base + home-manager + locale + fcitx5 + audio + niri
      development
      hdd
      yama
      suichan
    ];
    networking.hostName = "nixos-with-hdd";
  };
}
