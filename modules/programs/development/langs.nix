{ ... }:
{
  # development/langs (Home Manager): コンパイラ・リンカ・言語ツールチェーン。
  # 他の development/* ファイルと同じ flake.modules.homeManager.development に追記 (Collector Aspect)。
  flake.modules.homeManager.development = { pkgs, ... }: {
    home.packages = with pkgs; [
      rustc   # Rust コンパイラ
      clang   # C/C++ コンパイラ
      mold    # 高速リンカー
    ];
  };
}
