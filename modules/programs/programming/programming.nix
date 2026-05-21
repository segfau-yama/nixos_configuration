{ ... }:
{
  # programming (NixOS): nix-ld でパッチされていない ELF バイナリを実行可能にする。
  flake.modules.nixos.programming = { ... }: {
    programs.nix-ld.enable = true;
  };

  # programming (Home Manager): プログラミング環境に必要な全 HM 設定を一箇所に集約する。
  #
  # 含まれる設定:
  #   - editor  : Zed エディター (pkgs.unstable)
  #   - langs   : Rust / C-C++ / Python コンパイラ・ツールチェーン
  #   - nix     : nix-index / devenv / nil / nixfmt
  #   - shell   : direnv / nushell
  #   - tools   : git / xh / jaq / just / pkg-config
  flake.modules.homeManager.programming = { pkgs, ... }: {

    # ── Shell: Zsh (ログインシェル) ──────────────────────────────────────────
    # jade/jade.nix で users.users.jade.shell = pkgs.zsh に設定済み。
    # HM で管理することで補完・提案・ハイライトを宣言的に設定する。
    programs.zsh = {
      enable                     = true;
      autosuggestion.enable      = true;  # 履歴ベースの補完候補をグレーで表示
      syntaxHighlighting.enable  = true;  # コマンド入力時のシンタックスハイライト
      history = {
        size  = 10000;
        share = true;  # 複数端末で履歴を共有
      };
    };

    # ── Shell: Nushell (サブシェル / スクリプト用) ───────────────────────────
    # デフォルトログインシェルは zsh。nushell は構造化データ処理が必要な場面で
    # サブシェルとして呼び出す想定。
    programs.nushell.enable = true;

    # ── Shell: Direnv ────────────────────────────────────────────────────────
    programs.direnv = {
      enable            = true;  # .envrc を自動適用
      nix-direnv.enable = true;  # nix develop の評価をキャッシュして高速化
    };

    # ── Packages ─────────────────────────────────────────────────────────────
    home.packages = [
      # --- Editor ---
      pkgs.unstable.zed-editor  # 常に最新版を使用 (unstable)

      # --- Language Toolchains ---
      pkgs.rustc    # Rust コンパイラ
      pkgs.clang    # C/C++ コンパイラ
      pkgs.mold     # 高速リンカー
      pkgs.python3  # Python 3 インタープリター

      # --- Nix Ecosystem ---
      pkgs.nix-index         # nix-locate で Nix パッケージを検索
      pkgs.devenv            # 開発環境マネージャー
      pkgs.nil               # Nix LSP サーバー
      pkgs.nixfmt-rfc-style  # RFC 形式の Nix フォーマッター

      # --- General CLI Tools ---
      pkgs.git         # バージョン管理
      pkgs.xh          # 使いやすい HTTP クライアント
      pkgs.jaq         # jq 互換の高速 JSON プロセッサー
      pkgs.just        # コマンドランナー (Makefile 代替)
      pkgs.pkg-config  # ライブラリパス解決ヘルパー
    ];
  };
}
