{ inputs, ... }:
{
  # desktopHyprland (NixOS): Hyprland WM のシステム設定。
  flake.modules.nixos.desktopHyprland = { config, lib, pkgs, ... }: {
    config = lib.mkIf (config.my.capabilities.window_manager == "hyprland") {
      hardware.graphics.enable = true;

      programs.hyprland = {
        enable = true;
        xwayland.enable = true;
      };

      programs.dconf.enable = true;
      services.dbus.enable = true;
      security.polkit.enable = true;

      environment.sessionVariables.NIXOS_OZONE_WL = "1";

      services.greetd = {
        enable = true;
        settings.default_session = {
          command = "${pkgs.greetd.tuigreet}/bin/tuigreet --time --cmd ${config.programs.hyprland.package}/bin/Hyprland";
          user = "greeter";
        };
      };

      xdg.portal = {
        enable = true;
        xdgOpenUsePortal = true;
        extraPortals = with pkgs; [
          xdg-desktop-portal-hyprland
          xdg-desktop-portal-gtk
        ];
        config.hyprland = {
          default = [ "hyprland" "gtk" ];
          "org.freedesktop.impl.portal.FileChooser" = [ "gtk" ];
          "org.freedesktop.portal.ScreenCast" = [ "hyprland" ];
          "org.freedesktop.portal.Screenshot" = [ "hyprland" ];
        };
      };
    };
  };

  # desktopHyprland (Home Manager): Hyprland ユーザー設定。
  flake.modules.homeManager.desktopHyprland = { config, lib, pkgs, ... }:
  let
    screenshotRegion = pkgs.writeShellScript "wayshot-region" ''
      ${pkgs.coreutils}/bin/mkdir -p "$HOME/Pictures/Screenshots"
      exec ${pkgs.wayshot}/bin/wayshot "$HOME/Pictures/Screenshots/region-$(${pkgs.coreutils}/bin/date +%Y%m%d-%H%M%S).png" -g
    '';
    screenshotWindow = pkgs.writeShellScript "wayshot-window" ''
      ${pkgs.coreutils}/bin/mkdir -p "$HOME/Pictures/Screenshots"
      exec ${pkgs.wayshot}/bin/wayshot "$HOME/Pictures/Screenshots/window-$(${pkgs.coreutils}/bin/date +%Y%m%d-%H%M%S).png" --choose-toplevel
    '';
    screenshotOutput = pkgs.writeShellScript "wayshot-output" ''
      ${pkgs.coreutils}/bin/mkdir -p "$HOME/Pictures/Screenshots"
      exec ${pkgs.wayshot}/bin/wayshot "$HOME/Pictures/Screenshots/output-$(${pkgs.coreutils}/bin/date +%Y%m%d-%H%M%S).png"
    '';
  in {
    config = lib.mkIf (config.my.capabilities.window_manager == "hyprland") {
      home.packages = with pkgs; [
        brightnessctl
        cliphist
        hicolor-icon-theme
        hypridle
        hyprpaper
        ironbar
        mako
        papirus-icon-theme
        playerctl
        polkit_gnome
        rofi-wayland
        slurp
        wl-clipboard
        wlsunset
        wayshot
        wezterm
        yazi
      ];

      gtk = {
        enable = true;
        iconTheme = {
          name = "Papirus-Dark";
          package = pkgs.papirus-icon-theme;
        };
      };

      xdg.configFile."rofi/config.rasi".text = ''
        configuration {
            modi:                "drun,run,window";
            show-icons:          true;
            icon-theme:          "Papirus-Dark,hicolor";
            display-drun:        "󰣇";
            display-run:         "󰆍";
            display-window:      "󰖯";
            drun-display-format: "{name}";
            terminal:            "wezterm";
            font:                "Inter 13";
        }

        @theme "~/.config/rofi/themes/catppuccin-mocha-purple.rasi"
      '';

      xdg.configFile."rofi/themes/catppuccin-mocha-purple.rasi".source =
        ./catppuccin-mocha-purple.rasi;

      xdg.configFile."hypr/hyprland.conf".text =
        (builtins.readFile ./hyprland.conf) + ''

          exec-once = ${pkgs.polkit_gnome}/libexec/polkit-gnome-authentication-agent-1

          bind = $mainMod, P, exec, ${screenshotRegion}
          bind = , Print, exec, ${screenshotRegion}
          bind = alt, Print, exec, ${screenshotWindow}
          bind = control, Print, exec, ${screenshotOutput}
        '';

      xdg.configFile."hypr/hyprpaper.conf".source = ./hyprpaper.conf;

      xdg.configFile."hypr/hypridle.conf".text = ''
        general {
            after_sleep_cmd = hyprctl dispatch dpms on
            ignore_dbus_inhibit = false
        }

        listener {
            timeout = 900
            on-timeout = hyprctl dispatch dpms off
            on-resume = hyprctl dispatch dpms on
        }
      '';

      xdg.configFile."ironbar/config.json".source = ./ironbar-config.json;

      xdg.configFile."ironbar/style.css".source = ./ironbar-style.css;
    };
  };
}
