{ inputs, ... }:
{
  # nixos-amd: AMD GPU を搭載するホスト構成。
  # 共通デスクトップ層は system-desktop (Inheritance Aspect) から継承する。
  flake.modules.nixos.nixos-amd = { ... }: {
    imports = with inputs.self.modules.nixos; [
      system-desktop  # system-base + home-manager + locale + fcitx5 + audio + niri
      development
      amd             # opengl を内包 (Inheritance Aspect via DRY)
      gaming          # opengl を内包 (Inheritance Aspect via DRY)
      yama
      suichan
    ];
    networking.hostName = "nixos-amd";
  };
}
