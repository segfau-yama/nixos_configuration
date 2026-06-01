{ ... }:
{
  # sns (Home Manager): capability に応じて GUI/TUI のコミュニケーションツールを切り替える。
  flake.modules.homeManager.sns = { config, lib, pkgs, ... }: {
    config = lib.mkMerge [
      (lib.mkIf (config.my.capabilities.user_interface == "gui") {
        home.packages = with pkgs; [
          discord
        ];
      })
      (lib.mkIf (config.my.capabilities.user_interface != "gui") {
        home.packages = with pkgs; [
          weechat
        ];
      })
    ];
  };
}
