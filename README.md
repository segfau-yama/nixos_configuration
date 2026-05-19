# nixos_configuration
個人的 NixOS 用の設定リポジトリです。

## 概要
このリポジトリは flakes ベースの NixOS 設定です。

- default: GPU なし構成
- nvidia: NVIDIA GPU 構成
- amd: AMD GPU 構成

GTX 1080 を使う場合は nvidia プロファイルを使用します。

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
GTX 1080 は NVIDIA 構成を使います。

	nixos-install --flake .#nvidia

インストール後に再起動:

	reboot

## 既存システムへの反映
既存 NixOS でこの設定を使う場合:

	# NVIDIA (GTX 1080)
	sudo nixos-rebuild switch --flake /etc/nixos#nvidia

	# GPU なし
	sudo nixos-rebuild switch --flake /etc/nixos#default

	# AMD
	sudo nixos-rebuild switch --flake /etc/nixos#amd

## 初回起動後にやること
この設定では users.users.yama を作成しますが、初期パスワードは固定していません。
必要に応じて TTY で root ログイン後に設定してください。

	passwd yama

あわせて以下を確認:
- ログイン確認
- Niri, Ironbar, fcitx5, 音声の動作確認

## 途中からパッケージを反映する手順
インストール後にパッケージを追加/削除した場合の反映手順です。

### 1. パッケージを編集
主な編集先:

- システム全体: modules 配下 (例: modules/development/tools.nix)
- ユーザー環境: home/home.nix の home.packages

### 2. 変更を保存して Git に追加
flake は Git 追跡中のファイルを参照するため、変更後に add が必要です。

	cd /etc/nixos
	git add .

### 3. 設定を反映
使用中のホストプロファイルに合わせて実行してください。

	# NVIDIA (GTX 1080)
	sudo nixos-rebuild switch --flake /etc/nixos#nvidia

	# GPU なし
	sudo nixos-rebuild switch --flake /etc/nixos#default

	# AMD
	sudo nixos-rebuild switch --flake /etc/nixos#amd

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

	# 使っているプロファイルを指定
	sudo nixos-rebuild switch --flake /etc/nixos#nvidia

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
