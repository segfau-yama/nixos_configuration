{ ... }:
{
  # browser (Home Manager): capability に応じて GUI/TUI のブラウザーを切り替える。
  flake.modules.homeManager.browser = { config, lib, pkgs, ... }: {
    config = lib.mkMerge [
      (lib.mkIf (config.my.capabilities.user_interface == "gui") {
        home.packages = with pkgs; [
          chromium
        ];
      })
      (lib.mkIf (config.my.capabilities.user_interface == "tui") {
        home.packages = with pkgs; [
          w3m
        ];
      })
    ];
  };
}
