{ inputs, ... }:
{
  # nixos-default: GPU なし・HDD なしの基本ホスト構成。
  # 共通デスクトップ層は system-desktop (Inheritance Aspect) から継承する。
  flake.modules.nixos.nixos-default = { ... }: {
    imports = with inputs.self.modules.nixos; [
      system-desktop  # system-base + home-manager + locale + fcitx5 + audio + niri
      development
      yama
      suichan
    ];
    networking.hostName = "nixos-default";
  };
}
