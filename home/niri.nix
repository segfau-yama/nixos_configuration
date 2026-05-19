{ ... }:
{
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
      Mod+D { spawn "tofi-drun"; }
      Mod+W { spawn "chromium"; }
      Mod+E { spawn "thunar"; }
      Mod+C { close-window; }
      Mod+F { fullscreen-window; }
      Mod+P { spawn "wayshot" "--region"; }

      XF86AudioRaiseVolume { spawn "wpctl" "set-volume" "-l" "1" "@DEFAULT_AUDIO_SINK@" "5%+"; }
      XF86AudioLowerVolume { spawn "wpctl" "set-volume" "@DEFAULT_AUDIO_SINK@" "5%-"; }
      XF86AudioMute { spawn "wpctl" "set-mute" "@DEFAULT_AUDIO_SINK@" "toggle"; }
      XF86AudioMicMute { spawn "wpctl" "set-mute" "@DEFAULT_AUDIO_SOURCE@" "toggle"; }
    }
  '';
}
