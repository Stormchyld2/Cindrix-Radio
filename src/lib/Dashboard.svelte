<script lang="ts">
  import { onMount } from "svelte";
  import { open } from "@tauri-apps/plugin-dialog";
  import { isAndroid } from "./platform";
  import { api, type Counts, type Folders, type LibItem, type SourceStatus, type Track } from "./api";
  import { player, actions } from "./player.svelte";
  import { store, settingsActions } from "./settings.svelte";
  import { startTrack } from "./playlistStore.svelte";

  let { go }: { go: (tab: string) => void } = $props();

  const inTauri = "__TAURI_INTERNALS__" in window;
  let folders = $state<Folders | null>(null);
  let sources = $state<SourceStatus[]>([]);
  let counts = $state<Counts>({ music: 0, podcast: 0, audiobook: 0, video: 0 });
  let songs = $state<LibItem[]>([]);
  let message = $state<string | null>(null);

  onMount(async () => {
    const [f, s, c, p] = await Promise.all([api.folders(), api.libSources(), api.libCounts(), api.libSearch("", "Music", null, null, null, 8, 0)]);
    folders = f; sources = s; counts = c; songs = p.items.filter((i) => i.available);
  });

  async function playFile() {
    const picked = await open({ multiple: false, filters: [{ name: "Audio", extensions: ["mp3", "flac", "ogg", "wav", "m4a", "aac", "opus"] }] });
    if (typeof picked !== "string") return;
    const name = picked.split(/[\\/]/).pop() ?? picked;
    await run(() => actions.play({ id: picked, title: name.replace(/\.[^.]+$/, ""), subtitle: "Local file", kind: "Music", duration_secs: null, path: picked }));
  }
  async function run(job: () => Promise<unknown>) {
    message = null;
    try { await job(); } catch (e) { message = String(e instanceof Error ? e.message : e); }
  }
  const toTrack = (i: LibItem): Track => ({ id: i.id, title: i.title, subtitle: i.artist || i.group, kind: i.kind, duration_secs: i.duration_secs, path: i.path });
  const fmt = (s: number | null) => (s == null ? "" : `${Math.floor(s / 60)}:${String(s % 60).padStart(2, "0")}`);
  const nice = (s: SourceStatus) => (s.source.builtin ? "Downloads/Cindrix" : s.source.path);
  const tiles: { label: string; key: keyof Counts; tab: string }[] = [
    { label: "Music", key: "music", tab: "Library" }, { label: "Podcasts", key: "podcast", tab: "Podcasts" },
    { label: "Audiobooks", key: "audiobook", tab: "Library" }, { label: "Videos", key: "video", tab: "Videos" },
  ];
  const cur = $derived(player.s.current);
  const err = $derived(message ?? player.s.error ?? null);
</script>

<div class="cols">
  <section class="left">
    <div class="tiles">
      {#each tiles as t}
        <button class="tile" onclick={() => go(t.tab)}><b>{counts[t.key].toLocaleString()}</b><span>{t.label}</span></button>
      {/each}
    </div>

    <div class="card">
      <div class="head"><h3>Now playing</h3>{#if inTauri && !isAndroid}<button class="btn primary" onclick={playFile}>Play a file…</button>{/if}</div>
      {#if cur}
        <div class="np">
          <b>{cur.title}</b>
          <span>{cur.kind === "Radio" ? (player.s.connecting ? "Connecting…" : player.s.now_playing ?? cur.subtitle) : cur.subtitle}</span>
          <div class="btns">
            <button class="btn primary" onclick={actions.toggle}>{player.s.playing ? "Pause" : "Play"}</button>
            <button class="btn" onclick={actions.next}>Next</button>
            {#if cur.kind === "Music"}<button class="btn" onclick={() => go("Karaoke")}>Karaoke view</button>{/if}
          </div>
        </div>
      {:else}
        <p class="note">Nothing playing. Pick something below, or open Radio, Podcasts or Videos.</p>
      {/if}
      {#if err}<p class="err">{err}</p>{/if}
    </div>

    <div class="card">
      <div class="head"><h3>From your library</h3><button class="btn" onclick={() => go("Library")}>Open Library</button></div>
      <div class="rows">
        {#each songs as i (i.id)}
          <div class="row">
            <button class="play" aria-label="Play {i.title}" onclick={() => run(async () => { await startTrack(toTrack(i)); })}>▶</button>
            <div class="meta"><b>{i.title}</b><span>{[i.artist || i.group, i.album].filter(Boolean).join(" · ")}</span></div>
            <span class="time">{fmt(i.duration_secs)}</span>
            <button class="btn" onclick={() => run(() => actions.enqueue(toTrack(i)))}>+ Queue</button>
          </div>
        {:else}
          <p class="note">No music yet. Songs you download land in Downloads/Cindrix automatically, or add a drive in Settings → Library &amp; Sources.</p>
        {/each}
      </div>
    </div>

    {#if folders}<div class="card note">Saves go to <code>{folders.root}</code> by default. Change any folder in Settings.</div>{/if}
  </section>

  <aside>
    <div class="card fixed" style="height:246px">
      <h3>Library Sources</h3>
      <div class="list" style="height:120px">
        {#each sources as s (s.source.id)}
          <div class="drive"><span class="dot" class:off={!s.online}></span><b title={s.source.path}>{nice(s)}</b><span>{s.online ? `${s.items.toLocaleString()} items` : `${s.items.toLocaleString()} unavailable`}</span></div>
        {:else}
          <span class="muted">No sources yet.</span>
        {/each}
      </div>
      <div class="btns"><button class="btn primary" onclick={() => go("Library")}>Open Library</button><button class="btn" onclick={() => go("Settings")}>Manage</button></div>
    </div>
    <div class="card fixed" style="height:292px">
      <h3>Up Next</h3>
      <div class="list" style="height:220px">
        {#each player.s.queue as t}
          <div class="drive"><b>{t.title}</b><span>{t.subtitle}</span></div>
        {:else}
          <span class="muted">Queue is empty.</span>
        {/each}
      </div>
    </div>
    <div class="card fixed" style="height:112px">
      <h3>Connectivity</h3>
      <div class="seg" role="group" aria-label="Connection mode">
        <button class:on={!store.s?.grid_down} onclick={() => settingsActions.setGridDown(false)}>Online</button>
        <button class:on={store.s?.grid_down} onclick={() => settingsActions.setGridDown(true)}>Grid-Down</button>
      </div>
      <span class="muted">{store.s?.grid_down ? "Local and downloaded content only. No network use." : "Streams, feeds and downloads available."}</span>
    </div>
  </aside>
</div>

<style>
  .cols{display:flex;gap:20px;margin-top:16px}
  .left{flex:1;display:flex;flex-direction:column;gap:16px;min-width:0}
  aside{width:340px;display:flex;flex-direction:column;gap:16px}
  .tiles{display:grid;grid-template-columns:repeat(4,1fr);gap:12px}
  .tile{display:flex;flex-direction:column;align-items:flex-start;gap:2px;padding:14px;border-radius:14px;border:1px solid var(--line);background:var(--panel);color:var(--silver)}
  .tile:hover{border-color:var(--blue)}
  .tile b{font:700 26px Rajdhani,sans-serif}.tile span{font-size:12px;color:var(--muted)}
  .fixed{flex:none;display:flex;flex-direction:column;gap:10px}
  .list{overflow-y:auto;display:flex;flex-direction:column;gap:8px}
  .drive{flex:none;display:flex;align-items:center;gap:8px;font-size:13px}
  .drive b{min-width:0;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
  .drive span:last-child{margin-left:auto;color:var(--muted);flex:none}
  .dot{width:8px;height:8px;border-radius:4px;background:var(--green);flex:none}.dot.off{background:var(--muted)}
  .btns{display:flex;gap:8px;margin-top:10px}.fixed .btns{margin-top:auto}
  .rows{display:flex;flex-direction:column;gap:6px;margin-top:10px}
  .row{display:flex;align-items:center;gap:12px;height:52px}
  .play{width:36px;height:36px;border-radius:18px;border:0;background:var(--blue);color:#04121C;flex:none}
  .meta{display:flex;flex-direction:column;font-size:13px;min-width:0;flex:1}
  .meta b,.meta span{overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
  .meta span,.time,.muted{color:var(--muted);font-size:12px}
  .np{display:flex;flex-direction:column;gap:2px;margin-top:8px;font-size:14px}.np span{color:var(--muted);font-size:12px}
  .note{font-size:13px;color:var(--muted)}
  .seg{display:flex;border:1px solid #2A3747;border-radius:10px;overflow:hidden}
  .seg button{flex:1;height:32px;border:0;background:transparent;color:var(--silver);font-size:13px}
  .seg button.on{background:#1FA8FF18;color:var(--lblue)}
  .head{display:flex;justify-content:space-between;align-items:center}
  .err{color:var(--red);font-size:12px;margin:8px 0 0}
  @media (max-width:800px){
    .cols{flex-direction:column}
    aside{width:auto}
    .tiles{grid-template-columns:repeat(2,1fr)}
  }
</style>
