{ ... }:
{
  # niri/nixos (NixOS): Niri Wayland コンポジターに必要な全 NixOS 設定を一箇所に集約する。
  #
  # 含まれる設定:
  #   - Niri / dconf / dbus / polkit / seatd の有効化
  #   - greetd + tuigreet によるログインマネージャー
  #   - xdg-desktop-portal (gtk + gnome バックエンド)
  #   - tofi アプリランチャー (Catppuccin Mocha テーマ)
  #   - Wayland 向け補助パッケージ群
  #
  # ※ HM 側の config.kdl は niri/home.nix で管理する (Multi Context Aspect)。
  flake.modules.nixos.niri = { pkgs, ... }: {

    # ── Compositor & System Services ────────────────────────────────────────
    programs.niri.enable   = true;
    programs.dconf.enable  = true;
    services.dbus.enable   = true;
    security.polkit.enable = true;
    services.seatd.enable  = true;

    # ── Login Manager: greetd + tuigreet ────────────────────────────────────
    # tuigreet は TUI 上でユーザー選択し、dbus-run-session 経由で niri を起動する。
    services.greetd = {
      enable = true;
      settings.default_session = {
        command = "${pkgs.greetd.tuigreet}/bin/tuigreet --time --cmd 'dbus-run-session niri --session'";
        user    = "greeter";
      };
    };

    # ── XDG Desktop Portal ───────────────────────────────────────────────────
    # ポータルバックエンドを2つ用意する:
    #   gtk   : ファイル選択・URI ハンドリング・その他汎用インターフェイス
    #   gnome : スクリーンキャスト・スクリーンショット・リモートデスクトップ
    #           (PipeWire + zwlr_screencopy_v1 経由。Discord/OBS の画面共有に必要)
    xdg.portal = {
      enable           = true;
      xdgOpenUsePortal = true;
      extraPortals = with pkgs; [
        xdg-desktop-portal-gtk    # ファイル選択・URI ハンドリング
        xdg-desktop-portal-gnome  # スクリーンキャスト・スクリーンショット
      ];
      config.common = {
        default                               = [ "gtk" ];
        "org.freedesktop.portal.ScreenCast"   = [ "gnome" ];
        "org.freedesktop.portal.Screenshot"   = [ "gnome" ];
        "org.freedesktop.portal.RemoteDesktop" = [ "gnome" ];
      };
    };

    # ── App Launcher: tofi ─────────────────────────────────────────────────────
    # programs.tofi NixOS オプションは nixpkgs 25.05 には存在しない。
    # テーマ設定は HM 側 (niri/home.nix) で行う。
    # ここではシステム全体にパッケージをインストールするのみ。

    # ── System Packages ──────────────────────────────────────────────────
    # niri 本体は programs.niri.enable で自動インストール済み。
    # mako / swww は HM の services.mako / systemd.user.services で管理済み。
    environment.systemPackages = with pkgs; [
      wl-clipboard  # Wayland クリップボード
      wayshot       # スクリーンショット
      wlsunset      # ブルーライトカット (夜間モード)
      wezterm       # ターミナルエミュレーター
      pwvucontrol   # PipeWire ネイティブ GUI ボリュームコントローラー (Rust)
      playerctl     # メディアキーコントロール
      pamixer       # コマンドラインボリューム操作
      ironbar       # IronBar ステータスバー
      spacedrive    # ファイルマネージャー
      tofi          # Wayland アプリランチャー (設定は HM 側で管理)
    ];
  };
}
