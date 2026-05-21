{ inputs, ... }:
{
  # nixos-vm を nixosConfigurations に登録する。
  # 使い方: sudo nixos-rebuild switch --flake .#nixos-vm
  flake.nixosConfigurations = inputs.self.lib.mkNixos "x86_64-linux" "nixos-vm";
}
