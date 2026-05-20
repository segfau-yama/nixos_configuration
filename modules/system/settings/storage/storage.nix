{ ... }:
{
  # storage: 非ブートドライブ (HDD/SSD) を自動マウントし、最大容量ドライブへ /nix を配置する。
  # 注意: /nix は NixOS の中核パスのため、初回同期時は時間がかかることがある。
  flake.modules.nixos.storage = { pkgs, ... }: {
    systemd.services.nix-auto-storage = {
      description = "Auto mount non-boot drives and place /nix on the largest one";
      wantedBy = [ "multi-user.target" ];
      before = [ "nix-daemon.service" ];
      after = [ "local-fs.target" "systemd-udev-settle.service" ];
      wants = [ "systemd-udev-settle.service" ];
      path = with pkgs; [
        bash
        coreutils
        gnugrep
        gawk
        gnused
        util-linux
        rsync
      ];
      serviceConfig = {
        Type = "oneshot";
        RemainAfterExit = true;
      };
      script = ''
        set -euo pipefail

        root_source="$(findmnt -n -o SOURCE / || true)"
        root_pkname=""
        if [[ -n "$root_source" ]]; then
          root_pkname="$(lsblk -ndo PKNAME "$root_source" 2>/dev/null || true)"
        fi

        mkdir -p /mnt/storage

        # 非ブートドライブ上のファイルシステム付きパーティションを検出して自動マウント。
        mapfile -t candidates < <(
          lsblk -prno NAME,TYPE,FSTYPE,RM,PKNAME |
            awk -v root_pkname="$root_pkname" '
              $2 == "part" && $3 != "" && $4 == "0" {
                if (root_pkname != "" && $5 == root_pkname) next;
                if ($3 == "swap") next;
                print $1;
              }
            '
        )

        if [[ ''${#candidates[@]} -eq 0 ]]; then
          echo "nix-auto-storage: no extra partitions found"
          exit 0
        fi

        largest_mount=""
        largest_size=0

        for dev in "''${candidates[@]}"; do
          fstype="$(lsblk -ndo FSTYPE "$dev" 2>/dev/null || true)"
          case "$fstype" in
            ext4|xfs|btrfs) ;;
            *)
              continue
              ;;
          esac

          uuid="$(blkid -s UUID -o value "$dev" 2>/dev/null || true)"
          label="$(blkid -s LABEL -o value "$dev" 2>/dev/null || true)"
          safe_name="${label:-$uuid}"
          safe_name="$(echo "$safe_name" | sed 's/[^a-zA-Z0-9._-]/_/g')"
          if [[ -z "$safe_name" ]]; then
            safe_name="$(basename "$dev")"
          fi

          mount_dir="/mnt/storage/$safe_name"
          mkdir -p "$mount_dir"

          existing_target="$(findmnt -rn -o TARGET -S "$dev" | head -n 1 || true)"
          if [[ -n "$existing_target" ]]; then
            mount_dir="$existing_target"
          elif ! findmnt -rn -T "$mount_dir" >/dev/null 2>&1; then
            mount -o noatime "$dev" "$mount_dir" || continue
          fi

          size="$(df -B1 --output=size "$mount_dir" 2>/dev/null | tail -n 1 | tr -d '[:space:]')"
          if [[ "$size" =~ ^[0-9]+$ ]] && (( size > largest_size )); then
            largest_size="$size"
            largest_mount="$mount_dir"
          fi
        done

        if [[ -z "$largest_mount" ]]; then
          echo "nix-auto-storage: no supported mount targets (ext4/xfs/btrfs)"
          exit 0
        fi

        state_dir="/var/lib/nix-auto-storage"
        state_file="$state_dir/target-nix"
        mkdir -p "$state_dir"

        target_nix=""
        if [[ -f "$state_file" ]]; then
          remembered="$(cat "$state_file")"
          if [[ -n "$remembered" ]]; then
            remembered_parent="$(dirname "$remembered")"
            if findmnt -rn -T "$remembered_parent" >/dev/null 2>&1; then
              target_nix="$remembered"
            fi
          fi
        fi

        if [[ -z "$target_nix" ]]; then
          target_nix="$largest_mount/nix"
          echo "$target_nix" > "$state_file"
        fi

        mkdir -p "$target_nix"

        current_nix_source="$(findmnt -n -o SOURCE /nix || true)"
        if [[ "$current_nix_source" != "$target_nix" ]]; then
          # 初回または移行時に現在の /nix を同期してから bind mount する。
          rsync -aHAX --delete /nix/ "$target_nix/"
          mount --bind "$target_nix" /nix
        fi

        echo "nix-auto-storage: /nix -> $target_nix"
      '';
    };

    systemd.services.nix-daemon = {
      requires = [ "nix-auto-storage.service" ];
      after = [ "nix-auto-storage.service" ];
    };
  };
}
