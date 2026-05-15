{ pkgs, ... }:
{
  nix.settings = {
    experimental-features = [ "nix-command" "flakes" ];
    auto-optimise-store = true;
    trusted-users = [ "root" "@wheel" ];
  };

  # nixos-rebuild の世代一覧に表示するラベル。
  system.nixos.label = "tracked";

  # NVIDIA/Steam/Discord などの unfree ソフトを宣言的に許可。
  nixpkgs.config.allowUnfree = true;

  environment.systemPackages = [
    (pkgs.writeShellScriptBin "nixos-mark-generation" ''
      set -euo pipefail

      if [[ $# -lt 1 || $# -gt 2 ]]; then
        echo "Usage: nixos-mark-generation <tag> [generation-number|current]" >&2
        echo "Example: sudo nixos-mark-generation before-gpu-update 132" >&2
        echo "Example: sudo nixos-mark-generation baseline" >&2
        exit 1
      fi

      tag="$1"
      generation="${2:-current}"
      root_dir="/nix/var/nix/gcroots/important-generations"

      if [[ "$generation" == "current" ]]; then
        profile_link="/nix/var/nix/profiles/system"
      else
        profile_link="/nix/var/nix/profiles/system-$generation-link"
      fi

      if [[ ! -e "$profile_link" ]]; then
        echo "Generation link not found: $profile_link" >&2
        exit 2
      fi

      target="$(readlink -f "$profile_link")"
      mkdir -p "$root_dir"
      ln -sfn "$target" "$root_dir/$tag"

      echo "Marked generation as important: $tag"
      echo "Source link: $profile_link"
      echo "Target store path: $target"
      echo "GC root: $root_dir/$tag"
    '')
  ];
}