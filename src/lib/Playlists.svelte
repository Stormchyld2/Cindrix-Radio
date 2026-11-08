<script lang="ts">
  import { onMount } from "svelte";
  import type { Playlist, Track } from "./api";
  import { lists, listActions, playList, startTrack, episodes, loadEpisodes, downloadEpisode } from "./playlistStore.svelte";
  import { store } from "./settings.svelte";

  let openId = $state<string | null>(null);
  let creating = $state(false);
  let name = $state("");
  let renaming = $state(false);
  let message = $state<string | null>(null);
  const gridDown = $derived(store.s?.grid_down ?? false);
  const cur = $derived(lists.d.playlists.find((p) => p.id === openId) ?? null);

  onMount(() => { listActions.load().catch((e) => (message = String(e)));
    loadEpisodes().catch(() => {}); });
  async function guard(job: () => Promise<unknown>) {
    message = null;
    try { await job(); } catch (e) { message = String(e instanceof Error ? e.message : e); }
  }
  async function create() {
    if (!name.trim()) return;
    await guard(async () => { const p = await listActions.create(name); openId = p.id; creating = false; name = ""; });
  }
  async function rename() {
    if (!cur || !name.trim()) return;
    await guard(async () => { await listActions.rename(cur.id, name); renaming = false; });
  }
  async function del() {
    if (!cur) return;
    await guard(async () => { await listActions.remove(cur.id); openId = null; });
  }
  const initials = (n: string) => n.split(/\s+/).map((w) => w[0]).join("").slice(0, 2).toUpperCase();
  const mins = (items: Track[]) => { const s = items.reduce((a, t) => a + (t.duration_secs ?? 0), 0); return s ? ` · ${Math.round(s / 60)} min` : ""; };
  const count = (p: Playlist) => `${p.items.length} ${p.items.length === 1 ? "item" : "items"}${mins(p.items)}`;
  const label = (k: string) => (k === "Podcast" ? "Episode" : k === "Audiobook" ? "Book" : k === "Video" ? "Video" : "Song");
</script>

{#if message}<p class="err">{message}</p>{/if}

{#if cur}
  <div class="top">
    <button class="btn" onclick={() => (openId = null)}>← All playlists</button>
    {#if renaming}
      <input class="field" bind:value={name} onkeydown={(e) => e.key === "Enter" && rename()} aria-label="Playlist name" />
      <button class="btn primary" onclick={rename}>Save</button><button class="btn" onclick={() => (renaming = false)}>Cancel</button>
    {:else}
      <h2>{cur.name}</h2><span class="note">{count(cur)}</span>
      <span class="sp"></span>
      <button class="btn primary" disabled={!cur.items.length} onclick={() => guard(() => playList(cur.items))}>Play all</button>
      <button class="btn" disabled={!cur.items.length} onclick={() => guard(() => playList(cur.items, true))}>Shuffle</button>
      <button class="btn" onclick={() => { name = cur.name; renaming = true; }}>Rename</button>
      <button class="btn" onclick={del}>Delete</button>
    {/if}
  </div>
  {#if cur.items.length}
    <div class="rows">
      {#each cur.items as t, i (t.id)}
        <div class="row">
          <button class="play" aria-label="Play {t.title}" onclick={() => guard(async () => { await startTrack(t); })}>▶</button>
          <div class="meta"><b>{t.title}</b><span>{[label(t.kind), t.subtitle].filter(Boolean).join(" · ")}</span></div>
          {#if t.kind === "Podcast" && !t.path}
            {#if episodes.by[t.id]?.file}<span class="tag">Downloaded</span>
            {:else if episodes.by[t.id]}<button class="btn" hidden={gridDown} onclick={() => guard(() => downloadEpisode(t))}>Download</button>{/if}
          {/if}
          <button class="ic" aria-label="Move up" disabled={i === 0} onclick={() => guard(() => listActions.shift(cur.id, i, -1))}>↑</button>
          <button class="ic" aria-label="Move down" disabled={i === cur.items.length - 1} onclick={() => guard(() => listActions.shift(cur.id, i, 1))}>↓</button>
          <button class="btn" onclick={() => guard(() => listActions.removeAt(cur.id, i))}>Remove</button>
        </div>
      {/each}
    </div>
  {:else}
    <div class="card note">Empty. In the Library or Videos tab, tap ＋ on any item and pick this playlist.</div>
  {/if}
{:else}
  <div class="tiles">
    <button class="tile new" onclick={() => { creating = true; name = ""; }}><span class="plus">＋</span><b>New playlist</b></button>
    {#each lists.d.playlists as p (p.id)}
      <button class="tile" onclick={() => (openId = p.id)}>
        <span class="art">{initials(p.name)}</span>
        <b>{p.name}</b>
        <span class="note">{count(p)}</span>
      </button>
    {/each}
  </div>
  {#if !lists.d.playlists.length}<p class="note">No playlists yet. Make one, then add songs, episodes and videos with the ＋ button.</p>{/if}
{/if}

{#if creating}
  <div class="gate" role="dialog" aria-modal="true" aria-label="New playlist">
    <div class="card box">
      <h2>New playlist</h2>
      <input class="field" placeholder="Name" bind:value={name} onkeydown={(e) => e.key === "Enter" && create()} />
      <div class="r"><button class="btn" onclick={() => (creating = false)}>Cancel</button><button class="btn primary" disabled={!name.trim()} onclick={create}>Create</button></div>
    </div>
  </div>
{/if}

<style>
  .top{display:flex;gap:10px;align-items:center;flex-wrap:wrap;margin:8px 0 14px}
  .top h2{margin:0 8px 0 4px;font-size:22px}
  .sp{flex:1}
  .note{font-size:13px;color:var(--muted)}
  .err{color:var(--red);font-size:13px}
  .tiles{display:grid;grid-template-columns:repeat(auto-fill,minmax(190px,1fr));gap:16px;margin:12px 0}
  .tile{display:flex;flex-direction:column;gap:6px;align-items:flex-start;text-align:left;padding:14px;border-radius:16px;border:1px solid var(--line);background:var(--panel);color:var(--silver);font-size:14px}
  .tile:hover{border-color:var(--blue)}
  .art{width:100%;aspect-ratio:1;border-radius:12px;display:flex;align-items:center;justify-content:center;font:700 40px Rajdhani,sans-serif;background:linear-gradient(135deg,#1B2C44,#5F7FA6);color:#F1F4F8}
  .tile.new{justify-content:center;align-items:center;min-height:210px;border-style:dashed}
  .plus{font-size:42px;color:var(--blue)}
  .rows{display:flex;flex-direction:column;gap:6px}
  .row{display:flex;align-items:center;gap:10px;min-height:52px;padding:6px 10px;border-radius:12px;border:1px solid var(--line);background:var(--panel)}
  .play{width:36px;height:36px;border-radius:18px;border:0;background:var(--blue);color:#04121C;flex:none}
  .meta{flex:1;min-width:0;display:flex;flex-direction:column;font-size:13px}.meta b,.meta span{overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
  .meta span{color:var(--muted);font-size:12px}
  .ic{width:30px;height:30px;border:1px solid #2A3747;background:transparent;color:var(--silver);border-radius:8px}
  .field{height:38px;border-radius:10px;border:1px solid #2A3747;background:#0D1620;color:var(--silver);padding:0 12px}
  .gate{position:fixed;inset:0;background:#000000CC;display:flex;align-items:center;justify-content:center;z-index:10}
  .box{width:min(420px,92vw);display:flex;flex-direction:column;gap:12px}
  .r{display:flex;justify-content:flex-end;gap:10px}
  .tag{font-size:12px;color:var(--green)}
  button:disabled{opacity:.5;cursor:not-allowed}
</style>
