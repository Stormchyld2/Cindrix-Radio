<script lang="ts">
  import { onMount } from "svelte";
  import { api, type Counts, type Kind, type LibItem } from "./api";
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
  let counts = $state<Counts>({ music: 0, podcast: 0, audiobook: 0, video: 0 });
  let genres = $state<[string, number][]>([]);
  let items = $state<LibItem[]>([]);
  let total = $state(0);
  let timer: ReturnType<typeof setTimeout>;

  async function refresh() {
    const [p, g, c] = await Promise.all([api.libSearch(query, kind, genre, 200, 0), api.libGenres(kind), api.libCounts()]);
    items = p.items; total = p.total; genres = g; counts = c;
  }
  onMount(refresh);
  const pick = (k: Kind) => { kind = k; genre = null; refresh(); };
  const typing = () => { clearTimeout(timer); timer = setTimeout(refresh, 200); };

  const toTrack = (i: LibItem) => ({ id: i.id, title: i.title, subtitle: i.artist || i.group, kind: i.kind, duration_secs: i.duration_secs, path: i.path });
  const playable = (i: LibItem) => i.kind !== "Video" && i.available;
  async function shuffle() {
    const p = await api.libSearch(query, kind, genre, 2000, 0);
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

<input class="search" placeholder="Search {kind === 'Music' ? 'songs, artists, albums' : 'titles'}…" bind:value={query} oninput={typing} />

{#if genres.length}
  <div class="genres">
    <button class="chip" class:on={genre === null} onclick={() => { genre = null; refresh(); }}>All</button>
    {#each genres as [g, n]}
      <button class="chip" class:on={genre === g} onclick={() => { genre = g; refresh(); }}>{g} <span>{n}</span></button>
    {/each}
  </div>
{/if}

<div class="actions">
  <span class="note">{total.toLocaleString()} {total === 1 ? "item" : "items"}{total > items.length ? ` (showing ${items.length})` : ""}</span>
  <button class="btn primary" disabled={kind === "Video" || !total} onclick={shuffle}>Shuffle</button>
</div>

{#if total === 0 && !query && !genre}
  <div class="card empty">Nothing here yet. Add a drive or folder in Settings → Library &amp; Sources, then scan it.</div>
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
  button:disabled{opacity:.5;cursor:not-allowed}
</style>
