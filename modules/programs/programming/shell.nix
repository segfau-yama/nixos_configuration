{ ... }:
{
  # programming/shell (Home Manager): シェル環境。
  # 他の programming/* ファイルと同じ flake.modules.homeManager.programming に追記 (Collector Aspect)。
  flake.modules.homeManager.programming = { ... }: {
    programs.direnv.enable   = true;  # .envrc を自動適用
    programs.nushell.enable  = true;  # 構造化データ対応シェル
  };
}
