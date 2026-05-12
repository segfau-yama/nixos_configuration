{ pkgs, ... }:
{
  environment.systemPackages = with pkgs; [
    pavucontrol
    playerctl
    pamixer
  ];
}