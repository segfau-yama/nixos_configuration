あなたは NixOS の上級ユーザー兼システムアーキテクトです。

NixOS の設定ファイル群を生成してください。

目的は、
「Wayland + Hyprland + NVIDIA + Gaming + 開発 + CAD」
に対応した、保守性の高い NixOS デスクトップ環境を構築することです。

# 必須条件

- flakes を使用する
- configuration を用途別に細かく分割する
- 可読性と再利用性を重視する
- declarative configuration を徹底する
- NixOS らしい構成にする
- Wayland 前提で構成する
- NVIDIA GPU を考慮する

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
- Hyprland
- Waybar
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

# ディレクトリ構成

以下のように用途別に分割すること

/etc/nixos/
│
├── flake.nix
├── configuration.nix
├── hardware-configuration.nix
│
├── modules/
│   ├── core/
│   ├── locale/
│   ├── desktop/
│   ├── gaming/
│   ├── development/
│   ├── media/
│   ├── audio/
│   ├── hardware/
│   └── users/
│
├── home/
│   └── home.nix
│
└── hosts/
    └── default.nix

# モジュール分割ルール

core/
- boot
- nix settings
- garbage collection
- networking
- timezone

locale/
- locale
- fonts
- fcitx5
- mozc

desktop/
- hyprland
- waybar
- display manager
- terminal
- xdg portal

hardware/
- nvidia
- opengl
- vulkan

audio/
- pipewire
- wireplumber
- audio utils

gaming/
- steam
- lutris
- gamemode
- wine

media/
- music player
- video player

development/
- zed
- development tools

# 出力形式

以下を順番に出力する

1. 全体ディレクトリ構造
2. flake.nix
3. configuration.nix
4. 各 module のコード
5. home-manager 設定
6. なぜその分割にしたか
7. NVIDIA + Hyprland の注意点
8. Wayland 特有の注意点
9. rebuild 手順

# コード品質ルール

- nixpkgs の unstable/stable の使い分けを説明する
- コメントを適切に入れる
- コピペで動くことを重視する
- 省略しすぎない
- imports を整理する
- enable オプションを明示する

# NVIDIA 要件

以下を適切に設定する

- services.xserver.videoDrivers
- hardware.nvidia
- modesetting
- powerManagement
- open
- nvidiaSettings
- package

Wayland + Hyprland 前提で調整する

# Hyprland 要件

以下を含める

- xdg portal
- polkit
- seatd
- greetd または SDDM
- waybar
- wallpaper
- notification daemon

# PipeWire 要件

以下を設定する

- pipewire
- pulse compatibility
- alsa compatibility
- rtkit

# Steam/Lutris 要件

以下を考慮する

- allowUnfree
- gamemode
- wine
- Vulkan
- Proton

# 出力ルール

- 必ず完全な nix コードを出す
- "..." で省略しない
- ファイルごとに分けて出力する
- どのファイルに保存するか明示する

# 禁止事項

- monolithic configuration.nix
- 古い NixOS 設定
- X11 前提構成
- imperative package install
- apt/yum/pacman 的説明
- unwrap 的な雑な設定

# 追加対応

必要に応じて以下にも対応する

- home-manager
- stylix
- catppuccin
- gaming optimization
- flakes lock
- nix-index
- direnv
- devenv