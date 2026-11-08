# 010 — Settings Architecture, Subtabs, Defaults and Safety

**Status:** Approved settings structure and functional scope; implementation specification (not implemented)
**Date:** 2026-11-07
**Platforms:** Windows and Android
**Related:** [005 UI](005-user-interface.md), [009 Contracts](009-implementation-contracts.md)

## 1. Navigation and interaction contract

**Settings is one top-level application tab, with exactly 17 child subtabs**, not 17 top-level app tabs. On Windows, the main sidebar remains available and Settings contains a secondary list of subtabs in its content pane. On Android, Settings is entered through the application hamburger drawer; settings subtabs use a compact, touch-friendly internal list or selector that does not permanently consume a wide content column. General is the default subtab. Organizing headings are non-navigable: Application (1–5), Media & Libraries (6–10), Connectivity & Data (11–14), System (15–17). Avoid a third navigational hierarchy.

The Settings search at top searches labels, descriptions and related keywords across all subtabs; results navigate to the relevant control with accessible focus. All content scrolls vertically with mouse wheel/trackpad, touch, keyboard and assistive technology but **no visible scrollbar tracks/thumbs**. Avoid horizontal page scrolling, and preserve subtab and scroll position when returning. Use spacious layouts, consistent charcoal/metallic silver/electric blue, text labels rather than icon-only ambiguity, and accessible controls.

Settings are saved automatically upon valid ordinary change, with clear persistence/validation feedback; destructive operations, file relocation, sharing activation, secret changes, restores and resets require a preview/explicit confirmation. Changes affecting running playback use the shared playback coordinator. Controls unavailable on the platform are hidden only where obvious or shown as disabled with an accurate explanation; never display a simulated functional OS switch.

## 2. Default policy

The values below are **recommended starting defaults** unless marked **LOCKED**. Platform-dependent behavior must be detected, not invented. Users may change supported settings after validation; do not silently grant filesystem/network permission or delete source media.

### 01 — General
| Control | Default and allowed behavior |
|---|---|
| Connection mode | **Online / Grid-Down** (two states, no Auto), default Online. Top control on the General page. Grid-Down uses no network at all and shows only local and downloaded content. Also shown as a one-tap switch in the app header and on the dashboard Connectivity card |
| Startup page | Dashboard; alternatives: Last Opened or another available primary tab |
| Launch on startup | Off; Windows only, explicit opt-in |
| Start minimized | Off; Windows only |
| Close window behavior | Exit/stop by default; Windows may offer minimize to tray/keep playback when supported |
| Remember session | On, preserving valid navigation and queue |
| Application language | English initially; localization-ready |
| Time format | System; optional 12/24-hour format |
| Notifications | Important only; All / Important / None, subject to OS permissions |
| Notification sound | Off by default; optional |
| Updates | Notify of available updates when online; manual check and OS/store-specific policy |
| Exit confirmation | Off except when an action could terminate recording, active share or critical transfer |
| Reset General | Confirmation, resets this category without touching content |

### 02 — Appearance
| Control | Default and allowed behavior |
|---|---|
| Theme | Cindrix Dark (charcoal/graphite) default; System/Light optional once implemented |
| Accent | Electric blue, metallic silver for headings; custom theme only if supported |
| Density | Comfortable; Compact or Spacious |
| Font scale | System-respecting Medium; Small/Large/Extra Large where usable |
| Windows sidebar | Expanded default; collapse to icon-only and resize; persist width/state |
| Android navigation | **LOCKED:** hamburger menu + slide-out drawer, not a selectable bottom navigation |
| Dashboard widgets | Reorder, show/hide, restore layout; saved locally |
| Background treatment | Subtle gradient / solid |
| Album art and mini-player size | Medium / Compact by default; supported alternatives |
| Animation | System reduced-motion preference; full/reduced/off as supported |
| Visualizer | Off by default; opt-in |
| Now Playing background | Soft artwork blur or solid |
| Scrollbars | **LOCKED:** visually hidden throughout app while wheel/touch/keyboard scrolling remains functional; informational, not a toggle |

### 03 — Playback
| Control | Default and allowed behavior |
|---|---|
| Music track separation | **LOCKED:** 1.0-second silence; configurable 0 (gapless) or bounded custom value |
| Crossfade | Off; optional enhancement requiring engine proof; cannot overlap the 1-second gap unless user selects crossfade |
| Resume listening | On for podcasts/audiobooks and remembered session; music position optional |
| Startup shuffle/repeat | Off / Off |
| Queue persistence | On; persist current queue and relevant ordering |
| Previous-track semantics | Restart current track after threshold; previous item otherwise, configurable when implemented |
| Seek forward/back | Media-type-aware defaults (e.g. podcast 30/15s); editable |
| Playback speed | 1.0x; media-specific presets |
| Sleep timer | Off; timer and end-of-chapter where meaningful |
| Reconnect/recovery | Finite retries for reachable streaming sources; do not silently start playback after device disconnect |
| Media keys | On when platform supports |
| Buffer strategy | Balanced; Low Latency / Stable with measured tradeoffs |
Note: no artificial silence within uninterrupted streams or a continuous audiobook/podcast.

### 04 — Audio & Equalizer
| Control | Default and allowed behavior |
|---|---|
| Equalizer | Off / Flat default; **10-band EQ** with bounded dB controls |
| Presets | Flat, Rock, Pop, Classical, Vocal, Bass Boost, user-defined presets (calibrated, not marketing claims) |
| Preamp | 0 dB; bounded adjustable gain with clipping warnings |
| ReplayGain | Off by default, user-selectable Track / Album |
| Normalization | Off unless ReplayGain explicitly enabled; avoid double processing |
| Output limiter | On for clipping prevention where feasible; document measurable behavior |
| Balance / mono | Center / Off |
| Output sample format and rate | System/engine automatic; expose supported choices only |
| Audio buffer | Balanced |
| Preset management | Save, rename, delete user presets; never overwrite shipped presets silently |
| Reset EQ | Restore Flat after confirmation |
All DSP shares the same coordinator; changes must not produce damaging pops or runaway gain.

### 05 — Devices & Bluetooth
| Control | Default and allowed behavior |
|---|---|
| Output routing | System-selected compatible output; **one Bluetooth output** baseline |
| On Bluetooth disconnect | **LOCKED:** pause rather than surprise speaker fallback when app can control behavior |
| On reconnect | Remain paused; user may explicitly resume |
| Preferred output | Remember chosen supported device, avoid automatic re-routing to unavailable endpoints |
| Audio interruptions | Respect Android audio focus; pause or duck per system policy |
| Car Mode | Manual; optionally suggest on supported car connection |
| Headset/media buttons | On where supported |
| Device diagnostics | Current route, codec/rate when observable |
| Refresh output list | Requery OS-supported outputs |
Pairing and other OS-managed audio behavior must defer to system controls; no fake in-app pairing implementation.

### 06 — Music Library
- Manage watched music folders (add/remove; removal from index does **not** delete original files).
- Folder scan on demand; optional startup/periodic scan (startup scan On for modest libraries, throttled for large collections).
- Reference files in place **by default**; copy/import only via explicit user action.
- Detect duplicates for review only; never auto-delete.
- Metadata read/edit and cover-art preference: embedded, sidecar/local, optional online lookup requiring connectivity and permission.
- Missing path/removable media: status Missing or Offline, with relink flow; no automatic library purge.
- Rebuild disposable index, view scan results, library statistics, formats and storage usage.

### 07 — Podcasts
| Control | Default and allowed behavior |
|---|---|
| RSS refresh | Manual and on-demand by default; optional timed refresh with online reachability |
| Auto-download | Off |
| Network requirement | Prefer unmetered for automatic downloads; explicit override |
| Per-show retained episodes | Keep until user chooses policy |
| Completed download cleanup | Off by default; constrained to app-managed downloads |
| Playback speed | 1.0x, remember per podcast if selected |
| Skip backward/forward | 15/30 seconds by default; configurable |
| Intro/outro skip | Off; per-show configuration |
| Resume | On |
| OPML | Import/export subscriptions |
| Storage location | Managed podcast folder, user-changeable through safe migration |
Downloads and retention must not delete arbitrary third-party/source files.

### 08 — Internet Radio
- Station directory selection and refresh (online only); favorites/presets.
- Add/edit user-provided station stream URL; clearly distinguish reachable/live, cached metadata, and unavailable.
- Connection timeout/retry with bounded backoff; balanced buffer by default; network diagnostics.
- Recording of streams only when permitted: manual/scheduled **capture**, destination, supported encoding/quality, bounded duration and storage quotas.
- Explicitly disabled by default: recording automation and any lawful LAN relay, subject to separate permissions.
- Station/history retention and failure notification.
- **No live microphone studio, multi-source production mixer or scheduled broadcasting.**

### 09 — Audiobooks
| Control | Default and allowed behavior |
|---|---|
| Book folders | Explicit add/remove references |
| Position resume | On, per book/chapter |
| Speed | 1.0x and configurable |
| Skip backward/forward | 15/30 seconds |
| Chapter navigation | On where chapters exist |
| Bookmarks | Manual bookmarks; optionally add automatic checkpoints |
| Sleep timer | Off, configurable time/end of chapter |
| Progress indicator | Time remaining by default; percentage alternate |
| Mark completed | Manual by default / automatic when end reached |
| Metadata | Local first, optional online lookup |
| Finished books | Retain; optional archive/hide from active view |
Music queues and audiobook progress remain independent.

### 10 — Karaoke & Lyrics
| Control | Default and allowed behavior |
|---|---|
| Default audio treatment | **LOCKED:** Original; user opt-in Reduce / Attempt Removal |
| Processing preference | Balanced default; Fast / Best Available, limited by device resource checks |
| Reduction strength | Adjustable **only when the selected algorithm makes strength meaningful**; vocal-stem fader only for actual separated stems |
| Show lyrics viewer | **On/Off, default Off.** Off hides the lyrics window and shows a Cindrix Radio banner in its place. On shows only user-pasted or user-supplied lyrics; no lyrics ship with the app and no online lookup exists. |
| Lyrics sources | User-pasted text, sidecar .lrc/.txt, embedded tags. Stored locally; never uploaded or shared over LAN. Manage/clear stored lyrics here |
| Lyrics timing | **LOCKED:** plain manually scrollable text; no synchronization/timecodes required |
| Lyrics appearance | 24px illustrative default, scalable with accessibility settings; high contrast available |
| Full-screen lyrics | Optional; do not force on all music |
| Cancellation and feedback | Show processing status, resource usage and Cancel; playback original fallback |
| Processed song retention | **LOCKED:** never persist separated songs/instrumentals/stems as library tracks by default |
| Temporary workspace | Bounded RAM/ephemeral disk only, automatically cleaned on cancel, track change, normal exit and recovery from crash |
No claim of perfect vocal removal or real-time performance on all Android hardware.

### 11 — LAN & Sharing
| Control | Default and allowed behavior |
|---|---|
| Local device name | Editable non-sensitive display name |
| LAN discovery | Enabled for receiving when user opts into LAN features; manual IP/host always possible |
| Sharing service | **Off by default**; explicit user action starts server |
| Shared media | Explicitly selected existing files/collections only; **never expose whole filesystem** |
| Port | Auto/valid custom, display actual endpoint |
| Trusted clients / pairing | Require deliberate authorization for private collections; revoke immediately |
| Access controls | Read-only access where appropriate; authenticated/tokenized as supported |
| Maximum listeners | Conservative adjustable bound |
| Audio relay | Off by default; only when rights and connectivity permit |
| Diagnostics | Interface/IP, reachable status, activity, connection errors |
| Stop All Sharing | Immediate, prominent control |
LAN-only operation must work on isolated Wi-Fi/hotspots without a WAN, using manual connection if discovery fails. Never turn on sharing merely because Wi-Fi becomes available.

### 12 — Downloads & Storage
- Per-category managed directories: podcasts, recordings, downloaded artwork/metadata; display originals separately from copied managed files.
- Downloads: queue, retry, bounded concurrent connections, integrity checks, network/metered rules, optional quotas.
- Storage overview: originals (informational), app-owned downloads, disposable cache, temporary karaoke workspace.
- Cleanup defaults: **never automatically remove original/source audio**. App-owned podcast downloads may follow explicit user policy; show deletion preview and confirmation for destructive operations.
- Safe cache clear (rebuildable indexes/artwork), partial download cleanup, temporary karaoke cleanup and bounded cache sizes.
- Move app-owned storage to new path with capacity check, atomic/copy-verify transition where feasible, failure rollback.
- Missing removable media must be labeled unavailable rather than deleted or interpreted as empty.

### 13 — Privacy & Security
| Control | Default and allowed behavior |
|---|---|
| Listening history | On locally; user can disable or clear |
| Retention | Keep until user changes policy; optional 30/90-day/custom periods |
| Network access | Show what internet/LAN services can contact; optional metadata/lyrics lookups are transparent |
| LAN permissions | Manage trust, access and revocation |
| Diagnostic logs | Minimal error logs; optional expanded diagnostic logging |
| Sensitive data | Redact local paths, tokens, credentials from exported logs when appropriate |
| Clear history/logs | Separate explicit actions; confirm without touching audio |
| Accounts / analytics / ads | **LOCKED:** no required account, mandatory analytics, advertisements or cloud storage |
Disabling UI controls must be backed by true backend permission enforcement.

### 14 — Backup & Restore
- Export/import portable **settings plus metadata**: profiles, EQ presets, playlists/smart rules, podcast subscriptions/OPML, stations/favorites, bookmarks and listening positions.
- Default: exclude audio files and secret device credentials; allow separate explicit media-inclusive workflow only when implemented.
- On-demand backup and optional user-configured scheduled local backup, choosing directory/retention.
- Validate schema/version/checksum; preview included categories, conflicts, missing media paths and platform incompatibilities before restore.
- Backups use atomic writes and protect existing working state with recoverable rollback; no destructive overwrite without confirmation.
- Portable preferences cross Windows/Android; skip or safely remap OS-specific device routing, Windows startup/tray options, paths and permissions.

### 15 — Advanced & Diagnostics
- Read-only engine/build/decoder details, format support, playback state, supported route/sample properties.
- Stream health/retry status, LAN interface/discovery and manual connection test, download failures.
- Library index validate/rebuild without changing authoritative media.
- View/export redacted logs and app-level CPU/memory/cache diagnostics where measurable.
- Karaoke model/engine availability, size and performance diagnostics (without exposing unavailable processing as working).
- Reset Advanced settings with confirmation, not app data.
- Technical controls are guarded, validated and must not leave source files corrupt.

### 16 — About
- Approved logo/brand, app version, build/channel, OS and architecture.
- Release notes, documentation and GitHub repository links.
- Installed third-party software, codec and model licenses/attributions.
- Copyright/credits, support/issue reporting and online-only Check for Updates.
- Clear build status: planned/prototype/tested/released, never infer working capabilities from the documentation.

### 17 — Legal
- Separate subtab holding all legal documents, viewable in-app and offline: **End User License Agreement (EULA)**, **Terms of Service / Terms of Use**, **Privacy Policy**, plus third-party notices link (also in About).
- Each document shows version and last-updated date; the app records which version the user accepted and when (stored locally) and asks again only when a new version changes terms.
- States plainly: user is solely responsible for content they import or add; app does not condone or facilitate copyright infringement; user-added lyrics stay on the device.
- Privacy Policy states: no account, no analytics or ads, no server hosting user content; what stays local (library index, lyrics, settings); what leaves the device only when the user uses it (radio directory lookups, stream playback, LAN sharing they enable).
- Source texts: `docs/legal/`. AI-drafted; require lawyer review before release.

## 3. Persistence, validation and schema

Use versioned local portable files (JSON/YAML as suitable), atomic writes with recoverable previous version, stable IDs for playlists/profiles, and disposable indexes. No authoritative SQL database for user state. Separate:
1. **Portable preferences:** playback settings, appearance preferences where platform-neutral, EQ, media options and dashboard layout.
2. **Device preferences:** Windows sidebar geometry/tray; Android OS routing/permissions; device-specific paths, caches, performance thresholds.
3. **User library and media metadata:** references/IDs and content, backed up independently of app preferences.
4. **Secrets:** platform-protected credential storage only, not plain-text export and not portable by default.

Every setting needs a stable key, type, validated bounds/choices, effective default, applicable platform(s), change policy (immediate/restart/confirmation), privacy/security sensitivity and migration rule. Introduce schema version 1 when implementation begins; never claim a live file format exists before code is written.

## 4. Required acceptance tests

1. Exactly 17 subtabs under one Settings main tab; grouping headers not additional routes; General opens by default.
2. Windows secondary navigation and Android touch-friendly subtab picker have complete parity, searchable labels and accessible focus.
3. Hidden visual scrollbars do not disable mouse wheel, touch, keyboard, screen readers, or scroll-position restoration.
4. Each settings type validates input and persists across clean restart; corrupted config loads a safe recoverable version rather than silently losing data.
5. Reset a category does not alter other categories, media files, imported originals, playlists or subscriptions.
6. Portable export/restore preserves supported preferences but never exports secrets/device-only permissions.
7. Turning on LAN sharing requires explicit authorization, shares selected media only, and Stop All Sharing revokes access promptly.
8. Deleting downloads and moving storage require the applicable confirmation/preview, with no accidental deletion of original tracks.
9. Missing OS support shows honest unavailable/OS-managed explanation; no placebo switches.
10. Music 1-second default gap, Bluetooth disconnect pause, single output, original karaoke playback and no permanent processed copies remain invariant.
11. Audio settings do not create multiple playback engines; changes update the current shared session.
12. Android offline use and isolated-LAN use work without sign-in, ads, telemetry, or WAN dependence.

**Scope note:** This document specifies approved feature areas and recommended defaults. Items explicitly described as optional or conditional require implementation/OS validation before they may appear as active settings. No app implementation is asserted by this documentation.
