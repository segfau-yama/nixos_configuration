#!/usr/bin/env bash
set -euo pipefail

REPO_URL="${REPO_URL:-https://github.com/segfau-yama/nixos_configuration.git}"
SOURCE_REPO=""

# Minimal Bash installer modeled after jadeos_setting_tui/AGENT.md.
# The Rust TUI owns the full future flow. This script keeps the same safety
# checkpoints for CLI/live-ISO installs.
TARGET_DISK="${TARGET_DISK:-}"
BOOT_PART="${BOOT_PART:-}"
ROOT_PART="${ROOT_PART:-}"
SWAP_PART="${SWAP_PART:-}"
MOUNT_ROOT="${MOUNT_ROOT:-/mnt}"
BOOT_SIZE="${BOOT_SIZE:-512MiB}"
SWAP_SIZE="${SWAP_SIZE:-0}"
YES="${YES:-false}"
DRY_RUN="${DRY_RUN:-false}"
COUNTDOWN_SECONDS="${COUNTDOWN_SECONDS:-5}"

usage() {
  cat <<'EOF'
Usage:
  sudo bash setup.sh

Environment/configuration:
  REPO_URL           repository to clone when setup.sh is not run from a checkout
  TARGET_DISK        optional; when empty, select from lsblk interactively
  BOOT_PART          optional explicit boot partition path
  ROOT_PART          optional explicit root partition path
  SWAP_PART          optional explicit swap partition path
  MOUNT_ROOT         default: /mnt
  BOOT_SIZE          default: 512MiB
  SWAP_SIZE          default: 0 (disabled). Example: 2GiB
  YES=true           skip destructive confirmation prompt
  DRY_RUN=true       print commands without writing or installing

This script:
  1. Checks network connectivity.
  2. Clones/copies the configuration repository.
  3. Selects an install profile from modules/hosts.
  4. Selects a target disk and partition sizes.
  5. Confirms destructive partition/format operations.
  6. Writes nixos/<profile>/install-args.nix and generated hardware config.
  7. Runs nixos-install with the selected flake profile.

It does not create new GitHub repositories or custom user modules. Use the Rust
TUI installer for the full AGENT.md flow.
EOF
}

info() {
  printf '[INFO] %s\n' "$*"
}

warn() {
  printf '[WARN] %s\n' "$*" >&2
}

error() {
  printf '[ERROR] %s\n' "$*" >&2
  exit 1
}

is_true() {
  case "${1:-}" in
    true|1|yes|YES|y|Y) return 0 ;;
    *) return 1 ;;
  esac
}

need_cmd() {
  command -v "$1" >/dev/null 2>&1 || error "Required command not found: $1"
}

run() {
  if is_true "$DRY_RUN"; then
    printf '[DRY-RUN] %q' "$1"
    shift
    for arg in "$@"; do
      printf ' %q' "$arg"
    done
    printf '\n'
  else
    "$@"
  fi
}

prompt_default() {
  local -n out_var=$1
  local label="$2"
  local default_value="$3"
  local value=""

  read -r -p "$label [$default_value]: " value
  out_var="${value:-$default_value}"
}

is_repo_root() {
  local path="$1"
  [[ -f "$path/flake.nix" && -d "$path/modules/hosts" ]]
}

check_network() {
  if is_true "$DRY_RUN"; then
    info "Skipping network check in dry-run."
    return
  fi

  need_cmd ping
  info "Checking network connectivity..."
  ping -c 1 -W 3 8.8.8.8 >/dev/null 2>&1 || error "Network check failed. Connect to the network first."
}

prepare_source_repo() {
  local script_dir
  script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"

  if is_repo_root "$PWD"; then
    SOURCE_REPO="$PWD"
    return
  fi

  if is_repo_root "$script_dir"; then
    SOURCE_REPO="$script_dir"
    return
  fi

  prompt_default REPO_URL "Repository URL" "$REPO_URL"

  SOURCE_REPO="/tmp/nixos_configuration_setup_source"
  if is_repo_root "$SOURCE_REPO"; then
    info "Using setup source repository at $SOURCE_REPO"
    return
  fi

  info "Cloning setup source repository to $SOURCE_REPO"
  if is_true "$DRY_RUN"; then
    info "Would clone $REPO_URL into $SOURCE_REPO"
    error "Cannot discover install profiles in dry-run without a local repository checkout."
  fi

  git clone "$REPO_URL" "$SOURCE_REPO"
}

copy_or_clone_repo() {
  local target_repo="$1"

  run mkdir -p "$target_repo"

  if [[ -n "$SOURCE_REPO" && -f "$SOURCE_REPO/flake.nix" && -d "$SOURCE_REPO/modules" ]]; then
    info "Copying source repository from $SOURCE_REPO to $target_repo"
    if command -v rsync >/dev/null 2>&1; then
      run rsync -a --exclude '/jadeos_setting_tui/target' "$SOURCE_REPO"/ "$target_repo"/
    else
      if is_true "$DRY_RUN"; then
        info "Would copy local repository with tar fallback"
      else
        tar -C "$SOURCE_REPO" --exclude './jadeos_setting_tui/target' -cf - . | tar -C "$target_repo" -xf -
      fi
    fi
  elif [[ -d "$target_repo/.git" && -f "$target_repo/flake.nix" ]]; then
    info "Using existing repository at $target_repo"
  else
    info "Cloning repository to $target_repo"
    if is_true "$DRY_RUN"; then
      info "Would clone $REPO_URL into $target_repo"
    else
      git clone "$REPO_URL" "$target_repo"
    fi
  fi
}

discover_profiles() {
  local -n out_profiles=$1
  local host_dir
  local profile

  out_profiles=()
  while IFS= read -r host_dir; do
    profile="$(basename "$host_dir")"
    [[ -n "$profile" ]] || continue
    [[ "$profile" == ".gitkeep" ]] && continue
    out_profiles+=("$profile")
  done < <(find "$SOURCE_REPO/modules/hosts" -mindepth 1 -maxdepth 1 -type d | sort)

  ((${#out_profiles[@]} > 0)) || error "No valid hosts found in $SOURCE_REPO/modules/hosts"
}

select_profile() {
  local -n out_profile=$1
  local profiles=()
  local i
  local selection=""
  local max

  discover_profiles profiles

  info "Select install profile:"
  for i in "${!profiles[@]}"; do
    printf '  %d) %s\n' "$((i + 1))" "${profiles[$i]}"
  done

  max="${#profiles[@]}"
  while true; do
    read -r -p "Enter number [1-${max}]: " selection
    if [[ "$selection" =~ ^[0-9]+$ ]] && ((selection >= 1 && selection <= max)); then
      out_profile="${profiles[$((selection - 1))]}"
      return
    fi
    warn "Invalid selection: $selection"
  done
}

select_disk() {
  local disks=()
  local line
  local i
  local selection=""
  local default_answer=""

  if [[ -n "$TARGET_DISK" ]]; then
    read -r -p "Use configured target disk $TARGET_DISK? [Y/n]: " default_answer
    case "$default_answer" in
      n|N|no|NO) TARGET_DISK="" ;;
      *) return ;;
    esac
  fi

  need_cmd lsblk
  while IFS= read -r line; do
    [[ -n "$line" ]] || continue
    disks+=("$line")
  done < <(lsblk -dno NAME,SIZE,TYPE,MODEL | awk '$3 == "disk" { print }')

  ((${#disks[@]} > 0)) || error "No installable disks found by lsblk."

  info "Select target disk:"
  for i in "${!disks[@]}"; do
    printf '  %d) /dev/%s\n' "$((i + 1))" "${disks[$i]}"
  done

  while true; do
    read -r -p "Enter number [1-${#disks[@]}]: " selection
    if [[ "$selection" =~ ^[0-9]+$ ]] && ((selection >= 1 && selection <= ${#disks[@]})); then
      TARGET_DISK="/dev/$(awk '{ print $1 }' <<<"${disks[$((selection - 1))]}")"
      return
    fi
    warn "Invalid selection: $selection"
  done
}

validate_size() {
  local value="$1"
  [[ "$value" =~ ^[0-9]+([KMGTP]i?B|[KMGTP]B|%)?$ ]]
}

configure_partitions() {
  prompt_default BOOT_SIZE "EFI boot partition size" "$BOOT_SIZE"
  validate_size "$BOOT_SIZE" || error "Invalid BOOT_SIZE: $BOOT_SIZE"

  prompt_default SWAP_SIZE "Swap partition size (0 disables swap)" "$SWAP_SIZE"
  validate_size "$SWAP_SIZE" || error "Invalid SWAP_SIZE: $SWAP_SIZE"
}

has_swap_partition() {
  [[ -n "$SWAP_PART" ]] && return 0

  case "$SWAP_SIZE" in
    ""|0|0B|0K|0M|0G|0MiB|0GiB|none|false|no) return 1 ;;
    *) return 0 ;;
  esac
}

partition_path() {
  local disk="$1"
  local number="$2"

  if [[ "$disk" =~ (nvme|mmcblk|loop) ]]; then
    printf '%sp%s\n' "$disk" "$number"
  else
    printf '%s%s\n' "$disk" "$number"
  fi
}

derive_partition_paths() {
  PART_BOOT="${BOOT_PART:-$(partition_path "$TARGET_DISK" 1)}"
  PART_ROOT="${ROOT_PART:-$(partition_path "$TARGET_DISK" 2)}"

  if has_swap_partition; then
    PART_SWAP="${SWAP_PART:-$(partition_path "$TARGET_DISK" 3)}"
  else
    PART_SWAP=""
  fi
}

confirm_install() {
  local confirmation=""

  info "Selected profile : $PROFILE"
  info "Target disk      : $TARGET_DISK"
  info "Boot part        : $PART_BOOT ($BOOT_SIZE)"
  info "Root part        : $PART_ROOT (remaining space)"
  if has_swap_partition; then
    info "Swap part        : $PART_SWAP ($SWAP_SIZE from disk end)"
  else
    info "Swap part        : disabled"
  fi
  info "Mount root       : $MOUNT_ROOT"
  info "Install flake    : $MOUNT_ROOT/etc/nixos#$PROFILE"

  if is_true "$DRY_RUN"; then
    info "Dry-run: skipping destructive confirmation prompt."
    return
  fi

  if is_true "$YES"; then
    warn "YES=true: skipping destructive confirmation prompt."
    return
  fi

  warn "This will repartition and format $TARGET_DISK."
  read -r -p "Type INSTALL to continue: " confirmation
  [[ "$confirmation" == "INSTALL" ]] || error "Install cancelled."
}

auto_start_countdown() {
  local seconds="$COUNTDOWN_SECONDS"

  if ((seconds <= 0)); then
    return 0
  fi
  info "Starting install in ${seconds}s. Press Ctrl+C to cancel."
  while ((seconds > 0)); do
    printf '\r[INFO] Starting in %ds... ' "$seconds"
    sleep 1
    ((seconds--))
  done
  printf '\n'
}

partition_disk() {
  run parted -s "$TARGET_DISK" mklabel gpt
  run parted -s "$TARGET_DISK" mkpart ESP fat32 1MiB "$BOOT_SIZE"
  run parted -s "$TARGET_DISK" set 1 esp on

  if has_swap_partition; then
    run parted -s "$TARGET_DISK" -- mkpart nixos ext4 "$BOOT_SIZE" "-$SWAP_SIZE"
    run parted -s "$TARGET_DISK" -- mkpart swap linux-swap "-$SWAP_SIZE" 100%
  else
    run parted -s "$TARGET_DISK" mkpart nixos ext4 "$BOOT_SIZE" 100%
  fi

  if command -v partprobe >/dev/null 2>&1; then
    run partprobe "$TARGET_DISK"
  fi
}

format_and_mount() {
  run mkfs.fat -F 32 -n boot "$PART_BOOT"
  run mkfs.ext4 -L nixos -F "$PART_ROOT"

  if has_swap_partition; then
    run mkswap -L swap "$PART_SWAP"
  fi

  run mount "$PART_ROOT" "$MOUNT_ROOT"
  run mkdir -p "$MOUNT_ROOT/boot"
  run mount "$PART_BOOT" "$MOUNT_ROOT/boot"

  if has_swap_partition; then
    run swapon "$PART_SWAP"
  fi
}

write_install_args() {
  local path="$1"
  local boot_part="$2"
  local root_part="$3"
  local swap_part="$4"

  if is_true "$DRY_RUN"; then
    info "Would write install args: $path (dry-run: no file is created)"
    printf '[DRY-RUN] installDisk.boot = %q\n' "$boot_part"
    printf '[DRY-RUN] installDisk.root = %q\n' "$root_part"
    if [[ -n "$swap_part" ]]; then
      printf '[DRY-RUN] installDisk.swap = %q\n' "$swap_part"
    else
      printf '[DRY-RUN] installDisk.swap = null\n'
    fi
    return
  fi

  if [[ -n "$swap_part" ]]; then
    cat > "$path" <<EOF
{
  installDisk = {
    boot = "$boot_part";
    root = "$root_part";
    swap = "$swap_part";
  };
}
EOF
  else
    cat > "$path" <<EOF
{
  installDisk = {
    boot = "$boot_part";
    root = "$root_part";
    swap = null;
  };
}
EOF
  fi
}

generate_hardware_config() {
  local path="$1"

  if is_true "$DRY_RUN"; then
    info "Would generate hardware config: $path (dry-run: no file is created)"
    printf '[DRY-RUN] nixos-generate-config --root %q --show-hardware-config > %q\n' "$MOUNT_ROOT" "$path"
    return
  fi

  nixos-generate-config --root "$MOUNT_ROOT" --show-hardware-config > "$path"
}

check_prerequisites() {
  if [[ $# -gt 0 ]]; then
    case "$1" in
      -h|--help)
        usage
        exit 0
        ;;
      *)
        error "setup.sh does not accept install options. Use environment variables or interactive prompts."
        ;;
    esac
  fi

  if ! is_true "$DRY_RUN" && [[ "${EUID:-$(id -u)}" -ne 0 ]]; then
    error "Please run as root (use sudo)."
  fi

  need_cmd find
  need_cmd sort
  need_cmd awk
  need_cmd git

  if ! is_true "$DRY_RUN"; then
    for cmd in parted mkfs.fat mkfs.ext4 mkswap mount swapon mkdir nixos-generate-config nixos-install; do
      need_cmd "$cmd"
    done
  fi
}

main() {
  PROFILE=""
  PART_BOOT=""
  PART_ROOT=""
  PART_SWAP=""

  check_prerequisites "$@"
  check_network
  prepare_source_repo
  select_profile PROFILE
  select_disk
  configure_partitions
  derive_partition_paths

  if ! is_true "$DRY_RUN" && [[ ! -b "$TARGET_DISK" ]]; then
    error "Block device not found: $TARGET_DISK"
  fi

  confirm_install
  auto_start_countdown

  local target_repo="$MOUNT_ROOT/etc/nixos"
  local profile_dir="$target_repo/nixos/$PROFILE"
  local install_args="$profile_dir/install-args.nix"
  local generated_hardware="$profile_dir/generated-hardware-configuration.nix"

  partition_disk
  format_and_mount
  copy_or_clone_repo "$target_repo"

  run mkdir -p "$profile_dir"
  write_install_args "$install_args" "$PART_BOOT" "$PART_ROOT" "$PART_SWAP"
  generate_hardware_config "$generated_hardware"
  run git -C "$target_repo" add -f \
    "nixos/$PROFILE/install-args.nix" \
    "nixos/$PROFILE/generated-hardware-configuration.nix"

  info "Installing NixOS from $target_repo#$PROFILE"
  run nixos-install --flake "$target_repo#$PROFILE"

  info "Install finished. Reboot, then set user passwords with passwd."
}

main "$@"
