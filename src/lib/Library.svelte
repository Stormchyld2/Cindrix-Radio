<script lang="ts">
  import { onMount } from "svelte";
  import { api, type AlbumRow, type BookHit, type Counts, type Kind, type LibItem } from "./api";
  import { store } from "./settings.svelte";
  import { actions } from "./player.svelte";
  import { videoActions } from "./video.svelte";
  import RowMenu from "./RowMenu.svelte";

  const types: { kind: Kind; label: string; key: keyof Counts }[] = [
    { kind: "Music", label: "Music", key: "music" },
    { kind: "Podcast", label: "Podcasts", key: "podcast" },
    { kind: "Audiobook", label: "Audiobooks", key: "audiobook" },
    { kind: "Video", label: "Videos", key: "video" },
  ];
  let { initial = "Music" }: { initial?: Kind } = $props();
  let kind = $state<Kind>(initial);
  let query = $state("");
  let genre = $state<string | null>(null);
  let artist = $state<string | null>(null);
  let album = $state<string | null>(null);
  let view = $state<"songs" | "artists" | "albums" | "genres">("songs");
  let artists = $state<[string, number][]>([]);
  let albums = $state<AlbumRow[]>([]);
  let counts = $state<Counts>({ music: 0, podcast: 0, audiobook: 0, video: 0 });
  let genres = $state<[string, number][]>([]);
  let items = $state<LibItem[]>([]);
  let total = $state(0);
  let timer: ReturnType<typeof setTimeout>;

  async function refresh() {
    const music = kind === "Music";
    const [p, g, c, ar, al] = await Promise.all([
      api.libSearch(query, kind, genre, artist, album, 200, 0), api.libGenres(kind), api.libCounts(),
      music && view === "artists" ? api.libArtists(kind, query) : Promise.resolve([] as [string, number][]),
      music && view === "albums" ? api.libAlbums(kind, query, artist) : Promise.resolve([] as AlbumRow[]),
    ]);
    items = p.items; total = p.total; genres = g; counts = c; artists = ar; albums = al;
  }
  // the built-in Downloads/Cindrix folder is refreshed whenever the Library opens
  onMount(async () => { await refresh(); try { await api.libScan("downloads"); await refresh(); } catch { /* browser preview */ } });
  // ---- edit details (saved in the folder's metadata.txt)
  const inTauri = "__TAURI_INTERNALS__" in window;
  let edit = $state<LibItem | null>(null);
  let eTitle = $state(""); let eDate = $state(""); let eDesc = $state("");
  let eMsg = $state<string | null>(null);
  function startEdit(i: LibItem) { edit = i; eTitle = i.title; eDate = i.published ?? ""; eDesc = i.description ?? ""; eMsg = null; }
  async function saveEdit(clear = false) {
    if (!edit) return;
    try {
      await api.libSetDetails(edit.path, clear ? "" : eTitle, clear ? null : eDate || null, clear ? null : eDesc || null);
      edit = null; await refresh();
    } catch (e) { eMsg = String(e instanceof Error ? e.message : e); }
  }
  const pick = (k: Kind) => { kind = k; genre = null; artist = null; album = null; view = "songs"; refresh(); };
  const setView = (v: typeof view) => { view = v; genre = null; artist = null; album = null; refresh(); };
  const openArtist = (a: string) => { artist = a; album = null; view = "songs"; refresh(); };
  const openAlbum = (a: AlbumRow) => { artist = a.artist || null; album = a.album; view = "songs"; refresh(); };
  const openGenre = (g: string | null) => { genre = g; view = "songs"; refresh(); };
  // ---- find free audiobooks online (public domain, LibriVox)
  let bq = $state("");
  let books = $state<BookHit[]>([]);
  let bBusy = $state<string | null>(null);
  let bMsg = $state<string | null>(null);
  let saved = $state<Set<string>>(new Set());
  let getting = $state<string | null>(null);
  async function findBooks() {
    if (!bq.trim()) return;
    bBusy = "Searching…"; bMsg = null; books = [];
    try { books = await api.audiobookSearch(bq); if (!books.length) bMsg = "No free audiobooks matched. Try a shorter title or the author's name."; }
    catch (e) { bMsg = String(e instanceof Error ? e.message : e); }
    bBusy = null;
  }
  async function getBook(b: BookHit) {
    getting = b.id; bMsg = null;
    bBusy = `Downloading "${b.title}"… starting`;
    const poll = setInterval(async () => {
      try { const [d, t] = await api.audiobookProgress(); if (t) bBusy = `Downloading "${b.title}": part ${Math.min(d + 1, t)} of ${t}. You can keep using the app.`; } catch { /* ignore */ }
    }, 1000);
    try { const n = await api.audiobookDownload(b.id, b.title); saved = new Set([...saved, b.id]); bMsg = `Saved "${b.title}" as one file (${n} chapters joined). Find it under Audiobooks.`; await refresh(); }
    catch (e) { bMsg = String(e instanceof Error ? e.message : e); }
    clearInterval(poll);
    getting = null; bBusy = null;
  }
  const typing = () => { clearTimeout(timer); timer = setTimeout(refresh, 200); };

  const toTrack = (i: LibItem) => ({ id: i.id, title: i.title, subtitle: i.artist || i.group, kind: i.kind, duration_secs: i.duration_secs, path: i.path });
  const playable = (i: LibItem) => i.kind !== "Video" && i.available;
  const start = (i: LibItem) => (i.kind === "Video" ? videoActions.open({ title: i.title, file: i.path }) : actions.play(toTrack(i)));
  async function shuffle() {
    const p = await api.libSearch(query, kind, genre, artist, album, 2000, 0);
    const pool = p.items.filter(playable);
    for (let i = pool.length - 1; i > 0; i--) { const j = Math.floor(Math.random() * (i + 1)); [pool[i], pool[j]] = [pool[j], pool[i]]; }
    if (!pool.length) return;
    await actions.play(toTrack(pool[0]));
    for (const t of pool.slice(1, 40)) await actions.enqueue(toTrack(t));
  }
  const fmt = (s: number | null) => (s == null ? "" : `${Math.floor(s / 60)}:${String(s % 60).padStart(2, "0")}`);
</script>

<div class="types">
  {#each types as t}
    <button class="btn type" class:on={kind === t.kind} onclick={() => pick(t.kind)}>{t.label} <b>{counts[t.key].toLocaleString()}</b></button>
  {/each}
</div>

<form class="sbar" onsubmit={(e) => { e.preventDefault(); clearTimeout(timer); refresh(); }}>
  <input class="search" type="search" enterkeyhint="search" placeholder="Search {kind === 'Music' ? 'songs, artists, albums, genres' : 'titles'}…" bind:value={query} oninput={typing} />
  <button class="btn primary" type="submit">Search</button>
</form>

{#if kind === "Audiobook" && !store.s?.grid_down}
  <div class="find">
    <b>Find free audiobooks online</b>
    <span class="note">Public-domain books from LibriVox. They download to your phone and play offline.</span>
    <form class="sbar" onsubmit={(e) => { e.preventDefault(); findBooks(); }}>
      <input class="search" type="search" enterkeyhint="search" placeholder="Book title or author…" bind:value={bq} />
      <button class="btn primary" type="submit" disabled={!bq.trim() || !!bBusy}>Find</button>
    </form>
    {#if bBusy}<p class="note">{bBusy}</p>{/if}
    {#if bMsg}<p class="note">{bMsg}</p>{/if}
    {#each books as b (b.id)}
      <div class="book">
        <div class="bm"><b>{b.title}</b><span>{b.author}{b.runtime ? ` · ${b.runtime}` : ""}</span></div>
        <button class="btn" disabled={!!bBusy || saved.has(b.id)} onclick={() => getBook(b)}>{saved.has(b.id) ? "Saved ✓" : getting === b.id ? "Downloading…" : "Download"}</button>
      </div>
    {/each}
  </div>
{/if}

{#if kind === "Music"}
  <div class="genres">
    {#each [["songs", "Songs"], ["artists", "Artists"], ["albums", "Albums"], ["genres", "Genres"]] as [v, label]}
      <button class="chip" class:on={view === v} onclick={() => setView(v as typeof view)}>{label}</button>
    {/each}
  </div>
{/if}

{#if artist || album || genre}
  <div class="genres">
    {#if artist}<button class="chip on" onclick={() => { artist = null; album = null; refresh(); }}>Artist: {artist} ✕</button>{/if}
    {#if album}<button class="chip on" onclick={() => { album = null; refresh(); }}>Album: {album} ✕</button>{/if}
    {#if genre}<button class="chip on" onclick={() => { genre = null; refresh(); }}>Genre: {genre} ✕</button>{/if}
  </div>
{:else if genres.length && !(kind === "Music" && view !== "songs")}
  <div class="genres">
    <button class="chip" class:on={genre === null} onclick={() => openGenre(null)}>All</button>
    {#each genres as [g, n]}
      <button class="chip" class:on={genre === g} onclick={() => openGenre(g)}>{g} <span>{n}</span></button>
    {/each}
  </div>
{/if}

<div class="actions">
  <span class="note">{total.toLocaleString()} {total === 1 ? "item" : "items"}{total > items.length ? ` (showing ${items.length})` : ""}</span>
  <button class="btn primary" disabled={kind === "Video" || !total} onclick={shuffle}>Shuffle</button>
</div>

{#if total === 0 && !query && !genre && !artist && !album}
  <div class="card empty">Nothing here yet. Music you download lands in Downloads/Cindrix automatically. You can also add drives or folders in Settings → Library &amp; Sources.</div>
{:else if kind === "Music" && view === "artists"}
  <div class="rows">
    {#each artists as [a, n] (a)}
      <button class="row pick" onclick={() => openArtist(a)}><div class="meta"><b>{a}</b><span>{n} {n === 1 ? "song" : "songs"}</span></div></button>
    {:else}<p class="note">No artists found. Artists come from the tags in your files.</p>{/each}
  </div>
{:else if kind === "Music" && view === "albums"}
  <div class="rows">
    {#each albums as a (a.album + a.artist)}
      <button class="row pick" onclick={() => openAlbum(a)}><div class="meta"><b>{a.album}</b><span>{[a.artist, `${a.count} ${a.count === 1 ? "song" : "songs"}`].filter(Boolean).join(" · ")}</span></div></button>
    {:else}<p class="note">No albums found. Albums come from the tags in your files.</p>{/each}
  </div>
{:else if kind === "Music" && view === "genres"}
  <div class="rows">
    {#each genres as [g, n] (g)}
      <button class="row pick" onclick={() => openGenre(g)}><div class="meta"><b>{g}</b><span>{n} {n === 1 ? "song" : "songs"}</span></div></button>
    {:else}<p class="note">No genres found. Genres come from tags or from your folder names.</p>{/each}
  </div>
{:else}
  <div class="rows">
    {#each items as i (i.id)}
      <div class="row" class:off={!i.available}>
        <button class="play" disabled={!i.available} title={i.available ? "Play" : "Drive not connected"} aria-label="Play {i.title}" onclick={() => start(i)}>▶</button>
        <div class="meta"><b>{i.title}</b><span>{[i.artist || i.group, i.album].filter(Boolean).join(" · ")}</span></div>
        {#if !i.available}<span class="tag">Unavailable</span>{/if}
        <span class="time">{fmt(i.duration_secs)}</span>
        <RowMenu track={toTrack(i)} canQueue={playable(i)} onQueue={() => actions.enqueue(toTrack(i))} onEdit={inTauri && i.available ? () => startEdit(i) : undefined} />
      </div>
    {/each}
  </div>
{/if}

{#if edit}
  <div class="gate" role="dialog" aria-modal="true" aria-label="Edit details">
    <div class="card box">
      <h2>Edit details</h2>
      <p class="note">{edit.kind === "Audiobook" ? "Name a chapter or book part." : "Change how this appears in your Library."} Saved in metadata.txt in the same folder. Your file is never changed.</p>
      <label>Title<input class="field" bind:value={eTitle} /></label>
      <label>Date (YYYY-MM-DD, optional)<input class="field" placeholder="2026-11-08" bind:value={eDate} /></label>
      <label>Description (optional)<textarea class="field area" bind:value={eDesc}></textarea></label>
      {#if eMsg}<p class="err">{eMsg}</p>{/if}
      <div class="r">
        <button class="btn" onclick={() => saveEdit(true)}>Clear details</button>
        <span class="sp"></span>
        <button class="btn" onclick={() => (edit = null)}>Cancel</button>
        <button class="btn primary" onclick={() => saveEdit()}>Save</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .types{display:flex;gap:8px;flex-wrap:wrap;margin:12px 0}
  .type.on,.chip.on{border-color:var(--blue);color:var(--lblue);background:#1FA8FF14}
  .type b{margin-left:6px;color:var(--muted);font-weight:500}
  .sbar{display:flex;gap:8px;align-items:center;margin:8px 0}
  .find{border:1px solid var(--line);border-radius:14px;background:var(--panel);padding:12px 14px;margin:10px 0;display:flex;flex-direction:column;gap:6px}
  .find .note{font-size:12px;color:var(--muted);margin:0}
  .book{display:flex;gap:10px;align-items:center;padding:8px 0;border-top:1px solid var(--line)}
  .bm{flex:1;min-width:0;display:flex;flex-direction:column}.bm b{overflow-wrap:anywhere}.bm span{font-size:12px;color:var(--muted)}
  .search{flex:1;min-width:0;width:100%;height:44px;border-radius:12px;border:1px solid #2A3747;background:#0D1620;color:var(--silver);padding:0 14px;font-size:15px}
  .genres{display:flex;gap:8px;flex-wrap:wrap;margin:12px 0}
  .chip{height:30px;padding:0 12px;border-radius:15px;border:1px solid #2A3747;background:transparent;color:var(--silver);font-size:12px}
  .chip span{color:var(--muted);margin-left:4px}
  .actions{display:flex;justify-content:space-between;align-items:center;margin:12px 0}
  .note{font-size:12px;color:var(--muted)}
  .empty{color:var(--muted);font-size:14px}
  .rows{display:flex;flex-direction:column;gap:6px}
  .row{display:flex;align-items:center;gap:12px;min-height:52px;padding:6px 10px;border-radius:12px;border:1px solid var(--line);background:var(--panel)}
  .row.off{opacity:.55}
  .play{width:36px;height:36px;border-radius:18px;border:0;background:var(--blue);color:#04121C;flex:none}
  .play:disabled{background:#2A3747;color:var(--muted);cursor:not-allowed}
  .meta{flex:1;min-width:0;display:flex;flex-direction:column;font-size:13px}.meta b{display:-webkit-box;-webkit-line-clamp:2;line-clamp:2;-webkit-box-orient:vertical;overflow:hidden;overflow-wrap:anywhere;color:#F1F4F8;font-size:14px}.meta span{overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
  .meta span,.time{color:var(--muted);font-size:12px}
  .tag{font-size:12px;color:var(--muted)}
  .ic{width:32px;height:32px;border:0;background:transparent;color:var(--muted);font-size:16px;border-radius:8px}
  .gate{position:fixed;inset:0;background:#000000CC;display:flex;align-items:center;justify-content:center;z-index:20}
  .box{width:min(520px,94vw);display:flex;flex-direction:column;gap:10px}
  .box label{display:flex;flex-direction:column;gap:6px;font-size:13px}
  .field{height:38px;border-radius:10px;border:1px solid #2A3747;background:#0D1620;color:var(--silver);padding:0 12px}
  .area{height:120px;padding:10px 12px;resize:vertical;font:14px/1.5 Inter,sans-serif}
  .r{display:flex;gap:10px;align-items:center}.sp{flex:1}
  .err{color:var(--red);font-size:13px;margin:0}
  .pick{width:100%;text-align:left;color:var(--silver);cursor:pointer}
  .note{font-size:13px;color:var(--muted)}
  button:disabled{opacity:.5;cursor:not-allowed}
</style>
