{ inputs, ... }:
let
  username = "admin";
in
{
  # admin: 管理者ユーザー。GUI ホストでは niri がユーザー別 config.kdl を読むため、
  # 最小限の Home Manager desktop 設定を持たせる。
  flake.modules.nixos."${username}" = { pkgs, ... }: {
    users.users."${username}" = {
      isNormalUser = true;
      description = "Admin";
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
    imports = [
      inputs.self.modules.homeManager.desktop
    ];

    home.username      = "${username}";
    home.homeDirectory = "/home/${username}";
    home.stateVersion  = "25.05";

    programs.home-manager.enable = true;
  };
}
