<script lang="ts">
  import { onMount } from "svelte";
  import { open } from "@tauri-apps/plugin-dialog";
  import { api, type StorageReport, type Folders } from "../api";
  import { store, settingsActions } from "../settings.svelte";
  import { isAndroid } from "../platform";
  let rep = $state<StorageReport>({ rows: [], free: 0, total: 0, root_exists: false });
  const used = $derived(rep.rows.reduce((a, r) => a + r.bytes, 0));
  const nfiles = $derived(rep.rows.reduce((a, r) => a + r.files, 0));
  async function refresh() { rep = await api.storageReport(); }
  let folders = $state<Folders | null>(null);
  let typed = $state("");
  let msg = $state<string | null>(null);
  const inTauri = "__TAURI_INTERNALS__" in window;
  const size = (b: number) => (b > 1e12 ? `${(b / 1e12).toFixed(2)} TB` : b > 1e9 ? `${(b / 1e9).toFixed(1)} GB` : b > 1e6 ? `${(b / 1e6).toFixed(1)} MB` : b > 0 ? `${Math.max(1, Math.round(b / 1e3))} KB` : "0");
  onMount(async () => { folders = await api.folders(); await refresh(); });
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
  {#if rep.total > 0}
    <div class="meter" aria-label="Phone storage"><i style="width:{Math.min(100, Math.max(0.5, (used / rep.total) * 100))}%"></i></div>
    <p class="note"><b>{size(used)}</b> used by Cindrix in {nfiles.toLocaleString()} {nfiles === 1 ? "file" : "files"} · {size(rep.free)} free of {size(rep.total)} on this device</p>
  {/if}
  {#each rep.rows as r}
    <div class="row"><span>{r.label}</span><span class="p">{r.exists ? `${r.files.toLocaleString()} ${r.files === 1 ? "file" : "files"}` : "Nothing saved here yet"}</span><b>{size(r.bytes)}</b></div>
  {/each}
  {#if isAndroid && !rep.root_exists}
    <p class="note">The Cindrix folder is created the first time something is saved. If you have saved things and still see nothing, allow <b>All files access</b> for Cindrix Radio in Android Settings → Apps → Cindrix Radio → Permissions.</p>
  {/if}
  <div class="btns"><button class="btn" onclick={refresh}>Refresh</button></div>
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
  .meter{height:10px;border-radius:5px;background:#1B2735;overflow:hidden;margin:10px 0 4px}.meter i{display:block;height:100%;background:var(--blue)}
  .h2{margin-top:18px}.btns{display:flex;gap:8px;margin-top:10px;flex-wrap:wrap}
  .in{flex:1;min-width:0;height:36px;border-radius:10px;border:1px solid #2A3747;background:#0D1620;color:var(--silver);padding:0 12px}
</style>
