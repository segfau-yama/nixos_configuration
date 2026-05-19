{ ... }:
{
  # niri/tofi (NixOS): Catppuccin テーマの Wayland 専用アプリランチャー。
  # system.nix と同じ flake.modules.nixos.niri に追記する (Collector Aspect)。
  flake.modules.nixos.niri = { ... }: {
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
  };
}
