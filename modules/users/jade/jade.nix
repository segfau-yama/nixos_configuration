{ inputs, ... }:
let
  username = "jade";
in
{
  # jade (NixOS): メインユーザーの定義と Home Manager 統合。
  flake.modules.nixos."${username}" = { pkgs, ... }: {
    users.users."${username}" = {
      isNormalUser = true;
      description  = "Jade";
      extraGroups  = [
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

    # Home Manager モジュールを NixOS ユーザーに紐付ける。
    home-manager.users."${username}" = {
      imports = [ inputs.self.modules.homeManager."${username}" ];
    };
  };

  # jade (Home Manager): 最小限の GUI ユーザー設定 (desktop のみ)。
  flake.modules.homeManager."${username}" = { ... }: {
    imports = with inputs.self.modules.homeManager; [
      desktop     # Hyprland ユーザー設定
    ];

    home.username    = "${username}";
    home.homeDirectory = "/home/${username}";
    home.stateVersion  = "25.05";

    programs.home-manager.enable = true;

  };
}
