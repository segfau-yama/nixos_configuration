{ ... }:
{
  # programming/nixos (NixOS): nix-ld でパッチされていない ELF バイナリを実行可能にする。
  # 他の programming/* ファイルと同じ flake.modules.nixos.programming に追記 (Collector Aspect)。
  flake.modules.nixos.programming = { ... }: {
    programs.nix-ld.enable = true;
  };
}
