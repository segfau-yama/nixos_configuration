#!/usr/bin/env bash
set -euo pipefail

REPO_URL="${REPO_URL:-https://github.com/segfau-yama/nixos_configuration.git}"

# Edit this block before running in a minimal CLI environment.
# PROFILE is selected interactively from modules/hosts at runtime.
TARGET_DISK="/dev/vda"
BOOT_PART=""
ROOT_PART=""
SWAP_PART=""
MOUNT_ROOT="/mnt"
BOOT_END="512MiB"
ROOT_END="100GiB"
YES=true
DRY_RUN=false

usage() {
  cat <<'EOF'
Usage:
  sudo bash setup.sh

Configuration:
  Edit the variables near the top of setup.sh before running:
    TARGET_DISK, BOOT_PART, ROOT_PART, SWAP_PART,
    MOUNT_ROOT, BOOT_END, ROOT_END, YES, DRY_RUN.

This script does not generate Nix host or user modules.
The install profile is selected interactively from modules/hosts.
EOF
}

info() {
  printf '[INFO] %s\n' "$*"
}

error() {
  printf '[ERROR] %s\n' "$*" >&2
  exit 1
}

need_cmd() {
  command -v "$1" >/dev/null 2>&1 || error "Required command not found: $1"
}

run() {
  if [[ "$DRY_RUN" == true ]]; then
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

copy_or_clone_repo() {
  local target_repo="$1"

  run mkdir -p "$target_repo"

  if [[ -f ./flake.nix && -d ./modules ]]; then
    info "Copying local repository to $target_repo"
    if command -v rsync >/dev/null 2>&1; then
      run rsync -a --exclude '/jadeos_setting_tui/target' ./ "$target_repo"/
    else
      if [[ "$DRY_RUN" == true ]]; then
        info "Would copy local repository with tar fallback"
      else
        tar -C . --exclude './jadeos_setting_tui/target' -cf - . | tar -C "$target_repo" -xf -
      fi
    fi
  elif [[ -d "$target_repo/.git" && -f "$target_repo/flake.nix" ]]; then
    info "Using existing repository at $target_repo"
  else
    info "Cloning repository to $target_repo"
    if [[ "$DRY_RUN" == true ]]; then
      info "Would clone $REPO_URL into $target_repo"
    else
      git clone "$REPO_URL" "$target_repo"
    fi
  fi
}

write_install_args() {
  local path="$1"
  local boot_part="$2"
  local root_part="$3"
  local swap_part="$4"

  if [[ "$DRY_RUN" == true ]]; then
    info "Would write install args: $path (dry-run: no file is created)"
    printf '[DRY-RUN] installDisk.boot = %q\n' "$boot_part"
    printf '[DRY-RUN] installDisk.root = %q\n' "$root_part"
    printf '[DRY-RUN] installDisk.swap = %q\n' "$swap_part"
    return
  fi

  cat > "$path" <<EOF
{
  installDisk = {
    boot = "$boot_part";
    root = "$root_part";
    swap = "$swap_part";
  };
}
EOF
}

generate_hardware_config() {
  local path="$1"

  if [[ "$DRY_RUN" == true ]]; then
    info "Would generate hardware config: $path (dry-run: no file is created)"
    printf '[DRY-RUN] nixos-generate-config --root %q --show-hardware-config > %q\n' "$MOUNT_ROOT" "$path"
    return
  fi

  nixos-generate-config --root "$MOUNT_ROOT" --show-hardware-config > "$path"
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
  done < <(find modules/hosts -mindepth 1 -maxdepth 1 -type d | sort)

  ((${#out_profiles[@]} > 0)) || error "No valid hosts found in modules/hosts"
}

select_profile() {
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
      PROFILE="${profiles[$((selection - 1))]}"
      return
    fi
    info "Invalid selection: $selection"
  done
}

auto_start_countdown() {
  local seconds=5
  local part_boot="$1"
  local part_root="$2"
  local part_swap="$3"

  info "Selected profile: $PROFILE"
  info "Target disk      : $TARGET_DISK"
  info "Boot part        : $part_boot"
  info "Root part        : $part_root"
  info "Swap part        : $part_swap"
  info "Auto-starting install in ${seconds}s. Press Ctrl+C to cancel."

  while ((seconds > 0)); do
    printf '\r[INFO] Starting in %ds... ' "$seconds"
    sleep 1
    ((seconds--))
  done
  printf '\n'
}

main() {
  local PROFILE=""

  if [[ $# -gt 0 ]]; then
    case "$1" in
      -h|--help)
        usage
        exit 0
        ;;
      *)
        error "setup.sh no longer accepts install options. Edit the configuration block at the top of the file."
        ;;
    esac
  fi

  [[ -n "$TARGET_DISK" ]] || error "--target-disk is required"

  if [[ "$YES" != true && "$DRY_RUN" != true ]]; then
    error "--yes is required because this will partition and format $TARGET_DISK"
  fi

  if [[ "$DRY_RUN" != true && "${EUID:-$(id -u)}" -ne 0 ]]; then
    error "Please run as root (use sudo)."
  fi

  for cmd in parted mkfs.fat mkfs.ext4 mkswap mount swapon mkdir nixos-generate-config nixos-install git; do
    if [[ "$DRY_RUN" != true ]]; then
      need_cmd "$cmd"
    fi
  done

  if [[ "$DRY_RUN" != true && ! -b "$TARGET_DISK" ]]; then
    error "Block device not found: $TARGET_DISK"
  fi

  local part_boot="${TARGET_DISK}1"
  local part_root="${TARGET_DISK}2"
  local part_swap="${TARGET_DISK}3"
  if [[ "$TARGET_DISK" =~ (nvme|mmcblk) ]]; then
    part_boot="${TARGET_DISK}p1"
    part_root="${TARGET_DISK}p2"
    part_swap="${TARGET_DISK}p3"
  fi
  part_boot="${BOOT_PART:-$part_boot}"
  part_root="${ROOT_PART:-$part_root}"
  part_swap="${SWAP_PART:-$part_swap}"

  select_profile
  auto_start_countdown "$part_boot" "$part_root" "$part_swap"

  local target_repo="$MOUNT_ROOT/etc/nixos"
  local profile_dir="$target_repo/nixos/$PROFILE"
  local install_args="$profile_dir/install-args.nix"
  local generated_hardware="$profile_dir/generated-hardware-configuration.nix"

  info "Profile      : $PROFILE"
  info "Target disk  : $TARGET_DISK"
  info "Boot part    : $part_boot"
  info "Root part    : $part_root"
  info "Swap part    : $part_swap"
  info "Mount root   : $MOUNT_ROOT"
  info "Install flake: $target_repo#$PROFILE"

  run parted -s "$TARGET_DISK" mklabel gpt
  run parted -s "$TARGET_DISK" mkpart ESP fat32 1MiB "$BOOT_END"
  run parted -s "$TARGET_DISK" set 1 esp on
  run parted -s "$TARGET_DISK" mkpart nixos ext4 "$BOOT_END" "$ROOT_END"
  run parted -s "$TARGET_DISK" mkpart swap linux-swap "$ROOT_END" 100%

  run mkfs.fat -F 32 -n boot "$part_boot"
  run mkfs.ext4 -L nixos -F "$part_root"
  run mkswap -L swap "$part_swap"

  run mount "$part_root" "$MOUNT_ROOT"
  run mkdir -p "$MOUNT_ROOT/boot"
  run mount "$part_boot" "$MOUNT_ROOT/boot"
  run swapon "$part_swap"

  copy_or_clone_repo "$target_repo"

  run mkdir -p "$profile_dir"
  write_install_args "$install_args" "$part_boot" "$part_root" "$part_swap"
  generate_hardware_config "$generated_hardware"
  run git -C "$target_repo" add -f \
    "nixos/$PROFILE/install-args.nix" \
    "nixos/$PROFILE/generated-hardware-configuration.nix"

  info "Installing NixOS from $target_repo#$PROFILE"
  run nixos-install --flake "$target_repo#$PROFILE"

  info "Install finished. Reboot, then set user passwords with passwd."
}

main "$@"
