{ ... }:
{
  # fcitx5: 日本語入力エンジン (fcitx5-mozc) と Wayland 向け環境変数を設定する。
  flake.modules.nixos.fcitx5 = { pkgs, ... }: {
    i18n.inputMethod = {
      enable = true;
      type = "fcitx5";
      fcitx5.addons = with pkgs; [
        fcitx5-mozc
        fcitx5-gtk
        fcitx5-configtool
      ];
    };

    # GTK/Qt/X アプリが fcitx5 を入力方式として認識するための環境変数。
    environment.sessionVariables = {
      GTK_IM_MODULE = "fcitx";
      QT_IM_MODULE  = "fcitx";
      XMODIFIERS    = "@im=fcitx";
      INPUT_METHOD  = "fcitx";
    };
  };
}
