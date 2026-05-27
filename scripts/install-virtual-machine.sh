#!/usr/bin/env bash
set -euo pipefail

TARGET_DISK="${TARGET_DISK:-/dev/vda}"
ROOT_PART="${ROOT_PART:-/dev/vda2}"
BOOT_PART="${BOOT_PART:-/dev/vda1}"
SWAP_PART="${SWAP_PART:-/dev/vda3}"
MNT="${MNT:-/mnt}"
HOST_ID="${HOST_ID:-virtual_machine}"
TARGET_REPO="${TARGET_REPO:-$MNT/etc/nixos}"

need_cmd() {
  command -v "$1" >/dev/null 2>&1 || {
    echo "[ERROR] Required command not found: $1" >&2
    exit 1
  }
}

expect_fstype() {
  local dev="$1"
  local expected="$2"
  local actual
  actual="$(blkid -o value -s TYPE "$dev" 2>/dev/null || true)"
  if [[ "$actual" != "$expected" ]]; then
    echo "[ERROR] Filesystem type mismatch for $dev: expected '$expected', got '${actual:-<none>}'" >&2
    exit 1
  fi
}

is_mounted_target() {
  local dev="$1"
  local target="$2"
  findmnt -rn -S "$dev" -T "$target" >/dev/null 2>&1
}

if [[ "${EUID:-$(id -u)}" -ne 0 ]]; then
  echo "[ERROR] Please run as root (use sudo)." >&2
  exit 1
fi

for cmd in blkid findmnt mount mkdir swapon grep nixos-install git; do
  need_cmd "$cmd"
done

for dev in "$TARGET_DISK" "$ROOT_PART" "$BOOT_PART" "$SWAP_PART"; do
  if [[ ! -b "$dev" ]]; then
    echo "[ERROR] Block device not found: $dev" >&2
    exit 1
  fi
done

expect_fstype "$ROOT_PART" "ext4"
expect_fstype "$BOOT_PART" "vfat"
expect_fstype "$SWAP_PART" "swap"

mkdir -p "$MNT"
if ! is_mounted_target "$ROOT_PART" "$MNT"; then
  mount "$ROOT_PART" "$MNT"
fi

mkdir -p "$MNT/boot"
if ! is_mounted_target "$BOOT_PART" "$MNT/boot"; then
  mount "$BOOT_PART" "$MNT/boot"
fi

if ! grep -qE "^${SWAP_PART//\//\\/}[[:space:]]" /proc/swaps; then
  swapon "$SWAP_PART"
fi

if [[ ! -d "$TARGET_REPO/.git" || ! -f "$TARGET_REPO/flake.nix" ]]; then
  echo "[ERROR] Repository not found at $TARGET_REPO" >&2
  echo "Run the following before retrying:" >&2
  echo "  mkdir -p $TARGET_REPO" >&2
  echo "  cd $TARGET_REPO" >&2
  echo "  git init" >&2
  echo "  git remote add origin https://github.com/segfau-yama/nixos_configuration.git" >&2
  echo "  git fetch origin" >&2
  echo "  git checkout -t origin/main" >&2
  exit 1
fi

echo "[INFO] Installing host '$HOST_ID' from flake: $TARGET_REPO#$HOST_ID"
nixos-install --flake "$TARGET_REPO#$HOST_ID"

cat <<'EOF'
[DONE] NixOS installation finished.
Next steps:
  1) Reboot into installed system
  2) Set passwords:
     passwd jade
     passwd admin   # optional
EOF
