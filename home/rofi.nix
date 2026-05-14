{ ... }:
{
  xdg.configFile."rofi/config.rasi".text = ''
    configuration {
      modi: "drun,run,window";
      show-icons: true;
      icon-theme: "Papirus-Dark";
      drun-display-format: "{name}";
      disable-history: false;
      hover-select: true;
      me-select-entry: "MousePrimary";
      me-accept-entry: "MouseDPrimary";
      kb-cancel: "Escape,Control+g";
      kb-accept-entry: "Return,KP_Enter";
      kb-row-up: "Up,Control+k";
      kb-row-down: "Down,Control+j";
      kb-mode-next: "Control+Tab";
      kb-mode-previous: "Control+Shift+Tab";
      font: "Inter 12";
    }

    @theme "launcher.rasi"
  '';

  xdg.configFile."rofi/launcher.rasi".text = ''
    * {
      bg: #11111bcc;
      bg-alt: #1e1e2ecc;
      fg: #cdd6f4;
      fg-dim: #7f849c;
      accent: #cba6f7;
      accent-soft: #cba6f733;
      danger: #f38ba8;
      good: #a6e3a1;

      border-color: #cba6f74d;
      radius: 14px;
      border: 1px;
      spacing: 8px;
    }

    window {
      transparency: "real";
      background-color: @bg;
      border: @border;
      border-color: @border-color;
      border-radius: @radius;
      location: center;
      width: 44%;
      max-width: 860px;
      min-width: 560px;
      padding: 16px;
    }

    mainbox {
      background-color: transparent;
      spacing: 10px;
      children: [ inputbar, mode-switcher, listview ];
    }

    inputbar {
      background-color: @bg-alt;
      border: @border;
      border-color: transparent;
      border-radius: 12px;
      padding: 10px 12px;
      spacing: 10px;
      children: [ prompt, entry, case-indicator ];
    }

    prompt {
      text-color: @accent;
      enabled: true;
      str: "apps";
      padding: 0 2px 0 0;
    }

    entry {
      text-color: @fg;
      placeholder: "Search applications";
      placeholder-color: @fg-dim;
    }

    case-indicator {
      text-color: @fg-dim;
    }

    mode-switcher {
      spacing: 8px;
      background-color: transparent;
    }

    button {
      background-color: @bg-alt;
      text-color: @fg-dim;
      border-radius: 10px;
      border: @border;
      border-color: transparent;
      padding: 6px 12px;
    }

    button selected {
      background-color: @accent-soft;
      text-color: @accent;
      border-color: @border-color;
    }

    listview {
      lines: 9;
      columns: 1;
      fixed-height: true;
      dynamic: false;
      cycle: true;
      scrollbar: true;
      background-color: transparent;
      spacing: 6px;
      margin: 2px 0 0 0;
    }

    scrollbar {
      width: 6px;
      border-radius: 999px;
      handle-width: 6px;
      handle-color: @accent-soft;
      background-color: #00000000;
    }

    element {
      background-color: transparent;
      text-color: @fg;
      border: @border;
      border-color: transparent;
      border-radius: 11px;
      padding: 8px 10px;
      spacing: 10px;
    }

    element normal.urgent,
    element selected.urgent {
      text-color: @danger;
    }

    element selected {
      background-color: @accent-soft;
      text-color: @accent;
      border-color: @border-color;
    }

    element-icon {
      size: 1.15em;
      vertical-align: 0.5;
      background-color: transparent;
    }

    element-text {
      background-color: transparent;
      text-color: inherit;
      vertical-align: 0.5;
    }

    message {
      background-color: @bg-alt;
      border-radius: 10px;
      border: @border;
      border-color: transparent;
      padding: 8px 10px;
    }

    textbox {
      text-color: @fg-dim;
    }

    error-message {
      background-color: #f38ba822;
      border: @border;
      border-color: #f38ba855;
      border-radius: 10px;
      padding: 10px;
      text-color: @danger;
    }

    @media (max-width: 1024px) {
      window {
        width: 86%;
        min-width: 0;
        max-width: 0;
      }

      listview {
        lines: 8;
      }
    }
  '';
}
