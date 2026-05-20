{ ... }:
{
  # niri/home (Home Manager): niri の config.kdl と tofi アプリランチャーの設定を管理する。
  # flake.modules.homeManager.niri として登録 (Multi Context Aspect — nixos.niri とは別コンテキスト)。
  flake.modules.homeManager.niri = { ... }: {
    # ── App Launcher: tofi (Catppuccin Mocha テーマ) ─────────────────────────────
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

    # ── Niri config.kdl ───────────────────────────────────────────────────────────
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
  };
}
