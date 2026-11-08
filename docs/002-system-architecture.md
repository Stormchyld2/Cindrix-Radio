# 002 — System Architecture and Storage

## Architecture principle
Keep the user's content and preferences durable and portable, with networking and indexes disposable. Implement platform adapters rather than assuming Android and Windows have identical media controls or background-service rules.

## Proposed layers
```text
Presentation
  Dashboard | Library | Podcasts | Radio | LAN | Audio Sharing | Playlists | Downloads
             | Mini-player | Widgets | EQ/Mixer | Karaoke | Car Mode
               |
Application services
  PlaybackCoordinator | Queue | Library | Discovery | Subscriptions | Downloads
  AudioSharing | Recommendation | DeviceRouting | PlaybackHistory | Backup
               |
Domain + persistence
  Media IDs | Source IDs | Playlists | Feed entries | Bookmarks | Profiles
  Local folders + portable metadata + disposable search/index cache
               |
Platform adapters
  Windows desktop audio/media integration | Android MediaSession/background service
  Decoder | Audio DSP | Bluetooth route | Networking | Notifications | Filesystem
```

## Technology approach (proposed, not locked)
- Desktop shell: Tauri 2 + Svelte/TypeScript + Rust services is a candidate that aligns with other Cindrix products.
- Android: either Tauri 2 Android with native Kotlin service/media bridges or a native Kotlin/Jetpack Media3 audio host. Validate lock-screen/background playback, foreground-service restrictions and routing **before** fixing this choice.
- Reuse domain models and protocol test vectors across targets; avoid trying to force platform-specific audio APIs into a single lowest-common-denominator implementation.
- Prefer a standard media-decoder framework supported by the target platform (e.g. native Media3/ExoPlayer on Android). Confirm codec and licensing profiles per platform.

## Durable layout (conceptual)
```text
CindrixRadio/
  library/                 # user-imported audio only when managed-copy is selected
  podcasts/<feed-id>/       # downloads and stable episode metadata
  recordings/              # authorized radio recordings
  playlists/               # portable playlist documents
  profiles/                # EQ and app preferences
  subscriptions/           # OPML export and feed configuration
  metadata/                # book positions, source IDs, bookmarks, history
  cache/                   # expendable: artwork, waveform, scan indexes, karaoke temp
  logs/                    # bounded; no secret token dumps
```
Do not duplicate user media automatically; support references to externally managed folders and removable drives.

## Identity and file moves
Use stable opaque IDs for logical media, sources, subscriptions, and playlists. Do not use title or absolute path alone as identity. Preserve a canonical URI plus file fingerprint (size, modified time, optional content hash) and keep moved/missing media as recoverable references; let the library integrity checker relink them. Detect duplicates without silently removing media.

## Persistence
Authoritative user data is ordinary files and metadata (JSON/YAML/M3U/OPML as suitable); indexes can be rebuilt. Atomic writes (temporary file + fsync/rename where possible), schema versions, migrations, and backups are mandatory. A local database is **not assumed**; design works without authoritative SQL storage. Only use cache tech if explicitly approved and demonstrably disposable.

## Events and state
Publish typed internal events for PlayerState, QueueChanged, OutputRouteChanged, NetworkModeChanged, SourceAvailabilityChanged, DownloadState, SourceCatalogRefreshed and ShareSessionChanged. All UI panels and widgets observe a single authoritative playback coordinator so widgets never invent their own playback state.

## Network modes
- Online: internet-dependent catalog/feed requests allowed after configured consent/network policy.
- LAN Only: no external host dependencies; local discovery, playback and media sharing continue.
- Device Offline: no LAN requirement; downloaded/local media, playlists, metadata, EQ and karaoke remain usable.
Use reachability checks against the actual endpoint, not just Wi-Fi connectivity. A hotspot with no upstream internet must be recognized as LAN-only.

## Resource boundaries
Downloads, file scans, art extraction, radio recording, DSP and vocal processing run off the UI thread. Bound memory, worker concurrency and temporary disk usage. Recover from sudden removal of LAN storage, USB, Bluetooth, or internet without freezing the player.

## Staged implementation
1. Cross-platform domain schemas and library/queue engine.
2. Windows + Android media playback/background lifecycle.
3. Online podcast/radio clients and downloads.
4. LAN discovery/source play and LAN audio sharing.
5. EQ, karaoke engine, UI widgets, device routing.
6. Reliability, backup/restore, car mode and extended features.
