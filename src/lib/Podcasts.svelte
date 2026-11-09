<script lang="ts">
  import RowMenu from "./RowMenu.svelte";
  import { onMount } from "svelte";
  import { api, type Feed, type Episode, type PodHit } from "./api";
  import { actions } from "./player.svelte";
  import { store } from "./settings.svelte";

  let feeds = $state<Feed[]>([]);
  let openId = $state<string | null>(null);
  let url = $state("");
  let busy = $state<string | null>(null);
  let message = $state<string | null>(null);

  const gridDown = $derived(store.s?.grid_down ?? false);
  // Grid-Down shows only podcasts with downloaded episodes, and only those episodes
  const shown = $derived(gridDown ? feeds.filter((f) => f.episodes.some((e) => e.file)) : feeds);
  const episodesOf = (f: Feed) => (gridDown ? f.episodes.filter((e) => e.file) : f.episodes);
  const open = $derived(feeds.find((f) => f.id === openId) ?? null);

  onMount(async () => { feeds = await api.podcastList(); });

  async function run<T>(label: string, job: () => Promise<T>): Promise<T | undefined> {
    busy = label; message = null;
    try { return await job(); }
    catch (e) { message = String(e instanceof Error ? e.message : e); }
    finally { busy = null; }
  }

  const add = async () => {
    const r = await run("Adding feed…", () => api.podcastAdd(url.trim()));
    if (r) { feeds = r; url = ""; }
  };
  // ---- find podcasts by topic or idea (e.g. "prepper")
  let sq = $state("");
  let hits = $state<PodHit[]>([]);
  let searched = $state(false);
  const subscribed = (h: PodHit) => feeds.some((f) => f.url === h.feed_url);
  async function findPods() {
    if (!sq.trim()) return;
    const r = await run("Searching podcasts…", () => api.podcastSearch(sq));
    if (r) { hits = r; searched = true; if (!r.length) message = "No podcasts matched. Try a different word."; }
  }
  async function subscribe(h: PodHit, latest = false) {
    const r = await run(latest ? "Subscribing and downloading the newest episode…" : "Subscribing…", async () => {
      let list = await api.podcastAdd(h.feed_url);
      if (latest) {
        const f = list.find((x) => x.url === h.feed_url);
        const e = f?.episodes.find((x) => !x.file);
        if (f && e) list = await api.podcastDownload(f.id, e.guid);
      }
      return list;
    });
    if (r) feeds = r;
  }
  const refresh = async () => {
    const r = await run("Checking feeds…", () => api.podcastRefresh());
    if (r) { feeds = r.feeds; if (r.failed.length) message = `Could not update: ${r.failed.join("; ")}`; }
  };
  const importOpml = async (ev: Event) => {
    const input = ev.currentTarget as HTMLInputElement;
    const file = input.files?.[0];
    input.value = "";
    if (!file) return;
    const r = await run("Importing…", async () => api.podcastImportOpml(await file.text()));
    if (r) { feeds = r.feeds; if (r.failed.length) message = `Could not add: ${r.failed.join("; ")}`; }
  };
  const remove = async (f: Feed) => {
    const r = await run("Removing…", () => api.podcastRemove(f.id));
    if (r) { feeds = r; openId = null; }
  };
  const download = async (f: Feed, e: Episode) => {
    const r = await run(`Downloading “${e.title}”…`, () => api.podcastDownload(f.id, e.guid));
    if (r) feeds = r;
  };
  const togglePlayed = async (f: Feed, e: Episode) => {
    const r = await run("Saving…", () => api.podcastMarkPlayed(f.id, e.guid, !e.played));
    if (r) feeds = r;
  };
  // Downloaded episodes play from the phone; the rest stream from the internet (no seeking while streaming).
  const play = async (f: Feed, e: Episode) => {
    message = null;
    const tr = (x: Episode) => ({ id: x.guid, title: x.title, subtitle: f.title, kind: "Podcast" as const, duration_secs: x.duration_secs,
      path: x.file ?? null, url: x.file ? null : x.enclosure_url });
    const list = episodesOf(f);
    const at = list.findIndex((x) => x.guid === e.guid);
    // the episodes listed after this one play next
    await actions.playMany([tr(e), ...(at < 0 ? [] : list.slice(at + 1, at + 10).map(tr))]);
  };
  async function playNewest(h: PodHit) {
    const f = await run("Opening the podcast…", () => api.podcastPreview(h.feed_url));
    const e = f?.episodes[0];
    if (!f || !e) { if (f) message = "That podcast has no episodes yet."; return; }
    await play({ ...f, title: f.title || h.name }, e);
  }
  const fmt = (s: number | null) => (s == null ? "" : `${Math.floor(s / 3600) ? Math.floor(s / 3600) + ":" : ""}${String(Math.floor((s % 3600) / 60)).padStart(2, "0")}:${String(s % 60).padStart(2, "0")}`);
  const unplayed = (f: Feed) => f.episodes.filter((e) => !e.played).length;
</script>

{#if !open}
  {#if gridDown}
    <p class="note">Grid-Down mode: showing downloaded episodes only. Nothing is using the network.</p>
  {/if}
  <div class="bar" class:hide={gridDown}>
    <input class="field" placeholder="Paste a podcast RSS link (https://…)" bind:value={url} onkeydown={(e) => e.key === "Enter" && url.trim() && add()} />
    <button class="btn primary" disabled={!url.trim() || !!busy} onclick={add}>Add feed</button>
    <label class="btn file">Import OPML<input type="file" accept=".opml,.xml,text/xml" onchange={importOpml} hidden /></label>
    <button class="btn" disabled={!!busy || !feeds.length} onclick={refresh}>Check for new episodes</button>
  </div>
  <form class="bar" class:hide={gridDown} onsubmit={(e) => { e.preventDefault(); findPods(); }}>
    <input class="field" type="search" enterkeyhint="search" placeholder="Find podcasts by topic or idea (try: prepper)" bind:value={sq} />
    <button class="btn primary" type="submit" disabled={!sq.trim() || !!busy}>Search</button>
  </form>
  {#if busy}<p class="status">{busy}</p>{/if}
  {#if message}<p class="status err">{message}</p>{/if}

  {#if hits.length}
    <h3 class="sec">Results for “{sq}” <button class="btn" onclick={() => { hits = []; searched = false; }}>Clear</button></h3>
    <div class="hits">
      {#each hits as h (h.feed_url)}
        <div class="hit card">
          {#if h.image}<img src={h.image} alt="" />{:else}<div class="ph s">{h.name.slice(0, 2).toUpperCase()}</div>{/if}
          <div class="hm"><b>{h.name}</b><span>{h.author}{h.genre ? ` · ${h.genre}` : ""}</span></div>
          {#if subscribed(h)}<div class="ha"><span class="tag">Subscribed ✓</span><button class="btn primary" disabled={!!busy} onclick={() => playNewest(h)}>▶ Play newest</button></div>
          {:else}<div class="ha"><button class="btn primary" disabled={!!busy} onclick={() => playNewest(h)}>▶ Play newest</button>
            <button class="btn" disabled={!!busy} onclick={() => subscribe(h)}>Subscribe</button>
            <button class="btn" disabled={!!busy} onclick={() => subscribe(h, true)}>Subscribe + ⬇</button></div>{/if}
        </div>
      {/each}
    </div>
  {/if}

  {#if feeds.length === 0}
    <div class="card empty">No podcasts yet. Paste an RSS link above, or import an OPML file from another podcast app.</div>
  {:else}
    <div class="grid">
      {#each shown as f (f.id)}
        <button class="feed card" onclick={() => (openId = f.id)}>
          {#if f.image}<img src={f.image} alt="" />{:else}<div class="ph">{f.title.slice(0, 2).toUpperCase()}</div>{/if}
          <b>{f.title}</b>
          <span>{episodesOf(f).length} episodes · {episodesOf(f).filter((e) => !e.played).length} unplayed</span>
        </button>
      {/each}
    </div>
  {/if}
{:else}
  <div class="bar">
    <button class="btn" onclick={() => (openId = null)}>← All podcasts</button>
    <h3>{open.title}</h3>
    <button class="btn" onclick={() => remove(open)}>Unsubscribe</button>
  </div>
  <p class="note">Unsubscribing never deletes episodes you downloaded.</p>
  {#if busy}<p class="status">{busy}</p>{/if}
  {#if message}<p class="status err">{message}</p>{/if}
  <div class="eps">
    {#each episodesOf(open) as e (e.guid)}
      <div class="ep" class:done={e.played}>
        <button class="play" title={e.file ? "Play" : "Stream (uses data). Download to keep it."} aria-label="Play {e.title}" onclick={() => play(open, e)}>▶</button>
        <div class="meta"><b>{e.title}</b><span>{e.published ?? ""} {fmt(e.duration_secs)}</span></div>
        {#if e.file}<span class="tag">Downloaded</span>{/if}
        <RowMenu track={{ id: e.guid, title: e.title, subtitle: open.title, kind: "Podcast", duration_secs: e.duration_secs, path: e.file, url: e.file ? null : e.enclosure_url }} canQueue={false} extras={[...(!e.file && !gridDown ? [{ label: "Download", run: () => download(open, e) }] : []), { label: e.played ? "Mark unplayed" : "Mark played", run: () => togglePlayed(open, e) }]} />
      </div>
    {/each}
  </div>
{/if}

<style>
  .bar{display:flex;gap:10px;align-items:center;flex-wrap:wrap;margin:12px 0}
  .bar h3{flex:1}
  .hide{display:none}
  .sec{display:flex;align-items:center;gap:10px;font-size:16px;margin:14px 0 8px}
  .hits{display:flex;flex-direction:column;gap:8px;margin-bottom:14px}
  .hit{display:flex;align-items:center;gap:10px;padding:10px;flex-wrap:wrap}
  .hit img,.ph.s{width:52px;height:52px;border-radius:10px;flex:none;object-fit:cover;aspect-ratio:auto;font-size:18px}
  .hm{flex:1;min-width:140px;display:flex;flex-direction:column}.hm b{overflow-wrap:anywhere}.hm span{font-size:12px;color:var(--muted)}
  .ha{display:flex;gap:6px;flex-wrap:wrap}
  .field{flex:1;min-width:0;height:36px;border-radius:10px;border:1px solid #2A3747;background:#0D1620;color:var(--silver);padding:0 12px}
  .file{display:inline-flex;align-items:center;cursor:pointer}
  .status{font-size:13px;color:var(--lblue);margin:4px 0}.status.err{color:var(--red)}
  .note{font-size:12px;color:var(--muted);margin:0 0 8px}
  .empty{color:var(--muted);font-size:14px}
  .grid{display:grid;grid-template-columns:repeat(auto-fill,minmax(190px,1fr));gap:14px}
  .feed{display:flex;flex-direction:column;gap:6px;text-align:left;color:inherit;cursor:pointer}
  .feed img,.ph{width:100%;aspect-ratio:1;border-radius:12px;object-fit:cover;background:linear-gradient(135deg,#1B2C44,#5F7FA6);display:flex;align-items:center;justify-content:center;font:700 28px Rajdhani,sans-serif;color:#F1F4F8}
  .feed span{font-size:12px;color:var(--muted)}
  .eps{display:flex;flex-direction:column;gap:6px}
  .ep{display:flex;align-items:center;gap:12px;min-height:52px;padding:6px 10px;border-radius:12px;border:1px solid var(--line);background:var(--panel)}
  .ep.done .meta b{color:var(--muted)}
  .play{width:36px;height:36px;border-radius:18px;border:0;background:var(--blue);color:#04121C;flex:none}
  .play:disabled{background:#2A3747;color:var(--muted);cursor:not-allowed}
  .meta{flex:1;min-width:0;display:flex;flex-direction:column;font-size:13px}.meta span{color:var(--muted);font-size:12px}
  .tag{font-size:12px;color:var(--green)}
  button:disabled{opacity:.5;cursor:not-allowed}
</style>
