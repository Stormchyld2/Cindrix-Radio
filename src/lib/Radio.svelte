<script lang="ts">
  import { onMount } from "svelte";
  import { api, type Station, type StreamInfo } from "./api";
  import { player, actions } from "./player.svelte";
  import { store, settingsActions } from "./settings.svelte";

  let { go }: { go: (tab: string) => void } = $props();

  // Each chip is a type of station and maps to the directory tags that describe it.
  const cats: { label: string; tags: string[]; group: "type" | "music" }[] = [
    { label: "Talk", tags: ["talk"], group: "type" },
    { label: "News", tags: ["news"], group: "type" },
    { label: "Weather", tags: ["weather"], group: "type" },
    { label: "Prepper & survival", tags: ["prepper", "survival", "preparedness"], group: "type" },
    { label: "Public radio", tags: ["public radio", "npr"], group: "type" },
    { label: "Christian", tags: ["christian", "gospel"], group: "type" },
    { label: "Country", tags: ["country"], group: "music" },
    { label: "Rock", tags: ["rock"], group: "music" },
    { label: "Classic rock", tags: ["classic rock"], group: "music" },
    { label: "Oldies", tags: ["oldies"], group: "music" },
    { label: "Pop", tags: ["pop"], group: "music" },
    { label: "Hip hop", tags: ["hip hop"], group: "music" },
    { label: "Jazz", tags: ["jazz"], group: "music" },
    { label: "Blues", tags: ["blues"], group: "music" },
    { label: "Bluegrass", tags: ["bluegrass"], group: "music" },
    { label: "Folk", tags: ["folk"], group: "music" },
    { label: "Classical", tags: ["classical"], group: "music" },
  ];
  const sorts = [["votes", "Most voted"], ["clickcount", "Most played"], ["clicktrend", "Trending"], ["bitrate", "Best quality"], ["name", "A to Z"]];
  const countries = [["", "Any country"], ["US", "United States"], ["CA", "Canada"], ["GB", "United Kingdom"], ["AU", "Australia"]];
  let tag = $state<string | null>(null);
  let order = $state("votes");
  let country = $state("");
  const cat = $derived(cats.find((c) => c.label === tag));
  let saved = $state<Station[]>([]);
  let found = $state<Station[]>([]);
  let query = $state("");
  let busy = $state<string | null>(null);
  let message = $state<string | null>(null);
  let adding = $state(false);
  let timer: ReturnType<typeof setTimeout>;

  const gridDown = $derived(store.s?.grid_down ?? false);
  const savedIds = $derived(new Set(saved.map((s) => s.id)));
  const q = $derived(query.trim().toLowerCase());
  // saved stations that match the search, then directory results that are not already saved
  const mine = $derived(saved.filter((s) => (!q || `${s.name} ${s.tags ?? ""}`.toLowerCase().includes(q)) && (!cat || cat.tags.some((t) => (s.tags ?? "").toLowerCase().includes(t))) && (!country || s.country === country)));
  const more = $derived(found.filter((s) => !savedIds.has(s.id)));

  async function load() {
    saved = await api.radioStations();
    await search();
  }
  async function search() {
    if (gridDown) { found = []; return; }
    try { found = await api.radioSearch(query, cat?.tags ?? [], order, country || null, 40, 0); message = null; }
    catch (e) { found = []; message = String(e instanceof Error ? e.message : e); }
  }
  onMount(load);
  $effect(() => { gridDown; search(); });
  const typing = () => { clearTimeout(timer); timer = setTimeout(search, 300); };
  const pickTag = (t: string) => { tag = tag === t ? null : t; search(); };
  const changed = () => search();

  async function run<T>(label: string, job: () => Promise<T>): Promise<T | undefined> {
    busy = label; message = null;
    try { return await job(); }
    catch (e) { message = String(e instanceof Error ? e.message : e); }
    finally { busy = null; }
  }
  const play = (s: Station) => run("Connecting…", () => actions.playStation(s));
  const save = async (s: Station) => { const r = await run("Saving…", () => api.radioAdd(s)); if (r) saved = r; };
  const unsave = async (s: Station) => { const r = await run("Removing…", () => api.radioRemove(s.id)); if (r) saved = r; };
  const isPlaying = (s: Station) => player.s.current?.id === s.id && player.s.current?.kind === "Radio";

  async function record(s: Station) {
    const r = await run("", () => api.radioRecord(!player.s.recording, s.name));
    if (r) message = `Recording to ${r}`;
    await actions.init();
  }

  async function importList(ev: Event) {
    const input = ev.currentTarget as HTMLInputElement;
    const file = input.files?.[0];
    input.value = "";
    if (!file) return;
    const r = await run("Importing…", async () => api.radioImport(await file.text()));
    if (r) { saved = r.stations; message = `Added ${r.added} station${r.added === 1 ? "" : "s"}.`; }
  }

  // ---- add by URL
  let url = $state("");
  let name = $state("");
  let probe = $state<StreamInfo | null>(null);
  async function test() {
    const r = await run("Testing the stream…", () => api.radioProbe(url.trim()));
    if (r) { probe = r; if (!name) name = r.name ?? ""; }
  }
  async function addUrl() {
    const st: Station = { id: "", name: name.trim(), url: (probe?.url ?? url).trim(), favicon: null, tags: probe?.genre ?? null, country: null, codec: null, bitrate: probe?.bitrate ?? null, rb_uuid: null };
    const r = await run("Adding…", () => api.radioAdd(st));
    if (r) { saved = r; adding = false; url = ""; name = ""; probe = null; }
  }
  const initials = (n: string) => n.split(/\s+/).map((w) => w[0]).join("").slice(0, 2).toUpperCase();
  const badge = (s: Station) => [s.codec?.toUpperCase(), s.bitrate ? `${s.bitrate}k` : null].filter(Boolean).join(" · ");
</script>

<div class="bar">
  <input class="search" placeholder="Search by station name or genre (country, rock, talk…)" bind:value={query} oninput={typing} />
  <button class="btn" onclick={() => (adding = true)} disabled={gridDown}>Add by URL</button>
  <label class="btn file">Import list<input type="file" accept=".m3u,.m3u8,.pls,.txt" onchange={importList} hidden /></label>
  <button class="btn" onclick={() => go("Playlists")}>Playlists</button>
</div>

{#if gridDown}
  <div class="card gd">
    <b>Grid-Down mode is on.</b> Live radio needs the internet, so stations are hidden and nothing is using the network.
    <button class="btn primary" onclick={() => settingsActions.setGridDown(false)}>Switch to Online</button>
  </div>
{:else}
  <div class="chips">
    <span class="lab">Type</span>
    {#each cats.filter((c) => c.group === "type") as c}<button class="chip" class:on={tag === c.label} onclick={() => pickTag(c.label)}>{c.label}</button>{/each}
  </div>
  <div class="chips">
    <span class="lab">Music</span>
    {#each cats.filter((c) => c.group === "music") as c}<button class="chip" class:on={tag === c.label} onclick={() => pickTag(c.label)}>{c.label}</button>{/each}
  </div>
  <div class="chips">
    <select class="sel" aria-label="Sort stations" bind:value={order} onchange={changed}>{#each sorts as [v, l]}<option value={v}>{l}</option>{/each}</select>
    <select class="sel" aria-label="Country" bind:value={country} onchange={changed}>{#each countries as [v, l]}<option value={v}>{l}</option>{/each}</select>
  </div>
  {#if busy}<p class="status">{busy}</p>{/if}
  {#if message}<p class="status" class:err={message.startsWith("Directory") || message.startsWith("Could") || message.includes("not")}>{message}</p>{/if}
  {#if player.s.error && player.s.current?.kind === "Radio"}<p class="status err">{player.s.error}</p>{/if}

  {#snippet card(s: Station, isSaved: boolean)}
    <div class="st" class:on={isPlaying(s)}>
      <div class="logo">
        <span>{initials(s.name)}</span>
        {#if s.favicon}<img src={s.favicon} alt="" onerror={(e) => ((e.currentTarget as HTMLImageElement).style.display = "none")} />{/if}
      </div>
      <div class="meta">
        <b>{s.name}</b>
        <span>{isPlaying(s) ? (player.s.connecting ? "Connecting…" : player.s.now_playing ?? s.tags ?? "Live") : s.tags ?? s.country ?? ""}</span>
      </div>
      {#if badge(s)}<span class="badge">{badge(s)}</span>{/if}
      {#if isPlaying(s) && !player.s.connecting}
        <button class="rec" class:live={player.s.recording} title={player.s.recording ? "Stop recording" : "Record (only where you are allowed to)"} aria-label="Record" onclick={() => record(s)}>●</button>
      {/if}
      <button class="heart" class:on={isSaved} title={isSaved ? "Remove from my stations" : "Save to my stations"} aria-label={isSaved ? "Remove from my stations" : "Save to my stations"} onclick={() => (isSaved ? unsave(s) : save(s))}>{isSaved ? "♥" : "♡"}</button>
      <button class="play" aria-label="Play {s.name}" onclick={() => play(s)}>▶</button>
    </div>
  {/snippet}

  <h3 class="sec">My stations <span>{mine.length}</span></h3>
  {#if mine.length}
    <div class="grid">{#each mine as s (s.id)}{@render card(s, true)}{/each}</div>
  {:else}
    <p class="note">{saved.length ? "None of your saved stations match." : "No saved stations yet. Tap the heart on any station below, or add one by URL."}</p>
  {/if}

  <h3 class="sec">{q || tag ? "From Radio Browser" : `Popular on Radio Browser · ${sorts.find((x) => x[0] === order)?.[1]}`} <span>{more.length}</span></h3>
  {#if more.length}
    <div class="grid">{#each more as s (s.id)}{@render card(s, false)}{/each}</div>
  {:else}
    <p class="note">No directory results yet. Check your connection or try another search.</p>
  {/if}
  <p class="note small">Station directory by Radio Browser, a community project. Recording is only for streams you are permitted to record, for personal use.</p>
{/if}

{#if adding}
  <div class="gate" role="dialog" aria-modal="true" aria-label="Add a station">
    <div class="card box">
      <h2>Add a station</h2>
      <label>Stream address<input class="field" placeholder="https://… (stream, .pls or .m3u)" bind:value={url} /></label>
      <button class="btn" disabled={!url.trim() || !!busy} onclick={test}>Test stream</button>
      {#if probe}
        <p class="ok">Found it{probe.name ? `: ${probe.name}` : ""}{probe.bitrate ? ` · ${probe.bitrate}k` : ""}{probe.genre ? ` · ${probe.genre}` : ""}</p>
      {/if}
      <label>Name<input class="field" placeholder="Station name" bind:value={name} /></label>
      {#if busy}<p class="status">{busy}</p>{/if}
      {#if message}<p class="status err">{message}</p>{/if}
      <div class="row">
        <button class="btn" onclick={() => { adding = false; probe = null; }}>Cancel</button>
        <button class="btn primary" disabled={!url.trim() || !!busy} onclick={addUrl}>Add station</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .bar{display:flex;gap:10px;align-items:center;flex-wrap:wrap;margin:12px 0}
  .search{flex:1;min-width:260px;height:44px;border-radius:12px;border:1px solid #2A3747;background:#0D1620;color:var(--silver);padding:0 14px;font-size:15px}
  .file{display:inline-flex;align-items:center;cursor:pointer}
  .chips{display:flex;gap:8px;flex-wrap:wrap;margin:8px 0}
  .chip{height:30px;padding:0 12px;border-radius:15px;border:1px solid #2A3747;background:transparent;color:var(--silver);font-size:12px}
  .lab{font-size:11px;color:var(--muted);text-transform:uppercase;letter-spacing:.08em;align-self:center;width:44px}
  .sel{height:34px;border-radius:10px;border:1px solid #2A3747;background:#0D1620;color:var(--silver);padding:0 10px;font-size:13px}
  .chip.on{border-color:var(--blue);color:var(--lblue);background:#1FA8FF14}
  .gd{display:flex;gap:14px;align-items:center;flex-wrap:wrap;font-size:14px}
  .status{font-size:13px;color:var(--lblue);margin:4px 0}.status.err{color:var(--red)}
  .sec{margin:18px 0 8px;font-size:18px}.sec span{font-size:12px;color:var(--muted);margin-left:6px;font-family:Inter,sans-serif}
  .note{font-size:13px;color:var(--muted);margin:6px 0}.note.small{font-size:11px;margin-top:18px}
  .grid{display:grid;grid-template-columns:repeat(auto-fill,minmax(310px,1fr));gap:10px}
  .st{display:flex;align-items:center;gap:10px;height:68px;padding:0 12px;border-radius:14px;border:1px solid var(--line);background:var(--panel)}
  .st.on{border-color:var(--blue);background:#1FA8FF0F}
  .logo{position:relative;width:44px;height:44px;border-radius:10px;flex:none;overflow:hidden;background:linear-gradient(135deg,#1B2C44,#5F7FA6);display:flex;align-items:center;justify-content:center;font:700 16px Rajdhani,sans-serif;color:#F1F4F8}
  .logo img{position:absolute;inset:0;width:100%;height:100%;object-fit:cover;background:#0D1620}
  .meta{flex:1;min-width:0;display:flex;flex-direction:column;font-size:13px}
  .meta b,.meta span{overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
  .meta span{font-size:12px;color:var(--muted)}
  .st.on .meta span{color:var(--lblue)}
  .badge{font-size:11px;color:var(--muted);border:1px solid #2A3747;border-radius:8px;padding:2px 6px;flex:none}
  .heart{border:0;background:transparent;color:var(--muted);font-size:20px;flex:none}.heart.on{color:var(--blue)}
  .rec{width:28px;height:28px;border-radius:14px;border:1px solid var(--red);background:transparent;color:var(--red);font-size:12px;flex:none}
  .rec.live{background:var(--red);color:#fff}
  .play{width:36px;height:36px;border-radius:18px;border:0;background:var(--blue);color:#04121C;flex:none}
  .gate{position:fixed;inset:0;background:#000000CC;display:flex;align-items:center;justify-content:center;z-index:10}
  .box{width:min(480px,92vw);display:flex;flex-direction:column;gap:12px}
  .box label{display:flex;flex-direction:column;gap:6px;font-size:13px}
  .field{height:38px;border-radius:10px;border:1px solid #2A3747;background:#0D1620;color:var(--silver);padding:0 12px}
  .ok{color:var(--green);font-size:13px;margin:0}
  .row{display:flex;justify-content:flex-end;gap:10px}
  button:disabled{opacity:.5;cursor:not-allowed}
</style>
