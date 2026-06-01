{ ... }:
{
  # programming (Home Manager): シェル設定 (Zsh / Nushell / Direnv)。
  #
  # ログインシェルの Zsh 設定は base を参照する。
  # 言語ツールチェーンは lang、Nix ツールは nix-tools、
  # 汎用 CLI は cli-tools を参照して組み合わせる。
  flake.modules.homeManager.programming = { ... }: {
    # ── Shell: Nushell (サブシェル / スクリプト用) ───────────────────────────
    programs.nushell.enable = true;

    # ── Shell: Direnv ────────────────────────────────────────────────────────
    programs.direnv = {
      enable            = true;  # .envrc を自動適用
      nix-direnv.enable = true;  # nix develop の評価をキャッシュして高速化
    };
  };
}
