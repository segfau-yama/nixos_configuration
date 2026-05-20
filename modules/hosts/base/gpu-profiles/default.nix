{ inputs, ... }:
{
  # GPU なし構成
  flake.modules.nixos.default-gpu = { ... }: {
    imports = with inputs.self.modules.nixos; [
      # GPU 固有の設定は不要
    ];
  };
}
