# 003 — Audio Engine and DSP

> **Update 2026-11-08:** LAN streams are out of scope here (AirLink). Built: rodio/symphonia playback, EQ, vocal reduction, seek, HLS (unencrypted MPEG-TS audio; encrypted and fMP4 report a clear error). See 009 §11.

## Playback pipeline
```text
Source (file | downloaded episode | URL | LAN stream)
  → reader/buffer → demux/decode → time stretch (optional)
  → volume normalization / ReplayGain
  → 10-band EQ + preamp → karaoke stem control (when available)
  → limiter / master level → platform audio sink → selected output device
```
Exact DSP order can change after listening tests; avoid double normalization and clipping.

## Engine contract
The app exposes play, pause, stop, seek, queue-next, previous, playback speed, source switching, repeat, shuffle, listening bookmarks, output device identity and interruption notifications. Live streams may be nonseekable and cannot support standard duration-based progress. Playback state must outlive the screen and mirror to OS media controls/widgets.

## Music transitions
**Default track gap: 1.0 s** between discrete tracks. Settings allow 0 s (gapless) and custom delays. Do not insert silence within continuous streams or between chapters inside a continuous audiobook asset. Ensure pause is elapsed audio time, not the sum of UI load/network buffering. Re-check gap setting on queue transitions.

## Supported media targets
Common formats: MP3, AAC/M4A, WAV, FLAC, Ogg/Vorbis, Opus where decoder and container support are confirmed. Podcast streams may use HLS, HTTP(S), RSS enclosure URLs and redirect handling; internet radio may use MP3/AAC streams, M3U and PLS station playlists. Provide explicit unsupported-format errors instead of assuming all codecs work everywhere.

## 10-band mixer EQ
Center frequencies: 31, 62, 125, 250, 500 Hz; 1, 2, 4, 8, 16 kHz. Each slider configurable ±12 dB, with a preamp, bypass and named user presets. Map UI vertical sliders to precise dB values, not arbitrary percentages. Use stable filter coefficients, smooth parameter ramps, and output limiter/headroom; EQ should not create harsh pops when moved.

A visual mixer includes fader caps, rotary controls, optional level meter, numeric values, reset, preset selection and Basic/Advanced layouts. **Blue** is the active control/level accent, not orange. EQ processing is reusable across music, downloaded podcasts, locally decoded LAN media and compatible radio streams; stream passthrough may bypass DSP unless transcoded.

## Loudness and ReplayGain
Read track/album ReplayGain tags when present; optionally analyze untagged local content in the background with consent. Avoid altering files unless the user explicitly edits metadata. Provide per-track vs album modes and safe peak headroom. For spoken-word content, voice enhancement may be an optional DSP preset, not microphone mixing.

## Output selection and interruptions
One Bluetooth device at a time. Request audio focus according to platform rules; detect loss of route/focus, suspend on unplug/disconnect when configured, and prevent surprising switch to phone speakers. Support car stereo, headphones, device speaker and regular wired outputs. Platform API capabilities may restrict programmatic route choice.

## Stream buffering and resilience
Expose user-configurable buffer profiles (Low Latency, Balanced, Unstable Network) rather than assuming a universal latency. Reconnect only with exponential backoff and stop when user explicitly presses Stop. Differentiate silence, buffering, unavailable stream and paused states. Track buffer fill, retries, codec, sample rate and bitrate for diagnostics.

## Safe processing
DSP work must run away from presentation main thread. Bound float levels, protect against clipping and denormals, support 44.1/48 kHz and other platform-native sample rates, handle changing audio devices, and preserve play/stop responsiveness during processing.

## Acceptance tests
- 1-second track gap tolerance measured under local playback.
- 0-second gapless track test with continuous reference track.
- EQ bypass bitwise or perceptually neutral signal path within tolerance; no clipping at boosted settings.
- Bluetooth disconnect pauses playback instead of surprising output reroute.
- Online radio dropout recovers without hanging UI.
- Playback position restored across restart, including audiobook chapters.
