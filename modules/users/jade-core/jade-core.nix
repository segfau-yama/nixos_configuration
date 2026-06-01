{ inputs, ... }:
let
  username = "jade-core";
in
{
  # jade-core: TUI/CUI 向けプリセットユーザー。
  flake.modules.nixos."${username}" = { pkgs, ... }: {
    users.users."${username}" = {
      isNormalUser = true;
      description = "Jade Core";
      extraGroups = [
        "wheel"
        "networkmanager"
        "audio"
        "video"
        "input"
        "seat"
      ];
      shell = pkgs.zsh;
    };

    programs.zsh.enable = true;

    home-manager.users."${username}" = {
      imports = [ inputs.self.modules.homeManager."${username}" ];
    };
  };

  flake.modules.homeManager."${username}" = { ... }: {
    imports = with inputs.self.modules.homeManager; [
      base
      programming
      browser
      media
      sns
    ];

    my.capabilities = {
      user_interface = "tui";
      window_manager = "none";
    };

    home.username = username;
    home.homeDirectory = "/home/${username}";
    home.stateVersion = "25.05";

    programs.home-manager.enable = true;
  };
}
