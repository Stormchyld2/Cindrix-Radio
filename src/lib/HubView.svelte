<script lang="ts">
  import { onMount } from "svelte";
  import { api, type HubTrack, type HubFolder } from "./api";
  import { actions } from "./player.svelte";

  let hub = $state<string | null>(null);
  let addr = $state("192.168.43.1");
  let pin = $state("");
  let busy = $state<string | null>(null);
  let message = $state<string | null>(null);
  let path = $state<string[]>([]);
  let folders = $state<HubFolder[]>([]);
  let tracks = $state<HubTrack[]>([]);
  let total = $state(0);
  let query = $state("");
  let searching = $state(false);
  let token = 0;

  const folder = $derived(path.length ? path.join("/") : null);
  const err = (e: unknown) => String(e instanceof Error ? e.message : e);

  onMount(async () => {
    try { const a = localStorage.getItem("cindrix.hub.addr"); if (a) addr = a; } catch { /* storage unavailable */ }
    hub = await api.hubStatus();
    if (hub) await load();
  });

  async function connect() {
    busy = "Connecting…"; message = null;
    try {
      hub = await api.hubConnect(addr, pin); pin = "";
      try { localStorage.setItem("cindrix.hub.addr", addr); } catch { /* ignore */ }
      await load();
    } catch (e) { message = err(e); }
    busy = null;
  }
  async function disconnect() { await api.hubDisconnect(); hub = null; folders = []; tracks = []; path = []; message = null; }

  async function load() {
    busy = "Loading…"; message = null; searching = false;
    try {
      const [f, t] = await Promise.all([api.hubFolders(folder), folder ? api.hubQuery("", folder, true, 0) : Promise.resolve({ total: 0, tracks: [] })]);
      folders = f; tracks = t.tracks; total = t.total;
    } catch (e) { message = err(e); if (/signed this phone out/.test(message)) hub = null; }
    busy = null;
  }
  async function search() {
    if (!query.trim()) { await load(); return; }
    busy = "Searching…"; message = null; searching = true;
    try { const t = await api.hubQuery(query, null, false, 0); tracks = t.tracks; total = t.total; folders = []; }
    catch (e) { message = err(e); }
    busy = null;
  }
  const open = (f: HubFolder) => { path = [...path, f.name]; query = ""; load(); };
  const up = (n: number) => { path = path.slice(0, n); query = ""; load(); };

  const toTrack = (t: HubTrack, p: string) => ({ id: "hub:" + t.rel, title: t.title || t.rel.split("/").pop() || t.rel, subtitle: t.artist || t.album || "Cindrix Hub", kind: "Music" as const, duration_secs: null, path: p });

  // Play now: copy the song to the phone, play it, then quietly copy the next few so Next works.
  async function play(i: number) {
    const my = ++token;
    busy = `Getting “${tracks[i].title}”…`; message = null;
    try {
      const p = await api.hubFetch(tracks[i].rel, false);
      if (my !== token) return;
      await actions.playMany([toTrack(tracks[i], p)]);
      busy = null;
      for (const t of tracks.slice(i + 1, i + 6)) {
        if (my !== token) return;
        const q = await api.hubFetch(t.rel, false);
        if (my !== token) return;
        await actions.enqueue(toTrack(t, q));
      }
    } catch (e) { message = err(e); }
    if (my === token) busy = null;
  }
  async function keep(t: HubTrack) {
    busy = `Saving “${t.title}” to your music…`; message = null;
    try { await api.hubFetch(t.rel, true); message = `Saved “${t.title}” to your Music library.`; } catch (e) { message = err(e); }
    busy = null;
  }
  async function saveFolder(name: string) {
    const full = [...path, name].join("/");
    busy = `Copying “${name}” from the Hub…`;
    message = null;
    const poll = setInterval(async () => { try { const [d, t] = await api.hubProgress(); if (t) busy = `Copying “${name}”: song ${Math.min(d + 1, t)} of ${t}. You can keep using the app.`; } catch { /* ignore */ } }, 1000);
    try { const n = await api.hubSaveFolder(full); message = `Saved ${n} songs from “${name}” to your Music library.`; } catch (e) { message = err(e); }
    clearInterval(poll); busy = null;
  }
</script>

{#if !hub}
  <div class="card">
    <h3>Connect to Cindrix Hub</h3>
    <p class="note">Use the music on your Cindrix Ops Hub or Vault, whether it is on the 4 TB drive or a thumb drive. Join the Hub's Wi-Fi hotspot first (no internet needed), then enter its address and PIN. Both are shown in Cindrix Ops under Settings → Sharing.</p>
    <form class="f" onsubmit={(e) => { e.preventDefault(); connect(); }}>
      <label>Hub address<input class="field" bind:value={addr} inputmode="url" autocapitalize="off" placeholder="192.168.43.1" /></label>
      <div class="chips"><button type="button" class="chip" onclick={() => (addr = "192.168.43.1")}>Phone Hub</button><button type="button" class="chip" onclick={() => (addr = "192.168.137.1")}>Windows Vault</button></div>
      <label>PIN<input class="field" bind:value={pin} inputmode="numeric" autocomplete="off" placeholder="4 to 8 digits" /></label>
      <button class="btn primary" type="submit" disabled={!addr.trim() || !pin.trim() || !!busy}>Connect</button>
    </form>
    {#if busy}<p class="status">{busy}</p>{/if}
    {#if message}<p class="status err">{message}</p>{/if}
  </div>
{:else}
  <div class="bar">
    <span class="note">Connected to <b>{hub}</b></span>
    <button class="btn" onclick={disconnect}>Disconnect</button>
  </div>
  <form class="bar" onsubmit={(e) => { e.preventDefault(); search(); }}>
    <input class="field" type="search" enterkeyhint="search" placeholder="Search the Hub's music…" bind:value={query} />
    <button class="btn primary" type="submit" disabled={!!busy}>Search</button>
  </form>
  <div class="crumbs">
    <button class="link" onclick={() => up(0)}>Hub music</button>
    {#each path as p, n}<span>›</span><button class="link" onclick={() => up(n + 1)}>{p}</button>{/each}
    {#if searching}<span>›</span><b>Search results</b>{/if}
  </div>
  {#if busy}<p class="status">{busy}</p>{/if}
  {#if message}<p class="status" class:err={/could|lost|not|refus|signed|no songs/i.test(message)}>{message}</p>{/if}

  <div class="rows">
    {#each folders as f (f.folder || f.name)}
      <div class="row">
        <button class="pick" onclick={() => open(f)}><b>{f.name}</b><span>{f.count} {f.count === 1 ? "song" : "songs"}</span></button>
        <button class="btn" disabled={!!busy} onclick={() => saveFolder(f.name)}>⬇ Save all</button>
      </div>
    {/each}
    {#each tracks as t, i (t.rel)}
      <div class="row">
        <button class="play" aria-label="Play {t.title}" disabled={!!busy && !busy.startsWith("Getting")} onclick={() => play(i)}>▶</button>
        <div class="meta"><b>{t.title || t.rel.split("/").pop()}</b><span>{[t.artist, t.album].filter(Boolean).join(" · ")}</span></div>
        <button class="btn" aria-label="Save {t.title} to my music" disabled={!!busy} onclick={() => keep(t)}>⬇</button>
      </div>
    {/each}
    {#if !busy && !folders.length && !tracks.length}<p class="note">{searching ? "Nothing matched." : "No music found here. Is the drive plugged into the Hub?"}</p>{/if}
  </div>
  {#if total > tracks.length}<p class="note">Showing {tracks.length} of {total}. Narrow it with a folder or a search.</p>{/if}
{/if}

<style>
  .f{display:flex;flex-direction:column;gap:10px;margin-top:10px}
  .f label{display:flex;flex-direction:column;gap:6px;font-size:13px}
  .field{flex:1;min-width:0;height:44px;border-radius:12px;border:1px solid #2A3747;background:#0D1620;color:var(--silver);padding:0 14px;font-size:15px}
  .chips{display:flex;gap:8px;flex-wrap:wrap}.chip{height:30px;padding:0 12px;border-radius:15px;border:1px solid #2A3747;background:transparent;color:var(--silver);font-size:12px}
  .bar{display:flex;gap:10px;align-items:center;margin:10px 0}.bar .note{flex:1;min-width:0;overflow-wrap:anywhere}
  .note{font-size:12px;color:var(--muted);line-height:1.6}
  .status{font-size:13px;color:var(--lblue);margin:6px 0}.status.err{color:var(--red)}
  .crumbs{display:flex;gap:6px;flex-wrap:wrap;align-items:center;font-size:13px;margin:6px 0}
  .link{border:0;background:transparent;color:var(--lblue);padding:0;font-size:13px}
  .rows{display:flex;flex-direction:column;gap:6px}
  .row{display:flex;align-items:center;gap:10px;padding:8px 10px;border:1px solid var(--line);border-radius:12px;background:var(--panel);min-width:0}
  .pick{flex:1;min-width:0;display:flex;flex-direction:column;text-align:left;border:0;background:transparent;color:inherit;padding:0}
  .pick b,.meta b{overflow-wrap:anywhere}.pick span,.meta span{font-size:12px;color:var(--muted)}
  .meta{flex:1;min-width:0;display:flex;flex-direction:column}
  .play{width:36px;height:36px;border-radius:18px;border:0;background:var(--blue);color:#04121C;flex:none}
</style>
