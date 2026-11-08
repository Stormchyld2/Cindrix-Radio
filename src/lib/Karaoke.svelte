<script lang="ts">
  import { onMount } from "svelte";
  import { open } from "@tauri-apps/plugin-dialog";
  import { api, type LibItem, type LyricsSummary } from "./api";
  import { player, actions } from "./player.svelte";
  import { store, settingsActions } from "./settings.svelte";

  const inTauri = "__TAURI_INTERNALS__" in window;
  const lyricsOn = $derived(store.s?.lyrics_viewer ?? false);
  const reduction = $derived(store.s?.eq.vocal_reduction ?? 0);

  let view = $state<"pick" | "stage">("pick");
  let query = $state("");
  let songs = $state<LibItem[]>([]);
  let message = $state<string | null>(null);
  let stageEl = $state<HTMLElement | null>(null);
  let full = $state(false);
  let text = $state<string | null>(null);
  let timer: ReturnType<typeof setTimeout>;

  const toTrack = (i: LibItem) => ({ id: i.id, title: i.title, subtitle: i.artist || i.group, kind: i.kind, duration_secs: i.duration_secs, path: i.path });
  async function find() {
    const p = await api.libSearch(query, "Music", null, null, null, 60, 0);
    songs = p.items.filter((i) => i.available);
  }
  const typing = () => { clearTimeout(timer); timer = setTimeout(find, 200); };
  onMount(() => { find(); document.addEventListener("fullscreenchange", () => (full = !!document.fullscreenElement)); });

  async function sing(i: LibItem) {
    message = null;
    try { await actions.play(toTrack(i)); view = "stage"; } catch (e) { message = String(e instanceof Error ? e.message : e); }
  }
  // after the song, go back to choosing the next one
  $effect(() => { if (view === "stage" && !player.s.current) { view = "pick"; if (document.fullscreenElement) document.exitFullscreen().catch(() => {}); } });
  // pick up lyrics for whatever is playing (only when the viewer is on)
  $effect(() => {
    const path = player.s.current?.path; const on = lyricsOn;
    text = null;
    if (path && on) api.lyricsGet(path).then((t) => { if (player.s.current?.path === path) text = t; }).catch(() => {});
  });
  async function toggleFull() {
    try { if (document.fullscreenElement) await document.exitFullscreen(); else await stageEl?.requestFullscreen(); } catch { /* not allowed here */ }
  }
  const mmss = (s: number) => `${Math.floor(s / 60)}:${String(s % 60).padStart(2, "0")}`;

  // ---- lyrics editor
  let editing = $state(false);
  let linkText = $state("");
  let linkPath = $state<string | null>(null);
  let linkTitle = $state("");
  let matches = $state<LibItem[]>([]);
  let body = $state("");
  let saved = $state<LyricsSummary[]>([]);
  let ltimer: ReturnType<typeof setTimeout>;

  async function openEditor(path?: string | null, title?: string) {
    message = null; editing = true;
    saved = await api.lyricsList().catch(() => []);
    pickLink(path ?? player.s.current?.path ?? null, title ?? player.s.current?.title ?? "");
    body = linkPath ? ((await api.lyricsGet(linkPath)) ?? "") : "";
  }
  function pickLink(path: string | null, title: string) { linkPath = path; linkTitle = title; linkText = title; matches = []; }
  function linkTyping() {
    linkPath = null; clearTimeout(ltimer);
    ltimer = setTimeout(async () => {
      if (!linkText.trim()) { matches = []; return; }
      matches = (await api.libSearch(linkText, "Music", null, null, null, 8, 0)).items;
    }, 150);
  }
  async function chooseMatch(i: LibItem) { pickLink(i.path, i.title); body = (await api.lyricsGet(i.path)) ?? ""; }
  async function browse() {
    const f = await open({ multiple: false, filters: [{ name: "Audio", extensions: ["mp3", "flac", "ogg", "wav", "m4a", "aac", "opus"] }] });
    if (typeof f !== "string") return;
    const name = (f.split(/[\\/]/).pop() ?? f).replace(/\.[^.]+$/, "");
    pickLink(f, name); body = (await api.lyricsGet(f)) ?? "";
  }
  async function save() {
    if (!linkPath) { message = "Pick the song these lyrics belong to."; return; }
    try { saved = await api.lyricsSet(linkPath, linkTitle, body); editing = false; message = body.trim() ? "Lyrics saved on this device." : "Lyrics removed."; if (linkPath === player.s.current?.path) text = body.trim() ? body : null; }
    catch (e) { message = String(e instanceof Error ? e.message : e); }
  }
</script>

{#if view === "pick"}
  <input class="search" placeholder="Pick a song to sing…" bind:value={query} oninput={typing} />
  {#if player.s.current && player.s.current.kind === "Music"}
    <div class="card cur"><span>Now playing: <b>{player.s.current.title}</b></span><button class="btn primary" onclick={() => (view = "stage")}>Open karaoke view</button></div>
  {/if}
  {#if message}<p class="status">{message}</p>{/if}
  <div class="tools">
    <span class="note">Vocal remover works on any song with the voice in the centre. Results vary by recording.</span>
    {#if lyricsOn && inTauri}<button class="btn" onclick={() => openEditor(null, "")}>Add or edit my lyrics</button>{/if}
  </div>
  <div class="rows">
    {#each songs as i (i.id)}
      <div class="row">
        <button class="play" aria-label="Sing {i.title}" onclick={() => sing(i)}>🎤</button>
        <div class="meta"><b>{i.title}</b><span>{[i.artist || i.group, i.album].filter(Boolean).join(" · ")}</span></div>
      </div>
    {:else}
      <div class="card note">No songs found. Add music in Settings → Library &amp; Sources or download some to Downloads/Cindrix/Music.</div>
    {/each}
  </div>
{:else}
  <div class="stage" class:full bind:this={stageEl}>
    <div class="lyr">
      {#if lyricsOn && text}
        <pre>{text}</pre>
      {:else if lyricsOn}
        <div class="banner"><img src="/logo.png" alt="Cindrix Radio" /><p>No lyrics added for this song.</p>
          {#if inTauri && player.s.current?.path}<button class="btn" onclick={() => openEditor(player.s.current?.path, player.s.current?.title)}>Add my lyrics</button>{/if}</div>
      {:else}
        <div class="banner"><img src="/logo.png" alt="Cindrix Radio" /><p>Cindrix Radio karaoke</p></div>
      {/if}
    </div>
    <div class="panel">
      <div class="title"><b>{player.s.current?.title}</b><span>{player.s.current?.subtitle}</span></div>
      <label class="vr">Vocal remover
        <input type="range" min="0" max="100" step="5" value={Math.round(reduction * 100)} aria-label="Vocal remover amount"
          oninput={(e) => settingsActions.setVocal(Number(e.currentTarget.value) / 100)} />
        <output>{reduction > 0 ? `${Math.round(reduction * 100)}%` : "Off"}</output>
      </label>
      <div class="ctl">
        <span class="time">{mmss(player.s.position_secs)}</span>
        <button class="btn primary" onclick={actions.toggle}>{player.s.playing ? "Pause" : "Play"}</button>
        <button class="btn" onclick={actions.next}>Next</button>
        <label class="vol">Vol <input type="range" min="0" max="1" step="0.01" value={player.s.volume} aria-label="Volume" oninput={(e) => actions.volume(Number(e.currentTarget.value))} /></label>
        <span class="sp"></span>
        <button class="btn" onclick={toggleFull}>{full ? "Exit full screen" : "Full screen"}</button>
        <button class="btn" onclick={() => (view = "pick")}>Songs</button>
      </div>
    </div>
  </div>
{/if}

{#if editing}
  <div class="gate" role="dialog" aria-modal="true" aria-label="My lyrics">
    <div class="card box">
      <h2>My lyrics</h2>
      <p class="note">Paste lyrics you are allowed to use. They are saved only on this device and are never uploaded or shared. You are responsible for what you add.</p>
      <label>Song
        <div class="link">
          <input class="field" placeholder="Start typing the song name…" bind:value={linkText} oninput={linkTyping} aria-label="Song" />
          <button class="btn" onclick={browse}>Browse…</button>
        </div>
      </label>
      {#if matches.length && !linkPath}
        <div class="dd" role="listbox">{#each matches as m (m.id)}<button role="option" aria-selected="false" onclick={() => chooseMatch(m)}><b>{m.title}</b> <span>{m.artist || m.group}</span></button>{/each}</div>
      {/if}
      {#if linkPath}<p class="ok">Linked to: {linkTitle}</p>{/if}
      <textarea class="field area" placeholder="Paste or type lyrics here" bind:value={body} aria-label="Lyrics"></textarea>
      {#if saved.length}<p class="note">Songs with lyrics: {saved.map((s) => s.title || s.path).join(", ")}</p>{/if}
      {#if message}<p class="status err">{message}</p>{/if}
      <div class="r"><button class="btn" onclick={() => { editing = false; message = null; }}>Cancel</button><button class="btn primary" onclick={save}>{body.trim() ? "Save" : "Remove lyrics"}</button></div>
    </div>
  </div>
{/if}

<style>
  .search{width:100%;height:44px;border-radius:12px;border:1px solid #2A3747;background:#0D1620;color:var(--silver);padding:0 14px;font-size:15px;margin-bottom:10px}
  .cur{display:flex;justify-content:space-between;align-items:center;gap:12px;margin:8px 0;font-size:14px}
  .tools{display:flex;justify-content:space-between;align-items:center;gap:12px;flex-wrap:wrap;margin:8px 0}
  .note{font-size:13px;color:var(--muted)}
  .status{font-size:13px;color:var(--lblue)}.status.err{color:var(--red)}
  .rows{display:flex;flex-direction:column;gap:6px}
  .row{display:flex;align-items:center;gap:12px;min-height:52px;padding:6px 10px;border-radius:12px;border:1px solid var(--line);background:var(--panel)}
  .play{width:36px;height:36px;border-radius:18px;border:0;background:var(--blue);color:#04121C;flex:none}
  .meta{flex:1;min-width:0;display:flex;flex-direction:column;font-size:13px}.meta b,.meta span{overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
  .meta span{color:var(--muted);font-size:12px}
  .stage{display:flex;flex-direction:column;gap:12px;height:calc(100vh - 170px);min-height:420px}
  .stage.full{height:100vh;padding:20px;background:var(--bg,#0A0E13)}
  .lyr{flex:1;min-height:0;overflow:auto;border:1px solid var(--line);border-radius:16px;background:var(--panel);padding:24px;display:flex}
  .lyr pre{margin:auto;font:600 clamp(22px,3.2vw,40px)/1.5 Rajdhani,Inter,sans-serif;text-align:center;white-space:pre-wrap;color:var(--silver)}
  .banner{margin:auto;text-align:center;color:var(--muted);display:flex;flex-direction:column;align-items:center;gap:12px}
  .banner img{max-width:min(420px,70%);mix-blend-mode:screen}
  .panel{display:flex;flex-direction:column;gap:10px;padding:14px;border:1px solid var(--line);border-radius:16px;background:var(--panel)}
  .title{display:flex;flex-direction:column;font-size:14px}.title span{color:var(--muted);font-size:12px}
  .vr{display:flex;align-items:center;gap:12px;font-size:14px}.vr input{flex:1;max-width:360px}.vr output{width:44px;color:var(--lblue)}
  .ctl{display:flex;align-items:center;gap:10px;flex-wrap:wrap}
  .time{font-size:13px;color:var(--muted);min-width:40px}
  .vol{display:flex;align-items:center;gap:8px;font-size:13px}
  .sp{flex:1}
  .gate{position:fixed;inset:0;background:#000000CC;display:flex;align-items:center;justify-content:center;z-index:40}
  .box{width:min(620px,94vw);max-height:92vh;overflow:auto;display:flex;flex-direction:column;gap:10px}
  .box label{display:flex;flex-direction:column;gap:6px;font-size:13px}
  .link{display:flex;gap:8px}.link .field{flex:1}
  .field{height:38px;border-radius:10px;border:1px solid #2A3747;background:#0D1620;color:var(--silver);padding:0 12px}
  .area{height:240px;padding:10px 12px;resize:vertical;font:14px/1.5 Inter,sans-serif}
  .dd{display:flex;flex-direction:column;border:1px solid #2A3747;border-radius:10px;background:#0A0E13;max-height:200px;overflow:auto}
  .dd button{text-align:left;border:0;background:transparent;color:var(--silver);padding:8px 12px;font-size:13px}
  .dd button:hover{background:#1FA8FF18}.dd span{color:var(--muted);font-size:12px}
  .ok{color:var(--green);font-size:13px;margin:0}
  .r{display:flex;justify-content:flex-end;gap:10px}
  button:disabled{opacity:.5;cursor:not-allowed}
</style>
