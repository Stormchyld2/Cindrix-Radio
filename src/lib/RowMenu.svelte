<script lang="ts">
  import { onMount } from "svelte";
  import type { Track } from "./api";
  import { lists, listActions } from "./playlistStore.svelte";

  let { track, title = track.title, canQueue = true, onQueue, onEdit, extras = [] }:
    { track: Track; title?: string; canQueue?: boolean; onQueue?: () => void; onEdit?: () => void; extras?: { label: string; run: () => void }[] } = $props();

  let open = $state(false);
  let view = $state<"main" | "lists">("main");
  let name = $state("");
  let note = $state<string | null>(null);
  const fav = $derived(lists.d.favorites.some((t) => t.id === track.id));

  onMount(() => { if (!lists.d.playlists.length && !lists.d.favorites.length) listActions.load().catch(() => {}); });
  const show = () => { view = "main"; note = null; open = true; };
  const close = () => { open = false; };
  async function guard(job: () => Promise<unknown>, ok: string, then?: () => void) {
    note = null;
    try { await job(); note = ok; setTimeout(() => { note = null; then ? then() : close(); }, 700); }
    catch (e) { note = String(e instanceof Error ? e.message : e); }
  }
  const addTo = (id: string, n: string) => guard(() => listActions.add(id, track), `Added to ${n}`);
  async function addNew() {
    if (!name.trim()) return;
    const n = name.trim();
    await guard(async () => { const p = await listActions.create(n); await listActions.add(p.id, track); }, `Added to ${n}`);
    name = "";
  }
</script>

<button class="dots" aria-label="More options for {title}" aria-haspopup="menu" onclick={show}>
  <svg width="22" height="22" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true"><circle cx="5" cy="12" r="2" /><circle cx="12" cy="12" r="2" /><circle cx="19" cy="12" r="2" /></svg>
</button>

{#if open}
  <div class="scrim" role="presentation" onclick={close}></div>
  <div class="sheet" role="menu" aria-label="Options for {title}">
    <div class="hd"><b>{title}</b>{#if track.subtitle}<span>{track.subtitle}</span>{/if}</div>
    {#if view === "main"}
      {#if canQueue}<button class="it" role="menuitem" onclick={() => { onQueue?.(); close(); }}>Add to queue</button>{/if}
      <button class="it" role="menuitem" onclick={() => guard(() => listActions.toggleFavorite(track), fav ? "Removed from Favorites" : "Added to Favorites")}>{fav ? "♥  Remove from Favorites" : "♡  Add to Favorites"}</button>
      <button class="it" role="menuitem" onclick={() => (view = "lists")}>Add to a playlist…</button>
      {#each extras as x}<button class="it" role="menuitem" onclick={() => { close(); x.run(); }}>{x.label}</button>{/each}
      {#if onEdit}<button class="it" role="menuitem" onclick={() => { close(); onEdit?.(); }}>Edit title, date and description</button>{/if}
    {:else}
      <div class="list">
        {#each lists.d.playlists as p (p.id)}<button class="it" role="menuitem" onclick={() => addTo(p.id, p.name)}>{p.name}</button>{:else}<span class="note">No playlists yet. Make one below.</span>{/each}
      </div>
      <div class="new"><input placeholder="New playlist…" bind:value={name} onkeydown={(e) => e.key === "Enter" && addNew()} /><button class="btn primary" onclick={addNew}>Create</button></div>
      <button class="it back" onclick={() => (view = "main")}>‹ Back</button>
    {/if}
    {#if note}<p class="note ok">{note}</p>{/if}
    <button class="it cancel" onclick={close}>Close</button>
  </div>
{/if}

<style>
  .dots{width:40px;height:40px;border:0;background:transparent;color:var(--silver);border-radius:10px;flex:none;display:inline-flex;align-items:center;justify-content:center}
  .scrim{position:fixed;inset:0;z-index:50;background:#000A}
  .sheet{position:fixed;z-index:51;left:50%;top:50%;transform:translate(-50%,-50%);width:min(420px,92vw);max-height:84vh;overflow:auto;background:#0D1620;border:1px solid #2A3747;border-radius:16px;padding:10px;display:flex;flex-direction:column;gap:4px}
  .hd{display:flex;flex-direction:column;padding:6px 10px 10px;border-bottom:1px solid var(--line);margin-bottom:4px}
  .hd b{font-size:16px;color:#F1F4F8;overflow-wrap:anywhere}.hd span{font-size:12px;color:var(--muted)}
  .it{min-height:48px;text-align:left;padding:0 14px;border:0;border-radius:10px;background:transparent;color:var(--silver);font-size:15px}
  .it:hover{background:#1FA8FF14}.cancel{color:var(--muted);margin-top:4px;border-top:1px solid var(--line);border-radius:0 0 10px 10px}
  .back{color:var(--lblue)}
  .list{display:flex;flex-direction:column;max-height:40vh;overflow:auto}
  .new{display:flex;gap:8px;padding:8px 6px}.new input{flex:1;min-width:0;height:40px;border-radius:10px;border:1px solid #2A3747;background:#0A0E13;color:var(--silver);padding:0 12px;font-size:14px}
  .note{font-size:13px;color:var(--muted);padding:6px 10px;margin:0}.ok{color:var(--lblue)}
</style>
