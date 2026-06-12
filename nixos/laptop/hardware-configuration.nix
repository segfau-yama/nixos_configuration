{ config, lib, ... }:
let
  installArgsPath = ./install-args.nix;
  installArgs =
    if builtins.pathExists installArgsPath
    then import installArgsPath
    else { };
  installDisk = installArgs.installDisk or config.my.installDisk;
  # Captured from nixos-generate-config --show-hardware-config during install.
  generatedHardwareModule =
    { lib, modulesPath, ... }:
    {
      imports = [ (modulesPath + "/installer/scan/not-detected.nix") ];

      boot.initrd.availableKernelModules = [
        "xhci_pci"
        "nvme"
        "ahci"
        "usbhid"
        "usb_storage"
        "sd_mod"
        "rtsx_pci_sdmmc"
      ];
      boot.initrd.kernelModules = [ ];

      fileSystems."/" = {
        device = "/dev/disk/by-label/nixos";
        fsType = "ext4";
      };

      fileSystems."/boot" = {
        device = "/dev/disk/by-label/boot";
        fsType = "vfat";
      };

      swapDevices = [
        { device = "/dev/disk/by-label/swap"; }
      ];

      nixpkgs.hostPlatform = lib.mkDefault "x86_64-linux";
    };
in
{
  imports = [
    generatedHardwareModule
  ];

  fileSystems."/" = lib.mkForce {
    device = installDisk.root;
    fsType = "ext4";
  };

  fileSystems."/boot" = lib.mkForce {
    device = installDisk.boot;
    fsType = "vfat";
  };

  swapDevices = lib.mkForce (
    lib.optional (installDisk.swap != null && installDisk.swap != "") {
      device = installDisk.swap;
    }
  );

  networking.useDHCP = lib.mkDefault true;

  boot.loader.systemd-boot.enable = true;
  boot.loader.efi.canTouchEfiVariables = true;
}
