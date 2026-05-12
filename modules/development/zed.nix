{ unstablePkgs, ... }:
{
  environment.systemPackages = [
    unstablePkgs.zed-editor
  ];
}