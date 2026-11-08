# Cindrix Radio — Architecture Documentation

Cindrix Radio is a local-first Windows and Android audio application for podcasts, internet radio, local music and audiobooks, karaoke, Bluetooth playback, and internet-independent LAN media streaming/sharing.

**Status:** Product requirements approved; architecture proposed as implementation baseline. No application code is implemented by these documents.

## Architecture index

1. [001 — Product scope and requirements](001-product-scope.md)
2. [002 — System architecture and storage](002-system-architecture.md)
3. [003 — Audio engine and DSP](003-audio-engine.md)
4. [004 — Connectivity, LAN, Bluetooth and sharing](004-connectivity-and-sharing.md)
5. [005 — UI, dashboard, widgets and themes](005-user-interface.md)
6. [006 — Karaoke and lyrics](006-karaoke.md)
7. [007 — Media library, podcasts and radio](007-media-library-and-sources.md)
8. [008 — Security, reliability, testing and delivery](008-quality-and-delivery.md)
9. [009 — Implementation contracts, cross-platform parity and release gates](009-implementation-contracts.md)

## Locked decisions

- Windows and Android are target platforms; no cloud account is required.
- Core online podcast and internet radio functionality must degrade gracefully to LAN-only or device-offline operation.
- Blue, **not orange**, is the UI accent; charcoal and metallic silver are the other principal brand colors.
- Dedicated tabs: Dashboard, Podcasts, Internet Radio, Offline Library, LAN Streaming, Audio Sharing, Playlists, Downloads, History, Settings.
- Audio Sharing means sharing existing audio on a LAN. **No live microphone studio, multi-source broadcast mixer, or scheduled broadcast show system.**
- One Bluetooth audio output at a time is sufficient; playback to four Bluetooth speakers simultaneously is **not** a requirement.
- Local 10-band graphic EQ and karaoke controls use mixer-inspired faders, knobs and meters.
- Karaoke is optional vocal reduction/near-removal plus **manually scrollable, non-timed lyrics**. Original songs remain unchanged; no permanent processed copies.
- Smart playlists and user-authored playlists; default **1-second pause between tracks** with configurable gapless option.
- Two optional widgets: Now Playing and Favorites / Quick Access. Permanent in-app mini-player.
- Recording of permissible streams, audiobooks, ReplayGain, metadata editing, Car Mode, backup/restore, library integrity, and offline support are approved.

## Decision policy

Documented requirements are approved product scope; library/codec/framework selections in the following documents are **proposals**, subject to build-time validation. In particular, Bluetooth features depend on OS and device APIs, and high-quality vocal separation may need precomputation or stronger hardware.

## Implementation gate

Do not claim features work until Windows and Android builds have been tested, including a completely isolated hotspot/LAN test and one-device Bluetooth audio routing test.


## Brand assets

The approved Cindrix Radio logo belongs at `images/logo.png` at the repository root. A reusable full-resolution source should be retained where possible. The identity is a dark graphite/metallic shield with blue radio-wave accents and an antenna motif; avoid conflating branding with a working app build.

## Requirements-to-tests handoff

Use [009](009-implementation-contracts.md) as the cross-platform test contract when implementing documents 001–008. In particular, verify LAN Only with internet disconnected, music default one-second track gap, Bluetooth disconnect safety, and karaoke on-demand vocal reduction with **no permanent processed audio copies**.
