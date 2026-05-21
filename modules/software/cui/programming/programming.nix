{ ... }:
{
  # programming (NixOS): nix-ld でパッチされていない ELF バイナリを実行可能にする。
  flake.modules.nixos.programming = { ... }: {
    programs.nix-ld.enable = true;
  };

  # programming (Home Manager): シェル設定 (Zsh / Nushell / Direnv)。
  #
  # 言語ツールチェーンは lang、Nix ツールは nix-tools、
  # 汎用 CLI は cli-tools を参照して組み合わせる。
  flake.modules.homeManager.programming = { ... }: {

    # ── Shell: Zsh ───────────────────────────────────────────────────────────
    programs.zsh = {
      enable                    = true;
      autosuggestion.enable     = true;  # 履歴ベースの補完候補をグレーで表示
      syntaxHighlighting.enable = true;  # コマンド入力時のシンタックスハイライト
      history = {
        size  = 10000;
        share = true;  # 複数端末で履歴を共有
      };
    };

    # ── Shell: Nushell (サブシェル / スクリプト用) ───────────────────────────
    programs.nushell.enable = true;

    # ── Shell: Direnv ────────────────────────────────────────────────────────
    programs.direnv = {
      enable            = true;  # .envrc を自動適用
      nix-direnv.enable = true;  # nix develop の評価をキャッシュして高速化
    };
  };
}
