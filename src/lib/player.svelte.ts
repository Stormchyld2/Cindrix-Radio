import { api, type PlayerState, type Station, type Track } from "./api";

export const player = $state<{ s: PlayerState }>({
  s: { current: null, queue: [], playing: false, position_secs: 0, volume: 0.8 },
});

const set = (s: PlayerState) => { player.s = s; };
export const actions = {
  init: async () => set(await api.state()),
  play: async (t: Track) => set(await api.play(t)),
  playStation: async (st: Station) => set(await api.radioPlay(st)),
  toggle: async () => set(await api.toggle()),
  next: async () => set(await api.next()),
  enqueue: async (t: Track) => set(await api.enqueue(t)),
  volume: async (v: number) => set(await api.volume(v)),
};
