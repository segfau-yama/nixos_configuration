{ inputs, ... }:
{
  # nixos-base: 環境変数（NIX_CPU、NIX_GPU）でカスタマイズ可能な基本ホスト構成。
  # GPU設定は別途 gpu-profiles/ から読み込む。
  flake.modules.nixos.nixos-base = { ... }: {
    imports = with inputs.self.modules.nixos; [
      system-desktop  # system-base + home-manager + locale + fcitx5 + audio + niri
      programming
      storage         # single or multi-storage support
      admin
      jade
    ];
    networking.hostName = "nixos";
  };
}
