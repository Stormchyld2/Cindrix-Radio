<script lang="ts">
  import { onMount } from "svelte";
  import { api, type BackupInfo } from "../api";
  let list = $state<BackupInfo[]>([]);
  let msg = $state<string | null>(null);
  let busy = $state(false);
  let confirm = $state<string | null>(null);
  const when = (s: number) => (s ? new Date(s * 1000).toLocaleString() : "");
  const refresh = async () => { list = await api.backupList(); };
  onMount(refresh);
  async function run(job: () => Promise<string>) {
    busy = true; msg = null;
    try { msg = await job(); } catch (e) { msg = String(e instanceof Error ? e.message : e); }
    busy = false; await refresh();
  }
  const make = () => run(async () => { const b = await api.backupMake(); return `Saved ${b.name} in Downloads/Cindrix/Backups.`; });
  const restore = (n: string) => run(async () => { confirm = null; const c = await api.backupRestore(n); return `Restored ${c} items. Close and reopen the app to finish.`; });
  const del = (n: string) => run(async () => { await api.backupDelete(n); return "Deleted."; });
</script>
<div class="card">
  <h3>Backup &amp; Restore</h3>
  <p class="note">A backup holds your settings, playlists and favorites, saved stations, podcast subscriptions, your lyrics and listening positions. Music, podcast and video files are not included. Copy the file in Downloads/Cindrix/Backups to keep it somewhere safe.</p>
  <button class="btn primary" disabled={busy} onclick={make}>Back up now</button>
  {#each list as b (b.name)}
    <div class="it"><div><b>{when(b.made) || b.name}</b><span>{b.name} · {Math.round(b.bytes / 1024)} KB</span></div>
      {#if confirm === b.name}<button class="btn primary" disabled={busy} onclick={() => restore(b.name)}>Replace my data</button><button class="btn" onclick={() => (confirm = null)}>Cancel</button>
      {:else}<button class="btn" disabled={busy} onclick={() => (confirm = b.name)}>Restore</button><button class="btn" disabled={busy} onclick={() => del(b.name)}>Delete</button>{/if}</div>
  {:else}<p class="note">No backups yet.</p>{/each}
  {#if msg}<p class="note">{msg}</p>{/if}
</div>
<style>
  .note{font-size:12px;color:var(--muted);line-height:1.6}
  .it{display:flex;gap:8px;align-items:center;margin:12px 0}.it div{flex:1;display:flex;flex-direction:column;font-size:14px;min-width:0}.it span{font-size:12px;color:var(--muted);overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
  button:disabled{opacity:.5}
</style>
