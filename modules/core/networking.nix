{ ... }:
{
  networking.networkmanager.enable = true;

  services.resolved = {
    enable = true;
    dnssec = "allow-downgrade";
  };

  networking.firewall = {
    enable = true;
    allowedTCPPorts = [ ];
    allowedUDPPorts = [ ];
  };
}