{ inputs, ... }:
{
  # nixos-vm: 仮想マシン上でのテスト用ホスト構成。
  # nixos-base のすべての設定を継承しつつ、VM固有の問題を解決する。
  flake.modules.nixos.nixos-vm = { ... }: {
    imports = with inputs.self.modules.nixos; [
      nixos # system-desktop + programming + storage + admin + jade
    ];

    networking.hostName = "nixos-vm";

    # ハードウェア宣言: GPU なしの VM 構成（ゲーミング・ドライバーは自動無効化）。
    my.hardware.gpu = "none";
    my.hardware.cpu = "amd";

    # VM環境では NetworkManager-wait-online.service がネットワーク待機で
    # 無期限にブロックし、OSが起動しなくなる問題があるため無効化する。
    systemd.services.NetworkManager-wait-online.enable = false;

    # QEMU/KVM ゲストエージェント（ファイル転送・ホスト連携・スナップショット）。
    services.qemuGuest.enable = true;
  };
}
