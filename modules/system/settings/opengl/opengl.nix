{ ... }:
{
  # opengl: 3D アクセラレーションの共通基盤 (DRY Aspect)。
  # amd / nvidia / gaming が共通して必要とする hardware.graphics 設定を一箇所に集約。
  # 各 GPU フィーチャーからはこのモジュールを import して重複を排除する。
  flake.modules.nixos.opengl = { ... }: {
    hardware.graphics = {
      enable    = true;
      enable32Bit = true;  # Steam / Wine / Proton 向け 32bit 対応
    };
  };
}
