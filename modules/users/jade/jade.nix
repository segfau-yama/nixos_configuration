{ inputs, ... }:
let
  username = "jade";
in
{
  # jade (NixOS): メインユーザーの定義と Home Manager 統合。
  flake.modules.nixos."${username}" = { pkgs, ... }: {
    users.users."${username}" = {
      isNormalUser = true;
      description  = "Jade";
      extraGroups  = [
        "wheel"
        "networkmanager"
        "audio"
        "video"
        "input"
        "seat"
      ];
      shell = pkgs.zsh;
    };

    programs.zsh.enable = true;

    # Home Manager モジュールを NixOS ユーザーに紐付ける。
    home-manager.users."${username}" = {
      imports = [ inputs.self.modules.homeManager."${username}" ];
    };
  };

  # jade (Home Manager): デスクトップ全機能を組み合わせたメインユーザー設定。
  flake.modules.homeManager."${username}" = { pkgs, ... }: {
    imports = with inputs.self.modules.homeManager; [
      desktop       # デスクトップ環境 (Niri/IronBar/mako/swww 統合)
      programming   # プログラミングツール群
      gaming        # Lutris + Wine
      media         # Spotify + mpv
      browser       # Chromium
      sns           # Discord 他 SNS クライアント
      electronics   # KiCad + ngspice + gerbv
      mechanical    # FreeCAD + PrusaSlicer + MeshLab + OpenSCAD
    ];

    home.username    = "${username}";
    home.homeDirectory = "/home/${username}";
    home.stateVersion  = "25.05";

    programs.home-manager.enable = true;

    # ユーザーアプリ: デスクトップ基盤や専用モジュールに属さないアプリを直接管理する。
    home.packages = with pkgs; [
      wezterm      # ターミナルエミュレーター
      spacedrive   # ファイルマネージャー
      pwvucontrol  # PipeWire ネイティブ GUI ボリュームコントローラー
      pamixer      # コマンドライン ボリューム操作
    ];

    # Electron/Chromium 系アプリ向けの Wayland セッション変数。
    home.sessionVariables = {
      NIXOS_OZONE_WL               = "1";
      MOZ_ENABLE_WAYLAND           = "1";
      ELECTRON_OZONE_PLATFORM_HINT = "auto";
    };
  };
}
