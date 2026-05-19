{ inputs, ... }:
{
  # system-base: 全ホスト共通の基盤 NixOS 設定。
  # unstable overlay, nix 設定, GC, boot, networking, timezone, stateVersion を含む。
  flake.modules.nixos.system-base = { pkgs, ... }: {
    nixpkgs.overlays = [
      # pkgs.unstable でアンスタブル版パッケージを参照できるようにする。
      (final: _prev: {
        unstable = import inputs.nixpkgs-unstable {
          inherit (final.stdenv.hostPlatform) system;
          config.allowUnfree = true;
        };
      })
    ];

    nixpkgs.config.allowUnfree = true;
    system.stateVersion = "25.05";

    nix.settings = {
      experimental-features = [ "nix-command" "flakes" ];
      auto-optimise-store = true;
      trusted-users = [ "root" "@wheel" ];
    };

    # nixos-rebuild の世代一覧に表示するラベル。
    system.nixos.label = "tracked";

    nix.gc = {
      automatic = true;
      dates = "weekly";
      options = "--delete-older-than 14d";
    };
    nix.optimise.automatic = true;

    # 重要な世代を GC から保護するスクリプト。
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
        generation="''${2:-current}"
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

    boot.loader.systemd-boot.enable = true;
    boot.loader.efi.canTouchEfiVariables = true;

    networking.networkmanager.enable = true;
    services.resolved = {
      enable = true;
      dnssec = "allow-downgrade";
    };
    networking.firewall = {
      enable = true;
      allowedTCPPorts = [ ];
      allowedUDPPorts = [ ];
    };

    time.timeZone = "Asia/Tokyo";
  };
}
