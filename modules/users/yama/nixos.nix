{ ... }:
let
  username = "yama";
in
{
  # yama: 管理者ユーザー。Home Manager は持たない。
  flake.modules.nixos."${username}" = { pkgs, ... }: {
    users.users."${username}" = {
      isNormalUser = true;
      description = "Yama";
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
  };
}
