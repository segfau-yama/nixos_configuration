{ inputs, ... }:
{
  # system-desktop (NixOS): 日本語デスクトップ環境の共通基盤 (Inheritance Aspect)。
  # すべてのデスクトップホストが共通して必要とするフィーチャーを一箇所にまとめる。
  # GPU 固有 (nvidia / amd) やオプション機能 (development / storage) はホスト側で追加する。
  #
  # 継承関係:
  #   system-base     → Nix 設定 / GC / ブート / ネットワーク / stateVersion
  #   home-manager    → HM NixOS モジュール統合 (useGlobalPkgs など)
  #   locale          → 日本語ロケール / フォント
  #   fcitx5          → 日本語入力 (fcitx5-mozc)
  #   audio           → PipeWire + ALSA/JACK/PulseAudio 互換
  #   desktop         → Wayland コンポジター + greetd + portal + tofi (niri/ironbar/notifications 統合)
  flake.modules.nixos.system-desktop = {
    imports = with inputs.self.modules.nixos; [
      system-base
      home-manager
      locale
      fcitx5
      audio
      desktop
    ];
  };
}
