{ ... }:
{
  # desktop (NixOS): Niri Wayland コンポジターに必要な全 NixOS 設定を一箇所に集約する。
  #
  # 含まれる設定:
  #   - Niri / dconf / dbus / polkit / seatd の有効化
  #   - greetd + tuigreet によるログインマネージャー
  #   - xdg-desktop-portal (gtk + gnome バックエンド)
  #   - Wayland 基本ツール + ironbar (シェル UI)
  # ユーザーアプリ (wezterm / spacedrive 等) は homeManager.jade で管理する。
  flake.modules.nixos.desktop = { pkgs, ... }: {

    # ── Compositor & System Services ────────────────────────────────────────
    hardware.graphics.enable = true;

    programs.niri.enable   = true;
    programs.dconf.enable  = true;
    services.dbus.enable   = true;
    security.polkit.enable = true;
    services.seatd.enable  = true;

    # ── Login Manager: greetd + tuigreet ────────────────────────────────────
    # tuigreet は TUI 上でユーザー選択し、dbus-run-session 経由で niri を起動する。
    services.greetd = {
      enable = true;
      settings.default_session = {
        command = "${pkgs.greetd.tuigreet}/bin/tuigreet --time --cmd 'dbus-run-session niri --session'";
        user    = "greeter";
      };
    };

    # ── XDG Desktop Portal ───────────────────────────────────────────────────
    # ポータルバックエンドを2つ用意する:
    #   gtk   : ファイル選択・URI ハンドリング・その他汎用インターフェイス
    #   gnome : スクリーンキャスト・スクリーンショット・リモートデスクトップ
    #           (PipeWire + zwlr_screencopy_v1 経由。Discord/OBS の画面共有に必要)
    xdg.portal = {
      enable           = true;
      xdgOpenUsePortal = true;
      extraPortals = with pkgs; [
        xdg-desktop-portal-gtk    # ファイル選択・URI ハンドリング
        xdg-desktop-portal-gnome  # スクリーンキャスト・スクリーンショット
      ];
      config.common = {
        default                                = [ "gtk" ];
        "org.freedesktop.portal.ScreenCast"    = [ "gnome" ];
        "org.freedesktop.portal.Screenshot"    = [ "gnome" ];
        "org.freedesktop.portal.RemoteDesktop" = [ "gnome" ];
      };
    };

    # ── System Packages ──────────────────────────────────────────────────────
    # niri 本体は programs.niri.enable で自動インストール済み。
    # tofi は homeManager.desktop の programs.tofi.enable で管理済み。
    # ユーザーアプリ (wezterm / spacedrive / pwvucontrol 等) は homeManager.jade に移動済み。
    environment.systemPackages = with pkgs; [
      wl-clipboard  # Wayland クリップボード
      wayshot       # スクリーンショット
      wlsunset      # ブルーライトカット (夜間モード)
      ironbar       # IronBar ステータスバー (greetd セッション前から起動するため system 側)
    ];
  };

  # desktop (Home Manager): Niri の設定・IronBar・通知デーモン・壁紙を一括管理する。
  flake.modules.homeManager.desktop = { pkgs, ... }: {

    # Niri セッション復旧・操作用の最小 GUI ツール。
    # カスタム GUI ユーザーでも Mod+Return で必ず端末を開けるようにする。
    home.packages = with pkgs; [
      swww
      wezterm
    ];

    # ── App Launcher: tofi (Catppuccin Mocha テーマ) ─────────────────────────
    # programs.tofi は NixOS モジュールにはないため HM 側で管理する。
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

    # ── Niri config.kdl ──────────────────────────────────────────────────────
    xdg.configFile."niri/config.kdl".text = ''
      input {
        keyboard {
          xkb {
            layout "jp"
            model "jp106"
          }
        }
      }

      layout {
        gaps 8
        center-focused-column "never"
      }

      spawn-at-startup "fcitx5" "-d"
      spawn-at-startup "ironbar"
      spawn-at-startup "wlsunset" "-l" "35.7" "-L" "139.7"

      binds {
        Mod+Return { spawn "wezterm"; }
        Mod+D      { spawn "tofi-drun"; }
        Mod+W      { spawn "chromium"; }
        Mod+E      { spawn "spacedrive"; }
        Mod+C      { close-window; }
        Mod+F      { fullscreen-window; }
        Mod+P      { spawn "wayshot" "--region"; }

        XF86AudioRaiseVolume { spawn "wpctl" "set-volume" "-l" "1" "@DEFAULT_AUDIO_SINK@" "5%+"; }
        XF86AudioLowerVolume { spawn "wpctl" "set-volume" "@DEFAULT_AUDIO_SINK@" "5%-"; }
        XF86AudioMute        { spawn "wpctl" "set-mute" "@DEFAULT_AUDIO_SINK@" "toggle"; }
        XF86AudioMicMute     { spawn "wpctl" "set-mute" "@DEFAULT_AUDIO_SOURCE@" "toggle"; }
      }
    '';

    # ── IronBar config.json ──────────────────────────────────────────────────
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

    # ── IronBar style.css ────────────────────────────────────────────────────
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

    # ── 通知デーモン: mako (Catppuccin Mocha テーマ) ─────────────────────────
    services.mako = {
      enable = true;

      # HM 25.05 以降、オプションは services.mako.settings.* 以下に移動された。
      # キー名は camelCase → kebab-case に変更されている。
      settings = {
        default-timeout  = 5000;      # 通知を 5 秒後に自動で消す

        # Catppuccin Mocha ベースのカラーパレット
        background-color = "#11111bCC";  # 80% 不透明の暗いネイビー
        border-color     = "#cba6f747";  # パープルアクセント (28% 不透明)
        text-color       = "#cdd6f4";    # メインテキスト
        border-radius    = 12;
        border-size      = 1;

        font        = "Inter 13";
        padding     = "12 16";
        width       = 380;
        anchor      = "top-right";
        margin      = "10,10,0,0";  # 上・右に 10px マージン
        layer       = "overlay";    # 最前面に表示
        max-visible = 5;
      };
    };

    # ── 壁紙マネージャー: swww ───────────────────────────────────────────────
    # HM に services.swww が未実装のため systemd ユーザーサービスで管理する。
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
        Type           = "oneshot";
        ExecStartPre   = "${pkgs.coreutils}/bin/sleep 1";
        # swww clear <COLOR> で単色背景を表示する (Catppuccin Mocha Base: #1e1e2e)。
        # 独自の壁紙ファイルを使う場合はこの行を以下のように変更すること:
        #   ExecStart = "${pkgs.swww}/bin/swww img /path/to/wallpaper.png --transition-type fade";
        ExecStart      = "${pkgs.swww}/bin/swww clear 1e1e2e";
        RemainAfterExit = true;
      };
      Install.WantedBy = [ "graphical-session.target" ];
    };
  };
}
