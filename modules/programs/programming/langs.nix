{ ... }:
{
  # programming/langs (Home Manager): コンパイラ・リンカ・言語ツールチェーン。
  # 他の programming/* ファイルと同じ flake.modules.homeManager.programming に追記 (Collector Aspect)。
  flake.modules.homeManager.programming = { pkgs, ... }: {
    home.packages = with pkgs; [
      rustc   # Rust コンパイラ
      clang   # C/C++ コンパイラ
      mold    # 高速リンカー
      python3 # Python 3 インタープリター
    ];
  };
}
