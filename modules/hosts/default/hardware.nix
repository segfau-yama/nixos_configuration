{ ... }:
{
  # nixos-default: ハードウェア設定を nixos-default モジュールに追加する。
  # 実機のハードウェア構成に合わせて変更すること。
  flake.modules.nixos.nixos-default = { ... }: {
    imports = [ ../../../nixos/hardware-configuration.nix ];
  };
}
