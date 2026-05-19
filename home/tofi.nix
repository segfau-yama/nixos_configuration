{ ... }:
{
  programs.tofi = {
    enable = true;
    settings = {
      # ウィンドウサイズと配置
      width = "44%";
      height = 300;
      anchor = "center";

      # 枠線と角丸
      corner-radius = 14;
      border-width = 1;
      outline-width = 0;
      padding-x = 0;
      padding-y = 0;

      # 色設定 - Catppuccin テーマ
      background-color = "#11111bcc";
      text-color = "#cdd6f4";
      input-color = "#cdd6f4";
      selection-color = "#cba6f7";
      selection-background-color = "#cba6f73d";

      # 枠線
      border-color = "#cba6f74d";
      outline-color = "#cba6f74d";

      # フォント
      font = "Inter";
      font-size = 12;

      # UI の挙動
      result-format = "{name}";
      prompt-text = "apps ";
      prompt-color = "#cba6f7";
      prompt-background = "#1e1e2ecc";
      prompt-padding = 10;
      prompt-font-size = 12;

      # 入力欄のスタイル
      input-background-color = "#1e1e2ecc";
      input-background-padding = 10;
      input-padding = 0;

      # リストのスタイル
      list-items-per-column = 9;
      list-max-display-height = 300;

      # 水平方向レイアウト（縦積みを無効化）
      horizontal = false;

      # マッチング
      fuzzy-match = true;
      require-match = false;

      # ソート
      sort = true;
      sort-by = "frecency";

      # 履歴
      history-file = "~/.cache/tofi/history";
    };
  };
}
