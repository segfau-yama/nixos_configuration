{ ... }:
{
  # zed (Home Manager): Zed エディター (GUI) を提供する。
  # CLI 開発ツール (Rust / Python / Git 等) は modules/software/cui/programming/ を参照。
  flake.modules.homeManager.zed = { pkgs, ... }: {
    home.packages = [
      pkgs.unstable.zed-editor  # 常に最新版を使用 (unstable)
    ];
  };
}
