<script lang="ts">
  import { onMount } from "svelte";
  import { api, type AlbumRow, type Counts, type Kind, type LibItem } from "./api";
  import { actions } from "./player.svelte";

  const types: { kind: Kind; label: string; key: keyof Counts }[] = [
    { kind: "Music", label: "Music", key: "music" },
    { kind: "Podcast", label: "Podcasts", key: "podcast" },
    { kind: "Audiobook", label: "Audiobooks", key: "audiobook" },
    { kind: "Video", label: "Videos", key: "video" },
  ];
  let kind = $state<Kind>("Music");
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
  const pick = (k: Kind) => { kind = k; genre = null; artist = null; album = null; view = "songs"; refresh(); };
  const setView = (v: typeof view) => { view = v; genre = null; artist = null; album = null; refresh(); };
  const openArtist = (a: string) => { artist = a; album = null; view = "songs"; refresh(); };
  const openAlbum = (a: AlbumRow) => { artist = a.artist || null; album = a.album; view = "songs"; refresh(); };
  const openGenre = (g: string | null) => { genre = g; view = "songs"; refresh(); };
  const typing = () => { clearTimeout(timer); timer = setTimeout(refresh, 200); };

  const toTrack = (i: LibItem) => ({ id: i.id, title: i.title, subtitle: i.artist || i.group, kind: i.kind, duration_secs: i.duration_secs, path: i.path });
  const playable = (i: LibItem) => i.kind !== "Video" && i.available;
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

<input class="search" placeholder="Search {kind === 'Music' ? 'songs, artists, albums, genres' : 'titles'}…" bind:value={query} oninput={typing} />

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
        <button class="play" disabled={!playable(i)} title={i.kind === "Video" ? "Video playback comes with the video player" : i.available ? "Play" : "Drive not connected"} aria-label="Play {i.title}" onclick={() => actions.play(toTrack(i))}>▶</button>
        <div class="meta"><b>{i.title}</b><span>{[i.artist || i.group, i.album].filter(Boolean).join(" · ")}</span></div>
        {#if !i.available}<span class="tag">Unavailable</span>{/if}
        <span class="time">{fmt(i.duration_secs)}</span>
        <button class="btn" disabled={!playable(i)} onclick={() => actions.enqueue(toTrack(i))}>+ Queue</button>
      </div>
    {/each}
  </div>
{/if}

<style>
  .types{display:flex;gap:8px;flex-wrap:wrap;margin:12px 0}
  .type.on,.chip.on{border-color:var(--blue);color:var(--lblue);background:#1FA8FF14}
  .type b{margin-left:6px;color:var(--muted);font-weight:500}
  .search{width:100%;height:44px;border-radius:12px;border:1px solid #2A3747;background:#0D1620;color:var(--silver);padding:0 14px;font-size:15px}
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
  .meta{flex:1;min-width:0;display:flex;flex-direction:column;font-size:13px}.meta b,.meta span{overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
  .meta span,.time{color:var(--muted);font-size:12px}
  .tag{font-size:12px;color:var(--muted)}
  .pick{width:100%;text-align:left;color:var(--silver);cursor:pointer}
  .note{font-size:13px;color:var(--muted)}
  button:disabled{opacity:.5;cursor:not-allowed}
</style>
