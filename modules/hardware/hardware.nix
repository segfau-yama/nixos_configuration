{ lib, ... }:
{
  # hardware: ハードウェア抽象化モジュール。
  # my.hardware.gpu / cpu オプションを宣言し、宣言値に応じて
  # ドライバー・マイクロコード・ストレージツールを自動適用する。
  #
  # 使い方 (ホスト設定内で):
  #   my.hardware.gpu = "nvidia";   # "nvidia" | "amd" | "intel" | "none"
  #   my.hardware.cpu = "amd";      # "intel"  | "amd" | "aarch64"
  flake.modules.nixos.hardware = { lib, config, pkgs, ... }: {

    options.my.hardware = {
      gpu = lib.mkOption {
        type        = lib.types.enum [ "nvidia" "amd" "intel" "none" ];
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

      # ── 共通: GPU あり → OpenGL を有効化 ─────────────────────────────────
      (lib.mkIf (config.my.hardware.gpu != "none") {
        hardware.graphics.enable = true;
      })

      # ── 共通: GPU あり + x86_64 → 32bit OpenGL + Gaming を有効化 ──────────
      # enable32Bit / Steam は x86_64 専用（ARM では非対応）。
      (lib.mkIf (config.my.hardware.gpu != "none" && pkgs.stdenv.hostPlatform.isx86_64) {
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
