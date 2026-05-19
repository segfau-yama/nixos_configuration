{ ... }:
{
  # development/shell (Home Manager): シェル環境。
  # 他の development/* ファイルと同じ flake.modules.homeManager.development に追記 (Collector Aspect)。
  flake.modules.homeManager.development = { ... }: {
    programs.direnv.enable   = true;  # .envrc を自動適用
    programs.nushell.enable  = true;  # 構造化データ対応シェル
  };
}
