{ pkgs, ... }:
{
  home.username = "yama";
  home.homeDirectory = "/home/yama";
  home.stateVersion = "25.05";

  programs.home-manager.enable = true;

  # Wayland session variables for Electron/Chromium apps.
  home.sessionVariables = {
    NIXOS_OZONE_WL = "1";
    MOZ_ENABLE_WAYLAND = "1";
    ELECTRON_OZONE_PLATFORM_HINT = "auto";
  };

  home.packages = with pkgs; [
    chromium
    discord
    kicad
    freecad-wayland
  ];

  wayland.windowManager.hyprland = {
    enable = true;
    settings = {
      "$mainMod" = "SUPER";
      "$terminal" = "foot";
      "$fileManager" = "thunar";
      "$menu" = "hyprlauncher";

      monitor = [ ",preferred,auto,1.0" ];
      exec-once = [
        "fcitx5 -d"
        "mako"
        "/usr/lib/xfce-polkit/xfce-polkit"
        "hyprpaper"
        "hypridle"
        "ironbar"
        "wlsunset -l 35.7 -L 139.7"
        "wl-paste --watch cliphist store"
        "setxkbmap jp"
      ];

      env = [
        "XCURSOR_SIZE,24"
        "HYPRCURSOR_SIZE,24"
        "XKB_DEFAULT_LAYOUT,jp"
        "XMODIFIERS,@im=fcitx5"
        "GTK_IM_MODULE,fcitx5"
        "QT_IM_MODULE,fcitx5"
      ];

      input = {
        kb_layout = "jp";
        kb_model = "jp106";
        follow_mouse = 1;
        sensitivity = 0;
        touchpad = {
          natural_scroll = false;
        };
      };

      general = {
        gaps_in = 4;
        gaps_out = 8;
        border_size = 2;
        "col.active_border" = "rgba(cba6f7ee) rgba(64aaffee) 45deg";
        "col.inactive_border" = "rgba(45475aaa)";
        resize_on_border = false;
        allow_tearing = false;
        layout = "dwindle";
      };

      decoration = {
        rounding = 10;
        rounding_power = 2;
        active_opacity = 0.85;
        inactive_opacity = 0.55;
        shadow = {
          enabled = true;
          range = 12;
          render_power = 3;
          color = "rgba(11111bee)";
        };
        blur = {
          enabled = true;
          size = 2;
          passes = 2;
          vibrancy = 0.2;
        };
      };

      animations = {
        enabled = true;
        bezier = [
          "easeOutQuint, 0.23, 1, 0.32, 1"
          "easeInOutCubic, 0.65, 0.05, 0.36, 1"
          "linear, 0, 0, 1, 1"
          "almostLinear, 0.5, 0.5, 0.75, 1"
          "quick, 0.15, 0, 0.1, 1"
        ];
        animation = [
          "global, 1, 10, default"
          "border, 1, 5.39, easeOutQuint"
          "windows, 1, 4.79, easeOutQuint"
          "windowsIn, 1, 4.1, easeOutQuint, popin 87%"
          "windowsOut, 1, 1.49, linear, popin 87%"
          "fadeIn, 1, 1.73, almostLinear"
          "fadeOut, 1, 1.46, almostLinear"
          "fade, 1, 3.03, quick"
          "layers, 1, 3.81, easeOutQuint"
          "layersIn, 1, 4, easeOutQuint, fade"
          "layersOut, 1, 1.5, linear, fade"
          "fadeLayersIn, 1, 1.79, almostLinear"
          "fadeLayersOut, 1, 1.39, almostLinear"
          "workspaces, 1, 1.94, almostLinear, fade"
          "workspacesIn, 1, 1.21, almostLinear, fade"
          "workspacesOut, 1, 1.94, almostLinear, fade"
          "zoomFactor, 1, 7, quick"
        ];
      };

      dwindle = {
        pseudotile = true;
        preserve_split = true;
      };

      master = {
        new_status = "master";
      };

      misc = {
        force_default_wallpaper = -1;
        disable_hyprland_logo = false;
      };

      gesture = [ "3, horizontal, workspace" ];

      device = {
        name = "epic-mouse-v1";
        sensitivity = -0.5;
      };

      bind = [
        "$mainMod, Q, exec, $terminal"
        "$mainMod, C, killactive"
        "$mainMod, M, exec, command -v hyprshutdown >/dev/null 2>&1 && hyprshutdown || hyprctl dispatch exit"
        "$mainMod, E, exec, $fileManager"
        "$mainMod, V, togglefloating"
        "$mainMod, F, fullscreen"
        "$mainMod, R, exec, $menu"
        "$mainMod, P, pseudo"
        "$mainMod, J, layoutmsg, togglesplit"
        "$mainMod, D, exec, wofi --show drun"
        "SUPER SHIFT, right, movetoworkspace, +1"
        "SUPER SHIFT, left, movetoworkspace, -1"
        ", Print, exec, hyprshot -m region"
        "alt, Print, exec, hyprshot -m window"
        "control, Print, exec, hyprshot -m output"
        "$mainMod, left, movefocus, l"
        "$mainMod, right, movefocus, r"
        "$mainMod, up, movefocus, u"
        "$mainMod, down, movefocus, d"
        "$mainMod, 1, workspace, 1"
        "$mainMod, 2, workspace, 2"
        "$mainMod, 3, workspace, 3"
        "$mainMod, 4, workspace, 4"
        "$mainMod, 5, workspace, 5"
        "$mainMod, 6, workspace, 6"
        "$mainMod, 7, workspace, 7"
        "$mainMod, 8, workspace, 8"
        "$mainMod, 9, workspace, 9"
        "$mainMod, 0, workspace, 10"
        "$mainMod SHIFT, 1, movetoworkspace, 1"
        "$mainMod SHIFT, 2, movetoworkspace, 2"
        "$mainMod SHIFT, 3, movetoworkspace, 3"
        "$mainMod SHIFT, 4, movetoworkspace, 4"
        "$mainMod SHIFT, 5, movetoworkspace, 5"
        "$mainMod SHIFT, 6, movetoworkspace, 6"
        "$mainMod SHIFT, 7, movetoworkspace, 7"
        "$mainMod SHIFT, 8, movetoworkspace, 8"
        "$mainMod SHIFT, 9, movetoworkspace, 9"
        "$mainMod SHIFT, 0, movetoworkspace, 10"
        "$mainMod, S, togglespecialworkspace, magic"
        "$mainMod SHIFT, S, movetoworkspace, special:magic"
        "$mainMod, mouse_down, workspace, e+1"
        "$mainMod, mouse_up, workspace, e-1"
      ];

      bindm = [
        "$mainMod, mouse:272, movewindow"
        "$mainMod, mouse:273, resizewindow"
      ];

      bindel = [
        ",XF86AudioRaiseVolume, exec, wpctl set-volume -l 1 @DEFAULT_AUDIO_SINK@ 5%+"
        ",XF86AudioLowerVolume, exec, wpctl set-volume @DEFAULT_AUDIO_SINK@ 5%-"
        ",XF86AudioMute, exec, wpctl set-mute @DEFAULT_AUDIO_SINK@ toggle"
        ",XF86AudioMicMute, exec, wpctl set-mute @DEFAULT_AUDIO_SOURCE@ toggle"
        ",XF86MonBrightnessUp, exec, brightnessctl -e4 -n2 set 5%+"
        ",XF86MonBrightnessDown, exec, brightnessctl -e4 -n2 set 5%-"
      ];

      bindl = [
        ", XF86AudioNext, exec, playerctl next"
        ", XF86AudioPause, exec, playerctl play-pause"
        ", XF86AudioPlay, exec, playerctl play-pause"
        ", XF86AudioPrev, exec, playerctl previous"
      ];

      windowrule = [
        "suppressevent maximize, class:.*"
        "nofocus, class:^$, title:^$, xwayland:1, floating:1, fullscreen:0, pinned:0"
        "move 20 monitor_h-120, class:hyprland-run"
        "float, class:hyprland-run"
      ];

      xwayland = {
        force_zero_scaling = true;
        use_nearest_neighbor = true;
      };
    };
  };

  services.mako = {
    enable = true;
    defaultTimeout = 5000;
  };

  services.hyprpaper = {
    enable = true;
    settings = {
      splash = false;
      ipc = true;
      preload = [ "/usr/share/backgrounds/cyber.png" ];
      wallpaper = [ "eDP-1,/usr/share/backgrounds/cyber.png" ];
    };
  };

  # Manage Ironbar configuration declaratively with Home Manager.
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
                      "label": "",
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
}