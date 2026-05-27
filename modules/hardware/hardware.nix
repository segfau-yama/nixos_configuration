{ lib, ... }:
{
  # hardware: ハードウェア抽象化モジュール。
  # my.hardware.gpu / cpu オプションを宣言し、宣言値に応じて
  # ドライバー・マイクロコード・ストレージツールを自動適用する。
  #
  # 使い方 (ホスト設定内で):
  #   my.hardware.gpu = "nvidia";   # "nvidia" | "amd" | "intel" | "virtio" | "none"
  #   my.hardware.cpu = "amd";      # "intel"  | "amd" | "aarch64"
  flake.modules.nixos.hardware = { lib, config, pkgs, ... }:
  let
    hasPhysicalGpu = builtins.elem config.my.hardware.gpu [ "nvidia" "amd" "intel" ];
  in
  {

    options.my.hardware = {
      gpu = lib.mkOption {
        type        = lib.types.enum [ "nvidia" "amd" "intel" "virtio" "none" ];
        default     = "none";
        description = "GPU の種類。ドライバーと関連パッケージを自動適用する。";
      };
      cpu = lib.mkOption {
        type        = lib.types.enum [ "intel" "amd" "aarch64" ];
        default     = "amd";
        description = "CPU の製造元。マイクロコードを自動適用する。aarch64 は不要のためスキップ。";
      };
      storage.enable = lib.mkEnableOption "non-boot drive auto mount and /nix placement";
    };

    options.my.installDisk = {
      boot = lib.mkOption {
        type = lib.types.str;
        description = "EFI system partition device used by hardware-configuration.nix.";
      };
      root = lib.mkOption {
        type = lib.types.str;
        description = "Root partition device used by hardware-configuration.nix.";
      };
      swap = lib.mkOption {
        type = lib.types.str;
        description = "Swap partition device used by hardware-configuration.nix.";
      };
    };

    config = lib.mkMerge [

      # ── 共通: GUI / Wayland 用の Mesa/DRI 基盤 ────────────────────────────
      # VM の汎用GPU (none) でも Wayland 動作用の Mesa/DRI を有効化する。
      # Niri などの Wayland コンポジターには Mesa/DRI が必要。
      (lib.mkIf (config.my.hardware.gpu == "none") {
        hardware.graphics.enable = true;

        environment.systemPackages = with pkgs; [
          mesa
          vulkan-tools
        ];

        services.qemuGuest.enable = lib.mkDefault true;
        services.spice-vdagentd.enable = lib.mkDefault true;
      })

      # ── Virtio GPU (VM) ───────────────────────────────────────────────────
      (lib.mkIf (config.my.hardware.gpu == "virtio") {
        hardware.graphics.enable = true;
        services.xserver.videoDrivers = [ "modesetting" ];
        boot.initrd.kernelModules = [ "virtio_gpu" ];

        environment.systemPackages = with pkgs; [
          mesa
          vulkan-tools
        ];

        services.qemuGuest.enable = lib.mkDefault true;
        services.spice-vdagentd.enable = lib.mkDefault true;
      })

      # ── 共通: GPU あり → OpenGL を有効化 ─────────────────────────────────
      (lib.mkIf (config.my.hardware.gpu != "none") {
        hardware.graphics.enable = true;
      })

      # ── 共通: 物理 GPU あり + x86_64 → 32bit OpenGL + Gaming を有効化 ─────
      # enable32Bit / Steam は x86_64 専用（ARM では非対応）。
      (lib.mkIf (hasPhysicalGpu && pkgs.stdenv.hostPlatform.isx86_64) {
        hardware.graphics.enable32Bit = true; # Steam / Wine / Proton 向け 32bit 対応

        programs.gamemode.enable = true;

        programs.steam = {
          enable                       = true;
          remotePlay.openFirewall      = true;
          dedicatedServer.openFirewall = false;
        };
      })

      # ── NVIDIA ─────────────────────────────────────────────────────────────
      (lib.mkIf (config.my.hardware.gpu == "nvidia") {
        boot.kernelParams = [ "nvidia-drm.modeset=1" ];

        services.xserver.videoDrivers = [ "nvidia" ];

        hardware.nvidia = {
          modesetting.enable          = true;
          powerManagement.enable      = false;
          powerManagement.finegrained = false;
          open                        = false;
          nvidiaSettings              = true;
          package =
            let np = config.boot.kernelPackages.nvidiaPackages;
            in np.production or np.stable;
        };

        environment.sessionVariables.NIXOS_OZONE_WL = "1";

        environment.systemPackages = with pkgs; [
          nvtopPackages.nvidia
          vulkan-tools
          vulkan-loader
          vulkan-validation-layers
        ];
      })

      # ── AMD ────────────────────────────────────────────────────────────────
      (lib.mkIf (config.my.hardware.gpu == "amd") {
        services.xserver.videoDrivers = [ "amdgpu" ];

        environment.systemPackages = with pkgs; [
          mesa
          vulkan-tools
          vulkan-loader
          vulkan-validation-layers
        ];
      })

      # ── Intel 内蔵グラフィック ─────────────────────────────────────────────
      (lib.mkIf (config.my.hardware.gpu == "intel") {
        # modesetting: 現行 Intel GPU の推奨 DDX ドライバー（Iris/HD/Arc 対応）
        services.xserver.videoDrivers = [ "modesetting" ];

        hardware.graphics.extraPackages = with pkgs; [
          intel-media-driver     # VAAPI ハードウェアデコード（Broadwell 以降）
          intel-compute-runtime  # OpenCL（第7世代以降）
        ];

        environment.sessionVariables.NIXOS_OZONE_WL = "1";

        environment.systemPackages = with pkgs; [
          intel-gpu-tools
          mesa
          vulkan-tools
          vulkan-loader
          vulkan-validation-layers
        ];
      })

      # ── CPU: Intel マイクロコード ──────────────────────────────────────────
      (lib.mkIf (config.my.hardware.cpu == "intel") {
        hardware.cpu.intel.updateMicrocode = true;
      })

      # ── CPU: AMD マイクロコード ────────────────────────────────────────────
      (lib.mkIf (config.my.hardware.cpu == "amd") {
        hardware.cpu.amd.updateMicrocode = true;
      })

      # ── ストレージ: btrfs 検出 → btrfs-progs 自動インストール ─────────────
      (lib.mkIf (builtins.any (fs: fs.fsType == "btrfs")
        (builtins.attrValues config.fileSystems)) {
        environment.systemPackages = [ pkgs.btrfs-progs ];
      })

      # ── ストレージ: 非ブートドライブ自動マウント + /nix 配置 ─────────────
      (lib.mkIf config.my.hardware.storage.enable {
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
              safe_name="''${label:-$uuid}"
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
      })

    ];
  };
}
