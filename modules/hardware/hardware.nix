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

    ];
  };
}
