<script lang="ts">
  import { onMount } from "svelte";
  import { api, type Diagnostics } from "../api";
  let d = $state<Diagnostics | null>(null);
  let yt = $state<string | null>(null);
  let msg = $state<string | null>(null);
  let busy = $state(false);
  let log = $state<string[]>([]);
  const loadLog = async () => { log = await api.engineTrace(); };
  onMount(async () => { d = await api.diagnostics(); yt = await api.ytdlStatus(); await loadLog(); });
  async function update() {
    busy = true; msg = null;
    try { yt = await api.ytdlInstall(); msg = "Downloader updated."; } catch (e) { msg = String(e instanceof Error ? e.message : e); }
    busy = false;
  }
  async function rescan() {
    busy = true; msg = null;
    try { const r = await api.libScan(null); msg = `Rescanned: ${r.scanned.toLocaleString()} items.`; d = await api.diagnostics(); } catch (e) { msg = String(e instanceof Error ? e.message : e); }
    busy = false;
  }
  const report = $derived(d ? `Cindrix Radio ${d.version}\n${d.os} ${d.arch}\nLibrary items: ${d.library_items}\nData: ${d.data_dir}\nSave location: ${d.save_root}\nDownloader: ${yt ?? "none"}` : "");
  async function copy() { try { await navigator.clipboard.writeText(report + "\n\nPlayer log:\n" + log.join("\n")); msg = "Copied."; } catch { msg = "Could not copy; select the text and copy it."; } }
</script>
<div class="card">
  <h3>Advanced &amp; Diagnostics</h3>
  {#if d}<pre>{report}</pre>{/if}
  <div class="btns">
    <button class="btn" disabled={busy} onclick={rescan}>Rescan library</button>
    <button class="btn" onclick={copy}>Copy diagnostics</button>
    {#if d?.private_build}<button class="btn" disabled={busy} onclick={update}>{yt ? "Update downloader" : "Install downloader"}</button>{/if}
  </div>
  <h3 class="h2">Player log</h3>
  <div class="btns"><button class="btn" onclick={loadLog}>Refresh</button></div>
  <pre>{log.length ? log.join("\n") : "Nothing yet. Play a song, then refresh."}</pre>
  {#if msg}<p class="note">{msg}</p>{/if}
</div>
<style>
  pre{font:12px/1.6 ui-monospace,Consolas,monospace;color:var(--silver);white-space:pre-wrap;word-break:break-all;background:#0D1620;border:1px solid var(--line);border-radius:10px;padding:12px}
  .btns{display:flex;gap:8px;flex-wrap:wrap;margin-top:10px}
  .note{font-size:12px;color:var(--muted)}
  .h2{margin-top:16px}
</style>
