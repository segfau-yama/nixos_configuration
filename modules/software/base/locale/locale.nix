{ ... }:
{
  # locale: 日本語ロケール・フォント・コンソールキーマップを設定する。
  flake.modules.nixos.locale = { pkgs, ... }: {
    i18n.defaultLocale = "ja_JP.UTF-8";
    i18n.extraLocaleSettings = {
      LC_ADDRESS        = "ja_JP.UTF-8";
      LC_IDENTIFICATION = "ja_JP.UTF-8";
      LC_MEASUREMENT    = "ja_JP.UTF-8";
      LC_MONETARY       = "ja_JP.UTF-8";
      LC_NAME           = "ja_JP.UTF-8";
      LC_NUMERIC        = "ja_JP.UTF-8";
      LC_PAPER          = "ja_JP.UTF-8";
      LC_TELEPHONE      = "ja_JP.UTF-8";
      LC_TIME           = "ja_JP.UTF-8";
    };

    # コンソール用日本語キーボードレイアウト。
    console.keyMap = "jp106";

    fonts.packages = with pkgs; [
      noto-fonts
      noto-fonts-cjk-sans
      noto-fonts-cjk-serif
      noto-fonts-emoji
      source-han-sans
      source-han-serif
    ];
  };
}
