<script lang="ts">
  import { onMount } from "svelte";
  import { api, type Station } from "./api";
  import { actions } from "./player.svelte";
  import { lists, listActions, playList, startTrack } from "./playlists.svelte";
  import { store } from "./settings.svelte";

  let stations = $state<Station[]>([]);
  let message = $state<string | null>(null);
  const gridDown = $derived(store.s?.grid_down ?? false);

  onMount(async () => {
    listActions.load().catch((e) => (message = String(e)));
    stations = await api.radioStations();
  });
  async function guard(job: () => Promise<unknown>) {
    message = null;
    try { await job(); } catch (e) { message = String(e instanceof Error ? e.message : e); }
  }
  const initials = (n: string) => n.split(/\s+/).map((w) => w[0]).join("").slice(0, 2).toUpperCase();
  const label = (k: string) => (k === "Podcast" ? "Episode" : k === "Audiobook" ? "Book" : k === "Video" ? "Video" : "Song");
</script>

{#if message}<p class="err">{message}</p>{/if}

<h3 class="sec">Stations <span>{stations.length}</span></h3>
{#if stations.length}
  {#if gridDown}<p class="note">Grid-Down mode is on, so stations are paused. Switch to Online to listen.</p>{/if}
  <div class="grid">
    {#each stations as s (s.id)}
      <div class="st">
        <div class="logo">{initials(s.name)}</div>
        <div class="meta"><b>{s.name}</b><span>{s.tags ?? s.country ?? ""}</span></div>
        <button class="play" disabled={gridDown} aria-label="Play {s.name}" onclick={() => guard(() => actions.playStation(s))}>▶</button>
      </div>
    {/each}
  </div>
{:else}
  <p class="note">No saved stations yet. Tap the heart on a station in the Radio tab.</p>
{/if}

<h3 class="sec">Songs, episodes and videos <span>{lists.d.favorites.length}</span></h3>
{#if lists.d.favorites.length}
  <div class="act">
    <button class="btn primary" onclick={() => guard(() => playList(lists.d.favorites))}>Play all</button>
    <button class="btn" onclick={() => guard(() => playList(lists.d.favorites, true))}>Shuffle</button>
  </div>
  <div class="rows">
    {#each lists.d.favorites as t (t.id)}
      <div class="row">
        <button class="play" aria-label="Play {t.title}" onclick={() => guard(async () => { await startTrack(t); })}>▶</button>
        <div class="meta"><b>{t.title}</b><span>{[label(t.kind), t.subtitle].filter(Boolean).join(" · ")}</span></div>
        <button class="ic on" title="Remove from Favorites" aria-label="Remove {t.title} from Favorites" onclick={() => guard(() => listActions.toggleFavorite(t))}>♥</button>
      </div>
    {/each}
  </div>
{:else}
  <p class="note">Nothing yet. Tap the heart on a song, episode or video in the Library or Videos tab.</p>
{/if}

<style>
  .sec{margin:18px 0 8px;font-size:18px}.sec span{font-size:12px;color:var(--muted);margin-left:6px;font-family:Inter,sans-serif}
  .note{font-size:13px;color:var(--muted)}.err{color:var(--red);font-size:13px}
  .act{display:flex;gap:10px;margin:6px 0 10px}
  .grid{display:grid;grid-template-columns:repeat(auto-fill,minmax(310px,1fr));gap:10px}
  .st,.row{display:flex;align-items:center;gap:10px;min-height:52px;padding:6px 12px;border-radius:14px;border:1px solid var(--line);background:var(--panel)}
  .st{height:68px}
  .logo{width:44px;height:44px;border-radius:10px;flex:none;background:linear-gradient(135deg,#1B2C44,#5F7FA6);display:flex;align-items:center;justify-content:center;font:700 16px Rajdhani,sans-serif;color:#F1F4F8}
  .rows{display:flex;flex-direction:column;gap:6px}
  .meta{flex:1;min-width:0;display:flex;flex-direction:column;font-size:13px}.meta b,.meta span{overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
  .meta span{color:var(--muted);font-size:12px}
  .play{width:36px;height:36px;border-radius:18px;border:0;background:var(--blue);color:#04121C;flex:none}
  .play:disabled{background:#2A3747;color:var(--muted)}
  .ic{width:32px;height:32px;border:0;background:transparent;color:var(--muted);font-size:18px}.ic.on{color:var(--blue)}
  button:disabled{opacity:.5;cursor:not-allowed}
</style>
