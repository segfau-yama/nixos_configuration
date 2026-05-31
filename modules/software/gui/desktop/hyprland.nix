{ ... }:
{
  # desktopHyprland (NixOS): Hyprland と greetd/tuigreet ログイン。
  flake.modules.nixos.desktopHyprland = { config, pkgs, ... }:
  let
    desktopUser = "suichan";
    hyprlandSessionForDesktopUser = pkgs.writeShellScript "hyprland-session-for-${desktopUser}" ''
      session_user="$(${pkgs.coreutils}/bin/id -un)"
      if [ "$session_user" != "${desktopUser}" ]; then
        printf '%s\n' 'This Hyprland session is configured only for suichan. Use a TTY for admin.'
        ${pkgs.coreutils}/bin/sleep 3
        exit 1
      fi

      exec ${config.programs.hyprland.package}/bin/Hyprland
    '';
  in {
    hardware.graphics.enable = true;

    programs.hyprland = {
      enable = true;
      xwayland.enable = true;
    };

    programs.dconf.enable = true;
    services.dbus.enable = true;
    services.seatd.enable = true;
    security.polkit.enable = true;

    environment.sessionVariables.NIXOS_OZONE_WL = "1";

    services.greetd = {
      enable = true;
      settings.default_session = {
        command = "${pkgs.greetd.tuigreet}/bin/tuigreet --time --cmd ${hyprlandSessionForDesktopUser}";
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

    environment.systemPackages = with pkgs; [
      adwaita-icon-theme
      brightnessctl
      cliphist
      foot
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
  };

  # desktopHyprland (Home Manager): suichan の Hyprland 設定。
  flake.modules.homeManager.desktopHyprland = { pkgs, ... }:
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
    home.packages = with pkgs; [
      adwaita-icon-theme
      brightnessctl
      cliphist
      foot
      hicolor-icon-theme
      hypridle
      hyprpaper
      ironbar
      mako
      papirus-icon-theme
      playerctl
      polkit_gnome
      rofi-wayland
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
          terminal:            "foot";
          font:                "Inter 13";
      }

      @theme "~/.config/rofi/themes/catppuccin-mocha-purple.rasi"
    '';

    xdg.configFile."rofi/themes/catppuccin-mocha-purple.rasi".text = ''
      // ============================================================
      // Catppuccin Mocha Purple — Rofi Theme
      // Designed to match Hyprland + Waybar Catppuccin Mocha setup
      // ============================================================

      * {
          // ── Catppuccin Mocha Palette ──────────────────────────
          crust:       #11111b;
          base:        #1e1e2e;
          surface0:    #313244;
          surface1:    #45475a;
          surface2:    #585b70;
          overlay0:    #6c7086;
          text:        #cdd6f4;
          subtext1:    #bac2de;
          subtext0:    #a6adc8;
          mauve:       #cba6f7;
          blue:        #89b4fa;
          pink:        #f38ba8;
          green:       #a6e3a1;

          // ── Semantic aliases ──────────────────────────────────
          bg:           @crust;
          bg-alt:       @base;
          bg-surface:   @surface0;
          fg:           @text;
          fg-dim:       @subtext1;
          fg-faint:     @overlay0;
          accent:       @mauve;
          urgent-color: @pink;

          // ── Defaults ─────────────────────────────────────────
          background-color: transparent;
          text-color:       @fg;
      }

      // ── Window ───────────────────────────────────────────────────
      window {
          background-color: rgba(17, 17, 27, 0.92);
          border:           1px;
          border-color:     rgba(203, 166, 247, 0.22);
          border-radius:    14px;
          padding:          14px;
          width:            620px;
          location:         center;
          x-offset:         0;
          y-offset:         0;
      }

      // ── Layout ───────────────────────────────────────────────────
      mainbox {
          background-color: transparent;
          children:         [inputbar, listview, mode-switcher];
          spacing:          10px;
          padding:          0;
      }

      // ── Search bar ───────────────────────────────────────────────
      inputbar {
          background-color: rgba(49, 50, 68, 0.60);
          border:           1px;
          border-color:     rgba(203, 166, 247, 0.16);
          border-radius:    10px;
          padding:          10px 16px;
          children:         [prompt, entry];
          spacing:          10px;
      }

      prompt {
          background-color: transparent;
          text-color:       @accent;
          vertical-align:   0.5;
      }

      entry {
          background-color: transparent;
          text-color:       @fg;
          placeholder:      "Search...";
          placeholder-color: @fg-faint;
          vertical-align:   0.5;
          cursor-color:     @accent;
      }

      // ── Results ──────────────────────────────────────────────────
      listview {
          background-color: transparent;
          lines:            8;
          columns:          1;
          spacing:          3px;
          padding:          2px 0;
          scrollbar:        false;
          border:           none;
          cycle:            true;
      }

      element {
          background-color: transparent;
          border-radius:    9px;
          padding:          8px 12px;
          spacing:          12px;
          orientation:      horizontal;
          children:         [element-icon, element-text];
          cursor:           pointer;
      }

      // Normal items
      element.normal.normal {
          background-color: transparent;
          text-color:       @fg-dim;
      }
      element.normal.active {
          background-color: rgba(203, 166, 247, 0.10);
          text-color:       @accent;
      }
      element.normal.urgent {
          background-color: rgba(243, 139, 168, 0.10);
          text-color:       @urgent-color;
      }

      // Selected (highlighted) item
      element.selected.normal {
          background-color: rgba(203, 166, 247, 0.24);
          text-color:       @accent;
      }
      element.selected.active {
          background-color: rgba(203, 166, 247, 0.24);
          text-color:       @accent;
      }
      element.selected.urgent {
          background-color: rgba(243, 139, 168, 0.24);
          text-color:       @urgent-color;
      }

      // Alternating items
      element.alternate.normal {
          background-color: transparent;
          text-color:       @fg-dim;
      }
      element.alternate.active {
          background-color: transparent;
          text-color:       @accent;
      }
      element.alternate.urgent {
          background-color: transparent;
          text-color:       @urgent-color;
      }

      element-icon {
          background-color: transparent;
          size:             24px;
          cursor:           inherit;
      }

      element-text {
          background-color: transparent;
          text-color:       inherit;
          vertical-align:   0.5;
          cursor:           inherit;
      }

      // ── Mode switcher (drun / run / window) ──────────────────────
      mode-switcher {
          background-color: transparent;
          spacing:          8px;
          padding:          2px 0 0 0;
          border:           1px 0px 0px 0px;
          border-color:     rgba(203, 166, 247, 0.10);
      }

      button {
          background-color: rgba(49, 50, 68, 0.45);
          text-color:       @fg-faint;
          border-radius:    8px;
          padding:          4px 16px;
          cursor:           pointer;
      }

      button.selected {
          background-color: rgba(203, 166, 247, 0.24);
          text-color:       @accent;
      }

      // ── Scrollbar ────────────────────────────────────────────────
      scrollbar {
          background-color: rgba(49, 50, 68, 0.40);
          handle-color:     rgba(203, 166, 247, 0.50);
          handle-width:     4px;
          border-radius:    2px;
          width:            6px;
          padding:          2px;
      }

      // ── Message area ─────────────────────────────────────────────
      message {
          background-color: rgba(49, 50, 68, 0.40);
          border:           1px;
          border-color:     rgba(203, 166, 247, 0.12);
          border-radius:    9px;
          padding:          8px 14px;
          margin:           0 0 4px 0;
      }

      textbox {
          background-color: transparent;
          text-color:       @fg-dim;
      }
    '';

    xdg.configFile."hypr/hyprland.conf".text = ''
      monitor=,preferred,auto,1.0

      $terminal = wezterm
      $fileManager = wezterm start -- yazi
      $menu = rofi -show drun

      exec-once = fcitx5 -d
      exec-once = mako
      exec-once = ${pkgs.polkit_gnome}/libexec/polkit-gnome-authentication-agent-1
      exec-once = hyprpaper
      exec-once = hypridle
      exec-once = ${pkgs.ironbar}/bin/ironbar
      exec-once = wlsunset -l 35.7 -L 139.7
      exec-once = wl-paste --watch cliphist store

      env = XCURSOR_SIZE,24
      env = HYPRCURSOR_SIZE,24
      env = XKB_DEFAULT_LAYOUT,us
      env = XMODIFIERS,@im=fcitx5
      env = GTK_IM_MODULE,fcitx5
      env = QT_IM_MODULE,fcitx5
      env = NIXOS_OZONE_WL,1

      general {
          gaps_in = 4
          gaps_out = 8
          border_size = 2
          col.active_border = rgba(cba6f7ee) rgba(64aaffee) 45deg
          col.inactive_border = rgba(45475aaa)
          resize_on_border = false
          allow_tearing = false
          layout = dwindle
      }

      decoration {
          rounding = 10
          rounding_power = 2
          active_opacity = 0.85
          inactive_opacity = 0.55

          shadow {
              enabled = true
              range = 12
              render_power = 3
              color = rgba(11111bee)
          }

          blur {
              enabled = true
              size = 2
              passes = 2
              vibrancy = 0.2
          }
      }

      animations {
          enabled = true

          bezier = easeOutQuint,   0.23, 1,    0.32, 1
          bezier = easeInOutCubic, 0.65, 0.05, 0.36, 1
          bezier = linear,         0,    0,    1,    1
          bezier = almostLinear,   0.5,  0.5,  0.75, 1
          bezier = quick,          0.15, 0,    0.1,  1

          animation = global,        1, 10,   default
          animation = border,        1, 5.39, easeOutQuint
          animation = windows,       1, 4.79, easeOutQuint
          animation = windowsIn,     1, 4.1,  easeOutQuint, popin 87%
          animation = windowsOut,    1, 1.49, linear,       popin 87%
          animation = fadeIn,        1, 1.73, almostLinear
          animation = fadeOut,       1, 1.46, almostLinear
          animation = fade,          1, 3.03, quick
          animation = layers,        1, 3.81, easeOutQuint
          animation = layersIn,      1, 4,    easeOutQuint, fade
          animation = layersOut,     1, 1.5,  linear,       fade
          animation = fadeLayersIn,  1, 1.79, almostLinear
          animation = fadeLayersOut, 1, 1.39, almostLinear
          animation = workspaces,    1, 1.94, almostLinear, fade
          animation = workspacesIn,  1, 1.21, almostLinear, fade
          animation = workspacesOut, 1, 1.94, almostLinear, fade
      }

      dwindle {
          preserve_split = true
      }

      master {
          new_status = master
      }

      misc {
          force_default_wallpaper = -1
          disable_hyprland_logo = false
      }

      input {
          kb_layout = us
          kb_variant =
          kb_model =
          kb_options =
          kb_rules =
          follow_mouse = 1
          sensitivity = 0

          touchpad {
              natural_scroll = false
          }
      }

      device {
          name = epic-mouse-v1
          sensitivity = -0.5
      }

      $mainMod = SUPER

      bind = $mainMod, Q, exec, $terminal
      bind = $mainMod, C, killactive,
      bind = $mainMod, M, exec, command -v hyprshutdown >/dev/null 2>&1 && hyprshutdown || hyprctl dispatch exit
      bind = $mainMod, E, exec, $fileManager
      bind = $mainMod, V, togglefloating,
      bind = $mainMod, F, fullscreen,
      bind = $mainMod, R, exec, $menu
      bind = $mainMod, P, exec, ${screenshotRegion}
      bind = $mainMod, J, layoutmsg, togglesplit
      bind = $mainMod, D, exec, $menu
      bind = SUPER SHIFT, right, movetoworkspace, +1
      bind = SUPER SHIFT, left, movetoworkspace, -1

      bind = , Print, exec, ${screenshotRegion}
      bind = alt, Print, exec, ${screenshotWindow}
      bind = control, Print, exec, ${screenshotOutput}

      bind = $mainMod, left, movefocus, l
      bind = $mainMod, right, movefocus, r
      bind = $mainMod, up, movefocus, u
      bind = $mainMod, down, movefocus, d

      bind = $mainMod, 1, workspace, 1
      bind = $mainMod, 2, workspace, 2
      bind = $mainMod, 3, workspace, 3
      bind = $mainMod, 4, workspace, 4
      bind = $mainMod, 5, workspace, 5
      bind = $mainMod, 6, workspace, 6
      bind = $mainMod, 7, workspace, 7
      bind = $mainMod, 8, workspace, 8
      bind = $mainMod, 9, workspace, 9
      bind = $mainMod, 0, workspace, 10

      bind = $mainMod SHIFT, 1, movetoworkspace, 1
      bind = $mainMod SHIFT, 2, movetoworkspace, 2
      bind = $mainMod SHIFT, 3, movetoworkspace, 3
      bind = $mainMod SHIFT, 4, movetoworkspace, 4
      bind = $mainMod SHIFT, 5, movetoworkspace, 5
      bind = $mainMod SHIFT, 6, movetoworkspace, 6
      bind = $mainMod SHIFT, 7, movetoworkspace, 7
      bind = $mainMod SHIFT, 8, movetoworkspace, 8
      bind = $mainMod SHIFT, 9, movetoworkspace, 9
      bind = $mainMod SHIFT, 0, movetoworkspace, 10

      bind = $mainMod, S, togglespecialworkspace, magic
      bind = $mainMod SHIFT, S, movetoworkspace, special:magic

      bind = $mainMod, mouse_down, workspace, e+1
      bind = $mainMod, mouse_up, workspace, e-1

      bindm = $mainMod, mouse:272, movewindow
      bindm = $mainMod, mouse:273, resizewindow

      bindel = ,XF86AudioRaiseVolume, exec, wpctl set-volume -l 1 @DEFAULT_AUDIO_SINK@ 5%+
      bindel = ,XF86AudioLowerVolume, exec, wpctl set-volume @DEFAULT_AUDIO_SINK@ 5%-
      bindel = ,XF86AudioMute, exec, wpctl set-mute @DEFAULT_AUDIO_SINK@ toggle
      bindel = ,XF86AudioMicMute, exec, wpctl set-mute @DEFAULT_AUDIO_SOURCE@ toggle
      bindel = ,XF86MonBrightnessUp, exec, brightnessctl -e4 -n2 set 5%+
      bindel = ,XF86MonBrightnessDown, exec, brightnessctl -e4 -n2 set 5%-

      bindl = , XF86AudioNext, exec, playerctl next
      bindl = , XF86AudioPause, exec, playerctl play-pause
      bindl = , XF86AudioPlay, exec, playerctl play-pause
      bindl = , XF86AudioPrev, exec, playerctl previous

      xwayland {
          force_zero_scaling = true
          use_nearest_neighbor = true
      }
    '';

    xdg.configFile."hypr/hyprpaper.conf".text = ''
      preload = /home/suichan/Pictures/Wallpapers/cyber.png
      wallpaper = HDMI-A-1,/home/suichan/Pictures/Wallpapers/cyber.png
      splash = false
      ipc = on
    '';

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

    xdg.configFile."ironbar/config.json".text = ''
      {
        "icon_theme": "Papirus-Dark",
        "position": "top",
        "anchor_to_edges": true,
        "height": 36,
        "margin": {
          "top": 4,
          "left": 8,
          "right": 8
        },
        "start": [
          {
            "type": "workspaces",
            "format": "{name}"
          }
        ],
        "center": [
          {
            "type": "focused",
            "show_icon": false
          }
        ],
        "end": [
          {
            "type": "label",
            "name": "launcher",
            "label": "󰣇",
            "on_click_left": "rofi -show drun"
          },
          {
            "type": "volume",
            "format": "{icon}"
          },
          {
            "type": "label",
            "name": "network",
            "label": "{{3000:if nmcli -t -f STATE g 2>/dev/null | grep -q '^connected'; then if nmcli -t -f TYPE,STATE dev 2>/dev/null | grep -q '^ethernet:connected'; then echo '󰈀'; else echo '󰖩'; fi; else echo '󰖪'; fi}}"
          },
          {
            "type": "tray",
            "icon_size": 16,
            "prefer_theme_icons": true
          },
          {
            "type": "label",
            "name": "power",
            "label": "⏻",
            "tooltip": "Left: suspend | Right: reboot | Middle: poweroff",
            "on_click_left": "systemctl suspend",
            "on_click_middle": "systemctl poweroff",
            "on_click_right": "systemctl reboot"
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
        min-width: 0;
      }

      #bar {
        background: transparent;
        border-radius: 14px;
      }

      #bar,
      .background {
        border-radius: 14px;
      }

      .background {
        background: rgba(17, 17, 27, 0.7);
        color: #edf6f4;
        border-bottom: 1px solid rgba(203, 166, 247, 0.15);
      }

      tooltip,
      .popup {
        background: rgba(17, 17, 27, 0.96);
        color: #cdd6f4;
        border: 1px solid rgba(203, 166, 247, 0.28);
        border-radius: 12px;
      }

      tooltip label {
        color: #cdd6f4;
      }

      button {
        min-height: 0;
        min-width: 0;
        padding: 0;
        border: none;
        border-radius: 0;
        background: transparent;
        background-image: none;
        box-shadow: none;
        text-shadow: none;
        color: inherit;
      }

      button:hover,
      button:focus,
      button:checked,
      button:active {
        border: none;
        background-image: none;
        box-shadow: none;
        text-shadow: none;
      }

      #bar #start,
      #bar #center,
      #bar #end {
        margin: 0 6px;
      }

      .widget-container {
        min-height: 0;
      }

      .widget,
      .label,
      .volume,
      .clock,
      .tray {
        margin: 3px 1px;
        padding: 0 9px;
        border-radius: 9px;
        border: none;
        background: transparent;
        background-image: none;
        box-shadow: none;
        transition: background-color 0.2s ease, color 0.2s ease;
      }

      .widget:hover,
      .label:hover,
      .volume:hover,
      .tray .item:hover {
        background: rgba(203, 166, 247, 0.12);
      }

      .workspaces {
        margin: 0;
        padding: 0 2px;
        background: transparent;
        border: none;
      }

      .workspaces .item {
        padding: 0 8px;
        margin: 4px 2px;
        min-height: 24px;
        min-width: 22px;
        border-radius: 9px;
        border: none;
        background: transparent;
        background-image: none;
        box-shadow: none;
        color: #6c7086;
        font-size: 13px;
        font-weight: 700;
      }

      .workspaces .item:hover {
        background: rgba(203, 166, 247, 0.14);
        color: #cdd6f4;
      }

      .workspaces .item.focused,
      .workspaces .item.visible {
        background: rgba(203, 166, 247, 0.28);
        color: #cba6f7;
      }

      .workspaces .item.urgent {
        background: rgba(243, 139, 168, 0.24);
        color: #f38ba8;
      }

      .workspaces .item.inactive {
        color: #45475a;
      }

      .focused {
        color: #bac2de;
        padding: 0 9px;
      }

      .focused .label {
        margin: 0;
        padding: 0;
        background: transparent;
      }

      #launcher {
        color: #cba6f7;
        font-size: 17px;
        padding: 0 11px;
      }

      .volume,
      #network,
      .tray,
      #power {
        color: #cba6f7;
        font-size: 16px;
      }

      .volume.muted {
        color: #585b70;
      }

      .tray {
        padding: 0 7px;
      }

      .tray .item {
        padding: 0 3px;
        margin: 0 1px;
        border-radius: 8px;
        background: transparent;
        background-image: none;
      }

      .tray .item label {
        font-size: 0;
        min-width: 0;
        margin: 0;
        padding: 0;
      }

      .tray .item image {
        min-width: 16px;
        min-height: 16px;
      }

      .tray .item.urgent {
        background: rgba(243, 139, 168, 0.22);
      }

      .clock {
        color: #cdd6f4;
        background: rgba(49, 50, 68, 0.65);
        border-radius: 9px;
        padding: 0 14px;
        margin: 3px 4px 3px 7px;
      }

      .clock:hover {
        background: rgba(69, 71, 90, 0.8);
      }
    '';
  };
}
