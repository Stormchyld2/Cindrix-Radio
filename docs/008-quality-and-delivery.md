# 008 — Privacy, Security, Quality, Testing and Delivery

## Non-functional requirements
- No mandatory accounts, cloud synchronization, ads or analytics.
- Offline and LAN-only operation must be verifiable with WAN access physically disabled.
- Audio playback and queue remain functional when a feed, radio directory, or LAN source is unreachable.
- Protected media should not leak through accidental public sharing; no automatic access to all files.
- UX stays responsive during scans, downloads, separation and streaming reconnect.
- Sensible memory, CPU, battery, temporary-disk and network-use limits.
- Accessibility: keyboard and screen reader support, visible focus, readable visual meters, touch-friendly controls.

## Threat model
Inputs include potentially malicious RSS feeds, MP3 tags, artwork, station playlists, shared media filenames, URL redirects, and network server discovery records. Sanitize paths and HTML; prevent SSRF to privileged localhost addresses for untrusted automatic URLs as appropriate; reject directory traversal; cap payloads, artwork sizes and decompression. Sharing requires explicit selected collection and local authorization, with opt-in credentials when desired. Avoid embedding raw private paths or auth headers in logs.

## Protocol / license review
Test permissibility of bundled codec implementations, DSP, vocal-separation model weights, RSS/radio directory APIs, lyrics providers, stream recording and internet-to-LAN relays. A technically playable stream is not automatically distributable. Preserve third-party notices and comply with app store policy as applicable.

## Testing pyramid
### Domain/unit tests
Network-mode state machine; queue transitions with 1 s default gap; smart playlists; metadata and playlist serialization; position bookmarks; media identity/move; download retries; vocal modes and cleanup; user-selected share scope; EQ preset round-trip.

### Integration tests
Decoder → DSP → output route; Android native media service lifecycle; Windows tray playback; LAN discovery; manual IP; authenticated LAN share; HTTP range seeking; RSS redirect/refresh; download resume; storage interruption.

### Device matrix
- Windows desktop Bluetooth speaker, wired output, removable USB library.
- Android phone/tablet headset, speaker, car audio route, screen lock/background playback and OS battery policies.
- Isolated Wi-Fi with no WAN, hotspot as LAN, local LAN HTTP server.
- Two devices with hotspot reconnection, mDNS working and mDNS unavailable.
- Weak CPU Android hardware for karaoke fallback.
- Real sample-rate/codec coverage, including malformed or unsupported inputs.

## CI / build gates
Proposed GitHub Actions jobs: source lint/type check, domain unit tests, Rust backend tests, Android compile/instrumentation where runner permits, Windows build, dependency/security review, release artifact validation. Build matrix and installer signing should be designed when code/platform stack is chosen. Artifacts must record platform, commit, version and dependency notice package. A green compile is not proof of live audio, LAN discovery or Bluetooth routing—those require physical device testing.

## Release milestones
- **M0 Architecture**: docs, domain contracts, platform proof of concept; no false claims of completed software.
- **M1 Core Player**: local file playback, queue/progress, 1s music gap, offline library, Bluetooth route, dashboard and mini-player.
- **M2 Online Sources**: podcasts/RSS, internet radio, OPML, downloads and rules.
- **M3 LAN**: isolated-LAN source discovery, manual endpoint, receiving, selected-media sharing and secure access.
- **M4 DSP and Karaoke**: 10-band blue mixer EQ, loudness, lyric display, optional local vocal separation and bounded transient data.
- **M5 Experience**: smart/manual playlists, widgets, Car Mode, audiobook workflows, metadata, repair and backups.
- **M6 Hardening**: performance, cross-device sync (if feasible), accessibility, radio record controls, diagnostics, integration acceptance.

Phases are implementation milestones, not a license to omit approved scope; phase boundaries may be adjusted after actual platform validation.

## Completion criteria
Do not mark a feature delivered until it meets automated checks **and** an applicable real-device test. Record expected vs actual behavior, Android model/OS and Windows build versions, network topology, speaker routing, audio formats and errors. Specific acceptance demonstrations: 100% WAN-free LAN music sharing; offline local playback; safe Bluetooth disconnect; functional EQ; karaoke vocal reduction + manually scrolled lyrics without permanent audio duplication; portable playlist/metadata backup and restore.

## Open engineering decisions
Evaluate platform stack and native audio bridge, DSP/filter libraries, optional vocal-separation model and license, source catalog protocols, LAN sharing authentication model, Android widget/background policy and Windows widget packaging. These are design choices awaiting prototypes—not product requirement reversals.
