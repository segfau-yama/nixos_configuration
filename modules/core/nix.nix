{ ... }:
{
  nix.settings = {
    experimental-features = [ "nix-command" "flakes" ];
    auto-optimise-store = true;
    trusted-users = [ "root" "@wheel" ];
  };

  # Keep unfree software declarative for NVIDIA/Steam/Discord etc.
  nixpkgs.config.allowUnfree = true;
}