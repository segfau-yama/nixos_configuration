#!/usr/bin/env bash
# setup.sh — NixOS インタラクティブセットアップスクリプト
#
# 使い方:
#   bash setup.sh
#
# 流れ:
#   1. PC の設定 (ハードウェア検出 / パーティション / ネットワーク / GPU / CPU)
#   2. ユーザーの設定 (デフォルト or カスタム / GUI or CUI / プログラム選択)
#   3. インストール (パーティション作成 / nixos-install / 再起動)

set -euo pipefail

# ── 定数 ─────────────────────────────────────────────────────────────────────
REPO_URL="https://github.com/segfau-yama/nixos_configuration.git"
MOUNT_ROOT="/mnt"
STATE_VERSION="25.05"

# ── カラー定義 ────────────────────────────────────────────────────────────────
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
BLUE='\033[0;34m'
MAGENTA='\033[0;35m'
BOLD='\033[1m'
DIM='\033[2m'
RESET='\033[0m'

# ── ユーティリティ ────────────────────────────────────────────────────────────
info()      { echo -e "${CYAN}[INFO]${RESET}  $*"; }
success()   { echo -e "${GREEN}[ OK ]${RESET}  $*"; }
warn()      { echo -e "${YELLOW}[WARN]${RESET}  $*"; }
error()     { echo -e "${RED}[ERR ]${RESET}  $*" >&2; exit 1; }
step()      { echo -e "\n${BOLD}${BLUE}▶ $*${RESET}"; }
separator() { echo -e "${DIM}────────────────────────────────────────────────────────${RESET}"; }

banner() {
  echo -e "${BOLD}${CYAN}"
  echo "══════════════════════════════════════════════════════════"
  echo "  $*"
  echo "══════════════════════════════════════════════════════════"
  echo -e "${RESET}"
}

sub_banner() {
  echo -e "\n${BOLD}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}"
  echo -e "${BOLD}  $*${RESET}"
  echo -e "${BOLD}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}"
}

# ── グローバル変数 ─────────────────────────────────────────────────────────────
# Phase 1: PC 設定
DEVICE=""
PART_BOOT=""
PART_ROOT=""
PART_SWAP=""
BOOT_END="512MiB"
ROOT_END="100GiB"
HOSTNAME="nixos"
KEYBOARD="jp106"
LOCALE="ja_JP.UTF-8"
TIMEZONE="Asia/Tokyo"
SSH_ENABLED="false"
STORAGE_ENABLED="false"
GPU_TYPE="none"
GPU_BRAND="検出されず"
CPU_TYPE="amd"
CPU_BRAND="不明"
ARCH="x86_64-linux"

# Phase 2: ユーザー設定
declare -a USER_MODULE_NAMES=()   # host config の imports に並べるモジュール名
declare -a CUSTOM_USERS=()        # "username:type:description:prog1 prog2..."
JADE_SELECTED=false
ADMIN_SELECTED=false
HAS_GUI_USER=false
NEEDS_PROGRAMMING_CLI=false

# ─────────────────────────────────────────────────────────────────────────────
# ハードウェア検出
# ─────────────────────────────────────────────────────────────────────────────
detect_hardware() {
  # CPU
  CPU_BRAND=$(grep -m1 'model name' /proc/cpuinfo 2>/dev/null | cut -d: -f2 | xargs || echo "不明")
  CPU_ARCH=$(uname -m)
  case "$CPU_ARCH" in
    x86_64)
      if echo "$CPU_BRAND" | grep -qi "AMD"; then
        CPU_TYPE="amd"
        ARCH="x86_64-linux"
      else
        CPU_TYPE="intel"
        ARCH="x86_64-linux"
      fi
      ;;
    aarch64)
      CPU_TYPE="aarch64"
      ARCH="aarch64-linux"
      ;;
    *)
      CPU_TYPE="amd"
      ARCH="x86_64-linux"
      ;;
  esac

  # GPU
  local gpu_info
  gpu_info=$(lspci 2>/dev/null | grep -iE 'VGA|3D|Display' || echo "")
  if echo "$gpu_info" | grep -qi "NVIDIA"; then
    GPU_TYPE="nvidia"
    GPU_BRAND=$(echo "$gpu_info" | grep -i "NVIDIA" | head -1 \
      | sed -E 's/.*\[([^]]+)\].*/\1/' \
      | grep -v '^$' || echo "NVIDIA GPU")
  elif echo "$gpu_info" | grep -qi "AMD\|ATI"; then
    GPU_TYPE="amd"
    GPU_BRAND=$(echo "$gpu_info" | grep -iE "AMD|ATI" | head -1 \
      | sed -E 's/.*\[([^]]+)\].*/\1/' \
      | grep -v '^$' || echo "AMD GPU")
  elif echo "$gpu_info" | grep -qi "Intel"; then
    GPU_TYPE="intel"
    GPU_BRAND=$(echo "$gpu_info" | grep -i "Intel" | head -1 \
      | sed -E 's/.*\[([^]]+)\].*/\1/' \
      | grep -v '^$' || echo "Intel GPU")
  else
    GPU_TYPE="none"
    GPU_BRAND="検出されず (仮想マシン / GPU なし)"
  fi
}

# ─────────────────────────────────────────────────────────────────────────────
# Phase 1: PC の設定
# ─────────────────────────────────────────────────────────────────────────────
phase1_pc_config() {
  banner "NixOS インタラクティブセットアップ"
  echo -e "  ${BOLD}ステップ 1/3: PC の設定${RESET}\n"

  detect_hardware

  # ── ハードウェア表示 ──────────────────────────────────────────────────────
  step "ハードウェア検出結果"
  local mem_kb mem_gb
  mem_kb=$(grep MemTotal /proc/meminfo 2>/dev/null | awk '{print $2}' || echo 0)
  mem_gb=$(( mem_kb / 1024 / 1024 ))
  echo -e "  CPU    : ${BOLD}${CPU_BRAND}${RESET} (${CPU_TYPE})"
  echo -e "  GPU    : ${BOLD}${GPU_BRAND}${RESET} (${GPU_TYPE})"
  echo -e "  メモリ  : ${BOLD}${mem_gb} GB${RESET}"
  echo

  step "ストレージデバイス"
  local -a disk_names=()
  local -a disk_info=()
  local idx=1
  while IFS= read -r line; do
    local name size model
    name=$(echo "$line" | awk '{print $1}')
    size=$(echo "$line" | awk '{print $2}')
    model=$(echo "$line" | awk '{$1=$2=""; print $0}' | xargs)
    [[ -z "$name" ]] && continue
    disk_names+=("$name")
    disk_info+=("${name}  ${size}  ${model}")
    echo -e "  ${BOLD}${idx})${RESET} /dev/${name}  ${CYAN}${size}${RESET}  ${model}"
    (( idx++ ))
  done < <(lsblk -dpno NAME,SIZE,MODEL 2>/dev/null \
    | grep -v '^loop' \
    | grep -v '^sr' \
    | awk '{print $0}' \
    | sed 's|/dev/||')

  if [[ ${#disk_names[@]} -eq 0 ]]; then
    error "ブロックデバイスが見つかりません。"
  fi

  # ── デバイス選択 ─────────────────────────────────────────────────────────
  echo
  local sel
  while true; do
    read -rp "  インストール先の番号を入力 [1-${#disk_names[@]}]: " sel
    if [[ "$sel" =~ ^[0-9]+$ ]] && (( sel >= 1 && sel <= ${#disk_names[@]} )); then
      DEVICE="/dev/${disk_names[$(( sel - 1 ))]}"
      break
    fi
    warn "有効な番号を入力してください。"
  done
  success "インストール先: ${DEVICE}"

  # パーティション名を決定
  if [[ "$DEVICE" =~ nvme|mmcblk ]]; then
    PART_BOOT="${DEVICE}p1"
    PART_ROOT="${DEVICE}p2"
    PART_SWAP="${DEVICE}p3"
  else
    PART_BOOT="${DEVICE}1"
    PART_ROOT="${DEVICE}2"
    PART_SWAP="${DEVICE}3"
  fi

  # ── パーティション設定 ────────────────────────────────────────────────────
  echo
  step "パーティション設定"
  local input
  read -rp "  EFI パーティション終了位置 (デフォルト: ${BOOT_END}): " input
  [[ -n "$input" ]] && BOOT_END="$input"
  read -rp "  ルートパーティション終了位置 (デフォルト: ${ROOT_END}): " input
  [[ -n "$input" ]] && ROOT_END="$input"
  echo -e "  EFI: ${CYAN}${BOOT_END}${RESET}   Root: ${CYAN}${ROOT_END}${RESET}   Swap: 残り全て"

  # ── ホスト名 ─────────────────────────────────────────────────────────────
  echo
  step "ホスト名の設定"
  read -rp "  ホスト名 (デフォルト: ${HOSTNAME}): " input
  [[ -n "$input" ]] && HOSTNAME="$input"
  success "ホスト名: ${HOSTNAME}"

  # ── キーボードレイアウト ──────────────────────────────────────────────────
  echo
  step "キーボードレイアウトの設定"
  local kb_list=("jp106:日本語 JIS" "us:英語 US" "de:ドイツ語" "fr:フランス語")
  local i=1
  for kb in "${kb_list[@]}"; do
    local kb_key="${kb%%:*}"
    local kb_label="${kb##*:}"
    if [[ "$i" -eq 1 ]]; then
      echo -e "  ${BOLD}${i})${RESET} ${kb_key}  ${kb_label}  ${DIM}[デフォルト]${RESET}"
    else
      echo -e "  ${BOLD}${i})${RESET} ${kb_key}  ${kb_label}"
    fi
    (( i++ ))
  done
  echo -e "  ${BOLD}${i})${RESET} カスタム入力"
  read -rp "  選択 [1-${i}] (Enter でデフォルト): " sel
  case "$sel" in
    1|"") KEYBOARD="jp106" ;;
    2)    KEYBOARD="us" ;;
    3)    KEYBOARD="de" ;;
    4)    KEYBOARD="fr" ;;
    5)
      read -rp "  キーボードレイアウト名を入力: " input
      [[ -n "$input" ]] && KEYBOARD="$input"
      ;;
  esac
  success "キーボード: ${KEYBOARD}"

  # ── ロケール ─────────────────────────────────────────────────────────────
  echo
  step "言語・ロケールの設定"
  local lc_list=("ja_JP.UTF-8:日本語" "en_US.UTF-8:英語 US" "zh_CN.UTF-8:中国語簡体字" "ko_KR.UTF-8:韓国語")
  i=1
  for lc in "${lc_list[@]}"; do
    local lc_key="${lc%%:*}"
    local lc_label="${lc##*:}"
    if [[ "$i" -eq 1 ]]; then
      echo -e "  ${BOLD}${i})${RESET} ${lc_key}  ${lc_label}  ${DIM}[デフォルト]${RESET}"
    else
      echo -e "  ${BOLD}${i})${RESET} ${lc_key}  ${lc_label}"
    fi
    (( i++ ))
  done
  echo -e "  ${BOLD}${i})${RESET} カスタム入力"
  read -rp "  選択 [1-${i}] (Enter でデフォルト): " sel
  case "$sel" in
    1|"") LOCALE="ja_JP.UTF-8" ;;
    2)    LOCALE="en_US.UTF-8" ;;
    3)    LOCALE="zh_CN.UTF-8" ;;
    4)    LOCALE="ko_KR.UTF-8" ;;
    5)
      read -rp "  ロケール名を入力 (例: en_GB.UTF-8): " input
      [[ -n "$input" ]] && LOCALE="$input"
      ;;
  esac
  success "ロケール: ${LOCALE}"

  # ── タイムゾーン ──────────────────────────────────────────────────────────
  echo
  step "タイムゾーンの設定"
  local tz_list=("Asia/Tokyo" "UTC" "America/New_York" "America/Los_Angeles" "Europe/London" "Europe/Berlin")
  i=1
  for tz in "${tz_list[@]}"; do
    if [[ "$i" -eq 1 ]]; then
      echo -e "  ${BOLD}${i})${RESET} ${tz}  ${DIM}[デフォルト]${RESET}"
    else
      echo -e "  ${BOLD}${i})${RESET} ${tz}"
    fi
    (( i++ ))
  done
  echo -e "  ${BOLD}${i})${RESET} カスタム入力"
  read -rp "  選択 [1-${i}] (Enter でデフォルト): " sel
  case "$sel" in
    1|"") TIMEZONE="Asia/Tokyo" ;;
    2)    TIMEZONE="UTC" ;;
    3)    TIMEZONE="America/New_York" ;;
    4)    TIMEZONE="America/Los_Angeles" ;;
    5)    TIMEZONE="Europe/London" ;;
    6)    TIMEZONE="Europe/Berlin" ;;
    7)
      read -rp "  タイムゾーンを入力 (例: Asia/Seoul): " input
      [[ -n "$input" ]] && TIMEZONE="$input"
      ;;
  esac
  success "タイムゾーン: ${TIMEZONE}"

  # ── SSH ──────────────────────────────────────────────────────────────────
  echo
  step "SSH の設定"
  read -rp "  SSH を有効にしますか？ [y/N]: " input
  if [[ "$input" =~ ^[Yy]$ ]]; then
    SSH_ENABLED="true"
    success "SSH: 有効"
  else
    SSH_ENABLED="false"
    info "SSH: 無効"
  fi

  # ── nix-auto-storage ──────────────────────────────────────────────────────
  echo
  step "ストレージの設定"
  echo -e "  ${DIM}非ブートドライブが存在する場合、/nix をそこへ配置します。${RESET}"
  read -rp "  nix-auto-storage を有効にしますか？ [y/N]: " input
  if [[ "$input" =~ ^[Yy]$ ]]; then
    STORAGE_ENABLED="true"
    success "nix-auto-storage: 有効"
  else
    STORAGE_ENABLED="false"
    info "nix-auto-storage: 無効"
  fi

  # ── GPU 確認 ─────────────────────────────────────────────────────────────
  echo
  step "GPU の設定"
  echo -e "  検出された GPU: ${BOLD}${GPU_BRAND}${RESET} (${GPU_TYPE})"
  echo
  local gpu_opts=("nvidia:NVIDIA proprietary ドライバー" "amd:AMD open-source ドライバー" "intel:Intel modesetting ドライバー" "none:GPU なし / 仮想マシン")
  i=1
  for opt in "${gpu_opts[@]}"; do
    local key="${opt%%:*}"
    local label="${opt##*:}"
    if [[ "$key" == "$GPU_TYPE" ]]; then
      echo -e "  ${BOLD}${i})${RESET} ${key}  ${label}  ${GREEN}[検出値 ✓]${RESET}"
    else
      echo -e "  ${BOLD}${i})${RESET} ${key}  ${label}"
    fi
    (( i++ ))
  done
  read -rp "  選択 [1-4] (Enter で検出値を使用): " sel
  case "$sel" in
    1) GPU_TYPE="nvidia" ;;
    2) GPU_TYPE="amd" ;;
    3) GPU_TYPE="intel" ;;
    4) GPU_TYPE="none" ;;
    "") : ;; # 検出値を維持
  esac
  success "GPU: ${GPU_TYPE}"

  # ── CPU 確認 ─────────────────────────────────────────────────────────────
  echo
  step "CPU の設定"
  echo -e "  検出された CPU: ${BOLD}${CPU_BRAND}${RESET} (${CPU_TYPE})"
  echo
  local cpu_opts=("amd:AMD マイクロコード" "intel:Intel マイクロコード" "aarch64:ARM64 (マイクロコード不要)")
  i=1
  for opt in "${cpu_opts[@]}"; do
    local key="${opt%%:*}"
    local label="${opt##*:}"
    if [[ "$key" == "$CPU_TYPE" ]]; then
      echo -e "  ${BOLD}${i})${RESET} ${key}  ${label}  ${GREEN}[検出値 ✓]${RESET}"
    else
      echo -e "  ${BOLD}${i})${RESET} ${key}  ${label}"
    fi
    (( i++ ))
  done
  read -rp "  選択 [1-3] (Enter で検出値を使用): " sel
  case "$sel" in
    1) CPU_TYPE="amd";     ARCH="x86_64-linux" ;;
    2) CPU_TYPE="intel";   ARCH="x86_64-linux" ;;
    3) CPU_TYPE="aarch64"; ARCH="aarch64-linux" ;;
    "") : ;; # 検出値を維持
  esac
  success "CPU: ${CPU_TYPE}"

  echo
  success "Phase 1 完了: PC の設定が確定しました。"
}

# ─────────────────────────────────────────────────────────────────────────────
# GUI プログラム選択 (GUI アプリのみ・チェックボックス式)
# 引数: 選択結果を格納する名前参照変数 (bash nameref)
# ─────────────────────────────────────────────────────────────────────────────
select_programs_gui() {
  local -n _result_ref=$1

  # プログラム定義: "module_name:説明"
  local -a GUI_PROG_DEFS=(
    "browser:Chromium ウェブブラウザー"
    "gaming:Steam + Lutris + Wine (ゲーム)"
    "media:Spotify + mpv (音楽・動画)"
    "sns:Discord (SNS チャット)"
    "kicad:KiCad (電気・電子設計)"
    "freecad:FreeCAD + MeshLab (3D CAD)"
    "zed:Zed エディター (unstable 最新版)"
  )

  # 選択状態 (true/false の配列)
  local -a selected=()
  for _ in "${GUI_PROG_DEFS[@]}"; do
    selected+=(false)
  done

  local draw_menu
  draw_menu() {
    echo
    sub_banner "GUI アプリの選択"
    echo -e "  ${DIM}番号を入力して選択/解除、0 で確定${RESET}\n"
    echo -e "  ${MAGENTA}[★]${RESET} ${BOLD}desktop${RESET}      Niri デスクトップ環境 ${DIM}(必須・変更不可)${RESET}"
    local j
    for (( j=0; j<${#GUI_PROG_DEFS[@]}; j++ )); do
      local prog="${GUI_PROG_DEFS[$j]%%:*}"
      local desc="${GUI_PROG_DEFS[$j]##*:}"
      local num=$(( j + 1 ))
      if [[ "${selected[$j]}" == "true" ]]; then
        echo -e "  ${GREEN}[x]${RESET} ${BOLD}${num}.${RESET} ${prog}  ${DIM}${desc}${RESET}"
      else
        echo -e "  ${DIM}[ ]${RESET} ${BOLD}${num}.${RESET} ${prog}  ${DIM}${desc}${RESET}"
      fi
    done
    echo
  }

  while true; do
    draw_menu
    local input
    read -rp "  番号を入力 (0 = 確定): " input
    if [[ "$input" == "0" ]]; then
      break
    elif [[ "$input" =~ ^[0-9]+$ ]] && (( input >= 1 && input <= ${#GUI_PROG_DEFS[@]} )); then
      local idx=$(( input - 1 ))
      if [[ "${selected[$idx]}" == "true" ]]; then
        selected[$idx]=false
      else
        selected[$idx]=true
      fi
    else
      warn "有効な番号を入力してください (0-${#GUI_PROG_DEFS[@]})。"
    fi
  done

  # 結果を構築 (desktop は必須で先頭)
  _result_ref=("desktop")
  local j
  for (( j=0; j<${#GUI_PROG_DEFS[@]}; j++ )); do
    if [[ "${selected[$j]}" == "true" ]]; then
      _result_ref+=("${GUI_PROG_DEFS[$j]%%:*}")
    fi
  done
}

# ─────────────────────────────────────────────────────────────────────────────
# 開発ツール選択 (GUI / CUI 共通・チェックボックス式)
# ─────────────────────────────────────────────────────────────────────────────
select_programs_dev() {
  local -n _result_ref=$1

  local -a DEV_PROG_DEFS=(
    "programming:シェル設定 (Zsh / Nushell / Direnv)"
    "lang:言語ツールチェーン (Rust / C++ / Python)"
    "nix-tools:Nix エコシステム (nix-index / devenv / nil)"
    "cli-tools:汎用 CLI ツール (git / xh / jaq / just)"
  )
  local -a selected=()
  for _ in "${DEV_PROG_DEFS[@]}"; do
    selected+=(false)
  done

  local draw_menu
  draw_menu() {
    echo
    sub_banner "開発ツールの選択 (GUI / CUI 共通)"
    echo -e "  ${DIM}番号を入力して選択/解除、0 で確定${RESET}\n"
    local j
    for (( j=0; j<${#DEV_PROG_DEFS[@]}; j++ )); do
      local prog="${DEV_PROG_DEFS[$j]%%:*}"
      local desc="${DEV_PROG_DEFS[$j]##*:}"
      local num=$(( j + 1 ))
      if [[ "${selected[$j]}" == "true" ]]; then
        echo -e "  ${GREEN}[x]${RESET} ${BOLD}${num}.${RESET} ${prog}  ${DIM}${desc}${RESET}"
      else
        echo -e "  ${DIM}[ ]${RESET} ${BOLD}${num}.${RESET} ${prog}  ${DIM}${desc}${RESET}"
      fi
    done
    echo
  }

  while true; do
    draw_menu
    local input
    read -rp "  番号を入力 (0 = 確定): " input
    if [[ "$input" == "0" ]]; then
      break
    elif [[ "$input" =~ ^[0-9]+$ ]] && (( input >= 1 && input <= ${#DEV_PROG_DEFS[@]} )); then
      local idx=$(( input - 1 ))
      if [[ "${selected[$idx]}" == "true" ]]; then
        selected[$idx]=false
      else
        selected[$idx]=true
      fi
    else
      warn "有効な番号を入力してください。"
    fi
  done

  _result_ref=()
  local j
  for (( j=0; j<${#DEV_PROG_DEFS[@]}; j++ )); do
    if [[ "${selected[$j]}" == "true" ]]; then
      _result_ref+=("${DEV_PROG_DEFS[$j]%%:*}")
    fi
  done
}

# ─────────────────────────────────────────────────────────────────────────────
# カスタムユーザー追加
# ─────────────────────────────────────────────────────────────────────────────
add_custom_user() {
  echo
  sub_banner "カスタムユーザーの設定"

  # ユーザー名
  local uname=""
  while true; do
    read -rp "  ユーザー名を入力してください: " uname
    if [[ -z "$uname" ]]; then
      warn "ユーザー名を入力してください。"
      continue
    fi
    # 重複チェック
    local dup=false
    for existing in "${USER_MODULE_NAMES[@]:-}"; do
      if [[ "$existing" == "$uname" ]]; then
        dup=true
        break
      fi
    done
    if [[ "$dup" == "true" ]]; then
      warn "${uname} はすでに追加されています。別の名前を入力してください。"
      continue
    fi
    break
  done

  # 説明
  local default_desc
  default_desc="$(echo "${uname:0:1}" | tr '[:lower:]' '[:upper:]')${uname:1}"
  local udesc
  read -rp "  説明を入力してください (デフォルト: ${default_desc}): " udesc
  [[ -z "$udesc" ]] && udesc="$default_desc"

  # ユーザータイプ
  local utype=""
  echo
  echo -e "  ${BOLD}ユーザータイプを選択してください:${RESET}"
  echo -e "  ${BOLD}1)${RESET} GUI  デスクトップ環境あり (Niri / Wayland)"
  echo -e "  ${BOLD}2)${RESET} CUI  ターミナルのみ"
  while true; do
    read -rp "  選択 [1-2]: " sel
    case "$sel" in
      1) utype="gui"; break ;;
      2) utype="cui"; break ;;
      *) warn "1 または 2 を入力してください。" ;;
    esac
  done

  # プログラム選択
  # GUI ユーザー: GUI アプリ → 開発ツール の順に選択
  # CUI ユーザー: 開発ツールのみ選択
  local -a gui_progs=()
  local -a dev_progs=()

  if [[ "$utype" == "gui" ]]; then
    select_programs_gui gui_progs
    HAS_GUI_USER=true
  fi

  select_programs_dev dev_progs

  # 全プログラムを結合
  local -a uprograms=()
  uprograms=("${gui_progs[@]:-}" "${dev_progs[@]:-}")

  # programming が選択されているか確認 (NixOS 側で nix-ld を有効化するため)
  for p in "${uprograms[@]:-}"; do
    if [[ "$p" == "programming" ]]; then
      NEEDS_PROGRAMMING_CLI=true
      break
    fi
  done

  # CUSTOM_USERS に追加 (format: "username:type:description:prog1 prog2 ...")
  local prog_str="${uprograms[*]:-}"
  CUSTOM_USERS+=("${uname}:${utype}:${udesc}:${prog_str}")
  USER_MODULE_NAMES+=("$uname")

  success "${uname} を追加しました。(${utype} / プログラム: ${prog_str:-なし})"
}

# ─────────────────────────────────────────────────────────────────────────────
# Phase 2: ユーザーの設定
# ─────────────────────────────────────────────────────────────────────────────
phase2_user_config() {
  echo
  banner "ステップ 2/3: ユーザーの設定"

  while true; do
    # 現在のユーザー一覧
    if [[ ${#USER_MODULE_NAMES[@]} -gt 0 ]]; then
      echo -e "  ${DIM}追加済みユーザー: ${USER_MODULE_NAMES[*]}${RESET}"
      echo
    fi

    # 選択肢
    echo -e "  ${BOLD}ユーザーを追加してください:${RESET}"

    if [[ "$JADE_SELECTED" == "true" ]]; then
      echo -e "  ${DIM}1) jade   通常ユーザー (GUI デスクトップ)  [追加済み ✓]${RESET}"
    else
      echo -e "  ${BOLD}1)${RESET} jade   通常ユーザー (GUI デスクトップ)  ${DIM}[デフォルト設定]${RESET}"
    fi

    if [[ "$ADMIN_SELECTED" == "true" ]]; then
      echo -e "  ${DIM}2) admin  管理者 (CUI のみ)                [追加済み ✓]${RESET}"
    else
      echo -e "  ${BOLD}2)${RESET} admin  管理者 (CUI のみ)                ${DIM}[デフォルト設定]${RESET}"
    fi

    echo -e "  ${BOLD}3)${RESET} カスタムユーザーを追加"

    if [[ ${#USER_MODULE_NAMES[@]} -gt 0 ]]; then
      echo -e "  ${BOLD}0)${RESET} ユーザー設定を完了する"
    fi
    echo

    local sel
    read -rp "  選択 [0-3]: " sel

    case "$sel" in
      1)
        if [[ "$JADE_SELECTED" == "true" ]]; then
          warn "jade はすでに追加されています。"
        else
          JADE_SELECTED=true
          HAS_GUI_USER=true
          USER_MODULE_NAMES+=("jade")
          success "jade を追加しました。(既存の modules/users/jade/jade.nix を使用)"
        fi
        ;;
      2)
        if [[ "$ADMIN_SELECTED" == "true" ]]; then
          warn "admin はすでに追加されています。"
        else
          ADMIN_SELECTED=true
          USER_MODULE_NAMES+=("admin")
          success "admin を追加しました。(既存の modules/users/admin/nixos.nix を使用)"
        fi
        ;;
      3)
        add_custom_user
        ;;
      0)
        if [[ ${#USER_MODULE_NAMES[@]} -eq 0 ]]; then
          warn "少なくとも 1 人のユーザーを追加してください。"
          continue
        fi
        break
        ;;
      *)
        warn "有効な選択肢を入力してください。"
        continue
        ;;
    esac

    echo
    local more
    read -rp "  更にユーザーを追加しますか？ [y/N]: " more
    if [[ ! "$more" =~ ^[Yy]$ ]]; then
      if [[ ${#USER_MODULE_NAMES[@]} -eq 0 ]]; then
        warn "少なくとも 1 人のユーザーを追加してください。"
      else
        break
      fi
    fi
    echo
  done

  echo
  success "Phase 2 完了: ユーザー設定が確定しました。 (${USER_MODULE_NAMES[*]})"
}

# ─────────────────────────────────────────────────────────────────────────────
# Nix ファイル生成: ホスト設定
# ─────────────────────────────────────────────────────────────────────────────
generate_host_config() {
  local target_dir="${MOUNT_ROOT}/etc/nixos/modules/hosts/${HOSTNAME}"
  mkdir -p "$target_dir"

  # imports リスト構築
  local imports_lines=""
  if [[ "$HAS_GUI_USER" == "true" ]]; then
    # GUI ホスト: デスクトップ環境の全依存を展開
    imports_lines="      system-base   # コアシステム (ブート / ネットワーク / Nix 設定 / GC)"
    imports_lines+=$'\n'"      home-manager  # Home Manager 統合"
    imports_lines+=$'\n'"      locale        # ロケール / フォント"
    imports_lines+=$'\n'"      fcitx5        # 日本語入力"
    imports_lines+=$'\n'"      audio         # PipeWire"
    imports_lines+=$'\n'"      desktop       # Niri / greetd / XDG Portal"
  else
    # CUI ホスト: 最小構成
    imports_lines="      system-base   # コアシステム (ブート / ネットワーク / Nix 設定 / GC)"
    imports_lines+=$'\n'"      home-manager  # Home Manager 統合"
    imports_lines+=$'\n'"      locale        # ロケール / フォント"
  fi
  if [[ "$STORAGE_ENABLED" == "true" ]]; then
    imports_lines+=$'\n'"      storage"
  fi
  if [[ "$NEEDS_PROGRAMMING_CLI" == "true" ]]; then
    imports_lines+=$'\n'"      programming   # nix-ld (パッチなし ELF 実行)"
  fi
  for u in "${USER_MODULE_NAMES[@]}"; do
    imports_lines+=$'\n'"      ${u}"
  done

  # SSH 値
  local ssh_val="$SSH_ENABLED"

  cat > "${target_dir}/configuration.nix" <<EOF
{ inputs, ... }:
{
  # ${HOSTNAME}: ${HOSTNAME} ホストの設定。
  # setup.sh によって自動生成されました。
  flake.modules.nixos.${HOSTNAME} = { lib, ... }: {
    imports = with inputs.self.modules.nixos; [
${imports_lines}
    ] ++ [ "\${inputs.self}/nixos/${HOSTNAME}/hardware-configuration.nix" ];

    networking.hostName = "${HOSTNAME}";

    # ハードウェア設定
    my.hardware.gpu = lib.mkDefault "${GPU_TYPE}";
    my.hardware.cpu = lib.mkDefault "${CPU_TYPE}";

    # タイムゾーン
    time.timeZone = "${TIMEZONE}";

    # キーボードレイアウト
    console.keyMap = "${KEYBOARD}";

    # ロケール
    i18n.defaultLocale = "${LOCALE}";

    # SSH
    services.openssh.enable = ${ssh_val};
  };
}
EOF

  success "ホスト設定を生成しました: modules/hosts/${HOSTNAME}/configuration.nix"
}

generate_host_flake_parts() {
  local target_dir="${MOUNT_ROOT}/etc/nixos/modules/hosts/${HOSTNAME}"
  mkdir -p "$target_dir"

  cat > "${target_dir}/flake-parts.nix" <<EOF
{ inputs, ... }:
{
  # ${HOSTNAME} を nixosConfigurations に登録する。
  # 使い方: sudo nixos-rebuild switch --flake .#${HOSTNAME}
  flake.nixosConfigurations = inputs.self.lib.mkNixos "${ARCH}" "${HOSTNAME}";
}
EOF

  success "ホスト flake-parts を生成しました: modules/hosts/${HOSTNAME}/flake-parts.nix"
}

# ─────────────────────────────────────────────────────────────────────────────
# Nix ファイル生成: GUI カスタムユーザー
# ─────────────────────────────────────────────────────────────────────────────
generate_gui_user_nix() {
  local uname="$1"
  local udesc="$2"
  local uprogs="$3"   # スペース区切りのプログラム名

  local target_dir="${MOUNT_ROOT}/etc/nixos/modules/users/${uname}"
  mkdir -p "$target_dir"

  # imports リスト構築 (with ブロック用)
  local import_lines=""
  for p in $uprogs; do
    import_lines+=$'\n'"      ${p}"
  done

  cat > "${target_dir}/${uname}.nix" <<EOF
{ inputs, ... }:
let
  username = "${uname}";
in
{
  # ${uname} (NixOS): ユーザー定義と Home Manager 統合。
  # setup.sh によって自動生成されました。
  flake.modules.nixos."\${username}" = { pkgs, ... }: {
    users.users."\${username}" = {
      isNormalUser = true;
      description  = "${udesc}";
      extraGroups  = [
        "wheel"
        "networkmanager"
        "audio"
        "video"
        "input"
        "seat"
      ];
      shell = pkgs.zsh;
    };

    programs.zsh.enable = true;

    home-manager.users."\${username}" = {
      imports = [ inputs.self.modules.homeManager."\${username}" ];
    };
  };

  # ${uname} (Home Manager): デスクトップ環境設定。
  flake.modules.homeManager."\${username}" = { pkgs, ... }: {
    imports = with inputs.self.modules.homeManager; [
${import_lines}
    ];

    home.username      = "\${username}";
    home.homeDirectory = "/home/\${username}";
    home.stateVersion  = "${STATE_VERSION}";

    programs.home-manager.enable = true;
  };
}
EOF

  success "GUI ユーザー設定を生成しました: modules/users/${uname}/${uname}.nix"
}

# ─────────────────────────────────────────────────────────────────────────────
# Nix ファイル生成: CUI カスタムユーザー (programming-cli あり)
# ─────────────────────────────────────────────────────────────────────────────
generate_cui_user_with_hm_nix() {
  local uname="$1"
  local udesc="$2"

  local target_dir="${MOUNT_ROOT}/etc/nixos/modules/users/${uname}"
  mkdir -p "$target_dir"

  cat > "${target_dir}/nixos.nix" <<EOF
{ inputs, ... }:
let
  username = "${uname}";
in
{
  # ${uname} (NixOS): CUI ユーザーの定義。
  # setup.sh によって自動生成されました。
  flake.modules.nixos."\${username}" = { pkgs, ... }: {
    users.users."\${username}" = {
      isNormalUser = true;
      description  = "${udesc}";
      extraGroups  = [
        "wheel"
        "networkmanager"
      ];
      shell = pkgs.zsh;
    };

    programs.zsh.enable = true;

    home-manager.users."\${username}" = {
      imports = [ inputs.self.modules.homeManager."\${username}" ];
    };
  };

  # ${uname} (Home Manager): CLI 開発環境設定。
  flake.modules.homeManager."\${username}" = { ... }: {
    imports = [
      inputs.self.modules.homeManager."programming"
    ];

    home.username      = "\${username}";
    home.homeDirectory = "/home/\${username}";
    home.stateVersion  = "${STATE_VERSION}";

    programs.home-manager.enable = true;
  };
}
EOF

  success "CUI ユーザー設定を生成しました: modules/users/${uname}/nixos.nix"
}

# ─────────────────────────────────────────────────────────────────────────────
# Nix ファイル生成: CUI カスタムユーザー (プログラムなし)
# ─────────────────────────────────────────────────────────────────────────────
generate_cui_user_minimal_nix() {
  local uname="$1"
  local udesc="$2"

  local target_dir="${MOUNT_ROOT}/etc/nixos/modules/users/${uname}"
  mkdir -p "$target_dir"

  cat > "${target_dir}/nixos.nix" <<EOF
{ ... }:
let
  username = "${uname}";
in
{
  # ${uname} (NixOS): CUI ユーザーの定義 (Home Manager なし)。
  # setup.sh によって自動生成されました。
  flake.modules.nixos."\${username}" = { pkgs, ... }: {
    users.users."\${username}" = {
      isNormalUser = true;
      description  = "${udesc}";
      extraGroups  = [
        "wheel"
        "networkmanager"
      ];
      shell = pkgs.zsh;
    };

    programs.zsh.enable = true;
  };
}
EOF

  success "CUI ユーザー設定を生成しました: modules/users/${uname}/nixos.nix"
}

# ─────────────────────────────────────────────────────────────────────────────
# Phase 3: インストール
# ─────────────────────────────────────────────────────────────────────────────
phase3_install() {
  echo
  banner "ステップ 3/3: インストール"

  # ── 設定サマリー ──────────────────────────────────────────────────────────
  step "設定サマリー"
  echo -e "  ホスト名             : ${BOLD}${HOSTNAME}${RESET}"
  echo -e "  インストール先       : ${BOLD}${DEVICE}${RESET}"
  echo -e "  EFI 終了位置         : ${BOOT_END}"
  echo -e "  ルート終了位置       : ${ROOT_END}"
  echo -e "  GPU                  : ${GPU_TYPE}"
  echo -e "  CPU                  : ${CPU_TYPE}"
  echo -e "  アーキテクチャ       : ${ARCH}"
  echo -e "  キーボード           : ${KEYBOARD}"
  echo -e "  ロケール             : ${LOCALE}"
  echo -e "  タイムゾーン         : ${TIMEZONE}"
  echo -e "  SSH                  : ${SSH_ENABLED}"
  echo -e "  nix-auto-storage     : ${STORAGE_ENABLED}"
  echo -e "  ユーザー             : ${BOLD}${USER_MODULE_NAMES[*]}${RESET}"
  echo
  echo -e "  ${RED}${BOLD}警告: ${DEVICE} のすべてのデータが消去されます。${RESET}"
  echo

  local confirm
  read -rp "  インストールを開始しますか？ [yes/N]: " confirm
  if [[ "$confirm" != "yes" ]]; then
    info "インストールを中断しました。"
    exit 0
  fi

  # ── パーティション作成 ────────────────────────────────────────────────────
  step "パーティション作成"
  parted -s "$DEVICE" mklabel gpt
  parted -s "$DEVICE" mkpart ESP fat32 1MiB "$BOOT_END"
  parted -s "$DEVICE" set 1 esp on
  parted -s "$DEVICE" mkpart nixos ext4 "$BOOT_END" "$ROOT_END"
  parted -s "$DEVICE" mkpart swap linux-swap "$ROOT_END" 100%
  success "GPT パーティションテーブルを作成しました"

  step "ファイルシステム作成"
  mkfs.fat -F 32 -n boot "$PART_BOOT"
  mkfs.ext4 -L nixos -F "$PART_ROOT"
  mkswap -L swap "$PART_SWAP"
  success "ファイルシステムを作成しました"

  # ── マウント ─────────────────────────────────────────────────────────────
  step "マウント"
  mount /dev/disk/by-label/nixos "$MOUNT_ROOT"
  mkdir -p "${MOUNT_ROOT}/boot"
  mount /dev/disk/by-label/boot "${MOUNT_ROOT}/boot"
  swapon /dev/disk/by-label/swap
  success "マウント完了 (root=${MOUNT_ROOT})"

  # ── リポジトリをセットアップ ──────────────────────────────────────────────
  step "リポジトリを配置"
  mkdir -p "${MOUNT_ROOT}/etc/nixos"
  cd "${MOUNT_ROOT}/etc/nixos"
  git init
  git remote add origin "$REPO_URL"
  git fetch origin
  git checkout -t origin/main
  success "リポジトリを配置しました (${MOUNT_ROOT}/etc/nixos)"

  # ── ハードウェア設定を生成 ───────────────────────────────────────────────
  step "hardware-configuration.nix を生成"
  mkdir -p "${MOUNT_ROOT}/etc/nixos/nixos/${HOSTNAME}"
  nixos-generate-config \
    --root "$MOUNT_ROOT" \
    --dir "${MOUNT_ROOT}/etc/nixos/nixos/${HOSTNAME}"
  # nixos-generate-config が生成する configuration.nix は使わないので削除
  rm -f "${MOUNT_ROOT}/etc/nixos/nixos/${HOSTNAME}/configuration.nix"
  success "hardware-configuration.nix を生成しました: nixos/${HOSTNAME}/"

  # ── ホスト設定ファイルを生成 ─────────────────────────────────────────────
  step "ホスト設定ファイルを生成"
  generate_host_config
  generate_host_flake_parts

  # ── カスタムユーザーファイルを生成 ──────────────────────────────────────
  if [[ ${#CUSTOM_USERS[@]} -gt 0 ]]; then
    step "カスタムユーザー設定ファイルを生成"
    for entry in "${CUSTOM_USERS[@]}"; do
      # "username:type:description:prog1 prog2 ..." を分解
      local uname utype udesc uprogs_str
      uname="${entry%%:*}"
      local rest="${entry#*:}"
      utype="${rest%%:*}"
      rest="${rest#*:}"
      udesc="${rest%%:*}"
      uprogs_str="${rest#*:}"

      if [[ "$utype" == "gui" ]]; then
        generate_gui_user_nix "$uname" "$udesc" "$uprogs_str"
      else
        # programming が含まれているか確認
        if echo "$uprogs_str" | grep -qw "programming"; then
          generate_cui_user_with_hm_nix "$uname" "$udesc"
        else
          generate_cui_user_minimal_nix "$uname" "$udesc"
        fi
      fi
    done
  fi

  # ── NetworkManager 接続プロファイルを引き継ぎ ────────────────────────────
  local NM_SRC="/etc/NetworkManager/system-connections"
  local NM_DST="${MOUNT_ROOT}/etc/NetworkManager/system-connections"
  if [[ -d "$NM_SRC" ]] && [[ -n "$(ls -A "$NM_SRC" 2>/dev/null)" ]]; then
    step "NetworkManager 接続プロファイルを引き継ぎ"
    mkdir -p "$NM_DST"
    cp -r "${NM_SRC}/." "$NM_DST/"
    chmod 700 "$NM_DST"
    chmod 600 "$NM_DST"/* 2>/dev/null || true
    success "接続プロファイルをコピーしました"
  else
    warn "NetworkManager 接続プロファイルなし。初回起動後に nmtui で設定してください。"
  fi

  # ── flake 用に git add ───────────────────────────────────────────────────
  step "flake 用に git add ."
  cd "${MOUNT_ROOT}/etc/nixos"
  git add .
  success "追跡完了"

  # ── nixos-install ────────────────────────────────────────────────────────
  echo
  info "nixos-install を開始します: .#${HOSTNAME}"
  nixos-install --flake "${MOUNT_ROOT}/etc/nixos#${HOSTNAME}"

  # ── 完了 ─────────────────────────────────────────────────────────────────
  echo
  echo -e "${GREEN}${BOLD}"
  echo "══════════════════════════════════════════════════════════"
  echo "  インストール完了！"
  echo "══════════════════════════════════════════════════════════"
  echo -e "${RESET}"
  echo -e "  次のコマンドで再起動してください: ${BOLD}reboot${RESET}"
  echo
}

# ─────────────────────────────────────────────────────────────────────────────
# メイン
# ─────────────────────────────────────────────────────────────────────────────
main() {
  phase1_pc_config
  phase2_user_config
  phase3_install
}

main "$@"
