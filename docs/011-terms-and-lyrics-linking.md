# 011 — Terms of Use and Lyrics Linking

## Terms of Use / EULA (required)
Because users can add their own text (lyrics) and import their own media and stream URLs, the app includes Terms of Use inside the app and on any download page.

- Shown on first run; the user must accept to continue. Reachable any time at **Settings → Legal**, a separate subtab holding the EULA, Terms of Service and Privacy Policy (see 010, subtab 17).
- Shown again, briefly, the first time the lyrics viewer is switched on.
- Must state: the user is **solely responsible** for all content they import, add, paste, record or stream; Cindrix Radio **does not condone, facilitate or encourage copyright infringement**; the app ships with no lyrics and no licensed content; stream recording is only for content the user is permitted to record; the user must follow the laws where they live.
- Draft text: `docs/legal/TERMS-DRAFT.md`. **Draft only. A qualified IP lawyer must review it before any public release.**

## Local storage only
User-added lyrics are stored only on the device (local database or app-data file). They are never uploaded, synced to a server we control, included in LAN Share, or included in backup exports unless the user explicitly includes them in a local backup file. The app has no server that hosts user content.

## Labeling
The feature is labeled honestly: **My lyrics (added by you)**. It is not renamed to disguise its purpose. The Off default and the Terms are the safeguards. App-store listings describe the app as a media and radio player.

## Linking lyrics to a song
When adding lyrics (Karaoke screen, track menu):
1. **Song field with type-ahead.** As the user types, a dropdown autofills with matching songs from the library (title, artist, album, filename) and shows the source drive. Arrow keys and Enter select; tap selects on Android.
2. **Browse button** beside the field opens the file picker so the user can pick the song file directly (including files outside the library); the app matches it to the library or links by file path.
3. Paste or type text, or choose **Import .txt/.lrc** to load a file. Timing tags are ignored.
4. Save links the lyrics to that song's identity and survives renames/moves where identity can be established.
5. Manage lyrics lists every stored lyric set with search, edit, unlink and delete (per song or all).

Available only when **Show lyrics viewer** is On.
