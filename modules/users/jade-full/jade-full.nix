{ inputs, ... }:
let
  username = "jade-full";
in
{
  # jade-full: Hyprland と全用途パッケージのプリセットユーザー。
  flake.modules.nixos."${username}" = { pkgs, ... }: {
    imports = [ inputs.self.modules.nixos.desktopHyprland ];

    users.users."${username}" = {
      isNormalUser = true;
      description = "Jade Full";
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
      desktopHyprland
      programming
      browser
      gaming
      media
      sns
      electronics
      mechanical
    ];

    my.capabilities = {
      user_interface = "gui";
      window_manager = "hyprland";
    };

    home.username = username;
    home.homeDirectory = "/home/${username}";
    home.stateVersion = "25.05";

    programs.home-manager.enable = true;
  };
}
