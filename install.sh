#!/usr/bin/env bash
# install.sh — NixOS 自動インストールスクリプト
# README「新規インストール手順」のステップ 2〜7 を自動化する。
#
# 使い方:
#   bash install.sh -d /dev/vda -t nixos-vm              # VM
#   bash install.sh -d /dev/nvme0n1 -b 1GiB -r 57GiB    # 物理マシン

set -euo pipefail

# ── デフォルト値 ───────────────────────────────────────────────
DEVICE=""
BOOT_END="512MiB"          # EFI パーティション終了位置
ROOT_END="18GiB"           # ルートパーティション終了位置 (残りはスワップ)

FLAKE_TARGET="${FLAKE_TARGET:-nixos}"
REPO_URL="https://github.com/segfau-yama/nixos_configuration.git"
MOUNT_ROOT="/mnt"

# ── カラー出力 ─────────────────────────────────────────────────
RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'
CYAN='\033[0;36m'; BOLD='\033[1m'; RESET='\033[0m'

info()    { echo -e "${CYAN}[INFO]${RESET}  $*"; }
success() { echo -e "${GREEN}[ OK ]${RESET}  $*"; }
warn()    { echo -e "${YELLOW}[WARN]${RESET}  $*"; }
error()   { echo -e "${RED}[ERR ]${RESET}  $*" >&2; exit 1; }

# ── ヘルプ ─────────────────────────────────────────────────────
usage() {
  cat <<EOF
${BOLD}使い方:${RESET} $0 -d <デバイス> [オプション]

${BOLD}オプション:${RESET}
  -d <デバイス>    インストール先デバイス (例: /dev/vda, /dev/nvme0n1)  [必須]
  -g <GPU>         GPU プロファイル: default | nvidia | amd            (既定: default)
  -c <CPU>         CPU アーキテクチャ: x86_64-linux | aarch64-linux   (既定: x86_64-linux)
  -b <EFI終了>     EFI パーティション終了位置                          (既定: 512MiB)
  -r <Root終了>    ルートパーティション終了位置 (残りはスワップ)       (既定: 18GiB)
  -t <ターゲット>  flake ターゲット名                                  (既定: nixos)
  -h               このヘルプを表示

${BOLD}例:${RESET}
  # VM: 20GiB /dev/vda, GPU なし
  $0 -d /dev/vda

  # VM: nixos-vm ターゲット (NetworkManager-wait-online 無効)
  $0 -d /dev/vda -t nixos-vm

  # 物理マシン: NVMe, NVIDIA GPU
  $0 -d /dev/nvme0n1 -g nvidia -b 1GiB -r 57GiB

  # aarch64 VM
  $0 -d /dev/vda -c aarch64-linux

${BOLD}注意:${RESET}
  - root で実行するか、各コマンドに sudo が通る状態で実行してください。
  - 指定したデバイスの全データが消去されます。
EOF
}

# ── 引数パース ─────────────────────────────────────────────────
while getopts "d:b:r:t:h" opt; do
  case "$opt" in
    d) DEVICE="$OPTARG" ;;
    b) BOOT_END="$OPTARG" ;;
    r) ROOT_END="$OPTARG" ;;
    t) FLAKE_TARGET="$OPTARG" ;;
    h) usage; exit 0 ;;
    *) usage; exit 1 ;;
  esac
done

[[ -z "$DEVICE" ]]   && { usage; echo; error "-d でデバイスを指定してください。"; }
[[ ! -b "$DEVICE" ]] && error "ブロックデバイスが存在しません: $DEVICE"

# ── パーティション名を決定 ─────────────────────────────────────
# nvme / mmcblk は "p1/p2/p3"、それ以外 (sda, vda など) は "1/2/3"
if [[ "$DEVICE" =~ nvme|mmcblk ]]; then
  PART_BOOT="${DEVICE}p1"
  PART_ROOT="${DEVICE}p2"
  PART_SWAP="${DEVICE}p3"
else
  PART_BOOT="${DEVICE}1"
  PART_ROOT="${DEVICE}2"
  PART_SWAP="${DEVICE}3"
fi

# ── 確認プロンプト ─────────────────────────────────────────────
echo
echo -e "${BOLD}══════════════════════════════════════${RESET}"
echo -e "${BOLD}  NixOS 自動インストール${RESET}"
echo -e "${BOLD}══════════════════════════════════════${RESET}"
echo -e "  デバイス         : ${YELLOW}${DEVICE}${RESET}"
echo -e "  EFI 終了位置     : ${BOOT_END}"
echo -e "  Root 終了位置    : ${ROOT_END}"

echo -e "  flake ターゲット : .#${FLAKE_TARGET}"
echo
echo -e "  ${RED}${BOLD}警告: ${DEVICE} の全データが消去されます。${RESET}"
echo
read -rp "  続行しますか？ [yes/N] > " CONFIRM
[[ "$CONFIRM" != "yes" ]] && { info "中断しました。"; exit 0; }
echo

# ── ステップ 2: パーティション作成 ────────────────────────────
info "ステップ 2/6: パーティション作成"
parted -s "$DEVICE" mklabel gpt
parted -s "$DEVICE" mkpart ESP fat32 1MiB "$BOOT_END"
parted -s "$DEVICE" set 1 esp on
parted -s "$DEVICE" mkpart nixos ext4 "$BOOT_END" "$ROOT_END"
parted -s "$DEVICE" mkpart swap linux-swap "$ROOT_END" 100%
success "GPT パーティションテーブル作成"

info "ファイルシステム作成"
mkfs.fat -F 32 -n boot "$PART_BOOT"
mkfs.ext4 -L nixos -F "$PART_ROOT"
mkswap -L swap "$PART_SWAP"
success "ファイルシステム作成完了"

# ── ステップ 3: マウント ──────────────────────────────────────
info "ステップ 3/6: マウント"
mount /dev/disk/by-label/nixos "$MOUNT_ROOT"
mkdir -p "${MOUNT_ROOT}/boot"
mount /dev/disk/by-label/boot "${MOUNT_ROOT}/boot"
swapon /dev/disk/by-label/swap
success "マウント完了 (root=${MOUNT_ROOT})"

# ── ステップ 4: リポジトリを配置 ─────────────────────────────
info "ステップ 4/6: リポジトリを配置"
mkdir -p "${MOUNT_ROOT}/etc/nixos"
cd "${MOUNT_ROOT}/etc/nixos"
git init
git remote add origin "$REPO_URL"
git fetch origin
git checkout -t origin/main
success "リポジトリ配置完了 (${MOUNT_ROOT}/etc/nixos)"

# ── ステップ 5: ハードウェア設定を生成 ───────────────────────
info "ステップ 5/6: hardware-configuration.nix を生成"
nixos-generate-config --root "$MOUNT_ROOT"
success "hardware-configuration.nix 生成完了"

# ── NetworkManager 接続プロファイルを引き継ぎ ─────────────────
# インストーラ上で nmtui などで設定した接続を /mnt 側にコピーする。
# これにより初回起動時に NetworkManager-wait-online が成功しやすくなる。
NM_SRC="/etc/NetworkManager/system-connections"
NM_DST="${MOUNT_ROOT}/etc/NetworkManager/system-connections"
if [[ -d "$NM_SRC" ]] && [[ -n "$(ls -A "$NM_SRC" 2>/dev/null)" ]]; then
  info "NetworkManager 接続プロファイルを引き継ぎ"
  mkdir -p "$NM_DST"
  cp -r "${NM_SRC}/." "$NM_DST/"
  chmod 700 "$NM_DST"
  chmod 600 "$NM_DST"/*  2>/dev/null || true
  success "接続プロファイルをコピーしました (${NM_SRC} → ${NM_DST})"
else
  warn "接続プロファイルなし。初回起動後に nmtui で設定してください。"
fi

# ── ステップ 6: flake 用に Git 追跡 ──────────────────────────
info "ステップ 6/6: flake 用に git add ."
cd "${MOUNT_ROOT}/etc/nixos"
git add .
success "追跡完了"

# ── ステップ 7: nixos-install ─────────────────────────────────
echo
info "nixos-install 開始: .#${FLAKE_TARGET}"
nixos-install --flake "${MOUNT_ROOT}/etc/nixos#${FLAKE_TARGET}"

echo
echo -e "${GREEN}${BOLD}══════════════════════════════════════${RESET}"
echo -e "${GREEN}${BOLD}  インストール完了！${RESET}"
echo -e "${GREEN}${BOLD}══════════════════════════════════════${RESET}"
echo -e "  次のコマンドで再起動してください: ${BOLD}reboot${RESET}"
