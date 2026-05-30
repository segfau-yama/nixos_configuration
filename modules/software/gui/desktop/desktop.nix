{ inputs, ... }:
{
  imports = [
    ./hyprland.nix
  ];

  # desktop: Hyprland Wayland デスクトップ環境。
  flake.modules.nixos.desktop = { ... }: {
    imports = with inputs.self.modules.nixos; [
      greeter
      desktopHyprland
    ];
  };

  flake.modules.homeManager.desktop = { ... }: {
    imports = with inputs.self.modules.homeManager; [
      desktopHyprland
    ];
  };
}
