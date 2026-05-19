{ ... }:
{
  # browser (Home Manager): Chromium ウェブブラウザーを提供する。
  flake.modules.homeManager.browser = { pkgs, ... }: {
    home.packages = with pkgs; [
      chromium
    ];
  };
}
