{ inputs, ... }:
{
  # nvidia: NVIDIA プロプライエタリードライバー + Vulkan + Wayland 設定。
  # GTX 1080 等の旧世代 GPU では production ブランチを優先する。
  flake.modules.nixos.nvidia = { config, pkgs, ... }: {
    imports = [ inputs.self.modules.nixos.opengl ];  # Inheritance Aspect

    # カーネルパラメーター: Wayland での modesetting に必要。
    boot.kernelParams = [ "nvidia-drm.modeset=1" ];

    services.xserver.videoDrivers = [ "nvidia" ];

    hardware.nvidia = {
      modesetting.enable = true;
      powerManagement.enable = false;
      powerManagement.finegrained = false;
      open = false;           # オープンソースカーネルモジュールは無効
      nvidiaSettings = true;  # nvidia-settings GUI を有効化
      package = config.boot.kernelPackages.nvidiaPackages.production
        or config.boot.kernelPackages.nvidiaPackages.stable;
    };

    # OpenGL は opengl (DRY Aspect) から継承する。

    # Electron/Chromium 系アプリが Wayland バックエンドを使うための環境変数。
    environment.sessionVariables = {
      NIXOS_OZONE_WL = "1";
    };

    environment.systemPackages = with pkgs; [
      nvtopPackages.nvidia  # GPU 使用率モニター
      vulkan-tools
      vulkan-loader
      vulkan-validation-layers
    ];
  };
}
