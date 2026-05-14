{ ... }:
{
  programs.tofi = {
    enable = true;
    settings = {
      # Window sizing and positioning
      width = "44%";
      height = 300;
      anchor = "center";

      # Borders and corners
      corner-radius = 14;
      border-width = 1;
      outline-width = 0;
      padding-x = 0;
      padding-y = 0;

      # Colors - Catppuccin theme
      background-color = "#11111bcc";
      text-color = "#cdd6f4";
      input-color = "#cdd6f4";
      selection-color = "#cba6f7";
      selection-background-color = "#cba6f73d";

      # Borders
      border-color = "#cba6f74d";
      outline-color = "#cba6f74d";

      # Font
      font = "Inter";
      font-size = 12;

      # UI behavior
      result-format = "{name}";
      prompt-text = "apps ";
      prompt-color = "#cba6f7";
      prompt-background = "#1e1e2ecc";
      prompt-padding = 10;
      prompt-font-size = 12;

      # Input area styling
      input-background-color = "#1e1e2ecc";
      input-background-padding = 10;
      input-padding = 0;

      # List styling
      list-items-per-column = 9;
      list-max-display-height = 300;

      # Horizontal layout (stacked vertically)
      horizontal = false;

      # Matching
      fuzzy-match = true;
      require-match = false;

      # Sorting
      sort = true;
      sort-by = "frecency";

      # History
      history-file = "~/.cache/tofi/history";
    };
  };

  # Alternative: Using xdg.configFile for more advanced customization
  xdg.configFile."tofi/config".text = ''
    # Tofi Configuration (Catppuccin theme)
    width = 44%
    height = 300
    anchor = center
    
    # Borders and styling
    corner-radius = 14
    border-width = 1
    border-color = #cba6f74d
    padding-x = 0
    padding-y = 0
    
    # Colors - Catppuccin
    background-color = #11111bcc
    text-color = #cdd6f4
    input-color = #cdd6f4
    selection-color = #cba6f7
    selection-background-color = #cba6f73d
    
    # Prompt styling
    prompt-text = "apps "
    prompt-color = #cba6f7
    prompt-background-color = #1e1e2ecc
    prompt-padding = 10
    prompt-font-size = 12
    
    # Input area
    input-background-color = #1e1e2ecc
    input-padding = 10
    
    # List
    list-items-per-column = 9
    list-max-display-height = 300
    
    # UI
    horizontal = false
    fuzzy-match = true
    require-match = false
    sort = true
    sort-by = frecency
    
    # Font
    font = Inter
    font-size = 12
    
    # History
    history-file = ~/.cache/tofi/history
  '';
}
