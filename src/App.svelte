<script lang="ts">
  import { onMount } from "svelte";
  import { api, type Folders, type Track } from "./lib/api";
  import { open } from "@tauri-apps/plugin-dialog";
  import { player, actions } from "./lib/player.svelte";
  import { sampleTracks, sampleDrives } from "./lib/sample";
  import SettingsView from "./lib/SettingsView.svelte";
  import TermsGate from "./lib/TermsGate.svelte";
  import { store, settingsActions, TERMS_VERSION } from "./lib/settings.svelte";

  let folders = $state<Folders | null>(null);
  onMount(async () => {
    await settingsActions.load();
    await actions.init();
    folders = await api.folders();
  });

  const nav = ["Dashboard", "Library", "Radio", "Playlists", "Favorites", "Settings"];
  let active = $state("Dashboard");
  const inTauri = "__TAURI_INTERNALS__" in window;
  async function playFile() {
    const picked = await open({ multiple: false, filters: [{ name: "Audio", extensions: ["mp3", "flac", "ogg", "wav", "m4a", "aac", "opus"] }] });
    if (typeof picked !== "string") return;
    const name = picked.split(/[\\/]/).pop() ?? picked;
    const t: Track = { id: picked, title: name.replace(/\.[^.]+$/, ""), subtitle: "Local file", kind: "Music", duration_secs: null, path: picked };
    await actions.play(t);
  }
  // keep the position and queue in step with the audio engine
  onMount(() => { const id = setInterval(actions.init, 500); return () => clearInterval(id); });
  const fmt = (s: number | null) => (s == null ? "LIVE" : `${Math.floor(s / 60)}:${String(s % 60).padStart(2, "0")}`);
</script>

{#if store.s && store.s.terms_version !== TERMS_VERSION}
  <TermsGate />
{/if}

<div class="shell">
  <header><img src="/logo.png" alt="Cindrix Radio" height="36" /></header>
  <nav>
    {#each nav as n}
      <button class="navbtn" class:on={active === n} onclick={() => (active = n)}>{n}</button>
    {/each}
  </nav>

  <main>
    <h1>{active}</h1>
    {#if active === "Settings"}
      <SettingsView />
    {:else}
    <div class="cols">
      <section class="left">
        <div class="card">
          <div class="head"><h3>Quick play</h3>{#if inTauri}<button class="btn primary" onclick={playFile}>Play a file…</button>{/if}</div>
          {#if player.s.error}<p class="err">{player.s.error}</p>{/if}
          <div class="rows">
            {#each sampleTracks as t}
              <div class="row">
                <button class="play" aria-label="Play {t.title}" onclick={() => actions.play(t)}>▶</button>
                <div class="meta"><b>{t.title}</b><span>{t.subtitle}</span></div>
                <span class="time">{fmt(t.duration_secs)}</span>
                <button class="btn" onclick={() => actions.enqueue(t)}>+ Queue</button>
              </div>
            {/each}
          </div>
        </div>
        {#if folders}
          <div class="card note">Saves go to <code>{folders.root}</code> by default. Change any folder in Settings.</div>
        {/if}
      </section>

      <aside>
        <div class="card fixed" style="height:246px">
          <h3>Library Sources</h3>
          <div class="list" style="height:120px">
            {#each sampleDrives as d}
              <div class="drive"><span class="dot" class:off={!d.online}></span><b>{d.name}</b><span>{d.count}</span></div>
            {/each}
          </div>
          <div class="btns"><button class="btn primary" onclick={() => (active = "Library")}>Open Library</button><button class="btn">Manage</button></div>
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
      </aside>
    </div>
    {/if}
  </main>

  <footer>
    <div class="np">
      {#if player.s.current}<b>{player.s.current.title}</b><span>{player.s.current.subtitle}</span>{:else}<span>Nothing playing</span>{/if}
    </div>
    <div class="tx">
      <button class="btn" onclick={actions.toggle}>{player.s.playing ? "Pause" : "Play"}</button>
      <button class="btn" onclick={actions.next}>Next</button>
    </div>
    <label class="vol">Vol
      <input type="range" min="0" max="1" step="0.01" value={player.s.volume} oninput={(e) => actions.volume(+e.currentTarget.value)} />
    </label>
  </footer>
</div>

<style>
  .shell{display:grid;grid-template-columns:232px 1fr;grid-template-rows:60px 1fr 84px;height:100%}
  header{grid-column:1/3;display:flex;align-items:center;padding:0 20px;border-bottom:1px solid var(--line)}
  header img{mix-blend-mode:screen}
  nav{padding:16px 12px;display:flex;flex-direction:column;gap:6px;border-right:1px solid var(--line)}
  .navbtn{height:40px;text-align:left;padding:0 14px;border-radius:10px;border:0;background:transparent;color:var(--silver)}
  .navbtn.on{background:#1FA8FF18;color:var(--lblue)}
  main{padding:20px;overflow:auto}
  .cols{display:flex;gap:20px;margin-top:16px}
  .left{flex:1;display:flex;flex-direction:column;gap:16px;min-width:0}
  aside{width:340px;display:flex;flex-direction:column;gap:16px}
  .fixed{flex:none;display:flex;flex-direction:column;gap:10px}
  .list{overflow-y:auto;display:flex;flex-direction:column;gap:8px}
  .drive{flex:none;display:flex;align-items:center;gap:8px;font-size:13px}
  .drive span:last-child{margin-left:auto;color:var(--muted)}
  .dot{width:8px;height:8px;border-radius:4px;background:var(--green)}
  .dot.off{background:var(--muted)}
  .btns{display:flex;gap:8px;margin-top:auto}
  .rows{display:flex;flex-direction:column;gap:6px;margin-top:10px}
  .row{display:flex;align-items:center;gap:12px;height:52px}
  .play{width:36px;height:36px;border-radius:18px;border:0;background:var(--blue);color:#04121C}
  .meta{display:flex;flex-direction:column;font-size:13px}.meta span,.time,.muted{color:var(--muted);font-size:12px}
  .time{margin-left:auto}
  .note{font-size:13px;color:var(--muted)}
  .head{display:flex;justify-content:space-between;align-items:center}
  .err{color:var(--red);font-size:12px;margin:8px 0 0}
  footer{grid-column:1/3;display:flex;align-items:center;gap:20px;padding:0 20px;border-top:1px solid var(--line);background:var(--panel)}
  .np{display:flex;flex-direction:column;font-size:13px;min-width:200px}.np span{color:var(--muted)}
  .tx{display:flex;gap:8px}.vol{margin-left:auto;display:flex;gap:8px;align-items:center;font-size:12px}
</style>
