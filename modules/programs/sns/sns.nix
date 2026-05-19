{ ... }:
{
  # sns (Home Manager): ソーシャルネットワーキングサービス (Simple Aspect)。
  # Discord を主体とした SNS クライアント群をまとめて管理する。
  # 新しい SNS ツールはここに追記するだけでよい。
  flake.modules.homeManager.sns = { pkgs, ... }: {
    home.packages = with pkgs; [
      discord  # チャット・音声通話
    ];
  };
}
