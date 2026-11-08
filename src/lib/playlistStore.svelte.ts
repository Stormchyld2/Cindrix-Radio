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

/** Podcast episodes by id, so a saved episode knows whether it has been downloaded yet. */
export const episodes = $state<{ by: Record<string, { feedId: string; file: string | null }> }>({ by: {} });

export async function loadEpisodes() {
  const m: Record<string, { feedId: string; file: string | null }> = {};
  for (const f of await api.podcastList()) for (const e of f.episodes) m[e.guid] = { feedId: f.id, file: e.file };
  episodes.by = m;
}

/** Download a saved podcast episode so it plays offline (the podcast must still be in your list). */
export async function downloadEpisode(t: Track) {
  const ep = episodes.by[t.id];
  if (!ep) throw new Error("This episode's podcast is no longer in your list.");
  await api.podcastDownload(ep.feedId, t.id);
  await loadEpisodes();
}

/** A saved episode plays from its downloaded file once there is one. */
const resolved = (t: Track): Track => (t.kind === "Podcast" && !t.path && episodes.by[t.id]?.file ? { ...t, path: episodes.by[t.id].file } : t);
const needsLookup = (ts: Track[]) => ts.some((t) => t.kind === "Podcast" && !t.path);

/** Play one item: videos open the video player, everything else goes to the audio player. */
export async function startTrack(t: Track) {
  if (needsLookup([t])) await loadEpisodes().catch(() => {});
  t = resolved(t);
  return t.kind === "Video" ? videoActions.open({ title: t.title, file: t.path ?? undefined }) : actions.play(t);
}

/** Play a list in order (or shuffled): audio is queued, a video-only list opens the first video. */
export async function playList(tracks: Track[], shuffle = false) {
  if (needsLookup(tracks)) await loadEpisodes().catch(() => {});
  const pool = tracks.map(resolved).filter((t) => t.path || t.url);
  if (shuffle) for (let i = pool.length - 1; i > 0; i--) { const j = Math.floor(Math.random() * (i + 1)); [pool[i], pool[j]] = [pool[j], pool[i]]; }
  const audio = pool.filter((t) => t.kind !== "Video");
  if (audio.length) {
    await actions.play(audio[0]);
    for (const t of audio.slice(1, 100)) await actions.enqueue(t);
  } else if (pool.length) await startTrack(pool[0]);
}
