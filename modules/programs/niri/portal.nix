{ ... }:
{
  # niri/portal (NixOS): XDG Desktop Portal を GTK バックエンドで提供する。
  # ファイル選択・画面共有・URI ハンドリングなど Wayland アプリに必要。
  # system.nix と同じ flake.modules.nixos.niri に追記する (Collector Aspect)。
  flake.modules.nixos.niri = { pkgs, ... }: {
    xdg.portal = {
      enable            = true;
      xdgOpenUsePortal  = true;
      extraPortals      = with pkgs; [ xdg-desktop-portal-gtk ];
      config.common.default = [ "gtk" ];
    };
  };
}
