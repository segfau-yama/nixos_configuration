{ ... }:
{
  # notifications (Home Manager): mako 通知デーモン + swww 壁紙マネージャーを設定する。
  flake.modules.homeManager.notifications = { pkgs, ... }: {
    services.mako = {
      enable = true;
      defaultTimeout = 5000;       # 通知を 5 秒後に自動で消す

      # ironbar / niri と合わせたカラーパレット (Catppuccin Mocha ベース)
      backgroundColor = "#11111bCC";  # 80% 不透明の暗いネイビー
      borderColor = "#cba6f747";      # パープルアクセント (28% 不透明)
      textColor = "#cdd6f4";          # メインテキスト
      borderRadius = 12;
      borderSize = 1;

      font = "Inter 13";
      padding = "12 16";
      width = 380;
      anchor = "top-right";
      margin = "10,10,0,0";           # 上・右に 10px マージン
      layer = "overlay";              # 最前面に表示
      maxVisible = 5;
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
        Description = "Set wallpaper via swww";
        After = [ "swww-daemon.service" ];
        Requires = [ "swww-daemon.service" ];
        PartOf = [ "graphical-session.target" ];
      };
      Service = {
        Type = "oneshot";
        ExecStartPre = "${pkgs.coreutils}/bin/sleep 1";
        ExecStart = "${pkgs.swww}/bin/swww img /usr/share/backgrounds/cyber.png --transition-type fade";
        RemainAfterExit = true;
      };
      Install.WantedBy = [ "graphical-session.target" ];
    };
  };
}
