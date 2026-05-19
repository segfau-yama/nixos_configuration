{ ... }:
{
  # notifications (Home Manager): mako 通知デーモン + swaybg 壁紙マネージャーを設定する。
  flake.modules.homeManager.notifications = { ... }: {
    services.mako = {
      enable = true;
      defaultTimeout = 5000;  # 通知を 5 秒後に自動で消す
    };

    services.swaybg = {
      enable = true;
      image = "/usr/share/backgrounds/cyber.png";
      mode = "fill";
    };
  };
}
