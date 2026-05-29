{ ... }:
{
  # desktopPlasma (NixOS): KDE Plasma 6 と SDDM Wayland ログインマネージャー。
  flake.modules.nixos.desktopPlasma = { config, lib, pkgs, ... }:
  let
    desktopUser = "jade";
    niriSessionForDesktopUser = pkgs.writeShellScript "niri-session-for-${desktopUser}" ''
      session_user="$(${pkgs.coreutils}/bin/id -un)"
      if [ "$session_user" != "${desktopUser}" ]; then
        printf '%s\n' 'This niri session is configured only for jade. Use Plasma or a TTY for admin.'
        ${pkgs.coreutils}/bin/sleep 3
        exit 1
      fi

      exec ${config.programs.niri.package}/bin/niri-session
    '';
    niriDesktopSession = pkgs.runCommand "niri-jade-wayland-session" {
      passthru.providedSessions = [ "niri-jade" ];
    } ''
      mkdir -p "$out/share/wayland-sessions"
      cat > "$out/share/wayland-sessions/niri-jade.desktop" <<EOF
      [Desktop Entry]
      Name=Niri
      Comment=Scrollable-tiling Wayland compositor
      Exec=${niriSessionForDesktopUser}
      Type=Application
      DesktopNames=niri
      EOF
    '';
  in {
    services.desktopManager.plasma6.enable = true;

    services.displayManager = {
      defaultSession = lib.mkDefault "plasma";

      # programs.niri.enable also registers its upstream session package. Use an
      # explicit list so Plasma stays available and niri goes through the jade
      # guard above instead of the unguarded upstream desktop file.
      sessionPackages = lib.mkForce [
        pkgs.kdePackages.plasma-workspace
        niriDesktopSession
      ];

      sddm = {
        enable = true;
        theme = lib.mkDefault "breeze";
        wayland = {
          enable = true;
          compositor = "kwin";
        };
        settings.Users.HideUsers = "admin";
      };
    };
  };
}
