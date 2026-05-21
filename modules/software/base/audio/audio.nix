{ ... }:
{
  # audio: PipeWire + WirePlumber + rtkit + ALSA/JACK/PulseAudio 互換レイヤーを設定する。
  flake.modules.nixos.audio = { ... }: {
    # rtkit: PipeWire がリアルタイム優先度を取得するために必要。
    security.rtkit.enable = true;

    services.pipewire = {
      enable = true;
      alsa.enable = true;
      alsa.support32Bit = true;  # 32bit Steam/Wine ゲーム向け
      pulse.enable = true;       # PulseAudio 互換レイヤー
      jack.enable = true;        # JACK 互換レイヤー
      wireplumber.enable = true;
    };
  };
}
