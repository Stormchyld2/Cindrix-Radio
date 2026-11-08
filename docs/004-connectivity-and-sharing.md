# 004 — Connectivity, LAN, Bluetooth and Audio Sharing

> **Update 2026-11-08:** the LAN discovery, streaming, Audio Sharing, relay and sync sections below now belong to the separate **AirLink** app and are NOT built in Cindrix Radio. Only Online / Grid-Down modes remain (Grid-Down is enforced in the backend). Bluetooth output stays in scope, with full Android support planned (A2DP routing, MediaSession + foreground service, AVRCP metadata, audio focus, pause on disconnect, Android 12+ permission).

## Mandatory connectivity matrix
| Network state | Local files | LAN sources | Internet podcast/radio |
|---|---|---|---|
| Internet + LAN | Yes | Yes | Yes |
| Isolated Wi-Fi/hotspot/LAN | Yes | Yes | No (unless separately cached) |
| No network | Yes | No | No (downloads remain playable) |

Internet reachability and LAN reachability are separate signals. Never require DNS, cloud login, or external API for LAN playback.

## LAN source discovery
Use mDNS/DNS-SD and UPnP/DLNA where applicable; evaluate SMB shared folders and authenticated HTTP audio endpoints where target support is practical. Discovery may be blocked by hotspots, AP isolation or mobile multicast restrictions. Therefore include **manual add by IP/hostname, protocol, port/path**, saved endpoints, timeout/retry and a clear connection test.

## LAN streaming / receiving
Support browsing indexed media from compatible sources, playing shared tracks and playlists, and direct media URLs. Distinguish metadata catalogs from the media byte-stream endpoint. Correctly handle HTTP Range seek for files, MIME types, large files, interruptions and resuming. Never assert a device supports a protocol without probing it.

## Audio Sharing (not broadcast production)
Audio Sharing advertises a user-selected, explicitly shared collection or currently playing local media to other clients on the same trusted LAN. A lightweight HTTP server with read-only endpoints and Range support is a reasonable baseline proposal; add discovery announcements and optional password/pairing. Show share status, local address, listener count, active transfer and Stop Sharing.

Exclusions: no live microphone; no audio-input mixing board; no show/program scheduling. Sharing music should not automatically expose the entire device's filesystem. If sharing licensed external streams, validate redistribution rights; do not silently proxy protected content.

## Internet-to-LAN relay
An optional LAN relay can retrieve one internet stream and expose it to LAN clients where permitted by source terms. The receiving host needs internet; peers may only have LAN access. State should show the upstream dependency explicitly, and the relay must stop or show unavailable when upstream fails. Never promise this relay survives a complete internet outage without local copies.

## Device sync and rooms
Sync bookmarks, playlists and positions between trusted instances over LAN with explicit pairing and conflict-resolution rules. Future LAN listening rooms may align multiple Cindrix clients to one source using time stamps, buffering and drift adjustment. These are not the same as directing one Bluetooth signal to multiple independent Bluetooth speakers.

## Bluetooth
Bluetooth is an **audio output route**, not the LAN transport. User selects one paired system-supported Bluetooth output (speaker, headphones, car audio). Pairing often happens in OS Settings; the app must accurately reflect route availability and support a safe fallback/auto-pause when disconnected. Multiple Bluetooth speakers concurrently are out of scope.

## Security
Bind sharing service to configured LAN interfaces, never publicly expose by default, and display which interface/address is active. Require confirmation before initiating shares. Validate all URL inputs, prevent path traversal and arbitrary file exposure, rate-limit discovery, and support trusted-device permissions. Provide a one-click stop sharing action.

## Verification plan
1. Airplane mode plus explicitly enabled local Wi-Fi/hotspot, with upstream internet disabled.
2. Two devices: server on Windows, listener on Android, then reverse when supported.
3. mDNS unavailable → manual IP works.
4. AP isolation / lost hotspot → clear error, no false-online indicator.
5. Transfer of large audio file with seek and reconnect.
6. Bluetooth car and speaker switching; disconnection causes safe pause.
7. Share access denied for unlisted folder; concurrent clients within tested limits.
