{ ... }:
{
  # notifications (Home Manager): mako 通知デーモン + swww 壁紙マネージャーを設定する。
  flake.modules.homeManager.notifications = { pkgs, ... }: {
    services.mako = {
      enable = true;

      # HM 25.05 以降、オプションは services.mako.settings.* 以下に移動された。
      # キー名は camelCase → kebab-case に変更されている。
      settings = {
        default-timeout  = 5000;      # 通知を 5 秒後に自動で消す

        # ironbar / niri と合わせたカラーパレット (Catppuccin Mocha ベース)
        background-color = "#11111bCC";  # 80% 不透明の暗いネイビー
        border-color     = "#cba6f747";  # パープルアクセント (28% 不透明)
        text-color       = "#cdd6f4";    # メインテキスト
        border-radius    = 12;
        border-size      = 1;

        font        = "Inter 13";
        padding     = "12 16";
        width       = 380;
        anchor      = "top-right";
        margin      = "10,10,0,0";  # 上・右に 10px マージン
        layer       = "overlay";    # 最前面に表示
        max-visible = 5;
      };
    };

    home.packages = [ pkgs.swww ];

    # swww: Home Manager に services.swww が未実装のため systemd ユーザーサービスで管理する。
    systemd.user.services.swww-daemon = {
      Unit = {
        Description = "swww wallpaper daemon";
        After = [ "graphical-session.target" ];
        PartOf = [ "graphical-session.target" ];
      };
      Service = {
        ExecStart = "${pkgs.swww}/bin/swww-daemon";
        Restart = "on-failure";
      };
      Install.WantedBy = [ "graphical-session.target" ];
    };

    systemd.user.services.swww-init = {
      Unit = {
        Description = "Set initial wallpaper via swww";
        After = [ "swww-daemon.service" ];
        Requires = [ "swww-daemon.service" ];
        PartOf = [ "graphical-session.target" ];
      };
      Service = {
        Type = "oneshot";
        ExecStartPre = "${pkgs.coreutils}/bin/sleep 1";
        # swww clear <COLOR> で単色背景を表示する (Catppuccin Mocha Base: #1e1e2e)。
        # 独自の壁紙ファイルを使う場合はこの行を以下のように変更すること:
        #   ExecStart = "${pkgs.swww}/bin/swww img /path/to/wallpaper.png --transition-type fade";
        ExecStart = "${pkgs.swww}/bin/swww clear 1e1e2e";
        RemainAfterExit = true;
      };
      Install.WantedBy = [ "graphical-session.target" ];
    };
  };
}
