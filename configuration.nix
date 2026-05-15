{ ... }:
{
  imports = [
    # システム共通モジュール
    ./modules/core/boot.nix
    ./modules/core/nix.nix
    ./modules/core/gc.nix
    ./modules/core/networking.nix
    ./modules/core/timezone.nix

    # ロケールと言語入力
    ./modules/locale/locale.nix
    ./modules/locale/fonts.nix
    ./modules/locale/fcitx5-mozc.nix

    # デスクトップ構成
    ./modules/desktop/display-manager.nix
    ./modules/desktop/hyprland.nix
    ./modules/desktop/ironbar.nix
    ./modules/desktop/terminal.nix
    ./modules/desktop/xdg-portal.nix

    # ハードウェアアクセラレーション
    ./modules/hardware/opengl.nix
    ./modules/hardware/vulkan.nix

    # 音声構成
    ./modules/audio/pipewire.nix
    ./modules/audio/audio-utils.nix

    # ゲーム構成
    ./modules/gaming/steam.nix
    ./modules/gaming/gamemode.nix
    ./modules/gaming/wine.nix

    # 開発/CAD
    ./modules/development/zed.nix
    ./modules/development/tools.nix

    # メディアアプリ
    ./modules/media/music.nix
    ./modules/media/image.nix
    ./modules/media/video.nix

    # ユーザー設定
    ./modules/users/default.nix
  ];

  system.stateVersion = "25.05";
}