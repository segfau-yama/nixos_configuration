#!/usr/bin/env bash
# setup.sh - NixOS interactive setup script
#
# Usage:
#   bash setup.sh
#
# Phases:
#   1. PC configuration (hardware detection / partitioning / GPU / CPU)
#   2. User configuration (default or custom / GUI or CUI / program selection)
#   3. Installation (partition / nixos-install / reboot)

set -euo pipefail

# -- Constants -----------------------------------------------------------------
REPO_URL="https://github.com/segfau-yama/nixos_configuration.git"
MOUNT_ROOT="/mnt"
STATE_VERSION="25.05"

# -- Color definitions ---------------------------------------------------------
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
BLUE='\033[0;34m'
MAGENTA='\033[0;35m'
BOLD='\033[1m'
DIM='\033[2m'
RESET='\033[0m'

# -- Utilities -----------------------------------------------------------------
info()      { echo -e "${CYAN}[INFO]${RESET}  $*"; }
success()   { echo -e "${GREEN}[ OK ]${RESET}  $*"; }
warn()      { echo -e "${YELLOW}[WARN]${RESET}  $*"; }
error()     { echo -e "${RED}[ERR ]${RESET}  $*" >&2; exit 1; }
step()      { echo -e "\n${BOLD}${BLUE}> $*${RESET}"; }
separator() { echo -e "${DIM}--------------------------------------------------------${RESET}"; }

banner() {
  echo -e "${BOLD}${CYAN}"
  echo "=========================================================="
  echo "  $*"
  echo "=========================================================="
  echo -e "${RESET}"
}

sub_banner() {
  echo -e "\n${BOLD}------------------------------------------------------${RESET}"
  echo -e "${BOLD}  $*${RESET}"
  echo -e "${BOLD}------------------------------------------------------${RESET}"
}

# -- Global variables ----------------------------------------------------------
# Phase 1: PC configuration
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
GPU_BRAND="not detected"
CPU_TYPE="amd"
CPU_BRAND="unknown"
ARCH="x86_64-linux"
BOOT_TYPE="systemd-boot"  # "systemd-boot" | "grub"

# Phase 2: User configuration
declare -a USER_MODULE_NAMES=()   # module names to list in host config imports
declare -a CUSTOM_USERS=()        # "username:type:description:prog1 prog2..."
declare -a USER_PASSWORD_HASHES=() # "username:hashed-password"
JADE_SELECTED=false
ADMIN_SELECTED=false
HAS_GUI_USER=false
NEEDS_PROGRAMMING_CLI=false

# -----------------------------------------------------------------------------
# Hardware detection
# -----------------------------------------------------------------------------
detect_hardware() {
  # CPU
  CPU_BRAND=$(grep -m1 'model name' /proc/cpuinfo 2>/dev/null | cut -d: -f2 | xargs || echo "unknown")
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

  # -- VM / paravirtual GPUs (checked FIRST to prevent false AMD/Intel matches)
  # QXL, Virtio GPU, VMware SVGA, VirtualBox, Bochs VBE -- all need GPU_TYPE=none.
  if echo "$gpu_info" | grep -qiE \
       "QXL|Virtio.GPU|VMware.SVGA|VirtualBox.Graphics|Bochs|SVGA.II|paravirtual|Red Hat.*VGA"; then
    GPU_TYPE="none"
    GPU_BRAND=$(echo "$gpu_info" | head -1 \
      | sed -E 's/^[0-9a-f:.]+[[:space:]]+(VGA compatible controller|3D controller|Display controller):[[:space:]]*//' \
      | xargs)
    [[ -z "$GPU_BRAND" ]] && GPU_BRAND="VM / paravirtual GPU"
  elif echo "$gpu_info" | grep -qi "NVIDIA"; then
    GPU_TYPE="nvidia"
    GPU_BRAND=$(echo "$gpu_info" | grep -i "NVIDIA" | head -1 \
      | sed -E 's/.*\[([^]]+)\].*/\1/' \
      | grep -v '^$' || echo "NVIDIA GPU")
  elif echo "$gpu_info" | grep -qiE "AMD|ATI"; then
    GPU_TYPE="amd"
    GPU_BRAND=$(echo "$gpu_info" | grep -iE "AMD|ATI" | head -1 \
      | sed -E 's/.*\[([^]]+)\].*/\1/' \
      | grep -v '^$' || echo "AMD GPU")
  elif echo "$gpu_info" | grep -qi "Intel"; then
    GPU_TYPE="intel"
    GPU_BRAND=$(echo "$gpu_info" | grep -i "Intel" | head -1 \
      | sed -E 's/.*\[([^]]+)\].*/\1/' \
      | grep -v '^$' || echo "Intel GPU")
  elif [[ -n "$gpu_info" ]]; then
    # Other unrecognised VGA device
    GPU_TYPE="none"
    GPU_BRAND=$(echo "$gpu_info" | head -1 \
      | sed -E 's/^[0-9a-f:.]+[[:space:]]+(VGA compatible controller|3D controller|Display controller):[[:space:]]*//' \
      | xargs)
    [[ -z "$GPU_BRAND" ]] && GPU_BRAND="Generic VGA / VM GPU"
  else
    GPU_TYPE="none"
    GPU_BRAND="not detected (no GPU / lspci unavailable)"
  fi

  # Boot type detection
  if [[ -d /sys/firmware/efi/efivars ]]; then
    BOOT_TYPE="systemd-boot"
  else
    BOOT_TYPE="grub"
  fi
}

# -----------------------------------------------------------------------------
# Phase 1: PC configuration
# -----------------------------------------------------------------------------
phase1_pc_config() {
  banner "NixOS Interactive Setup"
  echo -e "  ${BOLD}Step 1/3: PC Configuration${RESET}\n"

  detect_hardware

  # -- Display hardware info --------------------------------------------------
  step "Hardware Detection Results"
  local mem_kb mem_gb
  mem_kb=$(grep MemTotal /proc/meminfo 2>/dev/null | awk '{print $2}' || echo 0)
  mem_gb=$(( mem_kb / 1024 / 1024 ))
  echo -e "  CPU    : ${BOLD}${CPU_BRAND}${RESET} (${CPU_TYPE})"
  echo -e "  GPU    : ${BOLD}${GPU_BRAND}${RESET} (${GPU_TYPE})"
  echo -e "  Memory : ${BOLD}${mem_gb} GB${RESET}"
  echo -e "  Boot   : ${BOLD}${BOOT_TYPE}${RESET}"
  echo

  step "Storage Devices"
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
    error "No block devices found."
  fi

  # -- Device selection ------------------------------------------------------
  echo
  local sel
  while true; do
    read -rp "  Select install target [1-${#disk_names[@]}]: " sel
    if [[ "$sel" =~ ^[0-9]+$ ]] && (( sel >= 1 && sel <= ${#disk_names[@]} )); then
      DEVICE="/dev/${disk_names[$(( sel - 1 ))]}"
      break
    fi
    warn "Please enter a valid number."
  done
  success "Install target: ${DEVICE}"

  # Determine partition names
  if [[ "$DEVICE" =~ nvme|mmcblk ]]; then
    PART_BOOT="${DEVICE}p1"
    PART_ROOT="${DEVICE}p2"
    PART_SWAP="${DEVICE}p3"
  else
    PART_BOOT="${DEVICE}1"
    PART_ROOT="${DEVICE}2"
    PART_SWAP="${DEVICE}3"
  fi

  # -- Partition sizes -------------------------------------------------------
  echo
  step "Partition Configuration"
  local input
  if [[ "$BOOT_TYPE" == "systemd-boot" ]]; then
    read -rp "  EFI partition end (default: ${BOOT_END}): " input
    [[ -n "$input" ]] && BOOT_END="$input"
    read -rp "  Root partition end (default: ${ROOT_END}): " input
    [[ -n "$input" ]] && ROOT_END="$input"
    echo -e "  EFI: ${CYAN}${BOOT_END}${RESET}   Root: ${CYAN}${ROOT_END}${RESET}   Swap: rest"
  else
    read -rp "  Root partition end (default: ${ROOT_END}): " input
    [[ -n "$input" ]] && ROOT_END="$input"
    echo -e "  ${DIM}BIOS mode: biosboot (2MiB fixed)${RESET}   Root: ${CYAN}${ROOT_END}${RESET}   Swap: rest"
  fi

  # -- Hostname --------------------------------------------------------------
  echo
  step "Hostname"
  read -rp "  Hostname (default: ${HOSTNAME}): " input
  [[ -n "$input" ]] && HOSTNAME="$input"
  success "Hostname: ${HOSTNAME}"

  # -- Keyboard layout -------------------------------------------------------
  echo
  step "Keyboard Layout"
  local kb_list=("jp106:Japanese JIS" "us:English US" "de:German" "fr:French")
  local i=1
  for kb in "${kb_list[@]}"; do
    local kb_key="${kb%%:*}"
    local kb_label="${kb##*:}"
    if [[ "$i" -eq 1 ]]; then
      echo -e "  ${BOLD}${i})${RESET} ${kb_key}  ${kb_label}  ${DIM}[default]${RESET}"
    else
      echo -e "  ${BOLD}${i})${RESET} ${kb_key}  ${kb_label}"
    fi
    (( i++ ))
  done
  echo -e "  ${BOLD}${i})${RESET} Custom"
  read -rp "  Select [1-${i}] (Enter for default): " sel
  case "$sel" in
    1|"") KEYBOARD="jp106" ;;
    2)    KEYBOARD="us" ;;
    3)    KEYBOARD="de" ;;
    4)    KEYBOARD="fr" ;;
    5)
      read -rp "  Enter keyboard layout name: " input
      [[ -n "$input" ]] && KEYBOARD="$input"
      ;;
  esac
  success "Keyboard: ${KEYBOARD}"

  # -- Locale ----------------------------------------------------------------
  echo
  step "Locale"
  local lc_list=("ja_JP.UTF-8:Japanese" "en_US.UTF-8:English US" "zh_CN.UTF-8:Chinese Simplified" "ko_KR.UTF-8:Korean")
  i=1
  for lc in "${lc_list[@]}"; do
    local lc_key="${lc%%:*}"
    local lc_label="${lc##*:}"
    if [[ "$i" -eq 1 ]]; then
      echo -e "  ${BOLD}${i})${RESET} ${lc_key}  ${lc_label}  ${DIM}[default]${RESET}"
    else
      echo -e "  ${BOLD}${i})${RESET} ${lc_key}  ${lc_label}"
    fi
    (( i++ ))
  done
  echo -e "  ${BOLD}${i})${RESET} Custom"
  read -rp "  Select [1-${i}] (Enter for default): " sel
  case "$sel" in
    1|"") LOCALE="ja_JP.UTF-8" ;;
    2)    LOCALE="en_US.UTF-8" ;;
    3)    LOCALE="zh_CN.UTF-8" ;;
    4)    LOCALE="ko_KR.UTF-8" ;;
    5)
      read -rp "  Enter locale (e.g. en_GB.UTF-8): " input
      [[ -n "$input" ]] && LOCALE="$input"
      ;;
  esac
  success "Locale: ${LOCALE}"

  # -- Timezone --------------------------------------------------------------
  echo
  step "Timezone"
  local tz_list=("Asia/Tokyo" "UTC" "America/New_York" "America/Los_Angeles" "Europe/London" "Europe/Berlin")
  i=1
  for tz in "${tz_list[@]}"; do
    if [[ "$i" -eq 1 ]]; then
      echo -e "  ${BOLD}${i})${RESET} ${tz}  ${DIM}[default]${RESET}"
    else
      echo -e "  ${BOLD}${i})${RESET} ${tz}"
    fi
    (( i++ ))
  done
  echo -e "  ${BOLD}${i})${RESET} Custom"
  read -rp "  Select [1-${i}] (Enter for default): " sel
  case "$sel" in
    1|"") TIMEZONE="Asia/Tokyo" ;;
    2)    TIMEZONE="UTC" ;;
    3)    TIMEZONE="America/New_York" ;;
    4)    TIMEZONE="America/Los_Angeles" ;;
    5)    TIMEZONE="Europe/London" ;;
    6)    TIMEZONE="Europe/Berlin" ;;
    7)
      read -rp "  Enter timezone (e.g. Asia/Seoul): " input
      [[ -n "$input" ]] && TIMEZONE="$input"
      ;;
  esac
  success "Timezone: ${TIMEZONE}"

  # -- SSH -------------------------------------------------------------------
  echo
  step "SSH"
  read -rp "  Enable SSH? [y/N]: " input
  if [[ "$input" =~ ^[Yy]$ ]]; then
    SSH_ENABLED="true"
    success "SSH: enabled"
  else
    SSH_ENABLED="false"
    info "SSH: disabled"
  fi

  # -- nix-auto-storage ------------------------------------------------------
  echo
  step "Storage Configuration"
  echo -e "  ${DIM}If a non-boot drive exists, /nix will be placed on it.${RESET}"
  read -rp "  Enable nix-auto-storage? [y/N]: " input
  if [[ "$input" =~ ^[Yy]$ ]]; then
    STORAGE_ENABLED="true"
    success "nix-auto-storage: enabled"
  else
    STORAGE_ENABLED="false"
    info "nix-auto-storage: disabled"
  fi

  # -- GPU -------------------------------------------------------------------
  echo
  step "GPU Configuration"
  echo -e "  Detected GPU: ${BOLD}${GPU_BRAND}${RESET} (${GPU_TYPE})"
  echo
  local gpu_opts=("nvidia:NVIDIA proprietary driver" "amd:AMD open-source driver" "intel:Intel modesetting driver" "none:Virtual Machine / basic Mesa graphics")
  i=1
  for opt in "${gpu_opts[@]}"; do
    local key="${opt%%:*}"
    local label="${opt##*:}"
    if [[ "$key" == "$GPU_TYPE" ]]; then
      echo -e "  ${BOLD}${i})${RESET} ${key}  ${label}  ${GREEN}[detected]${RESET}"
    else
      echo -e "  ${BOLD}${i})${RESET} ${key}  ${label}"
    fi
    (( i++ ))
  done
  read -rp "  Select [1-4] (Enter to use detected value): " sel
  case "$sel" in
    1) GPU_TYPE="nvidia" ;;
    2) GPU_TYPE="amd" ;;
    3) GPU_TYPE="intel" ;;
    4) GPU_TYPE="none" ;;
    "") : ;; # keep detected value
  esac
  success "GPU: ${GPU_TYPE}"

  # -- CPU -------------------------------------------------------------------
  echo
  step "CPU Configuration"
  echo -e "  Detected CPU: ${BOLD}${CPU_BRAND}${RESET} (${CPU_TYPE})"
  echo
  local cpu_opts=("amd:AMD microcode" "intel:Intel microcode" "aarch64:ARM64 (no microcode needed)")
  i=1
  for opt in "${cpu_opts[@]}"; do
    local key="${opt%%:*}"
    local label="${opt##*:}"
    if [[ "$key" == "$CPU_TYPE" ]]; then
      echo -e "  ${BOLD}${i})${RESET} ${key}  ${label}  ${GREEN}[detected]${RESET}"
    else
      echo -e "  ${BOLD}${i})${RESET} ${key}  ${label}"
    fi
    (( i++ ))
  done
  read -rp "  Select [1-3] (Enter to use detected value): " sel
  case "$sel" in
    1) CPU_TYPE="amd";     ARCH="x86_64-linux" ;;
    2) CPU_TYPE="intel";   ARCH="x86_64-linux" ;;
    3) CPU_TYPE="aarch64"; ARCH="aarch64-linux" ;;
    "") : ;; # keep detected value
  esac
  success "CPU: ${CPU_TYPE}"

  echo
  success "Phase 1 complete: PC configuration finalized."
}

# -----------------------------------------------------------------------------
# GUI program selection (checkbox style)
# Argument: nameref variable to store results
# -----------------------------------------------------------------------------
select_programs_gui() {
  local -n _result_ref=$1

  # Program definitions: "module_name:description"
  local -a GUI_PROG_DEFS=(
    "browser:Chromium web browser"
    "gaming:Steam + Lutris + Wine (gaming)"
    "media:Spotify + mpv (music & video)"
    "sns:Discord (chat)"
    "kicad:KiCad (EDA / PCB design)"
    "freecad:FreeCAD + MeshLab (3D CAD)"
    "zed:Zed editor (latest via unstable)"
  )

  # Selection state (true/false array)
  local -a selected=()
  for _ in "${GUI_PROG_DEFS[@]}"; do
    selected+=(false)
  done

  local draw_menu
  draw_menu() {
    echo
    sub_banner "Select GUI Applications"
    echo -e "  ${DIM}Enter a number to toggle, 0 to confirm${RESET}\n"
    echo -e "  ${MAGENTA}[*]${RESET} ${BOLD}desktop${RESET}      Niri desktop environment ${DIM}(required)${RESET}"
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
    read -rp "  Enter number (0 = confirm): " input
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
      warn "Please enter a valid number (0-${#GUI_PROG_DEFS[@]})."
    fi
  done

  # Build result (desktop is mandatory and comes first)
  _result_ref=("desktop")
  local j
  for (( j=0; j<${#GUI_PROG_DEFS[@]}; j++ )); do
    if [[ "${selected[$j]}" == "true" ]]; then
      _result_ref+=("${GUI_PROG_DEFS[$j]%%:*}")
    fi
  done
}

# -----------------------------------------------------------------------------
# Development tool selection (GUI / CUI common, checkbox style)
# -----------------------------------------------------------------------------
select_programs_dev() {
  local -n _result_ref=$1

  local -a DEV_PROG_DEFS=(
    "programming:Shell setup (Zsh / Nushell / Direnv)"
    "lang:Language toolchains (Rust / C++ / Python)"
    "nix-tools:Nix ecosystem (nix-index / devenv / nil)"
    "cli-tools:General CLI tools (git / xh / jaq / just)"
  )
  local -a selected=()
  for _ in "${DEV_PROG_DEFS[@]}"; do
    selected+=(false)
  done

  local draw_menu
  draw_menu() {
    echo
    sub_banner "Select Development Tools (GUI / CUI)"
    echo -e "  ${DIM}Enter a number to toggle, 0 to confirm${RESET}\n"
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
    read -rp "  Enter number (0 = confirm): " input
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
      warn "Please enter a valid number."
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

# -----------------------------------------------------------------------------
# User password input
# -----------------------------------------------------------------------------
set_user_password_hash() {
  local uname="$1"
  local pass1=""
  local pass2=""
  local hash=""

  echo
  step "Password for ${uname}"
  while true; do
    read -rsp "  Password: " pass1
    echo
    if [[ -z "$pass1" ]]; then
      warn "Password cannot be empty."
      continue
    fi

    read -rsp "  Confirm password: " pass2
    echo
    if [[ "$pass1" != "$pass2" ]]; then
      warn "Passwords do not match."
      continue
    fi

    break
  done

  if command -v mkpasswd >/dev/null 2>&1; then
    hash="$(mkpasswd -m yescrypt "$pass1")"
  elif command -v openssl >/dev/null 2>&1; then
    hash="$(openssl passwd -6 -stdin <<<"$pass1")"
  else
    error "Neither mkpasswd nor openssl is available to hash the password."
  fi

  USER_PASSWORD_HASHES+=("${uname}:${hash}")
  pass1=""
  pass2=""
  success "Password hash stored for ${uname}."
}

# -----------------------------------------------------------------------------
# Add custom user
# -----------------------------------------------------------------------------
add_custom_user() {
  echo
  sub_banner "Custom User Configuration"

  # Username
  local uname=""
  while true; do
    read -rp "  Enter username: " uname
    if [[ -z "$uname" ]]; then
      warn "Username cannot be empty."
      continue
    fi
    if [[ ! "$uname" =~ ^[a-z_][a-z0-9_-]*$ ]]; then
      warn "Use a Linux username such as 'jade' or 'admin' (lowercase letters, digits, '_' or '-')."
      continue
    fi
    # Duplicate check
    local dup=false
    for existing in "${USER_MODULE_NAMES[@]:-}"; do
      if [[ "$existing" == "$uname" ]]; then
        dup=true
        break
      fi
    done
    if [[ "$dup" == "true" ]]; then
      warn "${uname} is already added. Please choose a different name."
      continue
    fi
    break
  done

  # Display name
  local default_desc
  default_desc="$(echo "${uname:0:1}" | tr '[:lower:]' '[:upper:]')${uname:1}"
  local udesc
  read -rp "  Display name (default: ${default_desc}): " udesc
  [[ -z "$udesc" ]] && udesc="$default_desc"

  # User type
  local utype=""
  echo
  echo -e "  ${BOLD}Select user type:${RESET}"
  echo -e "  ${BOLD}1)${RESET} GUI  With desktop environment (Niri / Wayland)"
  echo -e "  ${BOLD}2)${RESET} CUI  Terminal only"
  while true; do
    read -rp "  Select [1-2]: " sel
    case "$sel" in
      1) utype="gui"; break ;;
      2) utype="cui"; break ;;
      *) warn "Please enter 1 or 2." ;;
    esac
  done

  # Program selection
  # GUI user: GUI apps -> dev tools
  # CUI user: dev tools only
  local -a gui_progs=()
  local -a dev_progs=()

  if [[ "$utype" == "gui" ]]; then
    select_programs_gui gui_progs
    HAS_GUI_USER=true
  fi

  select_programs_dev dev_progs

  # Merge all programs
  local -a uprograms=()
  uprograms=("${gui_progs[@]:-}" "${dev_progs[@]:-}")

  # Check if programming is selected (to enable nix-ld on NixOS side)
  for p in "${uprograms[@]:-}"; do
    if [[ "$p" == "programming" ]]; then
      NEEDS_PROGRAMMING_CLI=true
      break
    fi
  done

  # Add to CUSTOM_USERS (format: "username:type:description:prog1 prog2 ...")
  local prog_str="${uprograms[*]:-}"
  CUSTOM_USERS+=("${uname}:${utype}:${udesc}:${prog_str}")
  USER_MODULE_NAMES+=("$uname")
  set_user_password_hash "$uname"

  success "Added ${uname}. (${utype} / programs: ${prog_str:-none})"
}

# -----------------------------------------------------------------------------
# Phase 2: User configuration
# -----------------------------------------------------------------------------
phase2_user_config() {
  echo
  banner "Step 2/3: User Configuration"

  while true; do
    # Current user list
    if [[ ${#USER_MODULE_NAMES[@]} -gt 0 ]]; then
      echo -e "  ${DIM}Added users: ${USER_MODULE_NAMES[*]}${RESET}"
      echo
    fi

    # Choices
    echo -e "  ${BOLD}Add a user:${RESET}"

    if [[ "$JADE_SELECTED" == "true" ]]; then
      echo -e "  ${DIM}1) jade   standard user (GUI desktop)  [added]${RESET}"
    else
      echo -e "  ${BOLD}1)${RESET} jade   standard user (GUI desktop)  ${DIM}[default config]${RESET}"
    fi

    if [[ "$ADMIN_SELECTED" == "true" ]]; then
      echo -e "  ${DIM}2) admin  administrator (CUI only)      [added]${RESET}"
    else
      echo -e "  ${BOLD}2)${RESET} admin  administrator (CUI only)      ${DIM}[default config]${RESET}"
    fi

    echo -e "  ${BOLD}3)${RESET} Add custom user"

    if [[ ${#USER_MODULE_NAMES[@]} -gt 0 ]]; then
      echo -e "  ${BOLD}0)${RESET} Finish user configuration"
    fi
    echo

    local sel
    read -rp "  Select [0-3]: " sel

    case "$sel" in
      1)
        if [[ "$JADE_SELECTED" == "true" ]]; then
          warn "jade is already added."
        else
          JADE_SELECTED=true
          HAS_GUI_USER=true
          USER_MODULE_NAMES+=("jade")
          set_user_password_hash "jade"
          success "Added jade. (uses existing modules/users/jade/jade.nix)"
        fi
        ;;
      2)
        if [[ "$ADMIN_SELECTED" == "true" ]]; then
          warn "admin is already added."
        else
          ADMIN_SELECTED=true
          USER_MODULE_NAMES+=("admin")
          set_user_password_hash "admin"
          success "Added admin. (uses existing modules/users/admin/nixos.nix)"
        fi
        ;;
      3)
        add_custom_user
        ;;
      0)
        if [[ ${#USER_MODULE_NAMES[@]} -eq 0 ]]; then
          warn "Please add at least one user."
          continue
        fi
        break
        ;;
      *)
        warn "Please enter a valid choice."
        continue
        ;;
    esac

    echo
    local more
    read -rp "  Add another user? [y/N]: " more
    if [[ ! "$more" =~ ^[Yy]$ ]]; then
      if [[ ${#USER_MODULE_NAMES[@]} -eq 0 ]]; then
        warn "Please add at least one user."
      else
        break
      fi
    fi
    echo
  done

  echo
  success "Phase 2 complete: User configuration finalized. (${USER_MODULE_NAMES[*]})"
}

# -----------------------------------------------------------------------------
# Nix file generation: host configuration
# -----------------------------------------------------------------------------
generate_host_config() {
  local target_dir="${MOUNT_ROOT}/etc/nixos/modules/hosts/${HOSTNAME}"
  mkdir -p "$target_dir"

  # Build imports list
  local imports_lines=""
  if [[ "$HAS_GUI_USER" == "true" ]]; then
    # GUI host: expand all desktop dependencies
    imports_lines="      system-base   # core system (boot / network / Nix settings / GC)"
    imports_lines+=$'\n'"      home-manager  # Home Manager integration"
    imports_lines+=$'\n'"      locale        # locale / fonts"
    imports_lines+=$'\n'"      fcitx5        # Japanese input"
    imports_lines+=$'\n'"      audio         # PipeWire"
    imports_lines+=$'\n'"      desktop       # Niri / greetd / XDG Portal"
  else
    # CUI host: minimal setup
    imports_lines="      system-base   # core system (boot / network / Nix settings / GC)"
    imports_lines+=$'\n'"      home-manager  # Home Manager integration"
    imports_lines+=$'\n'"      locale        # locale / fonts"
  fi
  if [[ "$STORAGE_ENABLED" == "true" ]]; then
    imports_lines+=$'\n'"      storage"
  fi
  if [[ "$NEEDS_PROGRAMMING_CLI" == "true" ]]; then
    imports_lines+=$'\n'"      programming   # nix-ld (run unpatched ELF binaries)"
  fi
  for u in "${USER_MODULE_NAMES[@]}"; do
    imports_lines+=$'\n'"      ${u}"
  done

  # SSH value
  local ssh_val="$SSH_ENABLED"

  # Password config block
  local password_nix=""
  local entry uname hash
  for entry in "${USER_PASSWORD_HASHES[@]}"; do
    uname="${entry%%:*}"
    hash="${entry#*:}"
    password_nix+=$'\n'"    users.users.\"${uname}\".initialHashedPassword = \"${hash}\";"
  done

  # Boot loader config block
  local boot_loader_nix
  if [[ "$BOOT_TYPE" == "systemd-boot" ]]; then
    boot_loader_nix="    boot.loader.systemd-boot.enable = true;
    boot.loader.efi.canTouchEfiVariables = true;"
  else
    boot_loader_nix="    boot.loader.systemd-boot.enable = lib.mkForce false;
    boot.loader.efi.canTouchEfiVariables = lib.mkForce false;
    boot.loader.grub = {
      enable  = true;
      device  = \"${DEVICE}\";
      efiSupport = false;
    };"
  fi

  cat > "${target_dir}/configuration.nix" <<EOF
{ inputs, ... }:
{
  # ${HOSTNAME}: host configuration.
  # Auto-generated by setup.sh.
  flake.modules.nixos.${HOSTNAME} = { lib, ... }: {
    imports = with inputs.self.modules.nixos; [
${imports_lines}
    ] ++ [ "\${inputs.self}/nixos/${HOSTNAME}/hardware-configuration.nix" ];

    networking.hostName = "${HOSTNAME}";

    # Hardware
    my.hardware.gpu = lib.mkDefault "${GPU_TYPE}";
    my.hardware.cpu = lib.mkDefault "${CPU_TYPE}";

    # Timezone
    time.timeZone = "${TIMEZONE}";

    # Keyboard layout
    console.keyMap = "${KEYBOARD}";

    # Locale
    i18n.defaultLocale = "${LOCALE}";

    # SSH
    services.openssh.enable = ${ssh_val};

    # User passwords
${password_nix}

    # Boot loader
${boot_loader_nix}
  };
}
EOF

  success "Generated host config: modules/hosts/${HOSTNAME}/configuration.nix"
}

generate_host_flake_parts() {
  local target_dir="${MOUNT_ROOT}/etc/nixos/modules/hosts/${HOSTNAME}"
  mkdir -p "$target_dir"

  cat > "${target_dir}/flake-parts.nix" <<EOF
{ inputs, ... }:
{
  # Register ${HOSTNAME} in nixosConfigurations.
  # Usage: sudo nixos-rebuild switch --flake .#${HOSTNAME}
  flake.nixosConfigurations = inputs.self.lib.mkNixos "${ARCH}" "${HOSTNAME}";
}
EOF

  success "Generated host flake-parts: modules/hosts/${HOSTNAME}/flake-parts.nix"
}

# -----------------------------------------------------------------------------
# Nix file generation: GUI custom user
# -----------------------------------------------------------------------------
generate_gui_user_nix() {
  local uname="$1"
  local udesc="$2"
  local uprogs="$3"   # space-separated program names

  local target_dir="${MOUNT_ROOT}/etc/nixos/modules/users/${uname}"
  mkdir -p "$target_dir"

  # Build imports list (for with block)
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
  # ${uname} (NixOS): user definition and Home Manager integration.
  # Auto-generated by setup.sh.
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

  # ${uname} (Home Manager): desktop environment configuration.
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

  success "Generated GUI user config: modules/users/${uname}/${uname}.nix"
}

# -----------------------------------------------------------------------------
# Nix file generation: CUI custom user (with Home Manager / programming)
# -----------------------------------------------------------------------------
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
  # ${uname} (NixOS): CUI user definition.
  # Auto-generated by setup.sh.
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

  # ${uname} (Home Manager): CLI development environment.
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

  success "Generated CUI user config: modules/users/${uname}/nixos.nix"
}

# -----------------------------------------------------------------------------
# Nix file generation: CUI custom user (no Home Manager)
# -----------------------------------------------------------------------------
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
  # ${uname} (NixOS): CUI user definition (no Home Manager).
  # Auto-generated by setup.sh.
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

  success "Generated CUI user config: modules/users/${uname}/nixos.nix"
}

# -----------------------------------------------------------------------------
# Phase 3: Installation
# -----------------------------------------------------------------------------
phase3_install() {
  echo
  banner "Step 3/3: Installation"

  # -- Configuration summary -------------------------------------------------
  step "Configuration Summary"
  echo -e "  Hostname             : ${BOLD}${HOSTNAME}${RESET}"
  echo -e "  Install target       : ${BOLD}${DEVICE}${RESET}"
  echo -e "  EFI end              : ${BOOT_END}"
  echo -e "  Root end             : ${ROOT_END}"
  echo -e "  Boot loader          : ${BOOT_TYPE}"
  echo -e "  GPU                  : ${GPU_TYPE}"
  echo -e "  CPU                  : ${CPU_TYPE}"
  echo -e "  Architecture         : ${ARCH}"
  echo -e "  Keyboard             : ${KEYBOARD}"
  echo -e "  Locale               : ${LOCALE}"
  echo -e "  Timezone             : ${TIMEZONE}"
  echo -e "  SSH                  : ${SSH_ENABLED}"
  echo -e "  nix-auto-storage     : ${STORAGE_ENABLED}"
  echo -e "  Users                : ${BOLD}${USER_MODULE_NAMES[*]}${RESET}"
  echo
  echo -e "  ${RED}${BOLD}WARNING: ALL DATA on ${DEVICE} will be erased.${RESET}"
  echo

  local confirm
  read -rp "  Type 'yes' to start installation [yes/N]: " confirm
  if [[ "$confirm" != "yes" ]]; then
    info "Installation cancelled."
    exit 0
  fi

  # -- Partitioning ----------------------------------------------------------
  step "Creating Partitions"
  parted -s "$DEVICE" mklabel gpt
  if [[ "$BOOT_TYPE" == "systemd-boot" ]]; then
    # EFI: パーティション1 = ESP/FAT32、パーティション2 = Root、パーティション3 = Swap
    parted -s "$DEVICE" mkpart ESP fat32 1MiB "$BOOT_END"
    parted -s "$DEVICE" set 1 esp on
    parted -s "$DEVICE" mkpart nixos ext4 "$BOOT_END" "$ROOT_END"
    parted -s "$DEVICE" mkpart swap linux-swap "$ROOT_END" 100%
  else
    # BIOS/GPT: パーティション1 = BIOS boot (1MB)、パーティション2 = Root、パーティション3 = Swap
    parted -s "$DEVICE" mkpart grub 1MiB 2MiB
    parted -s "$DEVICE" set 1 bios_grub on
    parted -s "$DEVICE" mkpart nixos ext4 2MiB "$ROOT_END"
    parted -s "$DEVICE" mkpart swap linux-swap "$ROOT_END" 100%
  fi
  success "GPT partition table created"

  step "Formatting Filesystems"
  if [[ "$BOOT_TYPE" == "systemd-boot" ]]; then
    mkfs.fat -F 32 -n boot "$PART_BOOT"
  fi
  mkfs.ext4 -L nixos -F "$PART_ROOT"
  mkswap -L swap "$PART_SWAP"
  success "Filesystems formatted"

  # Wait for udev to create /dev/disk/by-label/* symlinks.
  # Without this, mount by label fails immediately after mkfs.
  udevadm trigger --action=add 2>/dev/null || true
  udevadm settle

  # -- Mounting --------------------------------------------------------------
  step "Mounting"
  mount /dev/disk/by-label/nixos "$MOUNT_ROOT"
  if [[ "$BOOT_TYPE" == "systemd-boot" ]]; then
    mkdir -p "${MOUNT_ROOT}/boot"
    mount /dev/disk/by-label/boot "${MOUNT_ROOT}/boot"
  fi
  swapon /dev/disk/by-label/swap
  success "Mounted (root=${MOUNT_ROOT})"

  # -- Clone repository ------------------------------------------------------
  step "Cloning Repository"
  mkdir -p "${MOUNT_ROOT}/etc/nixos"
  cd "${MOUNT_ROOT}/etc/nixos"
  git init
  git remote add origin "$REPO_URL"
  git fetch origin
  git checkout -t origin/main
  success "Repository cloned to ${MOUNT_ROOT}/etc/nixos"

  # -- Generate hardware configuration ---------------------------------------
  step "Generating hardware-configuration.nix"
  mkdir -p "${MOUNT_ROOT}/etc/nixos/nixos/${HOSTNAME}"
  nixos-generate-config \
    --root "$MOUNT_ROOT" \
    --dir "${MOUNT_ROOT}/etc/nixos/nixos/${HOSTNAME}"
  # Remove configuration.nix generated by nixos-generate-config (not used with flakes)
  rm -f "${MOUNT_ROOT}/etc/nixos/nixos/${HOSTNAME}/configuration.nix"
  success "Generated: nixos/${HOSTNAME}/hardware-configuration.nix"

  # -- Generate host configuration -------------------------------------------
  step "Generating Host Configuration"
  generate_host_config
  generate_host_flake_parts

  # -- Generate custom user configs ------------------------------------------
  if [[ ${#CUSTOM_USERS[@]} -gt 0 ]]; then
    step "Generating Custom User Configurations"
    for entry in "${CUSTOM_USERS[@]}"; do
      # Parse "username:type:description:prog1 prog2 ..."
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
        # Check if programming is included
        if echo "$uprogs_str" | grep -qw "programming"; then
          generate_cui_user_with_hm_nix "$uname" "$udesc"
        else
          generate_cui_user_minimal_nix "$uname" "$udesc"
        fi
      fi
    done
  fi

  # -- Copy NetworkManager profiles ------------------------------------------
  local NM_SRC="/etc/NetworkManager/system-connections"
  local NM_DST="${MOUNT_ROOT}/etc/NetworkManager/system-connections"
  if [[ -d "$NM_SRC" ]] && [[ -n "$(ls -A "$NM_SRC" 2>/dev/null)" ]]; then
    step "Copying NetworkManager Profiles"
    mkdir -p "$NM_DST"
    cp -r "${NM_SRC}/." "$NM_DST/"
    chmod 700 "$NM_DST"
    chmod 600 "$NM_DST"/* 2>/dev/null || true
    success "NetworkManager profiles copied"
  else
    warn "No NetworkManager profiles found. Configure network with nmtui after first boot."
  fi

  # -- git add for flake tracking --------------------------------------------
  step "Tracking files with git add ."
  cd "${MOUNT_ROOT}/etc/nixos"
  git add .
  success "All files tracked"

  # -- nixos-install ---------------------------------------------------------
  echo
  info "Starting nixos-install: .#${HOSTNAME}"
  nixos-install --flake "${MOUNT_ROOT}/etc/nixos#${HOSTNAME}"

  # -- Done ------------------------------------------------------------------
  echo
  echo -e "${GREEN}${BOLD}"
  echo "=========================================================="
  echo "  Installation complete!"
  echo "=========================================================="
  echo -e "${RESET}"
  echo -e "  Reboot with: ${BOLD}reboot${RESET}"
  echo
}

# -----------------------------------------------------------------------------
# Main
# -----------------------------------------------------------------------------
main() {
  phase1_pc_config
  phase2_user_config
  phase3_install
}

main "$@"
