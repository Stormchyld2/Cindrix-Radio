# 001 — Product Scope and Requirements

> **Update 2026-11-08:** connection modes are now only **Online** and **Grid-Down** (no LAN Only). LAN streaming/Audio Sharing moved out of this app into the separate **AirLink** file-sharing app. Videos (with an in-app player), HLS radio, playlists/favorites, karaoke with vocal reduction, the details editor and audiobook handling are built; see 009 §11 for the decision log. Where the text below mentions LAN, read it as AirLink.

## Purpose
Cindrix Radio is an audio library, podcast/radio client, and local-network audio sharing application for Windows and Android. The same library should remain usable in three modes: **Online**, **LAN Only**, and **Device Offline**.

## Features and navigation
| Tab | Responsibility |
|---|---|
| Dashboard | Branded logo header; connectivity; now playing; continue listening; recently added/played; recommendations from accessible catalogs; favorites; queue; downloads/storage; LAN sources; audio sharing |
| Podcasts | RSS/Atom discovery where applicable; subscriptions; episodes; auto-download; OPML import/export; bookmarks and playback position |
| Internet Radio | Directory discovery, manual URLs, favorites, listening history, supported stream recording |
| Offline Library | Music, audiobooks, recordings, downloaded episodes, metadata editing, albums/artists, local search |
| LAN Streaming | Discover and play reachable local servers, shares and streams; manual URL/IP fallback |
| Audio Sharing | Expose selected local music/audio to LAN listeners, optional access controls; not a broadcast studio |
| Playlists | User-created lists (create, edit, reorder, import/export) and rule-based smart playlists |
| Downloads | Queue, network policies, quota, retries, cleanup and integrity |
| History | Continue/resume, recents, bookmarks, configurable history retention |
| Settings | Sources, storage, playback, EQ, karaoke, Bluetooth/output routing, widgets, diagnostics, backup/restore |

## Locked functional behaviors
- Local-first: no required internet, account, analytics, or cloud service. Internet radio and remote podcast feed refresh naturally require connectivity.
- Bluetooth audio output: one selected system-supported speaker, headphones or car stereo at a time. Pause rather than unexpectedly playing on device speakers when Bluetooth disconnects.
- Wi-Fi, wired LAN and phone hotspot can carry local media streams without internet. Device discovery and manual connection must both be supported.
- Audio Sharing is file/stream sharing of existing audio, with simple permissions and listener status. **Exclude live microphones, multi-input broadcast mixing, and show scheduling.**
- Music default transition: **one-second silence between tracks**; allow user to select gapless and other interval settings. Do not insert pause within continuous radio streams or podcast episode content.
- 10-band graphical EQ with mixer-style faders and advanced knobs; custom presets, preamp, bypass and safe gain handling. ReplayGain/loudness matching.
- Karaoke: local/optional lead-vocal reduction or attempted removal; original/reduced/near-instrumental playback modes; plain lyrics with manual scroll, full screen and font size; no timed lyrics requirement. No permanent processed audio or duplicate songs.
- Podcasts: feed subscriptions, discovery, resume, optional automatic download, queue, speed, bookmarks, OPML portability.
- Internet radio: station search, user-added streams, presets/favorites, optional scheduled recording of a stream **only as a personal capture feature** if enabled and legally permitted; this is distinct from prohibited scheduled broadcasting. [Clarification: prior approval included scheduled recordings, not broadcast shows.]
- Audiobooks: dedicated browsing with chapters, reading/listening progress, speed and bookmarks. Support local and reachable LAN media.
- Lyrics: embedded and locally stored, optional internet retrieval subject to licenses/provider terms; no requirement for time synchronization.
- Metadata editor, duplicate detector (never delete without confirmation), library folder monitor, backup and recovery, integrity checker.
- Car Mode, sleep timer/alarm, background playback, playback queue, one or more offline-accessibility indicators, intelligent downloads.

## Dashboard recommendation rules
Recommendations must be computable from accessible information: locally indexed media, downloaded/subscribed content, current network sources, and optional online directories when reachable. No invented online availability. Prefer explainable categories (“New from subscriptions,” “Recently added on LAN,” “Based on favorites”). Cached recommendations must carry freshness/offline indicators.

## Approved future capabilities
LAN listening rooms/shared sessions and enhanced synchronized multi-device playback may be staged later. They are separate from multiple simultaneous Bluetooth output and do not justify adding broadcast production interfaces.

## Non-goals
- No mandatory cloud synchronization, login or telemetry.
- No four-speaker Bluetooth requirement.
- No live microphone broadcasting, broadcast studio/mixer, or broadcast scheduling.
- No automatic saving of vocals-separated instrumentals.
- No synchronized lyrics requirement.
- No assumption that a stream can be recorded or redistributed merely because it can be played.

## Product acceptance
A user can play a downloaded audio file with **no network**; play a remote library file via **isolated hotspot**; switch an ordinary single Bluetooth output; listen to radio/podcasts **when online**; configure a 10-band EQ; show manually scrollable lyrics during karaoke; create a manual playlist; use a smart playlist; and continue listening across app restarts.
