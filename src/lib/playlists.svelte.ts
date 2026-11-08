import { api, type PlaylistData, type Track } from "./api";
import { actions } from "./player.svelte";
import { videoActions } from "./video.svelte";

export const lists = $state<{ d: PlaylistData }>({ d: { playlists: [], favorites: [] } });
const set = (d: PlaylistData) => { lists.d = d; };

export const listActions = {
  load: async () => set(await api.playlistsGet()),
  create: async (name: string) => { set(await api.playlistCreate(name)); return lists.d.playlists[lists.d.playlists.length - 1]; },
  rename: async (id: string, name: string) => set(await api.playlistRename(id, name)),
  remove: async (id: string) => set(await api.playlistDelete(id)),
  add: async (id: string, t: Track) => set(await api.playlistAdd(id, t)),
  removeAt: async (id: string, i: number) => set(await api.playlistRemove(id, i)),
  shift: async (id: string, i: number, delta: number) => set(await api.playlistShift(id, i, delta)),
  toggleFavorite: async (t: Track) => set(await api.favoriteToggle(t)),
  isFavorite: (id: string) => lists.d.favorites.some((t) => t.id === id),
};

/** Play one item: videos open the video player, everything else goes to the audio player. */
export const startTrack = (t: Track) => (t.kind === "Video" ? videoActions.open({ title: t.title, file: t.path ?? undefined }) : actions.play(t));

/** Play a list in order (or shuffled): audio is queued, a video-only list opens the first video. */
export async function playList(tracks: Track[], shuffle = false) {
  const pool = tracks.filter((t) => t.path);
  if (shuffle) for (let i = pool.length - 1; i > 0; i--) { const j = Math.floor(Math.random() * (i + 1)); [pool[i], pool[j]] = [pool[j], pool[i]]; }
  const audio = pool.filter((t) => t.kind !== "Video");
  if (audio.length) {
    await actions.play(audio[0]);
    for (const t of audio.slice(1, 100)) await actions.enqueue(t);
  } else if (pool.length) await startTrack(pool[0]);
}
