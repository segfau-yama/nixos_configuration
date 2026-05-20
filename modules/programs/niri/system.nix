{ ... }:
{
  # niri/system (NixOS): Niri Wayland コンポジターの有効化と関連システムサービス・
  # パッケージを設定する。greetd / portal / tofi は別ファイルで追記 (Collector Aspect)。
  flake.modules.nixos.niri = { pkgs, ... }: {
    programs.niri.enable = true;
    programs.dconf.enable = true;
    services.dbus.enable   = true;
    security.polkit.enable = true;
    services.seatd.enable  = true;

    environment.systemPackages = with pkgs; [
      # niri は programs.niri.enable で自動インストール済み
      # mako / swww は HM services.mako / systemd.user.services で管理済み
      wl-clipboard  # Wayland クリップボード
      wayshot       # スクリーンショット
      wlsunset      # ブルーライトカット (夜間モード)
      wezterm       # ターミナルエミュレーター
      pwvucontrol   # PipeWire ネイティブ GUI ボリュームコントローラー (Rust)
      playerctl     # メディアキーコントロール
      pamixer       # コマンドラインボリューム操作
      ironbar       # IronBar ステータスバー
      spacedrive    # ファイルマネージャー
    ];
  };
}
