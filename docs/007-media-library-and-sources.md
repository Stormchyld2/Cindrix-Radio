# 007 — Media Library, Podcasts and Internet Radio

## Content ownership and discovery
Cindrix Radio supports user-owned local music and audio, removable drives, downloaded podcast episodes, audiobook collections, compatible LAN media sources, and online podcast/radio catalogs. Imported media defaults to **reference in place** rather than copied files. Optional managed copy is explicit and shows expected storage cost.

## Library model
Entities: Track, Album, Artist, Audiobook, Chapter, Recording, Episode, PodcastFeed, Station, Source, Playlist, PlaylistEntry, Bookmark, PlaybackProgress, Artwork, LyricsReference, DownloadJob, ShareCollection. IDs stable across file renames/moves where file identity can be established. Source reachability is separate from item identity; a temporarily missing LAN file is not deleted from the catalog.

## Playlists
Manual playlists: create/rename/delete, reorder, add/remove entries, duplicate-safe editing, export/import portable playlists (e.g. M3U8). Smart playlists: saved rule definitions (genre, artist, favorites, recently added, played/unplayed, available offline, source, duration). Smart playlists re-evaluate as content changes, but preserve user-specified ordering rules and never lose original rule configuration. Provide optional snapshot/export.

## Podcasts
Subscribe through feed URL and directory discovery. Parse feeds defensively, support common RSS enclosures and stable episode IDs, protect against feed replacement/redirect abuse, track read/download state and resume points. OPML import/export subscriptions. Auto-download rules: Wi-Fi vs metered, maximum episodes, storage quota, background retry, cleanup only within approved retention rules. Offline playback uses downloaded enclosure files, not current feed availability.

## Internet radio
Directory discovery may use Radio Browser or another vetted source; permit manual stream URLs and favorites. Detect actual content format/codec; support station playlist files, reconnect and station artwork when available. Explicitly indicate that internet-only streams are unavailable when disconnected. Stream recording where lawful/allowed is a user feature, including optional scheduled capture; it is **not** broadcast show scheduling and must never imply rights to rebroadcast station content.

## Audiobooks
Import local books and compatible LAN-hosted books, manage chapter metadata, resume/bookmarks and playback speed. Support common file/container formats when the platform decoder supports them. Treat audiobook progress separately from music track history. Never insert the default music-track 1-second gap inside a continuous audiobook recording.

## Metadata and lyrics
Read standard tags (ID3, Vorbis comments, MP4 metadata where supported). Editor offers preview, undo/backups as practical, safe atomic update, and respect read-only LAN sources. Sidecar artwork and lyric files supported. Any network-based lyrics retrieval requires provider terms/license review; local lyrics work offline.

## Scanning and repair
Watch user-selected music folders; debounce filesystem events; perform periodic reconciliation when watch APIs are unavailable or external storage remounts. Identify duplicates through layered fingerprints (source IDs, size, duration, optional hash), present groups and let users decide. Never auto-delete duplicates. Integrity checker reports missing paths, broken playlist entries, missing artwork, truncated downloads and inconsistent metadata; repair by relinking, redownloading where permitted or marking unavailable, not by deleting user records.

## Download engine
Queue states (queued, in progress, paused, verifying, complete, failed), resumable HTTP when supported, checksum/length checks, safe partial files, backoff, storage quotas, per-feed limits, conditional network rules and confirmation for bulk cleanup. Download storage policy is distinct from karaoke temporary buffers.

## Recommendation model
Start with transparent local heuristics: recently played, favorites, same genre/artist, new subscription episodes, recently discovered local/LAN audio. Use only available catalog entries; inaccessible sources may be shown as “unavailable” but not play-ready. No user profiling/cloud analytics needed.

## Portability
Backup/restore profiles, subscriptions/OPML, playlists, bookmarks, history, library references and app settings. Avoid bundling huge audio files by default; offer separately selected archive/media copy with clear size estimate. Migrate schemas safely and retain human-readable metadata exports.

## Acceptance tests
Manual and smart playlist tests; folder rename/relink; removable-drive removal/reappearance; offline episode play; OPML round-trip; truncated download recovery; duplicate detection with no deletion; audiobook chapter resume; offline source filtering in recommendations.
