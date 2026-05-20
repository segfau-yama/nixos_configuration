{ inputs, ... }:
{
  # NVIDIA GPU 構成
  flake.modules.nixos.nvidia-gpu = { ... }: {
    imports = with inputs.self.modules.nixos; [
      nvidia
      gaming
    ];
  };
}
