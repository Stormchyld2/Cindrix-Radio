import { invoke } from "@tauri-apps/api/core";

export type Kind = "Music" | "Radio" | "Podcast" | "Audiobook" | "Video";
export interface Track { id: string; title: string; subtitle: string; kind: Kind; duration_secs: number | null }
export interface PlayerState { current: Track | null; queue: Track[]; playing: boolean; position_secs: number; volume: number }
export interface Folders { root: string; radio: string; music: string; podcasts: string; videos: string; recordings: string }

const inTauri = typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

// Browser fallback so the UI can be developed with plain `npm run dev`.
let mock: PlayerState = { current: null, queue: [], playing: false, position_secs: 0, volume: 0.8 };
const call = async <T>(cmd: string, args: Record<string, unknown> = {}, fb: () => T): Promise<T> =>
  inTauri ? invoke<T>(cmd, args) : fb();

export const api = {
  folders: () => call<Folders>("get_folders", {}, () => ({ root: "Downloads/Cindrix", radio: "Downloads/Cindrix/Radio", music: "Downloads/Cindrix/Music", podcasts: "Downloads/Cindrix/Podcasts", videos: "Downloads/Cindrix/Videos", recordings: "Downloads/Cindrix/Radio" })),
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
