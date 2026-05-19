{ inputs, ... }:
let
  username = "suichan";
in
{
  # suichan (NixOS): メインユーザーの定義と Home Manager 統合。
  flake.modules.nixos."${username}" = { pkgs, ... }: {
    users.users."${username}" = {
      isNormalUser = true;
      description = "Suichan";
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

    # Home Manager モジュールを NixOS ユーザーに紐付ける。
    home-manager.users."${username}" = {
      imports = [ inputs.self.modules.homeManager."${username}" ];
    };
  };
}
