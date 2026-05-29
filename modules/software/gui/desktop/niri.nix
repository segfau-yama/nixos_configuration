{ ... }:
{
  # desktopNiri (NixOS): niri Wayland コンポジターと niri 用の基本サービス。
  flake.modules.nixos.desktopNiri = { lib, options, pkgs, ... }:
  let
    niriUserServiceOptions =
      options.systemd.user.services.type.getSubOptions [ "niri" ];
  in {
    hardware.graphics.enable = true;

    programs.niri.enable = true;
    programs.dconf.enable = true;

    services.dbus.enable  = true;
    services.seatd.enable = true;
    services.gnome.gnome-keyring.enable = true;

    security.polkit.enable = true;
    security.pam.services.swaylock = {};

    environment.sessionVariables.NIXOS_OZONE_WL = "1";

    systemd.user.services.niri = lib.optionalAttrs
      (niriUserServiceOptions ? enableDefaultPath)
      {
        # NixOS Wiki recommends this for niri-session so the niri user service
        # inherits the PATH imported by niri-session instead of a stripped one.
        enableDefaultPath = false;
      };

    xdg.portal = {
      enable = true;
      xdgOpenUsePortal = true;
      extraPortals = with pkgs; [
        xdg-desktop-portal-gnome
        xdg-desktop-portal-gtk
      ];
      config.niri = {
        default = [ "gtk" ];
        "org.freedesktop.impl.portal.FileChooser" = [ "gtk" ];
        "org.freedesktop.portal.ScreenCast" = [ "gnome" ];
        "org.freedesktop.portal.Screenshot" = [ "gnome" ];
        "org.freedesktop.portal.RemoteDesktop" = [ "gnome" ];
      };
    };

    environment.systemPackages = with pkgs; [
      wl-clipboard
      wayshot
      wlsunset
      ironbar
      wezterm
      tofi
      swayidle
      swaylock
      xwayland-satellite
    ];
  };

  # desktopNiri (Home Manager): niri の設定・IronBar・通知・壁紙。
  flake.modules.homeManager.desktopNiri = { pkgs, ... }: {
    home.packages = with pkgs; [
      swww
      wezterm
    ];

    programs.tofi = {
      enable   = true;
      settings = {
        width                      = "44%";
        height                     = 300;
        anchor                     = "center";
        corner-radius              = 14;
        border-width               = 1;
        outline-width              = 0;
        padding-x                  = 0;
        padding-y                  = 0;
        background-color           = "#11111bcc";
        text-color                 = "#cdd6f4";
        input-color                = "#cdd6f4";
        selection-color            = "#cba6f7";
        selection-background-color = "#cba6f73d";
        border-color               = "#cba6f74d";
        outline-color              = "#cba6f74d";
        font                       = "Inter";
        font-size                  = 12;
        result-format              = "{name}";
        prompt-text                = "apps ";
        prompt-color               = "#cba6f7";
        prompt-background          = "#1e1e2ecc";
        prompt-padding             = 10;
        prompt-font-size           = 12;
        input-background-color     = "#1e1e2ecc";
        input-background-padding   = 10;
        input-padding              = 0;
        list-items-per-column      = 9;
        list-max-display-height    = 300;
        horizontal                 = false;
        fuzzy-match                = true;
        require-match              = false;
        sort                       = true;
        sort-by                    = "frecency";
        history-file               = "~/.cache/tofi/history";
      };
    };

    xdg.configFile."niri/config.kdl".text = ''
      input {
        keyboard {
          xkb {
            layout "us"
          }
        }
      }

      layout {
        gaps 8
        center-focused-column "never"
      }

      environment {
        NIXOS_OZONE_WL "1"
      }

      spawn-at-startup "fcitx5" "-d"
      spawn-at-startup "ironbar"
      spawn-at-startup "wlsunset" "-l" "35.7" "-L" "139.7"

      binds {
        Mod+Return { spawn "wezterm"; }
        Mod+D { spawn "tofi-drun"; }
        Mod+W { spawn "chromium"; }
        Mod+E { spawn "spacedrive"; }
        Mod+C { close-window; }
        Mod+F { fullscreen-window; }
        Mod+L { spawn "swaylock" "-f" "-c" "1e1e2e"; }
        Mod+P { spawn "wayshot" "--region"; }

        XF86AudioRaiseVolume { spawn "wpctl" "set-volume" "-l" "1" "@DEFAULT_AUDIO_SINK@" "5%+"; }
        XF86AudioLowerVolume { spawn "wpctl" "set-volume" "@DEFAULT_AUDIO_SINK@" "5%-"; }
        XF86AudioMute { spawn "wpctl" "set-mute" "@DEFAULT_AUDIO_SINK@" "toggle"; }
        XF86AudioMicMute { spawn "wpctl" "set-mute" "@DEFAULT_AUDIO_SOURCE@" "toggle"; }
      }
    '';

    xdg.configFile."ironbar/config.json".text = ''
      {
        "position": "top",
        "height": 42,
        "margin": {
          "top": 6,
          "left": 8,
          "right": 8
        },
        "start": [
          {
            "type": "workspaces"
          },
          {
            "type": "launcher",
            "favorites": [],
            "icon_size": 20
          }
        ],
        "center": [
          {
            "type": "focused"
          }
        ],
        "end": [
          {
            "type": "volume",
            "format": "{icon}"
          },
          {
            "type": "label",
            "label": "{{3000:if nmcli -t -f STATE g 2>/dev/null | grep -q '^connected'; then echo '󰖩'; else echo '󰖪'; fi}}",
            "on_click": "nm-connection-editor"
          },
          {
            "type": "battery",
            "show_if": "ls /sys/class/power_supply/ | grep --quiet '^BAT'"
          },
          {
            "type": "tray"
          },
          {
            "type": "custom",
            "class": "power-menu",
            "bar": [
              {
                "type": "button",
                "name": "power-btn",
                "label": "⏻",
                "on_click": "popup:toggle"
              }
            ],
            "popup": [
              {
                "type": "box",
                "orientation": "vertical",
                "widgets": [
                  {
                    "type": "label",
                    "name": "header",
                    "label": "Power"
                  },
                  {
                    "type": "box",
                    "widgets": [
                      {
                        "type": "button",
                        "class": "power-action",
                        "label": "󰤄",
                        "on_click": "!systemctl suspend"
                      },
                      {
                        "type": "button",
                        "class": "power-action",
                        "label": "󰜉",
                        "on_click": "!systemctl reboot"
                      },
                      {
                        "type": "button",
                        "class": "power-action",
                        "label": "",
                        "on_click": "!systemctl poweroff"
                      }
                    ]
                  }
                ]
              }
            ]
          },
          {
            "type": "clock",
            "format": "%Y-%m-%d (%a) %H:%M"
          }
        ]
      }
    '';

    xdg.configFile."ironbar/style.css".text = ''
      * {
        font-family: "Inter", "Symbols Nerd Font", "Font Awesome 6 Free", "Noto Sans CJK JP", sans-serif;
        font-size: 13px;
        font-weight: 600;
        min-height: 0;
      }

      .background {
        background: rgba(17, 17, 27, 0.7);
        color: #edf6f4;
        border-bottom: 1px solid rgba(203, 166, 247, 0.15);
        border-radius: 14px;
      }

      .module {
        margin: 4px 2px;
        padding: 0 10px;
        border-radius: 9px;
        background: transparent;
        transition: background-color 0.2s ease;
      }

      .module:hover {
        background: rgba(203, 166, 247, 0.12);
      }

      #workspaces {
        margin: 0;
        padding: 0 2px;
      }

      #focused {
        color: #bac2de;
        padding: 0 10px;
      }

      #launcher {
        color: #cba6f7;
        font-size: 18px;
        padding: 0 12px;
      }

      #volume,
      #label,
      #battery,
      #tray,
      #power-btn {
        color: #cba6f7;
        font-size: 17px;
      }

      #clock {
        color: #cdd6f4;
        background: rgba(49, 50, 68, 0.65);
        border-radius: 9px;
        padding: 0 16px;
        margin: 4px 4px 4px 8px;
      }

      .popup {
        background: rgba(17, 17, 27, 0.96);
        border: 1px solid rgba(203, 166, 247, 0.28);
        border-radius: 12px;
        padding: 10px;
      }

      .power-menu .header {
        color: #cdd6f4;
        margin-bottom: 8px;
      }

      .power-menu .power-action {
        margin: 0 4px;
        padding: 6px 10px;
        border-radius: 8px;
        color: #cba6f7;
        background: rgba(203, 166, 247, 0.12);
      }
    '';

    services.mako = {
      enable = true;
      settings = {
        default-timeout  = 5000;
        background-color = "#11111bCC";
        border-color     = "#cba6f747";
        text-color       = "#cdd6f4";
        border-radius    = 12;
        border-size      = 1;
        font             = "Inter 13";
        padding          = "12 16";
        width            = 380;
        anchor           = "top-right";
        margin           = "10,10,0,0";
        layer            = "overlay";
        max-visible      = 5;
      };
    };

    systemd.user.services.swww-daemon = {
      Unit = {
        Description = "swww wallpaper daemon";
        After       = [ "graphical-session.target" ];
        PartOf      = [ "graphical-session.target" ];
      };
      Service = {
        ExecStart = "${pkgs.swww}/bin/swww-daemon";
        Restart   = "on-failure";
      };
      Install.WantedBy = [ "graphical-session.target" ];
    };

    systemd.user.services.swww-init = {
      Unit = {
        Description = "Set initial wallpaper via swww";
        After       = [ "swww-daemon.service" ];
        Requires    = [ "swww-daemon.service" ];
        PartOf      = [ "graphical-session.target" ];
      };
      Service = {
        Type            = "oneshot";
        ExecStartPre    = "${pkgs.coreutils}/bin/sleep 1";
        ExecStart       = "${pkgs.swww}/bin/swww clear 1e1e2e";
        RemainAfterExit = true;
      };
      Install.WantedBy = [ "graphical-session.target" ];
    };
  };
}
