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
      "seat"
    ];
    shell = pkgs.zsh;
  };

  programs.zsh.enable = true;
}