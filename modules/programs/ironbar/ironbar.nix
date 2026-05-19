{ ... }:
{
  # ironbar (Home Manager): IronBar ステータスバーの config.json と style.css を配置する。
  flake.modules.homeManager.ironbar = { ... }: {
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
  };
}
