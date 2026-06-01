{ inputs, ... }:
let
  username = "jade-gaming";
in
{
  # jade-gaming: KDE Plasma とゲーム用途のプリセットユーザー。
  flake.modules.nixos."${username}" = { pkgs, ... }: {
    imports = [ inputs.self.modules.nixos.desktopPlasma ];

    my.desktop.plasmaUsers = [ username ];

    users.users."${username}" = {
      isNormalUser = true;
      description = "Jade Gaming";
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
      desktopPlasma
      browser
      gaming
      media
      sns
    ];

    my.capabilities = {
      user_interface = "gui";
      window_manager = "plasma";
    };

    home.username = username;
    home.homeDirectory = "/home/${username}";
    home.stateVersion = "25.05";

    programs.home-manager.enable = true;
  };
}
