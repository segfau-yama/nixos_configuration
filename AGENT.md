あなたは NixOS の上級ユーザー兼システムアーキテクトです。

NixOS の設定リポジトリ一式を設計・生成してください。

目的は、
「Wayland + Niri + NVIDIA + Gaming + 開発 + CAD」
に対応した、保守性が高く再利用しやすい NixOS デスクトップ環境を構築することです。

このリポジトリは Dendritic Pattern を採用してください。
Dendritic Pattern とは、機能ごとに設定を枝分かれさせ、各機能が NixOS 設定・Home Manager 設定・補助ファイルを自己完結で持つ構成です。
従来の「core/desktop/gaming のような大分類モジュール」ではなく、feature 単位で完結させてください。

# 必須条件

- flakes を使用する
- Dendritic Pattern を採用する
- feature ごとに設定を自己完結させる
- Home Manager を統合する
- 可読性と再利用性を重視する
- declarative configuration を徹底する
- NixOS らしい構成にする
- Wayland 前提で構成する
- NVIDIA GPU を考慮する
- host 固有設定を最小化する
- 将来的に複数マシンへ展開しやすい構成にする

# 導入するもの

## 日本語環境
- 日本語 locale
- 日本語フォント
- fcitx5
- mozc

## GPU
- NVIDIA proprietary driver
- Wayland 対応
- OpenGL / Vulkan

## デスクトップ
- Niri
- IronBar
- notification daemon
- wallpaper manager
- terminal emulator
- display manager

## 音声
- PipeWire
- wireplumber
- pavucontrol

## アプリ
- Chromium
- Discord
- Lutris
- Steam
- KiCad
- FreeCAD
- Zed

## メディア
- 音楽再生ソフト
- 動画再生ソフト

# Dendritic Pattern のディレクトリ構成

以下のように、feature 単位で枝分かれする構成にすること。

/etc/nixos/
│
├── flake.nix
├── flake.lock
├── configuration.nix
├── hardware-configuration.nix
│
├── hosts/
│   └── default/
│       └── default.nix
│
├── features/
│   ├── locale/
│   │   ├── nixos.nix
│   │   ├── home.nix
│   │   └── fonts.nix
│   │
│   ├── fcitx5/
│   │   ├── nixos.nix
│   │   ├── home.nix
│   │   └── env.nix
│   │
│   ├── nvidia/
│   │   └── nixos.nix
│   │
│   ├── niri/
│   │   ├── nixos.nix
│   │   ├── home.nix
│   │   ├── config.kdl
│   │   └── packages.nix
│   │
│   ├── ironbar/
│   │   ├── home.nix
│   │   ├── config.json
│   │   └── style.css
│   │
│   ├── audio/
│   │   └── nixos.nix
│   │
│   ├── gaming/
│   │   ├── nixos.nix
│   │   ├── steam.nix
│   │   ├── lutris.nix
│   │   ├── gamemode.nix
│   │   └── wine.nix
│   │
│   ├── media/
│   │   ├── nixos.nix
│   │   ├── music-player.nix
│   │   └── video-player.nix
│   │
│   ├── development/
│   │   ├── nixos.nix
│   │   ├── home.nix
│   │   └── packages.nix
│   │
│   ├── browser/
│   │   └── home.nix
│   │
│   ├── discord/
│   │   └── home.nix
│   │
│   └── cad/
│       └── nixos.nix
│
├── modules/
│   └── lib/
│       └── default.nix
│
└── home/
    └── default.nix

# Dendritic Pattern のモジュール分割ルール

各 feature は自己完結させること。
feature ディレクトリの中に、その機能に必要な NixOS 設定と Home Manager 設定を閉じ込める。

例:
- features/niri/ には Niri 起動に必要な設定をすべて入れる
- features/gaming/ には Steam/Lutris/Gamemode/Wine 周辺をまとめる
- features/locale/ には locale と fonts をまとめる
- features/fcitx5/ には日本語入力関連をまとめる

host 側には「どの feature を有効化するか」だけを書き、詳細設定は feature に押し込むこと。

# feature ごとの責務

## locale/
- locale
- timezone
- fonts
- 日本語表示に必要な設定

## fcitx5/
- fcitx5
- mozc
- 入力方式関連の環境変数

## nvidia/
- NVIDIA proprietary driver
- modesetting
- powerManagement
- open
- nvidiaSettings
- package
- OpenGL / Vulkan
- 32bit support
- Wayland 前提の環境変数

## niri/
- Niri
- xdg portal
- polkit
- seatd
- greetd または SDDM
- terminal emulator
- wallpaper manager
- notification daemon
- Niri の config.kdl
- Niri 用の補助パッケージ

## ironbar/
- IronBar
- config.json
- style.css

## audio/
- PipeWire
- wireplumber
- pavucontrol
- pulse compatibility
- alsa compatibility
- rtkit

## gaming/
- Steam
- Lutris
- Gamemode
- Wine
- Vulkan 周辺
- allowUnfree に関する注意

## media/
- 音楽再生ソフト
- 動画再生ソフト

## development/
- Zed
- 開発ツール
- nix-index
- direnv
- devenv

## browser/
- Chromium

## discord/
- Discord

## cad/
- KiCad
- FreeCAD

# 出力形式

以下を順番に出力すること。

1. 全体ディレクトリ構造
2. flake.nix
3. configuration.nix
4. hosts/default/default.nix
5. 各 feature のコード
6. Home Manager 設定
7. なぜその Dendritic Pattern にしたか
8. NVIDIA + Niri の注意点
9. Wayland 特有の注意点
10. rebuild 手順

# コード品質ルール

- nixpkgs の unstable / stable の使い分けを説明する
- コメントを適切に入れる
- コピペで動くことを重視する
- 省略しすぎない
- imports を整理する
- enable オプションを明示する
- feature 間の依存は明確にする
- host 側は薄く保つ

# NVIDIA 要件

以下を適切に設定すること。

- services.xserver.videoDrivers
- hardware.nvidia
- modesetting
- powerManagement
- open
- nvidiaSettings
- package

Wayland + Niri 前提で調整すること。

# Niri 要件

以下を含めること。

- xdg portal
- polkit
- seatd
- greetd または SDDM
- ironbar
- wallpaper manager
- notification daemon
- terminal emulator

# PipeWire 要件

以下を設定すること。

- pipewire
- pulse compatibility
- alsa compatibility
- rtkit
- wireplumber

# Steam / Lutris 要件

以下を考慮すること。

- allowUnfree
- gamemode
- wine
- Vulkan
- Proton

# unstable / stable の使い分け

- OS の土台やドライバは stable を基本にする
- 新しい desktop / gaming / editor 系は unstable を使う選択肢を説明する
- もし unstable を使うなら、どの feature に限定するか明示する
- feature 単位で pkgs を切り替えられる設計にする

# 出力ルール

- 必ず完全な nix コードを出す
- "..." で省略しない
- ファイルごとに分けて出力する
- どのファイルに保存するか明示する
- feature ごとにコードを分ける

# 禁止事項

- monolithic configuration.nix
- 古い NixOS 設定
- X11 前提構成
- imperative package install
- apt/yum/pacman 的説明
- unwrap 的な雑な設定
- feature を大分類だけで分ける構成
- host 側へ設定を集中させる構成

# 追加対応

必要に応じて以下にも対応する。

- home-manager
- stylix
- catppuccin
- gaming optimization
- flakes lock
- nix-index
- direnv
- devenv

# 実装の基本方針

- Dendritic Pattern に従い、各 feature を自己完結させる
- ホストは feature の import だけを担当する
- 依存関係のある feature は、その feature 内で完結または明示的に依存を記述する
- feature ごとに NixOS 側と Home Manager 側を分けてよいが、同じ feature ディレクトリ配下にまとめる
- Wayland と Niri を中心に設計し、NVIDIA の注意点を反映する
- 実運用可能な構成を優先する