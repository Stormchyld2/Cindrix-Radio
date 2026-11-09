import { invoke } from "@tauri-apps/api/core";

export type Kind = "Music" | "Radio" | "Podcast" | "Audiobook" | "Video";
export interface Track { id: string; title: string; subtitle: string; kind: Kind; duration_secs: number | null; path?: string | null; url?: string | null }
export interface PlayerState { current: Track | null; queue: Track[]; playing: boolean; position_secs: number; volume: number; error?: string | null; now_playing?: string | null; recording?: boolean; connecting?: boolean }
export interface Folders { root: string; radio: string; music: string; podcasts: string; audiobooks: string; videos: string; recordings: string }

export interface EqSettings { bypass: boolean; preamp_db: number; gains_db: number[]; vocal_reduction: number }
export interface Prefs { text_scale: number; accent: string; high_contrast: boolean; reduce_motion: boolean; skip_back: number; skip_fwd: number; auto_refresh_podcasts: boolean; radio_country: string | null; radio_order: string | null; keep_awake: boolean }
export interface Settings { lyrics_viewer: boolean; terms_version: string | null; eq: EqSettings; grid_down: boolean; pause_on_disconnect: boolean; prefs: Prefs; download_root: string | null }
export interface StorageRow { label: string; path: string; bytes: number }
export interface BackupInfo { name: string; bytes: number; made: number }
export interface Diagnostics { version: string; os: string; arch: string; data_dir: string; save_root: string; library_items: number; private_build: boolean }

export interface Episode { guid: string; title: string; enclosure_url: string; published: string | null; duration_secs: number | null; file: string | null; played: boolean }
export interface Feed { id: string; url: string; title: string; image: string | null; episodes: Episode[] }
export interface ImportResult { feeds: Feed[]; failed: string[] }

export interface LibItem { id: string; source_id: string; path: string; kind: Kind; title: string; artist: string; album: string; genre: string | null; folder_genre: string | null; group: string; duration_secs: number | null; published?: string | null; description?: string | null; available: boolean }
export interface Source { id: string; path: string; kinds: Kind[]; builtin?: boolean }
export interface SourceStatus { source: Source; online: boolean; items: number }
export interface ScanSummary { scanned: number; added: number; skipped_offline: string[] }
export interface Page { items: LibItem[]; total: number }
export interface AlbumRow { album: string; artist: string; count: number }
export interface Counts { music: number; podcast: number; audiobook: number; video: number }

export interface LyricsSummary { path: string; title: string; chars: number }
export interface Playlist { id: string; name: string; items: Track[] }
export interface PlaylistData { playlists: Playlist[]; favorites: Track[] }
export interface VideoHit { id: string; source: string; title: string; channel: string; duration_secs: number | null; thumbnail: string | null; page_url: string; embed_url: string | null }
export interface VideoSource { id: string; label: string; can_download: boolean }

export interface Station { id: string; name: string; url: string; favicon: string | null; tags: string | null; country: string | null; codec: string | null; bitrate: number | null; rb_uuid: string | null }
export interface StreamInfo { url: string; name: string | null; genre: string | null; bitrate: number | null; content_type: string | null }
export interface ImportStations { stations: Station[]; added: number }

const inTauri = typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

// Browser fallback so the UI can be developed with plain `npm run dev`.
let mock: PlayerState = { current: null, queue: [], playing: false, position_secs: 0, volume: 0.8 };
const call = async <T>(cmd: string, args: Record<string, unknown> = {}, fb: () => T): Promise<T> =>
  inTauri ? invoke<T>(cmd, args) : fb();

const defaultPrefs: Prefs = { text_scale: 1, accent: "blue", high_contrast: false, reduce_motion: false, skip_back: 15, skip_fwd: 30, auto_refresh_podcasts: true, radio_country: null, radio_order: null, keep_awake: false };
let mockSettings: Settings = { lyrics_viewer: false, terms_version: null, grid_down: false, pause_on_disconnect: true, prefs: defaultPrefs, download_root: null, eq: { bypass: false, preamp_db: 0, gains_db: Array(10).fill(0), vocal_reduction: 0 } };

const desktopOnly = (): never => { throw new Error("Podcasts work in the desktop app, not in the browser preview."); };

export const api = {
  storageReport: () => call<StorageRow[]>("storage_report", {}, () => []),
  setPrefs: (prefs: Prefs) => call<Settings>("set_prefs", { prefs }, () => (mockSettings = { ...mockSettings, prefs })),
  setPauseOnDisconnect: (on: boolean) => call<Settings>("set_pause_on_disconnect", { on }, () => (mockSettings = { ...mockSettings, pause_on_disconnect: on })),
  setDownloadRoot: (path: string | null) => call<Settings>("set_download_root", { path }, () => (mockSettings = { ...mockSettings, download_root: path })),
  privacyClear: (what: "history" | "lyrics" | "library") => call<void>("privacy_clear", { what }, () => undefined),
  backupMake: () => call<BackupInfo>("backup_make", {}, desktopOnly),
  backupList: () => call<BackupInfo[]>("backup_list", {}, () => []),
  backupRestore: (name: string) => call<number>("backup_restore", { name }, desktopOnly),
  backupDelete: (name: string) => call<void>("backup_delete", { name }, desktopOnly),
  engineTrace: () => call<string[]>("engine_trace", {}, () => []),
  diagnostics: () => call<Diagnostics>("diagnostics", {}, () => ({ version: "browser", os: "browser", arch: "", data_dir: "", save_root: "", library_items: 0, private_build: false })),
  openBluetooth: () => call<void>("open_bluetooth_settings", {}, () => undefined),
  radioStations: () => call<Station[]>("radio_stations", {}, () => []),
  radioAdd: (station: Station) => call<Station[]>("radio_add", { station }, desktopOnly),
  radioRemove: (id: string) => call<Station[]>("radio_remove", { id }, desktopOnly),
  radioImport: (text: string) => call<ImportStations>("radio_import", { text }, desktopOnly),
  radioProbe: (url: string) => call<StreamInfo>("radio_probe", { url }, desktopOnly),
  radioSearch: (query: string, tags: string[], order: string | null, country: string | null, limit: number, offset: number) =>
    call<Station[]>("radio_search", { query, tags, order, country, limit, offset }, () => []),
  radioPlay: (station: Station) => call<PlayerState>("radio_play", { station }, desktopOnly),
  radioRecord: (start: boolean, name: string) => call<string | null>("radio_record", { start, name }, desktopOnly),
  playlistsGet: () => call<PlaylistData>("playlists_get", {}, () => ({ playlists: [], favorites: [] })),
  playlistCreate: (name: string) => call<PlaylistData>("playlist_create", { name }, desktopOnly),
  playlistRename: (id: string, name: string) => call<PlaylistData>("playlist_rename", { id, name }, desktopOnly),
  playlistDelete: (id: string) => call<PlaylistData>("playlist_delete", { id }, desktopOnly),
  playlistAdd: (id: string, track: Track) => call<PlaylistData>("playlist_add", { id, track }, desktopOnly),
  playlistRemove: (id: string, index: number) => call<PlaylistData>("playlist_remove", { id, index }, desktopOnly),
  playlistShift: (id: string, index: number, delta: number) => call<PlaylistData>("playlist_shift", { id, index, delta }, desktopOnly),
  favoriteToggle: (track: Track) => call<PlaylistData>("favorite_toggle", { track }, desktopOnly),
  setVocalReduction: (amount: number) => call<Settings>("set_vocal_reduction", { amount }, () => (mockSettings = { ...mockSettings, eq: { ...mockSettings.eq, vocal_reduction: amount } })),
  lyricsGet: (path: string) => call<string | null>("lyrics_get", { path }, () => null),
  lyricsSet: (path: string, title: string, text: string) => call<LyricsSummary[]>("lyrics_set", { path, title, text }, desktopOnly),
  lyricsList: () => call<LyricsSummary[]>("lyrics_list", {}, () => []),
  videoSources: () => call<VideoSource[]>("video_sources", {}, () => []),
  videoSearch: (source: string, query: string, limit: number, offset: number) => call<VideoHit[]>("video_search", { source, query, limit, offset }, () => []),
  videoDownload: (hit: VideoHit) => call<string>("video_download", { hit }, desktopOnly),
  ytdlStatus: () => call<string | null>("ytdl_status", {}, () => null),
  ytdlInstall: () => call<string>("ytdl_install", {}, desktopOnly),
  libSources: () => call<SourceStatus[]>("library_sources", {}, () => []),
  libAddSource: (path: string, kinds: Kind[]) => call<SourceStatus[]>("library_add_source", { path, kinds }, desktopOnly),
  libRemoveSource: (id: string) => call<SourceStatus[]>("library_remove_source", { id }, desktopOnly),
  libSetKinds: (id: string, kinds: Kind[]) => call<SourceStatus[]>("library_set_kinds", { id, kinds }, desktopOnly),
  libScan: (id: string | null) => call<ScanSummary>("library_scan", { id }, desktopOnly),
  libSearch: (query: string, kind: Kind | null, genre: string | null, artist: string | null, album: string | null, limit: number, offset: number) =>
    call<Page>("library_search", { query, kind, genre, artist, album, limit, offset }, () => ({ items: [], total: 0 })),
  libSetDetails: (path: string, title: string, date: string | null, description: string | null) => call<void>("library_set_details", { path, title, date, description }, desktopOnly),
  libArtists: (kind: Kind | null, query: string) => call<[string, number][]>("library_artists", { kind, query }, () => []),
  libAlbums: (kind: Kind | null, query: string, artist: string | null) => call<AlbumRow[]>("library_albums", { kind, query, artist }, () => []),
  libGenres: (kind: Kind | null) => call<[string, number][]>("library_genres", { kind }, () => []),
  libCounts: () => call<Counts>("library_counts", {}, () => ({ music: 0, podcast: 0, audiobook: 0, video: 0 })),
  podcastList: () => call<Feed[]>("podcast_list", {}, () => []),
  podcastAdd: (url: string) => call<Feed[]>("podcast_add", { url }, desktopOnly),
  podcastImportOpml: (xml: string) => call<ImportResult>("podcast_import_opml", { xml }, desktopOnly),
  podcastRefresh: () => call<ImportResult>("podcast_refresh", {}, desktopOnly),
  podcastRemove: (id: string) => call<Feed[]>("podcast_remove", { id }, desktopOnly),
  podcastDownload: (feedId: string, guid: string) => call<Feed[]>("podcast_download", { feedId, guid }, desktopOnly),
  podcastMarkPlayed: (feedId: string, guid: string, played: boolean) => call<Feed[]>("podcast_mark_played", { feedId, guid, played }, desktopOnly),
  settings: () => call<Settings>("get_settings", {}, () => mockSettings),
  setLyricsViewer: (on: boolean) => call<Settings>("set_lyrics_viewer", { on }, () => (mockSettings = { ...mockSettings, lyrics_viewer: on })),
  setGridDown: (on: boolean) => call<Settings>("set_grid_down", { on }, () => (mockSettings = { ...mockSettings, grid_down: on })),
  acceptTerms: (version: string) => call<Settings>("accept_terms", { version }, () => (mockSettings = { ...mockSettings, terms_version: version })),
  setEq: (eq: EqSettings) => call<Settings>("set_eq", { eq }, () => (mockSettings = { ...mockSettings, eq })),
  folders: () => call<Folders>("get_folders", {}, () => ({ root: "Downloads/Cindrix", radio: "Downloads/Cindrix/Radio", music: "Downloads/Cindrix/Music", podcasts: "Downloads/Cindrix/Podcasts", audiobooks: "Downloads/Cindrix/Audiobooks", videos: "Downloads/Cindrix/Videos", recordings: "Downloads/Cindrix/Radio" })),
  state: () => call<PlayerState>("player_state", {}, () => mock),
  play: (track: Track) => call<PlayerState>("player_play", { track }, () => (mock = { ...mock, current: track, playing: true, position_secs: 0 })),
  toggle: () => call<PlayerState>("player_toggle", {}, () => (mock = { ...mock, playing: mock.current ? !mock.playing : false })),
  next: () => call<PlayerState>("player_next", {}, () => {
    const [n, ...rest] = mock.queue;
    return (mock = n ? { ...mock, current: n, queue: rest, playing: true } : { ...mock, playing: false });
  }),
  enqueue: (track: Track) => call<PlayerState>("player_enqueue", { track }, () => (mock = { ...mock, queue: [...mock.queue, track] })),
  seek: (secs: number) => call<PlayerState>("player_seek", { secs }, () => (mock = { ...mock, position_secs: secs })),
  volume: (volume: number) => call<PlayerState>("player_volume", { volume }, () => (mock = { ...mock, volume })),
};
