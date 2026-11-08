<script lang="ts">
  import { onMount } from "svelte";
  import { open } from "@tauri-apps/plugin-dialog";
  import { api, type Kind, type SourceStatus } from "./api";

  const MAX = 5;
  const kinds: { k: Kind; label: string }[] = [
    { k: "Music", label: "Music" }, { k: "Podcast", label: "Podcasts" },
    { k: "Audiobook", label: "Audiobooks" }, { k: "Video", label: "Videos" },
  ];
  let sources = $state<SourceStatus[]>([]);
  let busy = $state<string | null>(null);
  let message = $state<string | null>(null);
  const inTauri = "__TAURI_INTERNALS__" in window;

  const mine = $derived(sources.filter((x) => !x.source.builtin));

  onMount(async () => { sources = await api.libSources(); });

  async function run<T>(label: string, job: () => Promise<T>): Promise<T | undefined> {
    busy = label; message = null;
    try { return await job(); }
    catch (e) { message = String(e instanceof Error ? e.message : e); }
    finally { busy = null; }
  }
  async function add() {
    const dir = await open({ directory: true, multiple: false });
    if (typeof dir !== "string") return;
    const r = await run("Adding…", () => api.libAddSource(dir, ["Music"]));
    if (r) { sources = r; await scan(r[r.length - 1].source.id); }
  }
  async function scan(id: string | null) {
    const r = await run("Scanning…", () => api.libScan(id));
    if (r) {
      sources = await api.libSources();
      message = r.skipped_offline.length ? `Not connected, kept as is: ${r.skipped_offline.join(", ")}` : `Found ${r.scanned.toLocaleString()} items.`;
    }
  }
  async function toggle(s: SourceStatus, k: Kind) {
    const next = s.source.kinds.includes(k) ? s.source.kinds.filter((x) => x !== k) : [...s.source.kinds, k];
    const r = await run("Saving…", () => api.libSetKinds(s.source.id, next));
    if (r) { sources = r; await scan(s.source.id); }
  }
  async function remove(s: SourceStatus) {
    const r = await run("Removing…", () => api.libRemoveSource(s.source.id));
    if (r) sources = r;
  }
</script>

<div class="card">
  <div class="head">
    <h3>Library &amp; Sources</h3>
    <span class="note">{mine.length} of {MAX} used</span>
  </div>
  <p class="note">Pick up to {MAX} drives or folders. Files stay where they are. Cindrix Radio only reads them and never copies, moves or deletes them.</p>

  {#each sources as s (s.source.id)}
    <div class="src">
      <div class="top">
        <span class="dot" class:off={!s.online}></span>
        <b class="path">{s.source.path}</b>
        <span class="note">{s.online ? `${s.items.toLocaleString()} items` : `Not connected · ${s.items.toLocaleString()} unavailable`}</span>
      </div>
      <div class="kinds">
        {#each kinds as k}
          <label><input type="checkbox" checked={s.source.kinds.includes(k.k)} disabled={!!busy || s.source.builtin} onchange={() => toggle(s, k.k)} /> {k.label}</label>
        {/each}
        <span class="sp"></span>
        <button class="btn" disabled={!!busy || !s.online} onclick={() => scan(s.source.id)}>Rescan</button>
        {#if s.source.builtin}<span class="note">Built in. Your Downloads/Cindrix folder.</span>
        {:else}<button class="btn" disabled={!!busy} onclick={() => remove(s)}>Remove</button>{/if}
      </div>
    </div>
  {/each}

  {#if inTauri}
    <button class="btn primary" disabled={!!busy || mine.length >= MAX} onclick={add}>
      {mine.length >= MAX ? "All 5 slots used" : `Add drive or folder · ${MAX - mine.length} slots left`}
    </button>
  {:else}
    <p class="note">Adding folders works in the desktop app.</p>
  {/if}
  {#if busy}<p class="status">{busy}</p>{/if}
  {#if message}<p class="status">{message}</p>{/if}
</div>

<style>
  .head{display:flex;justify-content:space-between;align-items:center}
  .note{font-size:12px;color:var(--muted);margin:6px 0}
  .src{border:1px solid var(--line);border-radius:12px;padding:12px;margin:10px 0;background:#0D1620}
  .top{display:flex;align-items:center;gap:10px}
  .path{flex:1;min-width:0;font-size:13px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
  .dot{width:8px;height:8px;border-radius:4px;background:var(--green)}.dot.off{background:var(--muted)}
  .kinds{display:flex;gap:14px;align-items:center;flex-wrap:wrap;margin-top:10px;font-size:13px}
  .sp{flex:1}
  .status{font-size:13px;color:var(--lblue)}
  button:disabled{opacity:.5;cursor:not-allowed}
</style>
