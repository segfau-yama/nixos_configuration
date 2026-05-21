{ inputs, ... }:
{
  # nixos: メインデスクトップホストの基本構成。
  # my.hardware.gpu / cpu でドライバー・マイクロコードが自動適用される。
  flake.modules.nixos.nixos = { lib, ... }: {
    imports = with inputs.self.modules.nixos; [
      system-desktop  # system-base + home-manager + locale + fcitx5 + audio + desktop
      programming
      storage         # single or multi-storage support
      admin
      jade
    ] ++ [ "${inputs.self}/nixos/hardware-configuration.nix" ];
    networking.hostName = lib.mkDefault "nixos";

    # ハードウェア宣言: hardware モジュールが自動的にドライバー・マイクロコードを適用する。
    # 継承するホスト（nixos-vm 等）は上書き可能（mkDefault より高優先度で設定する）。
    my.hardware.gpu = lib.mkDefault "nvidia";
    my.hardware.cpu = lib.mkDefault "amd";
  };
}
