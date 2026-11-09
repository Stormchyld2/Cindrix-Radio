<script lang="ts">
  import RowMenu from "./RowMenu.svelte";
  import { onMount } from "svelte";
  import { api, type Feed, type Episode } from "./api";
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
  const play = async (f: Feed, e: Episode) => {
    if (!e.file) return;
    await actions.play({ id: e.guid, title: e.title, subtitle: f.title, kind: "Podcast", duration_secs: e.duration_secs, path: e.file });
  };
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
  {#if busy}<p class="status">{busy}</p>{/if}
  {#if message}<p class="status err">{message}</p>{/if}

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
        <button class="play" disabled={!e.file} title={e.file ? "Play" : "Download to play"} aria-label="Play {e.title}" onclick={() => play(open, e)}>▶</button>
        <div class="meta"><b>{e.title}</b><span>{e.published ?? ""} {fmt(e.duration_secs)}</span></div>
        {#if e.file}<span class="tag">Downloaded</span>{:else}<button class="btn" hidden={gridDown} disabled={!!busy} onclick={() => download(open, e)}>Download</button>{/if}
        <RowMenu track={{ id: e.guid, title: e.title, subtitle: open.title, kind: "Podcast", duration_secs: e.duration_secs, path: e.file, url: e.file ? null : e.enclosure_url }} canQueue={false} />
        <button class="btn" onclick={() => togglePlayed(open, e)}>{e.played ? "Mark unplayed" : "Mark played"}</button>
      </div>
    {/each}
  </div>
{/if}

<style>
  .bar{display:flex;gap:10px;align-items:center;flex-wrap:wrap;margin:12px 0}
  .bar h3{flex:1}
  .hide{display:none}
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
