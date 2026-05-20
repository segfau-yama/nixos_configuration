# nixos_configuration
個人的 NixOS 用の設定リポジトリです。

## 概要
このリポジトリは flakes ベースの NixOS 設定です。

### CPU と GPU を環境変数で選択

CPU（`NIX_CPU`）と GPU（`NIX_GPU`）を環境変数で指定して、柔軟にカスタマイズできます。

**使用可能な値**：
- `NIX_CPU`: `x86_64-linux` (デフォルト), `aarch64-linux`
- `NIX_GPU`: `default` (デフォルト), `nvidia`, `amd`

> **重要 — `--impure` フラグについて**
> 
> このリポジトリは `builtins.getEnv` で環境変数を読み取ります。
> Nix のデフォルトは **pure evaluation** モードであり、このモードでは環境変数が常に空文字列として扱われます。
> `NIX_CPU` / `NIX_GPU` を有効にするには、**`nixos-install` および `nixos-rebuild` に `--impure` フラグを必ず付けてください**。
> 
> `--impure` なしで実行した場合、環境変数は無視され `x86_64-linux` + GPU なし のデフォルト構成になります。

### 例

```bash
# x86_64 + NVIDIA（デフォルト推奨）
sudo nixos-install --impure --flake .#nixos

# aarch64 + NVIDIA
NIX_CPU=aarch64-linux NIX_GPU=nvidia sudo nixos-install --impure --flake .#nixos

# x86_64 + AMD
NIX_GPU=amd sudo nixos-install --impure --flake .#nixos

# aarch64 + GPU なし
NIX_CPU=aarch64-linux NIX_GPU=default sudo nixos-install --impure --flake .#nixos
```

> **注意**: GTX 1080 を使う場合は `NIX_GPU=nvidia` を指定します。

## 新規インストール手順
以下は UEFI 前提の手順です。

### 1. NixOS インストーラで起動
- NixOS インストールメディアで起動
- ネットワーク接続を確認

### 2. ディスクを作成
以下は例として /dev/nvme0n1 を使用します。実際のデバイス名は lsblk で確認してください。

	lsblk -f
	parted -s /dev/nvme0n1 mklabel gpt
	parted -s /dev/nvme0n1 mkpart ESP fat32 1MiB 1GiB
	parted -s /dev/nvme0n1 set 1 esp on
	parted -s /dev/nvme0n1 mkpart nixos ext4 1GiB 57GiB
	parted -s /dev/nvme0n1 mkpart swap linux-swap 57GiB 100%
	mkfs.fat -F 32 -n boot /dev/nvme0n1p1
	mkfs.ext4 -L nixos /dev/nvme0n1p2
	mkswap -L swap /dev/nvme0n1p3

### 3. マウント
	mount /dev/disk/by-label/nixos /mnt
	mkdir -p /mnt/boot
	mount /dev/disk/by-label/boot /mnt/boot
	swapon /dev/disk/by-label/swap

### 4. このリポジトリを配置
インストーラが生成した /mnt/etc/nixos は非空のため、clone はできません。
既存ディレクトリに git を初期化して取り込みます。

	cd /mnt/etc/nixos
	git init
	git remote add origin https://github.com/segfau-yama/nixos_configuration.git
	git fetch origin
	git checkout -t origin/main

### 5. ハードウェア設定を実機で再生成
この手順で生成される hardware-configuration.nix は実機情報を含むため必須です。

	nixos-generate-config --root /mnt

### 6. flake 用にファイルを追跡
flake は Git 追跡ファイルのみ参照するため、インストール前に add します。

	git add .

### 7. インストール実行

使用するハードウェアに応じて以下を実行します：

**x86_64 + NVIDIA（推奨）**
```bash
sudo nixos-install --impure --flake .#nixos
```

**aarch64 + NVIDIA**
```bash
NIX_CPU=aarch64-linux NIX_GPU=nvidia sudo nixos-install --impure --flake .#nixos
```

**x86_64 + AMD**
```bash
NIX_GPU=amd sudo nixos-install --impure --flake .#nixos
```

**aarch64 + GPU なし**
```bash
NIX_CPU=aarch64-linux NIX_GPU=default sudo nixos-install --impure --flake .#nixos
```

インストール後に再起動:

	reboot

## 既存システムへの反映
既存 NixOS でこの設定を使う場合、インストール時の環境変数と同じ値を指定します：

	# x86_64 + NVIDIA（推奨）
	sudo nixos-rebuild switch --impure --flake /etc/nixos#nixos

	# aarch64 + NVIDIA
	NIX_CPU=aarch64-linux NIX_GPU=nvidia sudo nixos-rebuild switch --impure --flake /etc/nixos#nixos

	# x86_64 + AMD
	NIX_GPU=amd sudo nixos-rebuild switch --impure --flake /etc/nixos#nixos

	# aarch64 + GPU なし
	NIX_CPU=aarch64-linux NIX_GPU=default sudo nixos-rebuild switch --impure --flake /etc/nixos#nixos

## 初回起動後にやること
この設定では users.users.jade を作成しますが、初期パスワードは固定していません。
必要に応じて TTY で root ログイン後に設定してください。

	passwd jade

あわせて以下を確認:
- ログイン確認
- Niri, Ironbar, fcitx5, 音声の動作確認

## 途中からパッケージを反映する手順
インストール後にパッケージを追加/削除した場合の反映手順です。

### 1. パッケージを編集
主な編集先:

- システム全体の機能: modules 配下 (例: modules/gaming/steam.nix)
- ユーザーアプリ: home 配下 (例: home/development.nix, home/gaming.nix)

### 2. 変更を保存して Git に追加
flake は Git 追跡中のファイルを参照するため、変更後に add が必要です。

	cd /etc/nixos
	git add .

### 3. 設定を反映
インストール時に使用した環境変数を指定してください。わからない場合は、現在のシステムに合わせて指定します：

	# x86_64 + NVIDIA
	sudo nixos-rebuild switch --impure --flake /etc/nixos#nixos

	# aarch64 + NVIDIA
	NIX_CPU=aarch64-linux NIX_GPU=nvidia sudo nixos-rebuild switch --impure --flake /etc/nixos#nixos

	# x86_64 + AMD
	NIX_GPU=amd sudo nixos-rebuild switch --impure --flake /etc/nixos#nixos

	# aarch64 + GPU なし
	NIX_CPU=aarch64-linux NIX_GPU=default sudo nixos-rebuild switch --impure --flake /etc/nixos#nixos

### 4. 反映確認
- 追加したコマンドが実行できるか確認
- GUI アプリの場合はログアウト/再ログインで確認

### 5. 失敗時の切り戻し
直前世代に戻す場合:

	sudo nixos-rebuild switch --rollback

ブート時に previous generation を選ぶ方法でも復旧できます。

## 更新手順
	cd /etc/nixos
	sudo nix flake update

	# 使っている GPU プロファイルを指定（例: NVIDIA）
	NIX_GPU=nvidia sudo nixos-rebuild switch --impure --flake /etc/nixos#nixos

## 重要世代のマーク
この設定では、generation の識別と保護をしやすくするために以下を追加しています。

- system.nixos.label = "tracked"（必要に応じて modules/core/nix.nix で変更）
- nixos-mark-generation コマンド（GC root を作って世代を保護）

使い方:

	# 現在の世代を baseline として保護
	sudo nixos-mark-generation baseline

	# 指定した世代番号を保護
	sudo nixos-mark-generation before-gpu-update 132

作成された GC root は次に配置されます。

	/nix/var/nix/gcroots/important-generations/
