{ ... }:
{
  # lang (Home Manager): 言語ツールチェーン。
  # Rust / C-C++ / Python コンパイラ・ビルドツールを提供する。
  flake.modules.homeManager.lang = { pkgs, ... }: {
    home.packages = with pkgs; [
      rustc   # Rust コンパイラ
      clang   # C/C++ コンパイラ
      mold    # 高速リンカー
      python3 # Python 3 インタープリター
    ];
  };
}
