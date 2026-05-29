{ inputs, ... }:
{
  imports = [
    ./niri.nix
    ./plasma.nix
  ];

  # desktop: niri と Plasma を組み合わせた GUI デスクトップ環境。
  flake.modules.nixos.desktop = { ... }: {
    imports = with inputs.self.modules.nixos; [
      desktopNiri
      desktopPlasma
    ];
  };

  flake.modules.homeManager.desktop = { ... }: {
    imports = with inputs.self.modules.homeManager; [
      desktopNiri
    ];
  };
}
