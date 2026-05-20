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

    # Wayland 環境での入力方式環境変数。
    #
    # GTK_IM_MODULE / QT_IM_MODULE は設定しない:
    #   GTK4 / Qt6 の Wayland ネイティブアプリは text-input-v3 プロトコルを直接使う。
    #   これらの変数を設定すると XIM ブリッジ経由の入力になり、
    #   Wayland ネイティブ IME が無効化されて日本語入力が壊れるケースがある。
    #
    # XMODIFIERS は残す:
    #   XWayland 上で動作するレガシーアプリ (一部 Electron / X11 アプリ) に必要。
    environment.sessionVariables = {
      XMODIFIERS   = "@im=fcitx";
      INPUT_METHOD = "fcitx";
    };
  };
}
