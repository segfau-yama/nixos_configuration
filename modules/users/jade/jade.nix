{ inputs, ... }:
let
  mkUser = {
    username,
    description,
    gui ? false,
  }: { pkgs, ... }: {
    users.users.${username} = {
      isNormalUser = true;
      inherit description;
      extraGroups = [
        "wheel"
        "networkmanager"
      ] ++ pkgs.lib.optionals gui [
        "audio"
        "video"
        "input"
        "seat"
      ];
      shell = pkgs.zsh;
    };
  };

  mkHome = username: homeImports: { ... }: {
    imports = homeImports;

    home.username = username;
    home.homeDirectory = "/home/${username}";
    home.stateVersion = "25.05";

    programs.home-manager.enable = true;
  };
in
{
  # jadeUsers (NixOS): デフォルトユーザー群と Home Manager 統合。
  flake.modules.nixos.jadeUsers = { ... }: {
    imports = with inputs.self.modules.nixos; [
      jade-cui
      jade-gui
      jade-gaming
      jade-develop
      programming
    ];

    programs.zsh.enable = true;

    home-manager.users.jade-cui.imports = [
      inputs.self.modules.homeManager.jade-cui
    ];
    home-manager.users.jade-gui.imports = [
      inputs.self.modules.homeManager.jade-gui
    ];
    home-manager.users.jade-gaming.imports = [
      inputs.self.modules.homeManager.jade-gaming
    ];
    home-manager.users.jade-develop.imports = [
      inputs.self.modules.homeManager.jade-develop
    ];
  };

  # jade-cui: CUI 管理ユーザー。
  flake.modules.nixos.jade-cui = mkUser {
    username = "jade-cui";
    description = "Jade CUI";
  };

  # jade-gui: Hyprland GUI ユーザー。
  flake.modules.nixos.jade-gui = mkUser {
    username = "jade-gui";
    description = "Jade GUI";
    gui = true;
  };

  # jade-gaming: Plasma ゲーム用ユーザー。
  flake.modules.nixos.jade-gaming = mkUser {
    username = "jade-gaming";
    description = "Jade Gaming";
    gui = true;
  };

  # jade-develop: Hyprland 開発用ユーザー。
  flake.modules.nixos.jade-develop = mkUser {
    username = "jade-develop";
    description = "Jade Develop";
    gui = true;
  };

  flake.modules.homeManager.jade-cui = mkHome "jade-cui" (
    with inputs.self.modules.homeManager; [
      base
    ]
  );

  flake.modules.homeManager.jade-gui = mkHome "jade-gui" (
    with inputs.self.modules.homeManager; [
      base
      desktop
    ]
  );

  flake.modules.homeManager.jade-gaming = mkHome "jade-gaming" (
    with inputs.self.modules.homeManager; [
      base
      gaming
    ]
  );

  flake.modules.homeManager.jade-develop = mkHome "jade-develop" (
    with inputs.self.modules.homeManager; [
      base
      desktop
      cli-tools
      lang
      nix-tools
      programming
      zed
    ]
  );
}
