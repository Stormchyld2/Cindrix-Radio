<script lang="ts">
  import { onMount } from "svelte";
  import { open } from "@tauri-apps/plugin-dialog";
  import { api, type StorageRow, type Folders } from "../api";
  import { store, settingsActions } from "../settings.svelte";
  import { isAndroid } from "../platform";
  let rows = $state<StorageRow[]>([]);
  let folders = $state<Folders | null>(null);
  let typed = $state("");
  let msg = $state<string | null>(null);
  const inTauri = "__TAURI_INTERNALS__" in window;
  const size = (b: number) => (b > 1e9 ? `${(b / 1e9).toFixed(1)} GB` : b > 1e6 ? `${(b / 1e6).toFixed(1)} MB` : `${Math.round(b / 1e3)} KB`);
  onMount(async () => { folders = await api.folders(); rows = await api.storageReport(); });
  async function choose() {
    const dir = await open({ directory: true, multiple: false });
    if (typeof dir === "string") await save(dir);
  }
  async function save(p: string | null) {
    msg = null;
    try { await settingsActions.setDownloadRoot(p); msg = "Saved. Restart the app to start using this location."; typed = ""; }
    catch (e) { msg = String(e instanceof Error ? e.message : e); }
  }
</script>
<div class="card">
  <h3>Downloads &amp; Storage</h3>
  <p class="note">Everything you download is saved under <b>{store.s?.download_root ?? folders?.root ?? "Downloads/Cindrix"}</b>, in a folder for each kind.</p>
  {#each rows as r}
    <div class="row"><span>{r.label}</span><span class="p">{r.path}</span><b>{size(r.bytes)}</b></div>
  {/each}
  <h3 class="h2">Save location</h3>
  {#if inTauri && !isAndroid}
    <div class="btns"><button class="btn primary" onclick={choose}>Choose folder…</button>
      {#if store.s?.download_root}<button class="btn" onclick={() => save(null)}>Back to default</button>{/if}</div>
  {:else if inTauri}
    <div class="btns"><input class="in" placeholder="/storage/emulated/0/…" bind:value={typed} /><button class="btn primary" disabled={!typed.trim()} onclick={() => save(typed)}>Use</button>
      {#if store.s?.download_root}<button class="btn" onclick={() => save(null)}>Default</button>{/if}</div>
  {/if}
  {#if msg}<p class="note">{msg}</p>{/if}
</div>
<style>
  .note{font-size:12px;color:var(--muted);line-height:1.6}
  .row{display:flex;gap:12px;align-items:center;font-size:13px;margin:8px 0}.row span:first-child{width:130px;flex:none}.p{flex:1;min-width:0;color:var(--muted);overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
  .h2{margin-top:18px}.btns{display:flex;gap:8px;margin-top:10px;flex-wrap:wrap}
  .in{flex:1;min-width:0;height:36px;border-radius:10px;border:1px solid #2A3747;background:#0D1620;color:var(--silver);padding:0 12px}
</style>
