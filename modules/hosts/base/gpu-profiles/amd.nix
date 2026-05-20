{ inputs, ... }:
{
  # AMD GPU 構成
  flake.modules.nixos.amd-gpu = { ... }: {
    imports = with inputs.self.modules.nixos; [
      amd
      gaming
    ];
  };
}
