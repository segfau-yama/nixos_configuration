{ ... }:
{
  # development/editor (Home Manager): コードエディター。
  # pkgs.unstable を使用するため system-base の overlay が前提。
  # 他の development/* ファイルと同じ flake.modules.homeManager.development に追記 (Collector Aspect)。
  flake.modules.homeManager.development = { pkgs, ... }: {
    home.packages = [
      pkgs.unstable.zed-editor  # 常に最新版を使用 (unstable)
    ];
  };
}
