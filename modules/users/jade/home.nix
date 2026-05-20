{ inputs, ... }:
let
  username = "jade";
in
{
  # jade (Home Manager): デスクトップ全機能を組み合わせたメインユーザー設定。
  flake.modules.homeManager."${username}" = { ... }: {
    imports = with inputs.self.modules.homeManager; [
      niri          # Wayland コンポジター設定 (config.kdl)
      ironbar       # ステータスバー
      notifications # 通知 + 壁紙
      programming   # プログラミングツール群
      gaming        # Lutris + Wine
      media         # Spotify + mpv
      browser       # Chromium
      sns           # Discord 他 SNS クライアント
      electronics   # KiCad + ngspice + gerbv
      mechanical    # FreeCAD + PrusaSlicer + MeshLab + OpenSCAD
    ];

    home.username = "${username}";
    home.homeDirectory = "/home/${username}";
    home.stateVersion = "25.05";

    programs.home-manager.enable = true;

    # Electron/Chromium 系アプリ向けの Wayland セッション変数。
    home.sessionVariables = {
      NIXOS_OZONE_WL               = "1";
      MOZ_ENABLE_WAYLAND           = "1";
      ELECTRON_OZONE_PLATFORM_HINT = "auto";
    };
  };
}
