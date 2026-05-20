{ inputs, ... }:
{
  flake.nixosConfigurations = inputs.self.lib.mkNixosWithEnv;
}
