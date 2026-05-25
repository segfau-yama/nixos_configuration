{ ... }:
let
  username = "admin";
in
{
  # admin: 管理者向けの最小 CUI ユーザー。
  flake.modules.nixos."${username}" = { pkgs, ... }: {
    users.users."${username}" = {
      isNormalUser = true;
      description = "Admin";
      extraGroups = [
        "wheel"
        "networkmanager"
      ];
      shell = pkgs.zsh;
    };

    programs.zsh.enable = true;
  };
}
