# 005 — UI, Dashboard, Widgets and Theme

> **Update 2026-11-08:** current tabs are Dashboard, Library, Radio, Podcasts, Videos, Karaoke, Playlists, Favorites, Settings. LAN Streaming and Audio Sharing tabs are removed (AirLink). Of the 17 settings subtabs, 5 are built; the rest are pending. Phone layout (hamburger drawer) is next with the Android work.

## Design system
Follow Cindrix Overwatch family identity without reusing orange as the interactive accent. Primary app surfaces: charcoal / graphite (#111827 and #1E293B as initial candidates), metallic silver for branding/headers, **electric blue (#3B82F6 initial candidate)** for active navigation, focus, faders, rings, buttons and meters. Actual color values require design approval; role of blue is locked. Respect contrast, high DPI, dark mode and reduced motion.

## Windows and Android navigation
- Windows: **persistent, user-resizable left sidebar**, collapsible to an icon-only rail; top branding with Cindrix Radio logo, central workspace, persistent bottom mini-player, optional right context panel.
- Android: **hamburger menu in the top app bar opens a left slide-out navigation drawer** containing the same main sections as Windows. Closing behaviors: tap outside, swipe drawer away, Back, or select destination. When closed, page content uses full available width. No permanent left sidebar or bottom-tab replacement. Persistent compact playback controls and large touch targets.
- Tabs: Dashboard, Podcasts, Internet Radio, Offline Library, LAN Streaming, Audio Sharing, Playlists, Downloads, History, Settings.
- Mini-player remains visible across tab transitions and reflects one shared playback session. Opening EQ or karaoke should not interrupt audio. Tapping/clicking the mini-player opens a dedicated, expandable **Now Playing** view with artwork, queue, lyrics (when available) and relevant audio controls. Windows can expand it into an immersive/full-screen view; Android can present a full-screen player.

## Dashboard
Top section: logo, current playback state, current Online / LAN Only / Device Offline indicator; separate icons for Bluetooth output route and network availability.
Modules: Now Playing, Continue Listening, Recently Played, Recently Added, Recommended from Available, Favorites, Listening Queue, LAN Sources, Downloads/Storage, Audio Sharing status.

Dashboard widgets can be shown/hidden/reordered, with layout stored locally. The visible dashboard should prioritize only the most important cards above the fold; additional content belongs farther down the vertically scrollable page rather than compressed into an overcrowded grid. Never fabricate recommendations when the available source is offline. Label stale, cached and reachable items.

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

**Settings is one primary application tab containing 17 subtabs**, fully specified in [010 — Settings Architecture](010-settings-architecture.md). General opens by default. The subtabs, in order, are General; Appearance; Playback; Audio & Equalizer; Devices & Bluetooth; Music Library; Podcasts; Internet Radio; Audiobooks; Karaoke & Lyrics; LAN & Sharing; Downloads & Storage; Privacy & Security; Backup & Restore; Advanced & Diagnostics; About; Legal.

Show four non-clickable grouping headings inside Settings navigation: Application (1–5), Media & Libraries (6–10), Connectivity & Data (11–14), System (15–17). Windows uses secondary settings navigation within the content pane; Android uses a compact touch-friendly internal list/selector after entering Settings from the main hamburger drawer. Neither layout adds a third navigation level.

Provide cross-subtab settings search, stateful back navigation, category-scoped reset, platform capability labeling, and ordinary-setting autosave. Confirm file moves/deletions, sharing activation, settings restore and other destructive/sensitive actions. Retain the approved vertical scrolling **without visible scrollbars**. See 010 for each control, recommended default, data boundaries and acceptance criteria.

## State and feedback
Always distinguish paused, buffering, reconnecting, unavailable, offline-download available and sharing-active. Use blue for selected interaction; warning/error colors permitted for genuinely exceptional states. Show unsupported audio format or unavailable device concretely. Provide empty states and actionable retry.

## Acceptance
Screen-reader/keyboard operable desktop mixer and Android touch controls; active mini-player unaffected by navigation; dashboard recommendation list changes correctly with network mode; widget/media controls stay in sync; karaoke lyrics manually scroll without timing metadata.


## Approved UI refinement — navigation, scrolling and screen composition (2026-11-07)

**Visual direction:** use Cindrix-family charcoal/graphite, metallic silver and electric-blue accents. Layouts must be spacious and legible, with no requirement that all information fits in one viewport. The approved imagery is a concept reference, not a pixel-perfect UI specification.

### Scrolling
- Scroll each page **vertically** by mouse wheel/trackpad on Windows and touch gesture on Android. **Do not render visible scrollbar tracks or thumbs** in normal use, including nested content areas; scrolling and keyboard/screen-reader support must remain fully functional.
- No horizontal page scrolling on desktop. Content should wrap or reflow at narrower window widths. Avoid clipping important text or controls. Horizontal gestures within bounded media carousels can be allowed when clearly navigable, but must not cause the full page to overflow.
- Fixed navigation and bottom mini-player remain independently visible as page content scrolls. Account for their dimensions so the final card is not obscured.
- Preserve position and context when changing tabs and returning (scroll offset, active filter, selected item) when practical, without preserving stale content in a misleading way.

### Cross-platform layout
- **Windows**: resizable left navigation; collapse/expand to icon rail; remember sidebar width and state locally. Spacious content surface with responsive card columns. At narrow desktop widths collapse/reflow before text overlaps.
- **Android**: hamburger toggle and accessible slide-out drawer, closed by default for maximum content width. Use single-column cards on narrow screens, maintain adequate touch targets and safe areas, and provide readable vertical lists.
- Never show permanent scrollbars merely to indicate scrollability; use the natural continuation of cards, page headings and familiar swipe/wheel behavior to imply further content.

### Dedicated Now Playing
- Full player is reachable from the persistent mini-player and does **not** create a separate audio playback session. Artwork, title/creator/source, transport controls, progress or nonseekable stream status, output device and queue controls are primary.
- Show context-specific secondary actions: **music** uses shuffle/repeat, playlist, EQ, lyrics and optional karaoke; **podcasts** use playback speed, episode info, bookmarks and downloads; **audiobooks** use chapter navigation, speed, sleep timer, bookmarks/progress; **internet radio** uses station details, favorites, and recording controls only where allowed.
- Advanced control surfaces appear on demand, not as permanent dashboard panels. Lyrics are manually scrollable and karaoke remains opt-in.

### Discovery, dashboard and continuity
- Provide a **unified search** across indexed local music, audiobooks, podcasts, playlists and currently available internet radio catalogs, with type filters and explicit source reachability indicators. No invented offline results.
- Dashboard cards may be reordered/hidden; retain preferences locally and offer Restore defaults. Make room for vertical browsing rather than dense, tiny tiles. Detailed listening functions belong in their respective tabs.
- Maintain page navigation state across tab changes and reopen as practical (scroll offset, filters, selection); playback/queue persist independently.
- The desktop and Android experiences share features and design tokens but may use different navigation affordances. Android hamburger drawer **supersedes** the earlier bottom-navigation/side-overflow proposal.

### Implementation notes
- Hidden scrollbars are a **presentation** requirement only: do not disable overflow, wheel, touch, keyboard or assistive scrolling. Validate with desktop wheel, touchpad, Page Up/Down, focus navigation and Android swipes.
- Prefer responsive CSS layout constraints and logical spacing tokens. Do not force wide desktop mockup proportions onto Android.
- On smaller displays, reduce column counts before reducing font or controls to unusable sizes.

## UI acceptance additions

1. Windows sidebar resizes, collapses to icon-only mode, and persists its chosen width/state.
2. Android hamburger menu opens/closes a navigation drawer with all ten sections; choosing one navigates correctly and closes the drawer.
3. Dashboard and interior screens scroll vertically with wheel/touch but show no visible scrollbars; keyboard and screen-reader operation remain available.
4. Desktop page width reflows with no unintended horizontal page scroll; Android phone viewport stays readable as a single column where needed.
5. Mini-player stays fixed, always reflects one playback coordinator, and opens the Now Playing screen without interrupting playback.
6. Now Playing presents the correct music/podcast/audiobook/radio controls and does not overload the dashboard.
7. Dashboard card order/visibility and return navigation state (scroll position, selection, filters) persist as specified.
8. Unified search filters media types and correctly labels unavailable/online-only entries.
