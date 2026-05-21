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
nixos-install --impure --flake .#nixos

# aarch64 + NVIDIA
NIX_CPU=aarch64-linux NIX_GPU=nvidia nixos-install --impure --flake .#nixos

# x86_64 + AMD
NIX_GPU=amd nixos-install --impure --flake .#nixos

# aarch64 + GPU なし
NIX_CPU=aarch64-linux NIX_GPU=default nixos-install --impure --flake .#nixos
```

> **注意**: GTX 1080 を使う場合は `NIX_GPU=nvidia` を指定します。

## 自動インストール（推奨）

ステップ 2〜7 を自動化するスクリプトを用意しています。

```bash
# VM (20GiB /dev/vda)
bash install.sh -d /dev/vda

# VM (nixos-vm ターゲット — NetworkManager-wait-online 無効)
bash install.sh -d /dev/vda -t nixos-vm

# 物理マシン (NVMe + NVIDIA)
bash install.sh -d /dev/nvme0n1 -g nvidia -b 1GiB -r 57GiB

# aarch64 VM
bash install.sh -d /dev/vda -c aarch64-linux
```

| オプション | 説明 | 既定値 |
|---|---|---|
| `-d` | インストール先デバイス | なし（必須） |
| `-g` | GPU プロファイル (`default`/`nvidia`/`amd`) | `default` |
| `-c` | CPU アーキテクチャ | `x86_64-linux` |
| `-b` | EFI パーティション終了位置 | `512MiB` |
| `-r` | ルートパーティション終了位置 | `18GiB` |
| `-t` | flake ターゲット名 | `nixos` |

---

## 新規インストール手順（手動）
以下は UEFI 前提の手順です。

> **root シェルへの切り替え（推奨）**
>
> インストール作業は多くのコマンドに root 権限が必要です。
> 最初に以下を実行して root シェルに切り替えると、以降の `sudo` を省略できます。
>
> `sudo -i`
>
> root シェルに切り替えない場合は、各コマンドの先頭に `sudo` を付けて実行してください。

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
マウント直後は `/mnt/etc/nixos` が存在しないため、先にディレクトリを作成してから git を初期化します。
`/mnt` 以下は root 所有となるため、git コマンドにも `sudo` が必要です。

	mkdir -p /mnt/etc/nixos
	cd /mnt/etc/nixos
	git init
	git remote add origin https://github.com/segfau-yama/nixos_configuration.git
	git fetch origin
	git checkout -t origin/main

### 5. ハードウェア設定を実機で再生成
`nixos-generate-config` を実行すると、リポジトリ内の placeholder `hardware-configuration.nix` が
実機の実情に即した内容で上書きされます。

	nixos-generate-config --root /mnt

### 6. flake 用にファイルを追跡
flake は Git 追跡ファイルのみを参照するため、インストール前に add します。
(上書きされた hardware-configuration.nix もここで追跡します。)

	git add .

### 7. インストール実行

使用するハードウェアに応じて以下を実行します：

**x86_64 + NVIDIA（推奨）**
```bash
nixos-install --impure --flake .#nixos
```

**aarch64 + NVIDIA**
```bash
NIX_CPU=aarch64-linux NIX_GPU=nvidia nixos-install --impure --flake .#nixos
```

**x86_64 + AMD**
```bash
NIX_GPU=amd nixos-install --impure --flake .#nixos
```

**aarch64 + GPU なし**
```bash
NIX_CPU=aarch64-linux NIX_GPU=default nixos-install --impure --flake .#nixos
```

インストール後に再起動:

	reboot

## 他のインストール環境のメモ

### 仮想マシン (QEMU/KVM ・ VirtualBox ・ VMware)

QEMU/KVM などの VM にインストールする場合の手順です。ここでは 20GiB の `/dev/vda` (virtio ディスク) を例に使用します。

> **注意 — UEFI の有効化**
>
> この設定は `systemd-boot` を使用するため、VM 側で必ず **UEFI 起動** を有効にしてください。
> - **QEMU/KVM**: `virt-manager` の場合は「ファームウェア」に `OVMF` (レガシー BIOS ではなく) を選択
> - **VirtualBox**: 設定 → システム → マザーボードタブ → EFI を有効化
> - **VMware**: 　ファームウェアタイプに `UEFI` を選択

> **GPU 設定**: GPU パススルーを行わない通常の VM では `NIX_GPU` の指定は不要です。デフォルトの `default` 構成が適用されます。

#### パーティション構成表 (20GiB ディスク)

| デバイス | ラベル | サイズ | 用途 |
|---|---|---|---|
| `/dev/vda1` | `boot` | 512MiB | EFI システムパーティション |
| `/dev/vda2` | `nixos` | 約17.5GiB | NixOS ルート (ext4) |
| `/dev/vda3` | `swap` | 約2GiB | スワップ |

#### 0. root シェルに切り替え（推奨）

	sudo -i

以降のコマンドはすべて root として実行します。
`sudo -i` を使わない場合は各コマンドの先頭に `sudo` を付けてください。

#### 1. NixOS インストーラで起動
- NixOS インストール ISO を VM の CDROM/ISO に設定して起動
- ネットワーク接続を確認

#### 2. ディスクを作成

	parted -s /dev/vda mklabel gpt
	parted -s /dev/vda mkpart ESP fat32 1MiB 512MiB
	parted -s /dev/vda set 1 esp on
	parted -s /dev/vda mkpart nixos ext4 512MiB 18GiB
	parted -s /dev/vda mkpart swap linux-swap 18GiB 100%
	mkfs.fat -F 32 -n boot /dev/vda1
	mkfs.ext4 -L nixos /dev/vda2
	mkswap -L swap /dev/vda3

#### 3. マウント

	mount /dev/disk/by-label/nixos /mnt
	mkdir -p /mnt/boot
	mount /dev/disk/by-label/boot /mnt/boot
	swapon /dev/disk/by-label/swap

#### 4. リポジトリを配置

	mkdir -p /mnt/etc/nixos
	cd /mnt/etc/nixos
	git init
	git remote add origin https://github.com/segfau-yama/nixos_configuration.git
	git fetch origin
	git checkout -t origin/main

#### 5. ハードウェア設定を実機で再生成
リポジトリ内の placeholder `hardware-configuration.nix` が VM の実情に即した内容で上書きされます。

	nixos-generate-config --root /mnt

#### 6. flake 用にファイルを追跡
上書きされた hardware-configuration.nix もここで追跡します。

	git add .

#### 7. インストール実行

**x86_64 VM（通常の QEMU/KVM ・ VirtualBox ・ VMware）**
```bash
nixos-install --impure --flake .#nixos
```

**aarch64 VM（Apple Silicon Mac 上の UTM など）**
```bash
NIX_CPU=aarch64-linux nixos-install --impure --flake .#nixos
```

インストール後に再起動:

	reboot

#### VM 導入後の注意事項

- **生成される `hardware-configuration.nix` について**: `nixos-generate-config` が生成する `hardware-configuration.nix` には VM の実情に即した `hostPlatform` とディスク情報が含まれるため、そのまま使用してください。
- **Niri の画面描画**: QEMU/KVM で Virgil3D を有効にすると GPU アクセラレーションが利きます。この設定の範囲外のため手動設定が必要です。
- **クリップボード**: VM とホスト間のクリップボード共有には SPICE Guest Tools (別途インストール) が必要な場合があります。

## 既存システムへの反映
既存 NixOS でこの設定を使う場合、インストール時の環境変数と同じ値を指定します：

	# x86_64 + NVIDIA（推奨）
	nixos-rebuild switch --impure --flake /etc/nixos#nixos

	# aarch64 + NVIDIA
	NIX_CPU=aarch64-linux NIX_GPU=nvidia nixos-rebuild switch --impure --flake /etc/nixos#nixos

	# x86_64 + AMD
	NIX_GPU=amd nixos-rebuild switch --impure --flake /etc/nixos#nixos

	# aarch64 + GPU なし
	NIX_CPU=aarch64-linux NIX_GPU=default nixos-rebuild switch --impure --flake /etc/nixos#nixos

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

- システム全体の設定: `modules/system/` 配下
  - 例: `modules/system/settings/nvidia/nvidia.nix`（GPU 設定）
  - 例: `modules/system/settings/audio/audio.nix`（音声設定）
- アプリ・ツール: `modules/programs/` 配下
  - 例: `modules/programs/gaming/gaming.nix`（Steam / Lutris）
  - 例: `modules/programs/programming/home.nix`（エディター / 言語ツール）
  - 例: `modules/programs/niri/home.nix`（Niri キーバインド）
- ユーザー設定: `modules/users/jade/` 配下
  - 例: `modules/users/jade/home.nix`（有効にする HM フィーチャー）

### 2. 変更を保存して Git に追加
flake は Git 追跡中のファイルを参照するため、変更後に add が必要です。

	cd /etc/nixos
	git add .

### 3. 設定を反映
インストール時に使用した環境変数を指定してください。わからない場合は、現在のシステムに合わせて指定します：

	# x86_64 + NVIDIA
	nixos-rebuild switch --impure --flake /etc/nixos#nixos

	# aarch64 + NVIDIA
	NIX_CPU=aarch64-linux NIX_GPU=nvidia nixos-rebuild switch --impure --flake /etc/nixos#nixos

	# x86_64 + AMD
	NIX_GPU=amd nixos-rebuild switch --impure --flake /etc/nixos#nixos

	# aarch64 + GPU なし
	NIX_CPU=aarch64-linux NIX_GPU=default nixos-rebuild switch --impure --flake /etc/nixos#nixos

### 4. 反映確認
- 追加したコマンドが実行できるか確認
- GUI アプリの場合はログアウト/再ログインで確認

### 5. 失敗時の切り戻し
直前世代に戻す場合:

	nixos-rebuild switch --rollback

ブート時に previous generation を選ぶ方法でも復旧できます。

## 更新手順
	cd /etc/nixos
	nix flake update

	# 使っている GPU プロファイルを指定（例: NVIDIA）
	NIX_GPU=nvidia nixos-rebuild switch --impure --flake /etc/nixos#nixos

## 重要世代のマーク
この設定では、generation の識別と保護をしやすくするために以下を追加しています。

- `system.nixos.label = "tracked"`（必要に応じて `modules/system/system-types/system-base/system-base.nix` で変更）
- nixos-mark-generation コマンド（GC root を作って世代を保護）

使い方:

	# 現在の世代を baseline として保護
	nixos-mark-generation baseline

	# 指定した世代番号を保護
	nixos-mark-generation before-gpu-update 132

作成された GC root は次に配置されます。

	/nix/var/nix/gcroots/important-generations/
