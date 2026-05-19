{ inputs, ... }:
{
  # nixos-nvidia: NVIDIA GPU を搭載するホスト構成。
  # 共通デスクトップ層は system-desktop (Inheritance Aspect) から継承する。
  flake.modules.nixos.nixos-nvidia = { ... }: {
    imports = with inputs.self.modules.nixos; [
      system-desktop  # system-base + home-manager + locale + fcitx5 + audio + niri
      development
      nvidia          # opengl を内包 (Inheritance Aspect via DRY)
      gaming          # opengl を内包 (Inheritance Aspect via DRY)
      yama
      suichan
    ];
    networking.hostName = "nixos-nvidia";
  };
}
