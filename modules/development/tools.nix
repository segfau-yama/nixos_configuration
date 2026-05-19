{ pkgs, ... }:
{
  programs.direnv.enable = true;
  programs.nix-ld.enable = true;
  programs.nushell.enable = true;

  environment.systemPackages = with pkgs; [
    git
    xh
    jaq
    nix-index
    devenv
    rustc
    clang
    mold
    just
    pkg-config
    nil
    nixfmt-rfc-style
  ];
}