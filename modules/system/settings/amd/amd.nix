{ inputs, ... }:
{
  # amd: AMD GPU ドライバー (amdgpu) + Vulkan を設定する。
  # RADV (Mesa Vulkan) は AMD Wayland 環境で安定している。
  # hardware.graphics は opengl (DRY Aspect) から継承する。
  flake.modules.nixos.amd = { pkgs, ... }: {
    imports = [ inputs.self.modules.nixos.opengl ];  # Inheritance Aspect

    services.xserver.videoDrivers = [ "amdgpu" ];

    environment.systemPackages = with pkgs; [
      mesa
      vulkan-tools
      vulkan-loader
      vulkan-validation-layers
    ];
  };
}
