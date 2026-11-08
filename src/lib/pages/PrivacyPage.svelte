<script lang="ts">
  import { api } from "../api";
  let msg = $state<string | null>(null);
  let confirming = $state<"history" | "lyrics" | "library" | null>(null);
  const items = [
    { k: "history" as const, t: "Erase listening positions", d: "Forgets where you stopped in podcasts and audiobooks and which episodes you played." },
    { k: "lyrics" as const, t: "Erase my lyrics", d: "Deletes every lyric you pasted. They are stored only on this device." },
    { k: "library" as const, t: "Forget the library index", d: "Clears the scanned list of songs. Your files are not touched; rescan to rebuild it." },
  ];
  async function go() {
    if (!confirming) return;
    try { await api.privacyClear(confirming); msg = "Done."; } catch (e) { msg = String(e instanceof Error ? e.message : e); }
    confirming = null;
  }
</script>
<div class="card">
  <h3>Privacy &amp; Security</h3>
  <p class="note">Cindrix Radio has no account, no analytics and no ads. Your library, playlists and lyrics stay on this device. Network use is limited to what you ask for (radio streams, podcast feeds, downloads, the Radio Browser directory) and stops completely in Grid-Down mode. Files are only ever read; the app never moves or deletes your media.</p>
  {#each items as i}
    <div class="it"><div><b>{i.t}</b><span>{i.d}</span></div>
      {#if confirming === i.k}<button class="btn primary" onclick={go}>Confirm</button><button class="btn" onclick={() => (confirming = null)}>Cancel</button>
      {:else}<button class="btn" onclick={() => { confirming = i.k; msg = null; }}>Erase</button>{/if}</div>
  {/each}
  {#if msg}<p class="note">{msg}</p>{/if}
</div>
<style>
  .note{font-size:12px;color:var(--muted);line-height:1.6}
  .it{display:flex;gap:8px;align-items:center;margin:14px 0}.it div{flex:1;display:flex;flex-direction:column;font-size:14px;min-width:0}.it span{font-size:12px;color:var(--muted)}
</style>
