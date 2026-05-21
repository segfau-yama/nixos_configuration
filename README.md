# nixos_configuration

個人用の NixOS 設定リポジトリです。

**Wayland + Niri + NVIDIA + Gaming + 開発 + CAD** に対応した、保守性の高いデスクトップ環境を  
**Dendritic Pattern**（機能単位の自己完結モジュール構成）で構築しています。

---

## 技術スタック

| 項目 | 採用技術 |
|---|---|
| パッケージ管理 | Nix Flakes |
| モジュール管理 | [flake-parts](https://github.com/hercules-ci/flake-parts) + [import-tree](https://github.com/vic/import-tree) |
| ユーザー環境 | [Home Manager](https://github.com/nix-community/home-manager) |
| チャンネル | nixpkgs-25.05 (stable) / nixos-unstable (一部パッケージ) |
| デスクトップ | Niri (Wayland) + IronBar + greetd |
| 入力メソッド | fcitx5 + mozc |
| 音声 | PipeWire + WirePlumber |

---

## ディレクトリ構造

```
nixos_configuration/
│
├── flake.nix                    # flake 定義（import-tree で modules/ を自動インポート）
├── flake.lock
├── setup.sh                     # 対話型インストーラー
│
├── nixos/
│   └── <hostname>/
│       └── hardware-configuration.nix   # setup.sh が nixos-generate-config で生成
│
└── modules/
    ├── hardware/
    │   └── hardware.nix         # GPU/CPU ハードウェア抽象化
    │
    ├── hosts/                   # PC ごとのホスト設定（setup.sh が生成）
    │   └── <hostname>/
    │       ├── configuration.nix
    │       └── flake-parts.nix
    │
    ├── lib/
    │   └── helpers.nix          # mkNixos ヘルパー関数
    │
    ├── nix/
    │   └── home-manager/
    │       └── home-manager.nix # Home Manager NixOS 統合
    │
    ├── software/
    │   ├── base/
    │   │   ├── system-base/     # ブート・ネットワーク・Nix 設定・GC
    │   │   ├── locale/          # 日本語ロケール・フォント
    │   │   ├── fcitx5/          # 日本語入力（fcitx5-mozc）
    │   │   ├── audio/           # PipeWire + ALSA/JACK/PulseAudio
    │   │   └── storage/         # 非ブートドライブ自動マウント・/nix 配置
    │   │
    │   ├── gui/
    │   │   ├── desktop/         # Niri・greetd・XDG Portal・IronBar (NixOS + HM)
    │   │   ├── browser/         # Chromium
    │   │   ├── gaming/          # Lutris・Wine・Winetricks
    │   │   ├── media/           # Spotify・mpv・playerctl
    │   │   ├── sns/             # Discord
    │   │   ├── kicad/           # KiCad
    │   │   ├── freecad/         # FreeCAD + MeshLab
    │   │   └── zed/             # Zed エディター（unstable）
    │   │
    │   └── cui/
    │       ├── programming/     # nix-ld (NixOS) + Zsh/Nushell/Direnv (HM)
    │       ├── lang/            # Rust・Clang・Python
    │       ├── nix-tools/       # nix-index・devenv・nil・nixfmt
    │       └── cli-tools/       # git・xh・jaq・just・pkg-config
    │
    ├── users/
    │   ├── jade/
    │   │   └── jade.nix         # メインユーザー（GUI フルセット）
    │   └── admin/
    │       └── nixos.nix        # 管理者ユーザー
    │
    ├── devshell.nix             # nix develop 用シェル（nixd・alejandra）
    └── flake-parts.nix          # flake-parts modules エクストラのインポート
```

---

## モジュール一覧

### NixOS モジュール（`modules.nixos.*`）

| モジュール名 | 役割 |
|---|---|
| `system-base` | ブート・NM・Nix GC・stateVersion・unstable overlay |
| `hardware` | GPU/CPU ドライバー・マイクロコード（`my.hardware.*` オプション） |
| `home-manager` | Home Manager NixOS 統合 |
| `locale` | 日本語ロケール・フォント・コンソールキーマップ |
| `fcitx5` | fcitx5-mozc・Wayland 環境変数 |
| `audio` | PipeWire・rtkit・ALSA/JACK/PulseAudio 互換 |
| `desktop` | Niri・greetd・polkit・seatd・XDG Portal・IronBar |
| `storage` | 非ブートドライブ自動マウント・最大容量ドライブへ /nix 配置 |
| `programming` | nix-ld（パッチなし ELF バイナリ実行） |
| `jade` | jade ユーザー定義 + HM 統合 |
| `admin` | admin ユーザー定義 |

### Home Manager モジュール（`modules.homeManager.*`）

| モジュール名 | 役割 |
|---|---|
| `desktop` | Niri config.kdl・IronBar・mako・swww（tofi ランチャー含む） |
| `programming` | Zsh・Nushell・Direnv |
| `lang` | Rust・Clang・mold・Python |
| `nix-tools` | nix-index・devenv・nil・nixfmt-rfc-style |
| `cli-tools` | git・xh・jaq・just・pkg-config |
| `browser` | Chromium |
| `gaming` | Lutris・Wine・Winetricks |
| `media` | Spotify・mpv・oculante・playerctl |
| `sns` | Discord |
| `kicad` | KiCad |
| `freecad` | FreeCAD (Wayland)・MeshLab |
| `zed` | Zed エディター（unstable チャンネル） |
| `jade` | jade ユーザーの HM 設定（上記モジュールを組み合わせ） |

---

## ハードウェア設定

GPU と CPU の種類はホスト設定（`modules/hosts/<hostname>/configuration.nix`）で**宣言的**に指定します。  
環境変数や `--impure` フラグは不要です。

```nix
my.hardware.gpu = "nvidia";  # "nvidia" | "amd" | "intel" | "none"
my.hardware.cpu = "amd";     # "intel"  | "amd" | "aarch64"
```

`hardware` モジュールが宣言値に応じてドライバー・マイクロコード・Vulkan ツール・btrfs-progs を自動適用します。

| GPU 値 | 適用される設定 |
|---|---|
| `nvidia` | nvidia proprietary driver・modesetting・Vulkan・nvtop |
| `amd` | amdgpu driver・Mesa・Vulkan |
| `intel` | modesetting・intel-media-driver・OpenCL・Vulkan |
| `none` | OpenGL 無効（VM 用） |

GPU が `none` 以外のとき、x86_64 環境では `hardware.graphics.enable32Bit`・Steam・Gamemode も自動で有効になります。

---

## 新規インストール（自動: setup.sh 推奨）

`setup.sh` は対話型の 3 フェーズインストーラーです。

### ネットワークから取得して実行

NixOS ライブ環境（インストール ISO 起動直後）では、まずネットワーク経由で `setup.sh` を取得します。

```bash
# ネットワーク接続を確認
ping -c 1 github.com

# curl で取得して実行（推奨）
curl -fsSL https://raw.githubusercontent.com/segfau-yama/nixos_configuration/main/setup.sh -o setup.sh
sudo bash setup.sh
```

`curl` が使えない場合は `wget` を使用します。

```bash
wget -O setup.sh https://raw.githubusercontent.com/segfau-yama/nixos_configuration/main/setup.sh
sudo bash setup.sh
```

> **Wi-Fi の場合**  
> NixOS ライブ環境では `wpa_supplicant` または `nmtui` で接続してから実行してください。
> ```bash
> # nmtui で Wi-Fi 設定（テキスト UI）
> nmtui
> # または wpa_cli
> wpa_cli -i wlan0 scan
> wpa_cli -i wlan0 scan_results
> ```

### フェーズ 1: PC 設定

- ハードウェア自動検出（CPU・GPU・メモリ・ディスク一覧）
- インストール先デバイス・パーティションサイズの選択
- キーボードレイアウト・ロケール・タイムゾーンの選択
- SSH・nix-auto-storage・GPU/CPU の設定

### フェーズ 2: ユーザー設定

- デフォルトユーザー `jade`（GUI フルセット）と `admin`（管理者）の選択
- カスタムユーザーの追加（ユーザー名・説明・GUI/CUI 種別・プログラムセット選択）
  - **GUI ユーザー**: desktop・browser・gaming 等の GUI モジュールを選択可能
  - **CUI ユーザー**: programming・lang・nix-tools・cli-tools のみ選択可能

### フェーズ 3: インストール

1. GPT パーティション作成・フォーマット・マウント
2. リポジトリを `/mnt/etc/nixos` に clone
3. `nixos-generate-config` でハードウェア設定を `nixos/<hostname>/` に生成
4. ホスト設定ファイルを `modules/hosts/<hostname>/` に生成
5. カスタムユーザー設定を `modules/users/<username>/` に生成
6. `git add .` で全ファイルを追跡
7. `nixos-install --flake /mnt/etc/nixos#<hostname>` を実行

---

## 新規インストール（手動）

`setup.sh` を使わず手動でセットアップする場合の手順です。  
以下は UEFI 前提で、`/dev/nvme0n1` を例として使用します。

> **root シェルに切り替えておくと便利です（推奨）**
> ```bash
> sudo -i
> ```

### 1. NixOS インストーラで起動

- NixOS インストールメディアで起動
- ネットワーク接続を確認

### 2. ディスクを作成

```bash
lsblk -f   # デバイス名を確認

parted -s /dev/nvme0n1 mklabel gpt
parted -s /dev/nvme0n1 mkpart ESP fat32 1MiB 1GiB
parted -s /dev/nvme0n1 set 1 esp on
parted -s /dev/nvme0n1 mkpart nixos ext4 1GiB 57GiB
parted -s /dev/nvme0n1 mkpart swap linux-swap 57GiB 100%

mkfs.fat -F 32 -n boot /dev/nvme0n1p1
mkfs.ext4 -L nixos -F /dev/nvme0n1p2
mkswap -L swap /dev/nvme0n1p3
```

### 3. マウント

```bash
mount /dev/disk/by-label/nixos /mnt
mkdir -p /mnt/boot
mount /dev/disk/by-label/boot /mnt/boot
swapon /dev/disk/by-label/swap
```

### 4. リポジトリを配置

```bash
mkdir -p /mnt/etc/nixos
cd /mnt/etc/nixos
git init
git remote add origin https://github.com/segfau-yama/nixos_configuration.git
git fetch origin
git checkout -t origin/main
```

### 5. ハードウェア設定を生成

ホスト名（例: `mypc`）のディレクトリに `hardware-configuration.nix` を生成します。

```bash
HOSTNAME=mypc
mkdir -p nixos/${HOSTNAME}
nixos-generate-config --root /mnt --dir /mnt/etc/nixos/nixos/${HOSTNAME}
rm -f /mnt/etc/nixos/nixos/${HOSTNAME}/configuration.nix  # flake では不要
```

### 6. ホスト設定ファイルを作成

`modules/hosts/<hostname>/` に2ファイルを作成します。

**`modules/hosts/mypc/configuration.nix`**:
```nix
{ inputs, ... }:
{
  flake.modules.nixos.mypc = { lib, ... }: {
    imports = with inputs.self.modules.nixos; [
      system-base
      home-manager
      locale
      fcitx5
      audio
      desktop
      jade      # または任意のユーザーモジュール
      admin
    ] ++ [ "${inputs.self}/nixos/mypc/hardware-configuration.nix" ];

    networking.hostName = "mypc";
    my.hardware.gpu = lib.mkDefault "nvidia";  # "nvidia" | "amd" | "intel" | "none"
    my.hardware.cpu = lib.mkDefault "amd";     # "intel"  | "amd"
  };
}
```

**`modules/hosts/mypc/flake-parts.nix`**:
```nix
{ inputs, ... }:
{
  flake.nixosConfigurations = inputs.self.lib.mkNixos "x86_64-linux" "mypc";
}
```

### 7. flake 用にファイルを追跡

flake は Git 追跡ファイルのみを参照するため、新規ファイルはすべて `git add` が必要です。

```bash
git add .
```

### 8. インストール実行

```bash
nixos-install --flake /mnt/etc/nixos#mypc
```

インストール後に再起動:

```bash
reboot
```

---

## 仮想マシンへのインストール

QEMU/KVM などの VM にインストールする場合の補足です。  
ここでは 20GiB の `/dev/vda` を例として使用します。

> **UEFI 起動を有効にしてください**
> この設定は `systemd-boot` を使用します。
> - **QEMU/KVM**: virt-manager で「ファームウェア」に OVMF を選択
> - **VirtualBox**: 設定 → システム → マザーボードタブ → EFI を有効化
> - **VMware**: ファームウェアタイプに UEFI を選択

#### パーティション構成例 (20GiB ディスク)

| デバイス | ラベル | サイズ | 用途 |
|---|---|---|---|
| `/dev/vda1` | `boot` | 512MiB | EFI システムパーティション |
| `/dev/vda2` | `nixos` | 約17.5GiB | NixOS ルート (ext4) |
| `/dev/vda3` | `swap` | 約2GiB | スワップ |

```bash
parted -s /dev/vda mklabel gpt
parted -s /dev/vda mkpart ESP fat32 1MiB 512MiB
parted -s /dev/vda set 1 esp on
parted -s /dev/vda mkpart nixos ext4 512MiB 18GiB
parted -s /dev/vda mkpart swap linux-swap 18GiB 100%
mkfs.fat -F 32 -n boot /dev/vda1
mkfs.ext4 -L nixos -F /dev/vda2
mkswap -L swap /dev/vda3
```

ホスト設定では GPU を `"none"` に設定するとドライバーが無効化され VM に適した構成になります。

```bash
# VM 向けホスト設定での例
my.hardware.gpu = lib.mkDefault "none";
my.hardware.cpu = lib.mkDefault "amd";
```

> **VM 導入後の注意**
> - Niri の描画: QEMU/KVM で Virgil3D を有効にすると GPU アクセラレーションが利きます（設定は別途必要）
> - `NetworkManager-wait-online.service` が起動を遅延させる場合は `systemd.services.NetworkManager-wait-online.enable = false;` を追加してください
> - QEMU Guest Agent が必要な場合は `services.qemuGuest.enable = true;` を追加してください

---

## 初回起動後にやること

ユーザーパスワードを設定します（TTY から root でログイン）。

```bash
passwd jade
passwd admin  # 必要に応じて
```

あわせて以下を確認:
- Niri・IronBar の起動確認
- fcitx5 での日本語入力確認
- PipeWire による音声確認

---

## 設定変更・反映手順

### 1. ファイルを編集

主な編集先:

| 変更内容 | ファイル |
|---|---|
| GPU/CPU ドライバー設定 | `modules/hardware/hardware.nix` |
| ホスト固有設定（GPU 種別等） | `modules/hosts/<hostname>/configuration.nix` |
| システム基盤（Nix・GC・ブート） | `modules/software/base/system-base/system-base.nix` |
| 日本語ロケール・フォント | `modules/software/base/locale/locale.nix` |
| 日本語入力 | `modules/software/base/fcitx5/fcitx5.nix` |
| 音声設定 | `modules/software/base/audio/audio.nix` |
| Niri・greetd・IronBar | `modules/software/gui/desktop/desktop.nix` |
| ゲーミング (Lutris/Wine) | `modules/software/gui/gaming/gaming.nix` |
| 開発ツール（シェル・Direnv） | `modules/software/cui/programming/programming.nix` |
| 言語ツールチェーン | `modules/software/cui/lang/lang.nix` |
| ユーザー設定（jade） | `modules/users/jade/jade.nix` |

### 2. 変更を Git に追加

flake は Git 追跡中のファイルのみ参照します。**新規ファイル**を追加した場合は必ず `git add` が必要です。

```bash
cd /etc/nixos
git add .
```

### 3. 設定を反映

```bash
nixos-rebuild switch --flake /etc/nixos#<hostname>
```

### 4. 反映確認

- 追加したコマンドが実行できるか確認
- GUI アプリはログアウト → 再ログインで確認

### 5. 失敗時のロールバック

直前の世代に戻す場合:

```bash
nixos-rebuild switch --rollback
```

ブート選択画面で `previous generation` を選ぶことでも復旧できます。

---

## Flake 更新手順

```bash
cd /etc/nixos
nix flake update
nixos-rebuild switch --flake /etc/nixos#<hostname>
```

---

## 重要世代のマーク

`system-base` モジュールに `nixos-mark-generation` コマンドが含まれています。  
GC root を作成して、自動 GC で誤って削除されないよう世代を保護します。

```bash
# 現在の世代を baseline として保護
sudo nixos-mark-generation baseline

# 指定した世代番号を保護
sudo nixos-mark-generation before-gpu-update 132

# 現在のラベル確認
ls /nix/var/nix/gcroots/important-generations/
```

作成された GC root の保存先:

```
/nix/var/nix/gcroots/important-generations/<tag>
```

---

## 開発シェル

リポジトリ編集用の開発シェル（nixd + alejandra）を提供しています。

```bash
nix develop
```

| ツール | 用途 |
|---|---|
| `nixd` | Nix 言語サーバー（LSP） |
| `alejandra` | Nix フォーマッター |

---

## 設計メモ

### Dendritic Pattern について

各 feature を自己完結させ、ホスト側は「どの feature を有効化するか」のみを記述します。  
`import-tree` が `modules/` 配下の全 `.nix` ファイルを自動インポートし、  
`flake-parts` の `flake.modules.nixos.*` / `flake.modules.homeManager.*` オプションを通じてモジュールが登録されます。

```
flake.nix
└── import-tree ./modules        # 全 .nix ファイルを自動インポート
    └── flake.modules.nixos.*    # NixOS モジュール
    └── flake.modules.homeManager.*  # Home Manager モジュール
```

**新しいモジュールの追加手順:**

```bash
# 例: 新しいツールを追加
mkdir -p modules/software/gui/newapp
cat > modules/software/gui/newapp/newapp.nix <<'EOF'
{ ... }:
{
  flake.modules.homeManager.newapp = { pkgs, ... }: {
    home.packages = [ pkgs.newapp ];
  };
}
EOF

git add modules/software/gui/newapp/newapp.nix
```

あとはユーザーの HM 設定（`modules/users/<user>/<user>.nix`）の `imports` に `newapp` を追加するだけです。

### nixpkgs stable / unstable の使い分け

`system-base` モジュールが `pkgs.unstable` overlay を設定します。

| チャンネル | 用途 |
|---|---|
| stable (25.05) | OS 基盤・ドライバー・大多数のパッケージ |
| unstable (`pkgs.unstable.*`) | Zed エディター等、最新版が必要なパッケージのみ |

### git add を忘れずに

Nix flake は Git が追跡しているファイルのみを参照します。  
新規ファイルを追加した場合は `git add` しないとフレーク評価で無視されます。

```bash
# 新規ファイル追加後は必ず
git add <新規ファイル>
# または
git add .
```
