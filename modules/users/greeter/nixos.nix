{ ... }:
{
  # greeter: greetd が使用する最小システムユーザー。
  flake.modules.nixos.greeter = { lib, pkgs, ... }: {
    users.groups.greeter = lib.mkDefault {};

    users.users.greeter = {
      isSystemUser = lib.mkDefault true;
      group        = lib.mkDefault "greeter";
      description  = lib.mkDefault "greetd greeter user";
      home         = lib.mkDefault "/var/lib/greetd";
      createHome   = lib.mkDefault true;
      shell        = lib.mkDefault "${pkgs.shadow}/bin/nologin";
    };
  };
}
