{ ... }:
{
  # development/nixos (NixOS): nix-ld でパッチされていない ELF バイナリを実行可能にする。
  # 他の development/* ファイルと同じ flake.modules.nixos.development に追記 (Collector Aspect)。
  flake.modules.nixos.development = { ... }: {
    programs.nix-ld.enable = true;
  };
}
