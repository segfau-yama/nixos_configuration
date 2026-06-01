{ inputs, ... }:
{
  # base: 全ホスト共通の基盤設定を1ファイルへ集約。
  flake.modules.nixos.base = { pkgs, ... }: {
    imports = with inputs.self.modules.nixos; [
      hardware
    ];

    nixpkgs.overlays = [
      (final: _prev: {
        unstable = import inputs.nixpkgs-unstable {
          inherit (final.stdenv.hostPlatform) system;
          config.allowUnfree = true;
        };
      })
    ];

    nixpkgs.config.allowUnfree = true;
    system.stateVersion = "25.05";

    nix.settings = {
      experimental-features = [ "nix-command" "flakes" ];
      auto-optimise-store = true;
      trusted-users = [ "root" "@wheel" ];
    };

    programs.nix-ld.enable = true;

    system.nixos.label = "tracked";

    nix.gc = {
      automatic = true;
      dates = "weekly";
      options = "--delete-older-than 14d";
    };
    nix.optimise.automatic = true;

    environment.systemPackages = with pkgs; [
      git
    ];

    networking.networkmanager.enable = true;
    services.resolved = {
      enable = true;
      dnssec = "allow-downgrade";
    };
    networking.firewall = {
      enable = true;
      allowedTCPPorts = [ ];
      allowedUDPPorts = [ ];
    };

    time.timeZone = "Asia/Tokyo";

    i18n.defaultLocale = "ja_JP.UTF-8";
    i18n.extraLocaleSettings = {
      LC_ADDRESS = "ja_JP.UTF-8";
      LC_IDENTIFICATION = "ja_JP.UTF-8";
      LC_MEASUREMENT = "ja_JP.UTF-8";
      LC_MONETARY = "ja_JP.UTF-8";
      LC_NAME = "ja_JP.UTF-8";
      LC_NUMERIC = "ja_JP.UTF-8";
      LC_PAPER = "ja_JP.UTF-8";
      LC_TELEPHONE = "ja_JP.UTF-8";
      LC_TIME = "ja_JP.UTF-8";
    };

    fonts.packages = with pkgs; [
      font-awesome
      inter
      nerd-fonts.symbols-only
      noto-fonts
      noto-fonts-cjk-sans
      noto-fonts-cjk-serif
      noto-fonts-emoji
      source-han-sans
      source-han-serif
    ];

    i18n.inputMethod = {
      enable = true;
      type = "fcitx5";
      fcitx5.addons = with pkgs; [
        fcitx5-mozc
        fcitx5-gtk
        fcitx5-configtool
      ];
    };

    environment.sessionVariables = {
      XMODIFIERS = "@im=fcitx";
      INPUT_METHOD = "fcitx";
    };

    security.rtkit.enable = true;
    services.pipewire = {
      enable = true;
      alsa.enable = true;
      alsa.support32Bit = true;
      pulse.enable = true;
      jack.enable = true;
      wireplumber.enable = true;
    };
  };

  # base (Home Manager): 全ユーザー共通の基本ユーザー環境。
  flake.modules.homeManager.base = { ... }: {
    programs.zsh = {
      enable                    = true;
      autosuggestion.enable     = true;  # 履歴ベースの補完候補をグレーで表示
      syntaxHighlighting.enable = true;  # コマンド入力時のシンタックスハイライト
      history = {
        size  = 10000;
        share = true;  # 複数端末で履歴を共有
      };
    };
  };
}
