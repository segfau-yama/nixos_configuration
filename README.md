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
├── setup.sh                     # flake profile 指定型インストーラー
│
├── nixos/
│   └── <hostname>/
│       ├── hardware-configuration.nix   # profile ごとのハードウェア設定
│       ├── generated-hardware-configuration.nix # setup.sh が検出したハードウェア設定（必要時）
│       └── install-args.nix             # setup.sh が生成するインストール入力（必要時）
│
└── modules/
    ├── hardware/
    │   └── hardware.nix         # GPU/CPU ハードウェア抽象化
    │
    ├── hosts/                   # PC ごとのホスト設定
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
    │   ├── base.nix             # 共通基盤（Nix設定・locale・fcitx5・audio）
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
    │       └── admin.nix        # 管理者ユーザー
    │
    ├── devshell.nix             # nix develop 用シェル（nixd・alejandra）
    └── flake-parts.nix          # flake-parts modules エクストラのインポート
```

---

## モジュール一覧

### NixOS モジュール（`modules.nixos.*`）

| モジュール名 | 役割 |
|---|---|
| `base` | ブート・NM・Nix GC・stateVersion・unstable overlay・locale・fcitx5・audio |
| `hardware` | GPU/CPU ドライバー・マイクロコード・nix-auto-storage（`my.hardware.*` オプション） |
| `home-manager` | Home Manager NixOS 統合 |
| `desktop` | Niri・greetd・polkit・seatd・XDG Portal・IronBar |
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
my.hardware.gpu = "nvidia";  # "nvidia" | "amd" | "intel" | "virtio" | "none"
my.hardware.cpu = "amd";     # "intel"  | "amd" | "aarch64"
my.hardware.storage.enable = false;
```

`hardware` モジュールが宣言値に応じてドライバー・マイクロコード・Vulkan ツール・btrfs-progs・nix-auto-storage を自動適用します。

| GPU 値 | 適用される設定 |
|---|---|
| `nvidia` | nvidia proprietary driver・modesetting・Vulkan・nvtop |
| `amd` | amdgpu driver・Mesa・Vulkan |
| `intel` | modesetting・intel-media-driver・OpenCL・Vulkan |
| `virtio` | Virtio GPU (VM)・modesetting・virtio_gpu・Mesa・QEMU Guest Agent・SPICE Agent |
| `none` | VM / 汎用 Mesa・QEMU Guest Agent・SPICE Agent |

GPU が `nvidia` / `amd` / `intel` のとき、x86_64 環境では `hardware.graphics.enable32Bit`・Steam・Gamemode も自動で有効になります。

`my.hardware.storage.enable = true;` のとき、非ブートドライブを自動マウントし、最大容量ドライブへ `/nix` を配置する `nix-auto-storage` を有効化します。

---

## 新規インストール（setup.sh）

`setup.sh` は、リポジトリに定義済みの flake profile を選んで `nixos-install` するための薄いインストーラーです。  
Nix 設定本体は生成せず、インストール時のパーティション入力だけを `nixos/<profile>/install-args.nix` として渡します。

### ネットワークから取得して実行

NixOS ライブ環境（インストール ISO 起動直後）では、まずネットワーク経由で `setup.sh` を取得します。

```bash
curl -fsSL https://raw.githubusercontent.com/segfau-yama/nixos_configuration/main/setup.sh -o setup.sh
vi setup.sh
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

`setup.sh` 冒頭の設定ブロックを編集してから実行します。

```bash
PROFILE="virtual_machine"
TARGET_DISK="/dev/vda"
BOOT_PART=""      # 空なら TARGET_DISK から推定
ROOT_PART=""      # 空なら TARGET_DISK から推定
SWAP_PART=""      # 空なら TARGET_DISK から推定
MOUNT_ROOT="/mnt"
BOOT_END="512MiB"
ROOT_END="100GiB"
YES=true          # 実行前に true へ変更
DRY_RUN=false
```

動作確認だけしたい場合は、`DRY_RUN=true` に変更します。この場合、`install-args.nix` も作成されません。

このスクリプトが行うこと:
1. GPT パーティション作成・フォーマット・マウント
2. リポジトリを `/mnt/etc/nixos` に配置
3. `nixos/<profile>/install-args.nix` に boot/root/swap の割り当てを書き出し、flake 評価に含める
4. `nixos-generate-config --show-hardware-config` の結果を `nixos/<profile>/generated-hardware-configuration.nix` に書き出す
5. `nixos-install --flake /mnt/etc/nixos#<profile>` を実行

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

#### 安全寄り簡易インストールスクリプト（再パーティションなし）

既存パーティションをそのまま使ってインストールだけを行う場合は、以下の手順でスクリプトをダウンロードして実行します。

```bash
curl -fsSL https://raw.githubusercontent.com/segfau-yama/nixos_configuration/main/scripts/install-virtual-machine.sh -o install-virtual-machine.sh
chmod +x install-virtual-machine.sh
sudo ./install-virtual-machine.sh
```

このスクリプトは次を行います:
- `/dev/vda2` を `/mnt` にマウント
- `/dev/vda1` を `/mnt/boot` にマウント
- `/dev/vda3` を `swapon`
- `/mnt/etc/nixos#virtual_machine` で `nixos-install`

このスクリプトは**再パーティション・再フォーマットを行いません**。  
事前に以下が成立している必要があります:
- `/dev/vda1` が `vfat`
- `/dev/vda2` が `ext4`
- `/dev/vda3` が `swap`
- [hardware-configuration.nix](/workspaces/nixos_configuration/nixos/virtual_machine/hardware-configuration.nix) の `/dev/vda*` 設定と一致
- `/mnt/etc/nixos` にこのリポジトリ（`flake.nix` と `.git`）が存在

ホスト設定では GPU を `"virtio"`（推奨）または `"none"` に設定すると、VM 向け設定を有効化できます。`"virtio"` は `modesetting` / `virtio_gpu` を明示し、`"none"` は汎用 Mesa 構成として扱います。

```bash
# VM 向けホスト設定での例（virtio 推奨）
my.hardware.gpu = lib.mkDefault "virtio";
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
| nix-auto-storage 設定 | `modules/hardware/hardware.nix` |
| ホスト固有設定（GPU 種別等） | `modules/hosts/<hostname>/configuration.nix` |
| 共通基盤（Nix・GC・ブート・locale・入力・音声） | `modules/software/base.nix` |
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

## 重要世代の保護

現在は `nixos-mark-generation` コマンドを同梱していません。  
重要な世代を保護する場合は、`nix-env --list-generations --profile /nix/var/nix/profiles/system` で世代を確認し、
必要に応じて手動で GC root を作成してください。

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

`base` モジュールが `pkgs.unstable` overlay を設定します。

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
