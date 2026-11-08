import { invoke } from "@tauri-apps/api/core";

export type Kind = "Music" | "Radio" | "Podcast" | "Audiobook" | "Video";
export interface Track { id: string; title: string; subtitle: string; kind: Kind; duration_secs: number | null; path?: string | null }
export interface PlayerState { current: Track | null; queue: Track[]; playing: boolean; position_secs: number; volume: number; error?: string | null }
export interface Folders { root: string; radio: string; music: string; podcasts: string; audiobooks: string; videos: string; recordings: string }

export interface EqSettings { bypass: boolean; preamp_db: number; gains_db: number[] }
export interface Settings { lyrics_viewer: boolean; terms_version: string | null; eq: EqSettings; grid_down: boolean }

export interface Episode { guid: string; title: string; enclosure_url: string; published: string | null; duration_secs: number | null; file: string | null; played: boolean }
export interface Feed { id: string; url: string; title: string; image: string | null; episodes: Episode[] }
export interface ImportResult { feeds: Feed[]; failed: string[] }

export interface LibItem { id: string; source_id: string; path: string; kind: Kind; title: string; artist: string; album: string; genre: string | null; folder_genre: string | null; group: string; duration_secs: number | null; published?: string | null; description?: string | null; available: boolean }
export interface Source { id: string; path: string; kinds: Kind[] }
export interface SourceStatus { source: Source; online: boolean; items: number }
export interface ScanSummary { scanned: number; added: number; skipped_offline: string[] }
export interface Page { items: LibItem[]; total: number }
export interface Counts { music: number; podcast: number; audiobook: number; video: number }

const inTauri = typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

// Browser fallback so the UI can be developed with plain `npm run dev`.
let mock: PlayerState = { current: null, queue: [], playing: false, position_secs: 0, volume: 0.8 };
const call = async <T>(cmd: string, args: Record<string, unknown> = {}, fb: () => T): Promise<T> =>
  inTauri ? invoke<T>(cmd, args) : fb();

let mockSettings: Settings = { lyrics_viewer: false, terms_version: null, grid_down: false, eq: { bypass: false, preamp_db: 0, gains_db: Array(10).fill(0) } };

const desktopOnly = (): never => { throw new Error("Podcasts work in the desktop app, not in the browser preview."); };

export const api = {
  libSources: () => call<SourceStatus[]>("library_sources", {}, () => []),
  libAddSource: (path: string, kinds: Kind[]) => call<SourceStatus[]>("library_add_source", { path, kinds }, desktopOnly),
  libRemoveSource: (id: string) => call<SourceStatus[]>("library_remove_source", { id }, desktopOnly),
  libSetKinds: (id: string, kinds: Kind[]) => call<SourceStatus[]>("library_set_kinds", { id, kinds }, desktopOnly),
  libScan: (id: string | null) => call<ScanSummary>("library_scan", { id }, desktopOnly),
  libSearch: (query: string, kind: Kind | null, genre: string | null, limit: number, offset: number) =>
    call<Page>("library_search", { query, kind, genre, limit, offset }, () => ({ items: [], total: 0 })),
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
  volume: (volume: number) => call<PlayerState>("player_volume", { volume }, () => (mock = { ...mock, volume })),
};
