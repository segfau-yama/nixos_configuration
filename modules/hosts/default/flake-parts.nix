{ inputs, ... }:
{
  # nixos-default: nixosConfigurations 出力を生成する。
  flake.nixosConfigurations = inputs.self.lib.mkNixos "x86_64-linux" "nixos-default";
}
