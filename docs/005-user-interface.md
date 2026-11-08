# 005 — UI, Dashboard, Widgets and Theme

## Design system
Follow Cindrix Overwatch family identity without reusing orange as the interactive accent. Primary app surfaces: charcoal / graphite (#111827 and #1E293B as initial candidates), metallic silver for branding/headers, **electric blue (#3B82F6 initial candidate)** for active navigation, focus, faders, rings, buttons and meters. Actual color values require design approval; role of blue is locked. Respect contrast, high DPI, dark mode and reduced motion.

## Windows and Android navigation
- Windows: left module rail, top branding with Cindrix Radio logo, central workspace, persistent bottom mini-player, optional right context panel.
- Android: adaptive bottom/side navigation with overflow for ten sections, persistent compact playback controls; larger tap targets.
- Tabs: Dashboard, Podcasts, Internet Radio, Offline Library, LAN Streaming, Audio Sharing, Playlists, Downloads, History, Settings.
- Mini-player remains visible across tab transitions and reflects one shared playback session. Opening EQ or karaoke should not interrupt audio.

## Dashboard
Top section: logo, current playback state, current Online / LAN Only / Device Offline indicator; separate icons for Bluetooth output route and network availability.
Modules: Now Playing, Continue Listening, Recently Played, Recently Added, Recommended from Available, Favorites, Listening Queue, LAN Sources, Downloads/Storage, Audio Sharing status.

Dashboard widgets can be shown/hidden/reordered, with layout stored locally. Never fabricate recommendations when the available source is offline. Label stale, cached and reachable items.

## Mixer-style interaction
Karaoke controls and **10-band EQ** use common mixer design primitives: vertical fader controls, rotary dials, meters, numeric value labels, reset action, bypass, presets, blue illumination. Basic/Advanced mode reveals more controls without replacing the simple player. EQ frequency values are dB-based, not percentages. Each logical control must expose accessible labels/value/keyboard operation; on mobile support touch drag, tap-to-adjust and screen-reader adjustment. No hardware-style ornament should block use.

## Karaoke presentation
Toggle from now-playing or track menu. Full-screen or windowed lyrics; manual scroll only; font scale, high contrast, optional simple lyrics search. Vocal treatment controls: Original / Reduce / Attempt Removal; slider only when stems/process path supports meaningful mixing. Show processing/buffering state; do not promise perfect vocal elimination. Song original remains untouched.

## External widgets
1. **Now Playing:** source/track/cover, play/pause, next/previous as appropriate, position/progress for seekable tracks, current output route.
2. **Favorites & Quick Access:** favorite radio stations, downloaded podcasts, playlists and LAN collections with availability badges.

Size variants: Compact, Standard, Expanded when host platform supports. Windows: optional desktop or always-on-top floating panels plus system-tray controls; Android: native home-screen widget integration and system media/notification controls. OS widget actions operate the same playback session through native bridge; widgets never run a separate player.

## Car Mode
Large controls and uncluttered pages, route status and safe playback/queue controls; do not encourage distraction while driving. Automatic route detection may suggest Car Mode but should honor user preference.

## Settings organization
Audio (EQ, gap, ReplayGain, speed), Devices (single Bluetooth output), Network (LAN discovery/manual source/sharing), Downloads, Library, Playback, Karaoke, Appearance, Backup/Restore, Privacy and Diagnostics.

## State and feedback
Always distinguish paused, buffering, reconnecting, unavailable, offline-download available and sharing-active. Use blue for selected interaction; warning/error colors permitted for genuinely exceptional states. Show unsupported audio format or unavailable device concretely. Provide empty states and actionable retry.

## Acceptance
Screen-reader/keyboard operable desktop mixer and Android touch controls; active mini-player unaffected by navigation; dashboard recommendation list changes correctly with network mode; widget/media controls stay in sync; karaoke lyrics manually scroll without timing metadata.
