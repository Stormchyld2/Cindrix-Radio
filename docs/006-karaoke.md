# 006 — Karaoke and Lyrics Architecture

## Approved experience
Karaoke means playing a music track with **optional strong vocal reduction/attempted removal**, while displaying plain, manually scrollable lyrics. Timed text, word highlighting, lyric synchronization and live microphones are **not required**. The existing file is never modified or replaced.

## Signal path
```text
Original decoded audio
  ├─ Original mode -------------------------------------------┐
  └─ Vocal processing (when activated and supported)          |
       → instrumental estimate + isolated vocal estimate      |
       → Vocal-level blend (Original / Reduced / Near-Remove)  |
           → EQ / limiter / output device <-------------------┘
```
Precise processing stage relative to EQ must be validated with listening tests. Never mix incompatible channel layouts without explicit conversion.

## Separation strategies
1. Provide a minimal optional low-cost reduction technique for stereo recordings only if quality is acceptable, with clear limitations: center-channel cancellation removes other centered material and often leaves reverb/vocals.
2. For higher-quality separation, use a local/on-device stem separation engine with vetted model license, reproducible build, offline inference and benchmarked CPU/GPU/memory. Engine/model is a pluggable dependency, not permanently hard-coded; candidates require evaluation rather than assumption of real-time playback.
3. Android low-end devices may not sustain high-quality inference. Fall back to Original, a lower-cost algorithm, or prebuffered segment processing. Never freeze playback/UI and never require a cloud inference service.

## Storage policy (locked)
**Do not write permanent instrumental copies, separated stems, or processed duplicates.** Process on-demand with bounded RAM and temporary workspace, delete working files on completion/cancel/crash recovery, and cap disk/memory footprint. Temporary cache may be transient and automatically cleared; no persistent cache by default. Downloads/storage quota must account for transient work. Original track stays untouched.

## Performance
A high-quality algorithm may require startup delay/preprocessing. Show progress and a Cancel control; distinguish real-time mode from processing-first playback. Segment processing can prefetch the next chunk while playing current output, but test audible discontinuities, seeking, sample alignment and CPU throttling. Never imply zero-latency removal across all devices.

## Vocal level meaning
If real instrumental and vocal stems are available, the mixer Vocal Fader attenuates the separated vocal stem and Music Fader controls the instrumental stem. If using only stereo phase cancellation, present a “Reduction strength” control instead; do not label it as precise stem volume. Modes: Original (unprocessed), Reduced (user-chosen strength), Attempt Removal (maximum supported). Preserve the primary song volume independently of vocal setting.

## Lyrics
Sources: embedded metadata, sidecar .lrc/.txt (timing tags ignored if present), user-entered local lyrics, optionally licensed third-party lookup online. Match with track identity plus user override; avoid displaying wrong lyrics for similarly named songs. Show large readable text, manual scrolling, adjustable size, full-screen/television-friendly view, and optionally remember scroll position during the session. No timed lyrics fetch or synchronization requirement. Respect rights/licensing when fetching or sharing text.

## UX behavior
Karaoke toggle opens the mixer/lyrics experience without changing the ordinary library. Turning it off returns to original sound. Processed temporary data never appears as a second track in lists or duplicate detection. Skip, stop, switch track and device unplug cancel or retire work cleanly. Internet and LAN are not required for locally stored songs and lyrics.

## Acceptance tests
- Lyric files without timestamps display and scroll correctly.
- Changing vocal strength does not mutate input file, metadata or create persistent output.
- Stop/crash/relaunch cleans stale temporary files.
- Low-end device timeout offers safe fallback; playback engine remains responsive.
- Original mode sounds unchanged; Near-Remove warns of artifacts and limits.
- EQ and karaoke faders remain operable offline and with Bluetooth output.
