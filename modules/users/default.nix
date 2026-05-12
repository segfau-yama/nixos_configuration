{ pkgs, ... }:
{
  users.users.yama = {
    isNormalUser = true;
    description = "Yama";
    extraGroups = [
      "wheel"
      "networkmanager"
      "audio"
      "video"
      "input"
    ];
    shell = pkgs.zsh;
  };

  programs.zsh.enable = true;
}