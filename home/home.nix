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
      "$mod" = "SUPER";
      monitor = [ ",preferred,auto,1" ];
      exec-once = [
        "ironbar"
        "mako"
        "hyprpaper"
        "fcitx5"
      ];
      input = {
        kb_layout = "jp";
        follow_mouse = 1;
        sensitivity = 0;
      };
      general = {
        gaps_in = 4;
        gaps_out = 8;
        border_size = 2;
        layout = "dwindle";
      };
      decoration = {
        rounding = 8;
      };
      bind = [
        "$mod, RETURN, exec, wezterm"
        "$mod, B, exec, chromium"
        "$mod, Q, killactive"
        "$mod, M, exit"
        "$mod, V, togglefloating"
        "$mod, F, fullscreen"
      ];
    };
  };

  services.mako = {
    enable = true;
    defaultTimeout = 5000;
  };

  services.hyprpaper = {
    enable = true;
    settings = {
      preload = [ "/home/yama/.config/wallpapers/default.jpg" ];
      wallpaper = [ ",/home/yama/.config/wallpapers/default.jpg" ];
    };
  };

  # Manage Ironbar configuration declaratively with Home Manager.
  xdg.configFile."ironbar/config.json".text = ''
    {
      "position": "top",
      "height": 32,
      "start": [
        {
          "type": "workspaces"
        }
      ],
      "center": [
        {
          "type": "focused"
        }
      ],
      "end": [
        {
          "type": "volume"
        },
        {
          "type": "clock",
          "format": "%Y-%m-%d %H:%M"
        }
      ]
    }
  '';

  xdg.configFile."ironbar/style.css".text = ''
    * {
      font-family: "Noto Sans CJK JP", sans-serif;
      font-size: 13px;
    }

    .background {
      background: rgba(20, 24, 28, 0.88);
      border-bottom: 1px solid rgba(255, 255, 255, 0.12);
    }

    .module {
      margin: 0 8px;
      padding: 0 8px;
    }
  '';
}