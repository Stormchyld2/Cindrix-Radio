<script lang="ts">
  import { onMount } from "svelte";
  import { api, type LibItem, type VideoHit, type VideoSource } from "./api";
  import { store, settingsActions } from "./settings.svelte";
  import { videoActions } from "./video.svelte";
  import RowMenu from "./RowMenu.svelte";

  let sources = $state<VideoSource[]>([]);
  let source = $state("archive");
  let query = $state("");
  let results = $state<VideoHit[]>([]);
  let mine = $state<LibItem[]>([]);
  let total = $state(0);
  let busy = $state<string | null>(null);
  let message = $state<string | null>(null);
  let ytdl = $state<string | null>(null);
  let saving = $state<Set<string>>(new Set());
  let saved = $state<Set<string>>(new Set());

  const gridDown = $derived(store.s?.grid_down ?? false);
  const cur = $derived(sources.find((s) => s.id === source));
  const needsHelper = $derived((source === "youtube" || source === "web") && cur?.can_download && !ytdl);

  async function refreshMine() {
    const p = await api.libSearch("", "Video", null, null, null, 200, 0);
    mine = p.items; total = p.total;
  }
  onMount(async () => {
    sources = await api.videoSources();
    ytdl = await api.ytdlStatus();
    if (sources.some((s) => s.id === "youtube")) source = "youtube";
    await refreshMine();
    try { await api.libScan("downloads"); await refreshMine(); } catch { /* browser preview */ }
  });

  async function run<T>(label: string, job: () => Promise<T>): Promise<T | undefined> {
    busy = label; message = null;
    try { return await job(); }
    catch (e) { message = String(e instanceof Error ? e.message : e); }
    finally { busy = null; }
  }
  async function search() {
    if (!query.trim()) return;
    const r = await run("Searching…", () => api.videoSearch(source, query, 24, 0));
    if (r) { results = r; if (!r.length) message = "Nothing found. Try other words."; }
  }
  async function installHelper() {
    const v = await run("Downloading the helper…", () => api.ytdlInstall());
    if (v) { ytdl = v; message = `Helper ready (${v}).`; }
  }
  async function download(h: VideoHit) {
    const k = `${h.source}:${h.id}`;
    saving = new Set(saving).add(k); message = null;
    try {
      const path = await api.videoDownload(h);
      saved = new Set(saved).add(k);
      message = `Saved to ${path}`;
      try { await api.libScan("downloads"); await refreshMine(); } catch { /* ignore */ }
    } catch (e) { message = String(e instanceof Error ? e.message : e); }
    finally { const n = new Set(saving); n.delete(k); saving = n; }
  }
  const watch = (h: VideoHit) => videoActions.open({ title: h.title, embed: h.embed_url ?? undefined });
  const playMine = (i: LibItem) => videoActions.open({ title: i.title, file: i.path });
  const canDownload = (h: VideoHit) => sources.find((s) => s.id === h.source)?.can_download ?? false;
  const fmt = (s: number | null) => (s == null ? "" : `${Math.floor(s / 60)}:${String(s % 60).padStart(2, "0")}`);
</script>

{#if gridDown}
  <div class="card gd">
    <b>Grid-Down mode is on.</b> Web search and downloads are paused. Your saved videos below still play.
    <button class="btn primary" onclick={() => settingsActions.setGridDown(false)}>Switch to Online</button>
  </div>
{:else}
  <div class="bar">
    <select class="sel" aria-label="Where to search" bind:value={source}>{#each sources as s}<option value={s.id}>{s.label}</option>{/each}</select>
    <input class="search" placeholder={source === "youtube" ? "Search YouTube or paste a video link…" : source === "web" ? "Paste a link to any page with a video…" : "Search videos…"} bind:value={query} onkeydown={(e) => e.key === "Enter" && search()} />
    <button class="btn primary" disabled={!!busy || !query.trim() || needsHelper} onclick={search}>Search</button>
  </div>
  {#if needsHelper}
    <div class="card gd">This source uses a free helper program (yt-dlp), which is not installed yet.
      <button class="btn primary" disabled={!!busy} onclick={installHelper}>Install helper</button></div>
  {:else if (source === "youtube" || source === "web") && ytdl}
    <p class="note">Downloader helper {ytdl}. <button class="link" disabled={!!busy} onclick={installHelper}>Update</button> if downloads stop working.</p>
  {/if}
  {#if busy}<p class="status">{busy}</p>{/if}
  {#if message}<p class="status" class:err={/not|could|fail|bad|unavailable|nothing/i.test(message)}>{message}</p>{/if}

  {#if results.length}
    <div class="grid">
      {#each results as h (h.source + h.id)}
        {@const k = `${h.source}:${h.id}`}
        <div class="vc">
          <button class="thumb" disabled={!h.embed_url} aria-label="Watch {h.title}" onclick={() => watch(h)}>
            {#if h.thumbnail}<img src={h.thumbnail} alt="" loading="lazy" />{/if}
            {#if h.duration_secs}<span class="dur">{fmt(h.duration_secs)}</span>{/if}
          </button>
          <b title={h.title}>{h.title}</b>
          <span class="by">{h.channel}</span>
          <div class="acts">
            <button class="btn" disabled={!h.embed_url} onclick={() => watch(h)}>Watch</button>
            {#if canDownload(h)}
              <button class="btn primary" disabled={saving.has(k) || saved.has(k)} onclick={() => download(h)}>{saved.has(k) ? "Saved ✓" : saving.has(k) ? "Downloading…" : "Download"}</button>
            {:else}<span class="note">Watch only</span>{/if}
          </div>
        </div>
      {/each}
    </div>
  {/if}
{/if}

<h3 class="sec">My videos <span>{total}</span></h3>
{#if mine.length}
  <div class="rows">
    {#each mine as i (i.id)}
      <div class="row" class:off={!i.available}>
        <button class="play" disabled={!i.available} aria-label="Play {i.title}" onclick={() => playMine(i)}>▶</button>
        <div class="meta"><b>{i.title}</b><span>{i.group}</span></div>
        {#if !i.available}<span class="note">Unavailable</span>{/if}
        <RowMenu track={{ id: i.id, title: i.title, subtitle: i.group, kind: i.kind, duration_secs: i.duration_secs, path: i.path }} canQueue={false} />
      </div>
    {/each}
  </div>
{:else}
  <p class="note">No videos yet. Downloads land in Downloads/Cindrix/Videos. Videos on your drives show up once you add them in Settings → Library &amp; Sources.</p>
{/if}

<style>
  .bar{display:flex;gap:10px;align-items:center;flex-wrap:wrap;margin:12px 0}
  .search{flex:1;min-width:260px;height:44px;border-radius:12px;border:1px solid #2A3747;background:#0D1620;color:var(--silver);padding:0 14px;font-size:15px}
  .sel{height:44px;border-radius:12px;border:1px solid #2A3747;background:#0D1620;color:var(--silver);padding:0 10px;font-size:14px}
  .gd{display:flex;gap:14px;align-items:center;flex-wrap:wrap;font-size:14px;margin:8px 0}
  .status{font-size:13px;color:var(--lblue);margin:4px 0;word-break:break-all}.status.err{color:var(--red)}
  .note{font-size:13px;color:var(--muted);margin:6px 0}
  .link{border:0;background:transparent;color:var(--lblue);text-decoration:underline;padding:0}
  .sec{margin:22px 0 8px;font-size:18px}.sec span{font-size:12px;color:var(--muted);margin-left:6px;font-family:Inter,sans-serif}
  .grid{display:grid;grid-template-columns:repeat(auto-fill,minmax(230px,1fr));gap:14px;margin-top:10px}
  .vc{display:flex;flex-direction:column;gap:6px;padding:10px;border:1px solid var(--line);border-radius:14px;background:var(--panel);font-size:13px}
  .vc b{display:-webkit-box;-webkit-line-clamp:2;-webkit-box-orient:vertical;overflow:hidden}
  .by{color:var(--muted);font-size:12px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
  .thumb{position:relative;aspect-ratio:16/9;border:0;border-radius:10px;overflow:hidden;background:#0D1620;padding:0}
  .thumb img{width:100%;height:100%;object-fit:cover}
  .dur{position:absolute;right:6px;bottom:6px;background:#000000CC;color:#fff;font-size:11px;padding:1px 6px;border-radius:6px}
  .acts{display:flex;gap:8px;align-items:center;margin-top:2px}
  .rows{display:flex;flex-direction:column;gap:6px}
  .row{display:flex;align-items:center;gap:12px;min-height:52px;padding:6px 10px;border-radius:12px;border:1px solid var(--line);background:var(--panel)}
  .row.off{opacity:.55}
  .play{width:36px;height:36px;border-radius:18px;border:0;background:var(--blue);color:#04121C;flex:none}
  .play:disabled{background:#2A3747;color:var(--muted)}
  .meta{flex:1;min-width:0;display:flex;flex-direction:column;font-size:13px}.meta b,.meta span{overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
  .meta span{color:var(--muted);font-size:12px}
  button:disabled{opacity:.5;cursor:not-allowed}
</style>
