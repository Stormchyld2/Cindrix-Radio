# 009 — Implementation Contracts, Cross-Platform Parity and Release Gates

**Status:** Approved implementation requirements and traceability addendum  
**Date:** 2026-11-07  
**Scope:** Cindrix Radio — Windows and Android

This document supplements, rather than replaces, 001–008. It resolves how approved behaviors must be tested, integrated and presented. It does **not** claim any implementation exists.

## 1. Non-negotiable product contract

- Run in **Online**, **LAN Only**, and **Device Offline** states. Check source-specific reachability; Wi-Fi does not imply internet access.
- A user-owned local file plays with no internet, LAN, account, cloud, or AI dependencies.
- Persistent media is never copied, transcoded, deleted or mutated without an authorized operation. An imported path is a reference by default.
- The music default is **1.0 second of deliberate silence between discrete tracks**; gapless/other duration is user-selectable. Do not impose this on uninterrupted radio audio or a continuous audiobook/episode.
- Exactly one user-selected system-supported Bluetooth audio output is the baseline requirement. On disconnect, prefer pause over surprise speaker fallback.
- Provide **10-band graphic equalizer**, ReplayGain/normalization, named presets, and mixer-like faders/knobs; distinguish active blue controls from orange warning accents.
- Provide user playlists and rule-driven smart playlists, podcast subscriptions/OPML, internet radio favorites and manual URLs, audiobook bookmarks/progress, authorized stream recording, playback history, Car Mode, two optional external widgets, and an always-present mini-player.
- **Audio Sharing** distributes only explicitly selected existing files/streams to trusted LAN clients; it is not a live microphone broadcast studio or multi-source production mixer. Scheduled *capture* of a permitted radio stream is different from scheduled broadcasting.
- **Karaoke:** best practical local lead-vocal attenuation/near-removal with manually scrollable plain lyrics; no sync requirement; original unchanged; **no persistent processed songs/stems by default**. Temporary data must be bounded, disposable and recovered after crashes.
- No mandatory ads, analytics, sign-in, cloud account or third-party audio storage.

## 2. Network mode and capability contract

| Capability | Online | LAN Only | Device Offline |
|---|---|---|---|
| Own local music, audiobook, recording, downloaded episode | Yes | Yes | Yes |
| Local metadata, playlist, resume, EQ, lyrics and karaoke | Yes | Yes | Yes |
| Saved reachable LAN file or share | When connected to LAN | Yes | No |
| Share selected local audio with LAN clients | When connected to LAN | Yes | No |
| Internet-only station stream | If station reachable | Unavailable | Unavailable |
| RSS subscription refresh and episode downloading | If feed reachable | Unavailable | Unavailable |
| Previously downloaded podcast episode | Yes | Yes | Yes |
| Optional internet-to-LAN relay | When upstream stream and rights allow | Unavailable without upstream | Unavailable |

Each item stores **source availability** independently from identity. UI statuses include Available, Cached/Downloaded, Stale, Missing, Unauthorized, Buffering, Unavailable and Retry. An unavailable stream is never presented as playable because a historical catalog entry exists.

A host can have concurrent internet and LAN connectivity. The three user-facing states are simplified capability summaries, not a restriction on independent reachability tests.

## 3. Playback and shared session contract

Exactly one playback coordinator owns playing/paused/buffering/seeking/stopped state, queue, seek position, route, output level, repeat/shuffle and interruption state. All screens, widgets, notifications and platform transport controls dispatch commands to that coordinator. UI controls must not create a second player.

For local files, a deterministic elapsed-time test verifies the one-second default gap, excluding decode/preload delay. Gapless must be gapless at an audio-sample or perceptual continuity level (tested with reference material), not only an instantaneous UI transition. Episodes and audiobooks persist position per item/chapter; radio live streams distinguish a nonseekable timeline. Resume must survive unexpected process shutdown.

Use backoff and bounded stream buffers, explicit Stop semantics, device-change handling and output-interruption policy. DSP must maintain responsiveness during karaoke processing, download, scanning and network transitions.

## 4. LAN discovery and source-sharing contract

A trusted LAN can be Wi-Fi, Ethernet, or phone hotspot with **zero internet**. Implement discovery where available and a manual entry fallback (IP/host, port, path, protocol). Do not make mDNS multicast a prerequisite.

Never share the device's filesystem wholesale. User must select collections or tracks, explicitly start sharing, see address/listeners/activity, revoke permissions, and Stop Sharing in one action. Read-only stream endpoints use sanitized IDs/path resolution, HTTP Range/seek when practical, and access controls appropriate to the selected network. Do not relay externally licensed station streams unless rights permit.

Test bidirectional Windows/Android cases where supported, two-device isolated hotspots, IP-only connectivity, network reconnection, denial for unshared files and large-file seeking.

## 5. Audio processing and karaoke contract

Primary normal playback runs the original decoded signal. Vocal removal is optional and opt-in per karaoke session. Evaluate strong **on-device** stem separation candidates with model license, model size, CPU/RAM and battery requirements, processing delay, audible quality and failure fallback. Algorithm/framework selection is an open engineering decision; neither near-perfect elimination nor realtime performance is guaranteed on all Android devices.

Expose Original / Reduce / Attempt Removal with accurate control labeling. A vocal fader controls the *vocal stem* only when actual separated stems are available; center-channel attenuation uses the label **Reduction strength**. Lyrics can be embedded, sidecar or user-entered and must display as plain scalable, manually scrollable text without timecodes or mandatory synchronization.

**Storage invariant:** the original stays unchanged; processing cannot create persistent output tracks, saved instrumentals or duplicated media library entries. Prefer bounded RAM; any temporary disk workspace is limited, process-private and removed on normal exit, cancel, next track and startup recovery. If preprocessing needs too many resources, display a transparent delay/fallback and allow Cancel. A temporary storage override is not silently enabled.

## 6. Portable media, IDs and backups

Store authoritative profiles, playlists, subscriptions, downloaded-episode metadata, bookmarks, track references, lyrics references, EQ presets and state in versioned portable files. Use stable IDs that survive rename/move where identity remains established. Indexes and caches may be discarded and rebuilt. Treat external removable media as offline, not deleted, until explicitly reconciled. Duplicates must never be automatically deleted.

Atomic writes, previous-version backup on risky mutations, bounded logs, safe schema migrations and recovery tests are mandatory. Explicitly distinguish backup of **metadata only** from user-requested copying of large audio files.

## 7. Windows–Android parity table

| Behavior | Windows | Android | Gate |
|---|---|---|---|
| Local/offline playback, queues and bookmarks | Required | Required | Physical device smoke test |
| Music gap / gapless setting | Required | Required | Audio continuity test |
| 10-band EQ, loudness and karaoke | Required with device-based fallback | Required with device-based fallback | Real CPU, thermal, sample rate tests |
| Background and lock-screen media controls | Tray/media integration | Foreground playback service / MediaSession | OS policy validation |
| Output routing | Wired/system/Bluetooth | Android audio focus, wired/Bluetooth | Disconnection and focus interruption |
| Isolated-LAN receive and sharing | Required | Required subject to OS background restrictions | Two-device test in both directions |
| Podcast/radio and downloads | Required | Required | No-WAN degradation tests |
| Now Playing and Favorites widgets | Desktop widget/floating surface where supported | Native home-screen widgets | Same coordinator and state |
| Car Mode | Simplified controls | Simplified controls and OS policy compliance | Usability test |

The UI can adapt to form factor; capability gaps must be labeled, not concealed. No assumption that Android or Windows offers equivalent media, Bluetooth or notification API behavior.

## 8. Acceptance checklist

### Storage and identity
- [ ] No file copying during normal folder import.
- [ ] Folder rename/remount retains references or offers a non-destructive relink.
- [ ] Broken/corrupt metadata is reported, not silently overwritten.
- [ ] Backup/export/restore preserves playlist order, listening position, subscriptions and EQ preferences.

### Playback
- [ ] Music plays with Wi-Fi and mobile data disabled.
- [ ] 1.0-second track gap measured; gapless selectable and tested.
- [ ] Audiobook and podcast resume after kill/restart.
- [ ] One Bluetooth speaker/headset works; disconnect does not unexpectedly play on speakers.
- [ ] EQ and ReplayGain are safe from clipping/pops and survive restart.

### Network and downloads
- [ ] Offline, isolated hotspot, and internet conditions clearly differentiated.
- [ ] Manual-IP LAN playback works without discovery.
- [ ] LAN file seek and permissions validated with no internet.
- [ ] Radio and RSS content unavailable offline unless cached/downloaded.
- [ ] Download retries, checksum/integrity and quota/cleanup policies verified.

### Karaoke
- [ ] Original mode is unchanged.
- [ ] Strongest viable on-device processing selected for the test platform, with quality/performance evidence.
- [ ] Lyrics scroll without timestamps.
- [ ] No persistent separated output after stop, cancel, crash, relaunch.
- [ ] Processing does not freeze UI or exhaust storage; Cancel/fallback works.

### Product delivery
- [ ] Windows executable/build artifact and Android APK produced by reproducible CI.
- [ ] OS media controls and Android background playback tested on physical devices.
- [ ] Source and third-party codec/model licenses audited.
- [ ] No mandatory cloud account, analytics, advertisements or internet at startup.
- [ ] Versioned release notes distinguish implemented, tested, deferred and unsupported features.

## 9. Repository implementation guidance

Suggested separation (names may be reconciled with actual code when created):

```text
docs/                       approved scope and technical contracts
images/logo.png              approved Cindrix Radio identity
src/                         cross-platform UI and application/domain logic
src-tauri/                   candidate Rust/Tauri desktop/mobile shell
android/                     native media bridge only if chosen
tests/                       playback, persistence, LAN and DSP validation
.github/workflows/           source quality and target builds
```

Do not assume a framework choice is already approved: the Tauri 2 vs native Android service split remains pending platform proof of concept.

**Decision record:** v1 product behaviors are approved; implementation/framework and third-party libraries require verification. This file does not override the more detailed specifications in 001–008.


## 10. Approved UI navigation and accessibility gates (2026-11-07)

Refer to [005 — UI, Dashboard, Widgets and Theme](005-user-interface.md) for the authoritative layout specification. These are implementation-level release gates and supersede any older suggestion of Android bottom navigation:

- **Windows:** persistent left sidebar is user-resizable, collapses to an icon-only rail and remembers its state; responsive layouts prevent main-page horizontal overflow.
- **Android:** hamburger menu in the app bar opens a left slide-out drawer holding all primary sections. The drawer closes on destination selection, outside tap, Back and supported swipe gesture; no permanently occupied left panel.
- **Every screen:** vertical scroll with mouse wheel/trackpad, touch and keyboard while **scrollbar tracks/thumbs remain hidden**. Hiding scrollbar visuals must not disable accessibility or scrolling.
- **Persistent mini-player:** remains available and shares one playback coordinator; selecting it opens a dedicated expandable/full-screen **Now Playing** page.
- **Now Playing context:** music, podcast, audiobook and radio show their own relevant controls rather than displaying all controls at once.
- **Dashboard:** spacious cards with user-configurable visibility/order and saved preferences; overflow content belongs below the fold, not packed into miniature panels.
- **Unified search:** music, podcasts, audiobooks, playlists and accessible radio catalogs with filters and source availability.
- **Navigation continuity:** return to the prior section's scroll offset, filters and selection where valid; Android and Windows share user-facing functionality but adapt navigation per platform.

UI testing must include narrow Windows window, desktop wheel and keyboard, Android portrait gestures, screen-reader navigation, drawer accessibility, and hidden-but-functional scroll. These design approvals do not imply implementation is complete.


## 11. Settings delivery gates (2026-11-07)

Implement the complete Settings specification in [010 — Settings Architecture](010-settings-architecture.md). **Settings remains a single top-level tab with 17 subtabs.** General opens by default; Windows uses secondary Settings navigation and Android uses a compact in-Settings subtab selector entered via the main hamburger drawer. Four category headings organize rather than add hierarchy. Include searchable controls, proper persistence and safe recovery, supported platform-only options, category-specific resets, and backup/restore with no secret export. Validate that no Settings action silently deletes user originals, enables LAN sharing, or creates persistent karaoke stems. Test hidden-but-functional scrolling, dangerous-action confirmations, single playback-coordinator updates, and Windows/Android setting parity. Defaults in 010 that are not marked LOCKED are suggested v1 choices and may be refined during platform testing; conditional or optional features must never be displayed as available before implementation.


## 11. Decision log — mockup review (2026-11-08)
Approved by the product owner during UI mockup review; supersedes earlier wording in 001–007 and 010 where they differ.

- **Scope:** Cindrix Radio is a general media player (music, radio, podcasts, audiobooks, **videos**, karaoke, LAN share). Video is in scope, including locally stored YouTube downloads; first release plays and lists them.
- **Stack:** Tauri 2 + Svelte/TypeScript + Rust on Windows; Tauri 2 Android for the phone. The Android background/lock-screen playback proof of concept stays a gate; keep the audio core in Rust so a native Media3 host remains possible.
- **Sources:** up to **5** drives/folders, each tagged for Music, Podcasts, Audiobooks and/or Videos. Settings subtab 06 is named **Library & Sources**.
- **EQ:** one global EQ control for all audio **except** audio with its own mixer (Karaoke uses its faders). The global EQ is not applied to karaoke mixer output.
- **Dashboard:** no second Now Playing module (the persistent bottom player is the only one). Stat buttons: Library, Radio, Playlists. Favorites carousel (two rows, 20 slots) sits under Quick Tools. Right-column cards are fixed size and scroll internally with no visible scrollbar. Record icon is red.
- **Bottom bar:** EQ button, Library button, expand button.
- **Downloads:** no dashboard card; lives in Settings → Downloads & Storage plus offline badges/filters.
- **Saves:** default root is `Downloads/Cindrix/<Kind>` (Radio, Music, Podcasts, Videos). Radio favorites and recordings default to `Downloads/Cindrix/Radio`. The root and each kind can be changed.
- **Radio directory:** Radio Browser plus manual stream URLs and M3U/PLS import. SHOUTcast/iHeart directories are not used. Stream-rights documentation is required before store release.
- **Lyrics:** no bundled lyrics, no online lookup. A Settings toggle (default Off) enables a lyrics viewer showing only text the user pasted or supplied; when Off the space shows a Cindrix Radio banner. See 006 and 010.
- **Android karaoke view:** when a karaoke song starts playing, Android switches to a full-screen Karaoke view: lyrics (or the Cindrix Radio banner when the lyrics setting is off), the vocal remover/reducer controls (mode and strength, or Music/Vocal faders when stems exist), and the normal playback controls (play/pause, previous/next, seek). When the song ends or the user stops, the app returns to the karaoke song selection. Back/collapse returns to selection without losing playback. Windows may use the same layout in its expanded Now Playing view.
- **Podcasts tab:** a dedicated sidebar tab (and Android drawer entry) shows subscribed podcast feeds as cards (cover art, title, new-episode count). Opening a card lists episodes with Play, Queue, Download and Mark played. Feeds are added by RSS URL or OPML import; downloads default to `Downloads/Cindrix/Podcasts`. Directory search is deferred. Starter prepper feeds are suggestions the user picks, and each is verified before inclusion.
- **Station list file (2026-11-08):** the user's radio stations (favorites, manual stream URLs, and favorited Radio Browser stations) are stored in a human-readable `radio_stations.json` in the Radio folder (default `Downloads/Cindrix/Radio`, changeable), each with a stable ID, name, stream URL and optional artwork/tags. It is importable/exportable and portable between devices. Opening a station reads the URL from this file and hands it to the player. Starter station suggestions are verified before inclusion; none are hardcoded from memory.
- **Podcast feed handling:** there is no server. The app itself refreshes subscribed RSS feeds on a user-controlled schedule (default: on launch and while on unmetered networks), reads each episode's `<enclosure>` and GUID, and downloads missing episodes into `Downloads/Cindrix/Podcasts/<Show Name>/`. Episodes are matched by feed GUID, not filename. Default policy keeps the newest episodes only (user-set count and disk quota); nothing is deleted automatically. Truncated downloads are detected and retried. Offline, the app lists and plays from the saved episode index and local files.
- **Audiobooks:** default folder `Downloads/Cindrix/Audiobooks`, plus any drive tagged Audiobooks. A folder is a book (folder name is the title fallback; `Author/Book/` layout supported); chapter files are ordered naturally (2 before 10); tags supply title, author, chapter names and cover when present; resume position and bookmarks are kept per book.
- **Android Bluetooth and signing (2026-11-08):** on Android, Bluetooth audio to a car stereo, speaker or headphones must work fully: playback routes through the system A2DP output; the app registers an Android MediaSession and a foreground media service so lock-screen, notification and car/headset controls (play, pause, next, previous) and track/artist/art metadata work (AVRCP); audio focus is handled correctly (pause for calls, duck for navigation); the Output button and Settings → Devices & Bluetooth show and switch the connected audio device (requires the Android 12+ nearby-devices/Bluetooth-connect permission); a Bluetooth disconnect pauses instead of switching to the phone speaker. Bluetooth stays an audio route only. File and library sharing between devices (including via Cindrix AirLink) uses the local network, not Bluetooth. This work is part of the Android proof-of-concept gate (doc 002) and needs a native bridge. Android release builds are signed in CI with one persistent release key held in encrypted repository secrets (never committed), with an always-increasing version code, so updates install over existing installs; the key is backed up by the owner. Windows stays an unsigned exe for now.
- **Download for offline (2026-11-08):** wherever content can be fetched from somewhere other than this device, the item gets a Download / "Make available offline" control and an Offline badge, with an Offline-only filter: podcast episodes, audiobooks and videos held on LAN/other-device sources, and any shared item the user is allowed to access. Downloads save to the matching folder under `Downloads/Cindrix/` (Podcasts, Audiobooks, Videos, Music) or a folder the user picks, show progress, resume or retry on failure, check integrity, and never delete automatically. Live radio cannot be downloaded; it can be recorded only where lawful and permitted (see Recording). Items already stored on the user's own drives need no download. Purpose is personal offline listening, like offline mode in streaming services. The app does not bypass DRM, copy protection, logins or access controls, and does not download from services that forbid it.
- **Connection mode (2026-11-08):** a global mode with three states: **Auto** (follows actual connectivity), **Online**, and **Grid-Down**. Grid-Down hides everything that needs the internet (directory search, live streams, undownloaded podcast episodes, update checks) and shows only local and downloaded content from the user's drives and `Downloads/Cindrix/` folders. It also stops all background network activity (feed refresh, directory pings) to save power and send nothing out. The dashboard Connectivity card shows and switches the mode. Switching is instant and never interrupts what is playing.
- **Folder metadata file:** podcast, audiobook and other folders may contain an optional human-editable `metadata.txt`, one entry per line: `filename | display title | date (YYYY-MM-DD) | description`; blank lines and lines starting with `#` are ignored; a literal pipe in text is written `\|`; UTF-8. Filenames match case-insensitively. Precedence: `metadata.txt` entry, then embedded tags, then the file name. Files without an entry just show their file name. The app writes this file into each podcast folder it downloads into, so a folder stays self-describing and portable. The library index is a rebuildable cache: files and these text files are the source of truth, and a rescan rebuilds the index.
- **Resume playback:** podcasts and audiobooks (and any item the user bookmarks) remember their exact position. The position is saved every 15 seconds while playing and on pause, stop, track change and app close, so a crash, a dead battery or Android killing the app loses at most seconds. Stored in the app data folder (not on media drives, which may be read-only or unplugged) as a human-readable `playback_history` file written atomically (temp file then rename) so a power loss cannot corrupt it. Entries are keyed by source plus path relative to the source (falling back to full path), so drive-letter changes do not lose progress. Items finished are marked played; reopening an item resumes at its saved position.
- **Connection mode placement:** the canonical control is Settings → General (top row, large and clearly labeled), with a one-tap switch always visible in the app header and a copy on the dashboard Connectivity card.

- **Video player and web video (decided Nov 8, 2026).** Videos play in the built-in HTML5 player (mp4/m4v/webm; local files through Tauri's asset protocol, scope `**`). The Videos tab searches YouTube (private build), Internet Archive and PeerTube (Sepia Search), each result with Watch and Download. YouTube is watch-by-embed and download-by-yt-dlp, which lives behind the `ytdl` cargo feature (default on). **Production builds must use `--no-default-features`** (CI runs `cargo check --no-default-features` on every push to prove it compiles out). yt-dlp is not bundled; it is downloaded to the app data folder on demand. Downloads go to Downloads/Cindrix/Videos/<channel>/. Grid-Down blocks search and download but local videos still play.

- **Any-site download and bundled yt-dlp (decided Nov 8, 2026).** In the private build the Videos tab has an "Any website (paste a link)" source that hands the link to yt-dlp (its supported-sites list; DRM-protected streams are not supported and will not be added). yt-dlp.exe is embedded in the private Windows exe (CI fetches the latest release into `src-tauri/ytdlp/`, gitignored; `build.rs` embeds it, or an empty placeholder when absent) and unpacked to the app data folder on first use; Update fetches a newer one. ffmpeg is not bundled, so only formats that need no merging are downloaded. Production builds (`--no-default-features`) contain none of this.

- **Android YouTube/any-site download (decided Nov 8, 2026).** The private Android build will include the open-source yt-dlp Android library (youtubedl-android, which packages yt-dlp with its own Python and ffmpeg; about 40 to 60 MB extra) so the phone can download from YouTube and any site yt-dlp supports, matching the private Windows build. Android cannot run a helper downloaded at runtime, so it must be bundled in the APK. It goes behind the same `ytdl` switch and is excluded from production/store builds. To be built when the Android build starts (after the Tauri Android proof of concept). Need to verify at that time: the library's current maintenance status and license terms, ABI size impact (use per-ABI APK splits), and how it is called from the Tauri Android plugin layer.
