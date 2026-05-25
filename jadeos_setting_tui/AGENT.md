# AGENT.md

## Rust製 setup TUI アプリケーションの役割

### 機能責務
1. jadeos_setup で起動し、対話的にインストール設定を収集する
2. NixOS 構成リポジトリを読み取り、必要な設定を反映する
3. PC 設定を収集する（hardware detection / partitioning / GPU / CPU）
4. ユーザー設定を収集する（default or custom / GUI or CUI / program selection）
5. インストール処理を実行する（partition / nixos-install / reboot）
6. 作成または既存の private repository へ configuration を反映する

### 非責務（初期フェーズ）
1. GitHub private repository の新規作成自動化は必須にしない
2. まずは既存リポジトリの読み取りと設定反映を優先する
3. setup.sh との完全同等は段階的に到達する

## ファイル分割

### 現在の責務
- src/main.rs: TUI ランタイム初期化、イベントループ、終了処理
- src/app.rs: 画面状態（Screen）と遷移、入力イベント処理
- src/ui.rs: 描画専用（state を受けて表示）
- src/config.rs: 設定データモデル（InstallConfig など）

### 追加方針
- src/logic/: 副作用なしのドメインロジック
	- 入力検証、選択肢生成、設定変換
- src/components/: 再利用可能な TUI コンポーネント
	- selector、form、confirm dialog、progress view
- src/infra/: 副作用あり処理
	- コマンド実行、git 操作、ファイル I/O

## 画面フロー仕様

1. Welcome
2. PC Config
3. Users
4. Summary
5. Installing
6. Done

各画面で定義する項目:
- 入力: ユーザー操作と既存 state
- 出力: 更新された state
- 遷移条件: next/back のガード条件

## 画面設計

### レイアウト方針
1. 3段レイアウトを維持する
	- 上段: フェーズ表示（Tabs）
	- 中段: 作業領域（main ブロック + current config ブロック）
	- 下段: controls ブロック（操作ヘルプ）
2. main ブロックは現在画面の入力・選択に集中させる
3. current config ブロックは現在の設定サマリを常時表示する
4. controls ブロックは操作ヘルプのみを表示する
5. 上段の `jadeos installer` ブロックは削除する

### ブロック仕様
1. Header（上段）
	- Tabs 自体は維持する
	- `Block::title("jadeos installer")` を廃止する
	- 必要なら選択中タブの強調（bold / color）のみ残す
2. Main（中段）
	- Screen ごとの入力・選択フォームを表示する
	- 現在操作中の項目を marker / bold / color で強調する
	- current config のサマリ情報は main へ混在させない
3. Current Config（中段）
	- main と同じ中段作業領域に独立した Block として表示する
	- hostname / device / locale / GPU / CPU / users など、主要な設定値を一覧する
	- 空欄、真偽値、カスタム値は main と同じ強調ルールで表示する
	- 入力操作は持たせず、読み取り専用の確認領域にする
4. Controls（下段）
	- 既存の controls ブロック構造を維持する
	- キーバインド表記（q / Enter / Esc など）は継続利用する

### デザイン指針（Ratatui 参考）
1. Ratatui 公式ドキュメントのレイアウト/スタイル指針を踏襲する
	- 参考: https://ratatui.rs/
2. 情報密度より視認性を優先する
	- 余白、行間、ブロック境界を一定に保つ
3. 強調ルールを統一する
	- フェーズ選択中: 強調
	- 通常情報: プレーン
	- 警告/失敗: 高コントラスト
4. 色に依存しない可読性を確保する
	- 太字、記号、文言で状態差を表現する

## 設計パターン

1. 継承より合成を使う
2. TUI 表示とロジックを分離する
3. 使い回す処理は logic と component に分離する
4. 副作用処理は infra 層に隔離する
5. app 層から直接 shell コマンドを呼ばない

## 実装ロードマップ

### Phase A: 設定収集 MVP
- 目的: setup.sh 相当の設定入力を TUI で完結する
- 完了条件: Summary で設定値を確認できる

### Phase B: 実行フェーズ
- 目的: partition / nixos-install を段階実行する
- 完了条件: Installing 画面に進捗と失敗理由を表示できる

### Phase C: リポジトリ同期
- 目的: install 後に branch へ commit/push する
- 完了条件: push 成否と再実行手順を Done 画面に表示できる

## レビュー観点（PR チェック）

1. 変更が責務境界（app/ui/logic/infra）を守っているか
2. state 遷移が単方向で追跡可能か
3. 失敗時のユーザー向けメッセージが明確か
4. setup.sh との差分仕様が記録されているか
