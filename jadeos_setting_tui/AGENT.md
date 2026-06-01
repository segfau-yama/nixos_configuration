# AGENT.md

## Rust製 TUI インストーラーの役割

### 機能責務
1. jadeos_setup で起動し、ネットワーク確認から reboot まで対話的にインストールを完結する
2. nixos_configuration リポジトリを fork/clone し、ホスト設定を反映する
3. PC 設定を収集する（hardware detection / partitioning / GPU / CPU / boot type）
4. ユーザー設定を収集する（preset or custom / GUI or CUI or TUI / program selection）
5. インストール処理を実行する（partition / format / mount / nixos-install / reboot）
6. インストール後に設定を fork リポジトリへ commit/push する

### 非責務（初期フェーズ）
1. GitHub private repository の新規作成自動化は必須にしない
2. まずは fork 済みリポジトリの選択と設定反映を優先する
3. setup.sh との完全同等は段階的に到達する

---

## 画面フロー仕様

### 画面一覧と遷移順

| # | Screen | 説明 | ガード条件 |
|---|--------|------|----------|
| 1 | Welcome | 起動・ネットワーク確認 | 接続確認成功 |
| 2 | GitHubLogin | gh auth / リポジトリ選択 | clone 成功 |
| 3 | DeviceSelect | インストール先ディスク選択 | device 未選択なら next 不可 |
| 4 | PartitionConfig | boot/swap/root 容量入力 | 値が正の整数 |
| 5 | PartitionConfirm | 破壊的操作の確認 overlay | OK 選択必須 |
| 6 | HostSelect | デフォストホストを使うか選択 | - |
| 7 | HostnameInput | ホスト名入力 | 非空・英数字 |
| 8 | HardwareDetect | CPU/GPU 自動検出・確認 | - |
| 9 | GpuSelect | GPU 種別確認・変更 | - |
| 10 | CpuSelect | CPU 種別確認・変更 | - |
| 11 | BootTypeSelect | systemd-boot / GRUB 選択 | - |
| 12 | LocaleSelect | ロケール選択 | - |
| 13 | TimezoneSelect | タイムゾーン選択 | - |
| 14 | KeyboardSelect | キーボードレイアウト選択 | - |
| 15 | SshToggle | SSH 有効/無効 | - |
| 16 | UserMenu | ユーザー管理（追加/削除/一覧） | ユーザー1件以上 |
| 17 | PresetUserPassword | プリセットユーザーのパスワード入力 | - |
| 18 | CustomUserBasic | カスタムユーザー名入力 | - |
| 19 | CustomUserType | ユーザータイプ選択 | - |
| 20 | CustomUserPrograms | インストールパッケージ選択 | - |
| 21 | CustomUserPassword | カスタムユーザーのパスワード入力 | - |
| 22 | Summary | 設定サマリ確認 | - |
| 23 | InstallConfirm | インストール開始確認 overlay | OK 選択必須 |
| 24 | Installing | インストール進捗・ログ表示 | 完了/失敗まで待機 |
| 25 | Done | 成功/失敗・次のステップ表示 | - |

各画面で定義する項目:
- 入力: ユーザー操作と既存 state
- 出力: 更新された state
- 遷移条件: next/back のガード条件

---

## 各フェーズ詳細仕様

### 1. Welcome / ネットワーク確認
- `ping -c 1 8.8.8.8` でネットワーク疎通を確認する
- 失敗時はエラーメッセージを表示してインストーラーを終了する
- 成功時のみ次画面へ進む

### 2. GitHub ログイン
- `gh auth status` でログイン済みか確認する
- 未ログインなら `gh auth login` フローを案内する
- ログイン済みアカウントの username を取得し state に保持する（ハードコードしない）
- nixos_configuration を fork したリポジトリ名を入力または選択する
- `gh repo list --json name,owner` で候補を絞り込む
- 選択したリポジトリを `git clone` で live USB 上の作業ディレクトリ（`/tmp/nixos_config/`）へコピーする
- clone 失敗時はエラーを表示してリトライを促す

### 3. インストールディスクの設定
- `lsblk -dno NAME,SIZE,TYPE` の出力をパースしてディスク一覧をセレクトメニューで表示する
- 選択後、以下の容量を入力する（デフォルト: boot=512MiB, swap=0 または有効/無効トグル）
  - **boot**: EFI パーティション（推奨デフォルト: 512MiB。2GiB は不要に大きい）
  - **swap**: swap パーティション。`0` または無効化で swapfile に切り替える選択肢を持つ
  - **root**: 残り全量を自動割り当て（手動入力不要）
- パーティション分割・フォーマット・マウントは破壊的操作のため **PartitionConfirm overlay** で明示的に確認する
- マウントポイント: root→`/mnt`、boot→`/mnt/boot`

```
# パーティション構成例（UEFI + swap あり）
/dev/sdX1  512MiB   EFI System   /mnt/boot
/dev/sdX2  2GiB     Linux swap
/dev/sdX3  残り全量  Linux root   /mnt
```

### 4. 初期 config ファイル作成
- デフォルトホスト（`laptop` または `virtual_machine`）を流用するか、新規ホストを作成するか選択する
- 新規ホスト名を入力する（英数字・ハイフンのみ）
- 以下のコマンドで `/mnt/etc/nixos/{hostname}/` にホスト設定を生成する

```bash
mkdir -p /mnt/etc/nixos/{hostname}
nixos-generate-config \
  --root /mnt \
  --dir /mnt/etc/nixos/{hostname}
```

- 生成された `hardware-configuration.nix` は clone したリポジトリの `nixos/{hostname}/` へコピーする
- flake ベースでインストールするため、clone リポジトリを `/mnt/etc/nixos/` にも配置する

### 5. デバイスドライバの設定
- `lscpu`, `lspci -nn`, `cat /proc/cpuinfo` でハードウェアを自動検出する
- 検出結果を GpuSelect・CpuSelect 画面で確認・上書き可能にする
- boot 方式は `/sys/firmware/efi/efivars` の存在で判定し BootTypeSelect 画面で確認する
- CPU/GPU ドライバ設定・PipeWire・linux-firmware 等の共通設定は `configuration.nix` に書き込む
- `base.nix` は直接編集しない。`configuration.nix` の imports / options 経由で渡す

### 6. ロケーション・キーボード・タイムゾーン設定
- locale / keyboard / timezone はそれぞれ専用選択画面で設定する
- `configuration.nix` の `i18n.defaultLocale`, `console.keyMap`, `time.timeZone` に書き込む
- `base.nix` は直接編集しない

### 7. SSH 設定
- SSH 有効/無効をトグルで選択する
- 有効時は `services.openssh.enable = true;` を `configuration.nix` に書き込む

### 8. ユーザーの追加

#### プリセットユーザー一覧

| プリセット名 | タイプ | DE | インストールパッケージ |
|-------------|--------|----|-----------------------|
| jade-core | TUI（CUI + TUI + マウス対応） | なし | programming, browser, media, sns |
| jade-office | GUI | KDE Plasma | browser, media, sns, office |
| jade-gaming | GUI | KDE Plasma | browser, gaming, media, sns |
| jade-develop | GUI | Hyprland | programming, browser, media, sns, electronics, mechanical |
| jade-full | GUI | Hyprland | programming, browser, gaming, media, sns, electronics, mechanical |
| custom | 任意 | 任意 | 任意 |

> **jade-develop と jade-full の差分**: jade-full は gaming を追加で含む完全セット

- ユーザーは名前を変更して追加できる
- プリセットを選択するとパスワード入力のみ追加の手順が必要
- ユーザーは複数追加できる（UserMenu で管理）

#### カスタムユーザー設定（overlay で表示）
- ユーザー名・パスワードを入力する
- ユーザータイプを選択する（`cui`・`tui`・`gui` の3種）
  - `cui`: DEV_PROGRAM_OPTIONS のみ選択可
  - `tui`: DEV_PROGRAM_OPTIONS + マウス対応 CUI ツール
  - `gui`: GUI_PROGRAM_OPTIONS + DEV_PROGRAM_OPTIONS すべて選択可
- ユーザータイプに応じてパッケージ選択肢を絞り込む
- モーダルに相当する表示は TUI では overlay（Popup Block）として実装する

### 9. インストールの開始
- Summary 画面で全設定値を確認する
- **InstallConfirm overlay** で「インストール開始 / キャンセル」を選択する（破壊的操作のため確認必須）
- flake ベースのインストールコマンドを実行する

```bash
nixos-install --flake /mnt/etc/nixos#{hostname} --no-root-password
```

- Installing 画面にリアルタイムでログを表示する
- 完了後、clone リポジトリに変更をコミットして fork リポジトリへ push する

```bash
cd /tmp/nixos_config
git add .
git commit -m "add host: {hostname}"
git push origin main
```

- push には `gh auth login` 済みの認証情報を使用する（SSH キー不要）
- Done 画面で成功/失敗と手動 push 手順を表示する

---

## エラーハンドリング方針

| フェーズ | 失敗時の挙動 |
|---------|-------------|
| ネットワーク確認 | エラーメッセージ表示 → 終了 |
| GitHub clone | エラー表示 → リトライ or 中断 |
| パーティション/フォーマット | エラー表示 → 手動対応案内 → 終了 |
| nixos-install | ログ全文を表示 → リトライ or 終了 |
| git push | エラー表示 → 手動 push 手順を Done 画面に表示（インストール自体は成功扱い） |

---

## ファイル分割と設計パターン

### ディレクトリ構成
```
src/
  main.rs         # TUI ランタイム初期化・イベントループ・終了処理
  app.rs          # Screen 状態と遷移・入力イベントのルーティング
  ui.rs           # 描画専用（state を受けて表示）
  config.rs       # 設定データモデル（InstallConfig など）
  components/     # ロジックを持たない再利用可能な UI パーツ
    selector.rs   # 上下キー選択リスト
    form.rs       # テキスト入力フォーム
    popup.rs      # overlay / confirm dialog（モーダル相当）
    progress.rs   # インストール進捗バー・ログビュー
    controls.rs   # キーバインドヘルプ
    current_config.rs  # 設定サマリ読み取り専用パネル
    mod.rs
  hooks/          # 再利用可能な状態ロジック（React hooks に相当する独自パターン）
                  # ※ Rust の hooks は trait + 関数として定義する。React hooks とは別物
    network.rs    # ネットワーク状態管理
    github.rs     # gh コマンド認証・リポジトリ操作
    hardware.rs   # ハードウェア検出ロジック
    partition.rs  # パーティション計算・検証
    mod.rs
  pages/          # ロジックと components を合成した各 Screen の実装
    welcome.rs
    github_login.rs
    device_select.rs
    partition_config.rs
    hostname_input.rs
    hardware_detect.rs
    locale_select.rs
    user_menu.rs
    summary.rs
    installing.rs
    done.rs
    mod.rs
  logic/          # 副作用なしのドメインロジック（既存）
  infra/          # 副作用あり処理（既存）
    command_runner.rs
    install.rs
    password_hasher.rs
    state_store.rs
    mod.rs
```

### 設計原則
1. `components/` はロジックを持たず props（引数）を受けて描画のみ行う
2. `hooks/` は state と副作用のない計算ロジックをまとめる。`infra/` の呼び出しは行わない
3. `pages/` は `hooks/` のロジックと `components/` の描画を合成して Screen を構成する
4. `infra/` 層のみが shell コマンドやファイル I/O を実行する。`app.rs` から直接呼ばない
5. overlay（確認ダイアログ）は `components/popup.rs` を使い全 Screen から呼び出せる形にする
6. state 遷移は `app.rs` 内で単方向に管理し、pages から直接 Screen を書き換えない

---

## 画面レイアウト仕様

### 3段レイアウト
- **上段**: フェーズ表示（Tabs）。選択中タブを bold / color で強調
- **中段**: `main` ブロック（入力・選択フォーム）+ `current_config` ブロック（設定サマリ、読み取り専用）
- **下段**: `controls` ブロック（キーバインドヘルプのみ）

### overlay（モーダル相当）
- `PartitionConfirm` / `InstallConfirm` / カスタムユーザー入力は overlay で表示する
- overlay は中段の上に重ねて描画し、背景を dim にする
- `components/popup.rs` が描画を担当する

### デザイン指針
1. 情報密度より視認性を優先し余白・行間を一定に保つ
2. 強調ルールを統一する
   - 現在選択中: bold + color
   - 通常: プレーン
   - 警告/エラー: 高コントラスト（red / bold）
3. 色に依存しない可読性を確保する（太字・記号・文言で状態差を表現）
4. Ratatui 公式ガイドラインを参考: https://ratatui.rs/

---

## 実装ロードマップ

### Phase A: 設定収集 MVP
- 目的: Welcome → Summary まで全画面の入力を TUI で完結する
- 完了条件: Summary で全設定値を確認でき、`InstallConfig` に正しく格納される

### Phase B: インストール実行フェーズ
- 目的: partition / nixos-install を段階実行し進捗とログを表示する
- 完了条件: Installing 画面に進捗と失敗理由を表示でき、Done 画面へ遷移できる

### Phase C: リポジトリ同期
- 目的: インストール後に fork リポジトリへ commit/push する
- 完了条件: push 成否と再実行手順を Done 画面に表示できる

---

## レビュー観点（PR チェック）

1. 変更が責務境界（pages/hooks/components/infra）を守っているか
2. ハードコードされたユーザー名・URL・パスが含まれていないか
3. 破壊的操作（フォーマット・インストール）の前に確認 overlay があるか
4. state 遷移が単方向で追跡可能か
5. 失敗時のユーザー向けメッセージが明確で、リトライ/中断の案内があるか
6. `base.nix` を直接変更するコードが含まれていないか
7. setup.sh との差分仕様が記録されているか
