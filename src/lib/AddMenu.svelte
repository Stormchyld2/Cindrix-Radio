<script lang="ts">
  import { onMount } from "svelte";
  import type { Track } from "./api";
  import { lists, listActions } from "./playlists.svelte";

  let { track }: { track: Track } = $props();
  let open = $state(false);
  let name = $state("");
  let note = $state<string | null>(null);
  const fav = $derived(lists.d.favorites.some((t) => t.id === track.id));

  onMount(() => { if (!lists.d.playlists.length && !lists.d.favorites.length) listActions.load().catch(() => {}); });
  async function guard(job: () => Promise<unknown>, ok: string) {
    note = null;
    try { await job(); note = ok; setTimeout(() => { note = null; open = false; }, 900); }
    catch (e) { note = String(e instanceof Error ? e.message : e); }
  }
  const addTo = (id: string, n: string) => guard(() => listActions.add(id, track), `Added to ${n}`);
  async function addNew() {
    if (!name.trim()) return;
    await guard(async () => { const p = await listActions.create(name); await listActions.add(p.id, track); }, `Added to ${name.trim()}`);
    name = "";
  }
</script>

<span class="am">
  <button class="ic" class:on={fav} title={fav ? "Remove from Favorites" : "Add to Favorites"} aria-label={fav ? "Remove from Favorites" : "Add to Favorites"} onclick={() => guard(() => listActions.toggleFavorite(track), fav ? "Removed" : "Favorited")}>{fav ? "♥" : "♡"}</button>
  <button class="ic" title="Add to a playlist" aria-label="Add to a playlist" aria-expanded={open} onclick={() => (open = !open)}>＋</button>
  {#if open}
    <div class="pop" role="menu">
      {#each lists.d.playlists as p (p.id)}<button class="it" onclick={() => addTo(p.id, p.name)}>{p.name}</button>{/each}
      <div class="new"><input placeholder="New playlist…" bind:value={name} onkeydown={(e) => e.key === "Enter" && addNew()} /><button class="it" onclick={addNew}>Create</button></div>
      {#if note}<span class="note">{note}</span>{/if}
    </div>
  {/if}
</span>

<style>
  .am{position:relative;display:inline-flex;gap:2px}
  .ic{width:32px;height:32px;border:0;background:transparent;color:var(--muted);font-size:18px;border-radius:8px}
  .ic.on{color:var(--blue)}
  .pop{position:absolute;right:0;top:36px;z-index:5;min-width:200px;max-height:260px;overflow:auto;background:#0D1620;border:1px solid #2A3747;border-radius:12px;padding:6px;display:flex;flex-direction:column;gap:2px}
  .it{text-align:left;border:0;background:transparent;color:var(--silver);padding:8px 10px;border-radius:8px;font-size:13px}
  .it:hover{background:#1FA8FF18}
  .new{display:flex;gap:4px;margin-top:4px;border-top:1px solid var(--line);padding-top:6px}
  .new input{flex:1;min-width:0;height:32px;border-radius:8px;border:1px solid #2A3747;background:#0A0E13;color:var(--silver);padding:0 8px;font-size:13px}
  .note{font-size:12px;color:var(--lblue);padding:4px 10px}
</style>
