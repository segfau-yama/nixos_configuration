{ inputs, ... }:
{
  # gaming (NixOS): Steam + Gamemode を設定する。
  # hardware.graphics は opengl (DRY Aspect) から継承する。
  # allowUnfree は system-base の nixpkgs.config.allowUnfree で許可済み。
  flake.modules.nixos.gaming = { ... }: {
    imports = [ inputs.self.modules.nixos.opengl ];  # Inheritance Aspect

    programs.gamemode.enable = true;

    programs.steam = {
      enable = true;
      remotePlay.openFirewall = true;
      dedicatedServer.openFirewall = false;
    };
  };

  # gaming (Home Manager): Lutris + Wine + Winetricks を提供する。
  flake.modules.homeManager.gaming = { pkgs, ... }: {
    home.packages = with pkgs; [
      lutris
      wineWowPackages.stable  # 32/64bit Wine を同梱
      winetricks
    ];
  };
}
