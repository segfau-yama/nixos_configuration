{ ... }:
{
  # office (Home Manager): オフィススイートと辞書を提供する。
  flake.modules.homeManager.office = { pkgs, ... }: {
    home.packages = with pkgs; [
      libreoffice-qt
      hunspell
      hunspellDicts.en_US
    ];
  };
}
