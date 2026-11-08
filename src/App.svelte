<script lang="ts">
  import { onMount } from "svelte";
  import { player, actions } from "./lib/player.svelte";
  import Dashboard from "./lib/Dashboard.svelte";
  import SettingsView from "./lib/SettingsView.svelte";
  import Podcasts from "./lib/Podcasts.svelte";
  import Library from "./lib/Library.svelte";
  import Radio from "./lib/Radio.svelte";
  import Videos from "./lib/Videos.svelte";
  import Karaoke from "./lib/Karaoke.svelte";
  import Playlists from "./lib/Playlists.svelte";
  import Favorites from "./lib/Favorites.svelte";
  import VideoPlayer from "./lib/VideoPlayer.svelte";
  import TermsGate from "./lib/TermsGate.svelte";
  import NowPlaying from "./lib/NowPlaying.svelte";
  import { ui } from "./lib/ui.svelte";
  import { api } from "./lib/api";
  import { store, settingsActions, TERMS_VERSION } from "./lib/settings.svelte";

  onMount(async () => {
    await settingsActions.load();
    await actions.init();
    if (store.s?.prefs.auto_refresh_podcasts && !store.s.grid_down) api.podcastRefresh().catch(() => {});
  });

  // look and feel from Settings → Appearance
  $effect(() => {
    const p = store.s?.prefs;
    const r = document.documentElement;
    r.style.setProperty("--scale", String(p?.text_scale ?? 1));
    r.dataset.accent = p?.accent ?? "blue";
    if (p?.high_contrast) r.dataset.contrast = "high"; else delete r.dataset.contrast;
    if (p?.reduce_motion) r.dataset.motion = "reduced"; else delete r.dataset.motion;
  });

  // keep the screen on while playing, if asked
  let lock: { release: () => Promise<void> } | null = null;
  $effect(() => {
    const want = !!store.s?.prefs.keep_awake && player.s.playing;
    const wl = (navigator as unknown as { wakeLock?: { request: (t: string) => Promise<{ release: () => Promise<void> }> } }).wakeLock;
    if (want && !lock && wl) wl.request("screen").then((l) => (lock = l)).catch(() => {});
    if (!want && lock) { lock.release().catch(() => {}); lock = null; }
  });

  // sleep timer: pause when it runs out
  onMount(() => {
    const id = setInterval(() => {
      if (ui.sleepEnd && Date.now() >= ui.sleepEnd) { ui.sleepEnd = null; if (player.s.playing) actions.toggle(); }
    }, 1000);
    return () => clearInterval(id);
  });

  // seek bar: only for files whose length is known (live radio cannot be seeked)
  let dragging = $state<number | null>(null);
  const dur = $derived(player.s.current?.duration_secs ?? 0);
  const seekable = $derived(!!player.s.current?.path && dur > 0);
  const mmss = (s: number) => `${Math.floor(s / 3600) ? Math.floor(s / 3600) + ":" : ""}${String(Math.floor((s % 3600) / 60)).padStart(Math.floor(s / 3600) ? 2 : 1, "0")}:${String(s % 60).padStart(2, "0")}`;

  const nav = ["Dashboard", "Library", "Radio", "Podcasts", "Videos", "Karaoke", "Playlists", "Favorites", "Settings"];
  let active = $state("Dashboard");
  let menu = $state(false);
  const go = (n: string) => { active = n; menu = false; };
  // keep the position and queue in step with the audio engine
  onMount(() => { const id = setInterval(actions.init, 500); return () => clearInterval(id); });
</script>

{#if store.s && store.s.terms_version !== TERMS_VERSION}
  <TermsGate />
{/if}

<VideoPlayer />
<NowPlaying />

<div class="shell">
  <header>
    <button class="burger" aria-label="Menu" aria-expanded={menu} onclick={() => (menu = !menu)}>☰</button>
    <img src="/logo.png" alt="Cindrix Radio" height="36" />
    <button class="mode" class:grid={store.s?.grid_down} onclick={() => settingsActions.setGridDown(!store.s?.grid_down)}
      title="Switch between Online and Grid-Down mode" aria-label="Connection mode: {store.s?.grid_down ? 'Grid-Down' : 'Online'}. Click to switch.">
      <span class="d"></span>{store.s?.grid_down ? "Grid-Down" : "Online"}
    </button>
  </header>
  {#if menu}<button class="scrim" aria-label="Close menu" onclick={() => (menu = false)}></button>{/if}
  <nav class:open={menu}>
    {#each nav as n}
      <button class="navbtn" class:on={active === n} onclick={() => go(n)}>{n}</button>
    {/each}
  </nav>

  <main>
    <h1>{active}</h1>
    {#if active === "Settings"}
      <SettingsView />
    {:else if active === "Radio"}
      <Radio go={(t) => (active = t)} />
    {:else if active === "Karaoke"}
      <Karaoke />
    {:else if active === "Playlists"}
      <Playlists />
    {:else if active === "Favorites"}
      <Favorites />
    {:else if active === "Library"}
      <Library />
    {:else if active === "Podcasts"}
      <Podcasts />
    {:else if active === "Videos"}
      <Videos />
    {:else}
      <Dashboard go={(t) => (active = t)} />
    {/if}
  </main>

  <footer>
    <button class="np" aria-label="Open the full player" onclick={() => (ui.np = true)}>
      {#if player.s.current}<b>{player.s.current.title}</b><span>{player.s.current.subtitle}</span>{:else}<span>Nothing playing</span>{/if}
    </button>
    <div class="tx">
      <button class="btn" onclick={actions.toggle}>{player.s.playing ? "Pause" : "Play"}</button>
      <button class="btn" onclick={actions.next}>Next</button>
    </div>
    {#if seekable}
      <div class="seek">
        <span>{mmss(dragging ?? player.s.position_secs)}</span>
        <input type="range" min="0" max={dur} step="1" value={dragging ?? player.s.position_secs} aria-label="Seek"
          oninput={(e) => (dragging = +e.currentTarget.value)}
          onchange={async (e) => { const v = +e.currentTarget.value; await actions.seek(v); dragging = null; }} />
        <span>{mmss(dur)}</span>
      </div>
    {/if}
    <label class="vol">Vol
      <input type="range" min="0" max="1" step="0.01" value={player.s.volume} oninput={(e) => actions.volume(+e.currentTarget.value)} />
    </label>
  </footer>
</div>

<style>
  .shell{display:grid;grid-template-columns:232px 1fr;grid-template-rows:60px 1fr 84px;height:100%}
  header{grid-column:1/3;display:flex;align-items:center;padding:0 20px;border-bottom:1px solid var(--line)}
  header img{mix-blend-mode:screen}
  header{justify-content:space-between}
  .mode{height:32px;padding:0 14px;border-radius:16px;border:1px solid #2A3747;background:transparent;color:var(--silver);font-size:12px;display:flex;align-items:center;gap:8px}
  .mode .d{width:8px;height:8px;border-radius:4px;background:var(--green)}
  .mode.grid{border-color:var(--blue);color:var(--lblue);background:#1FA8FF14}
  .mode.grid .d{background:var(--blue)}
  nav{padding:16px 12px;display:flex;flex-direction:column;gap:6px;border-right:1px solid var(--line)}
  .navbtn{height:40px;text-align:left;padding:0 14px;border-radius:10px;border:0;background:transparent;color:var(--silver)}
  .navbtn.on{background:#1FA8FF18;color:var(--lblue)}
  main{padding:20px;overflow:auto}
  footer{grid-column:1/3;display:flex;align-items:center;gap:20px;padding:0 20px;border-top:1px solid var(--line);background:var(--panel)}
  .np{display:flex;flex-direction:column;align-items:flex-start;text-align:left;font-size:13px;min-width:200px;border:0;background:transparent;color:inherit;padding:0;font-family:inherit}.np span{color:var(--muted)}
  .tx{display:flex;gap:8px}
  .seek{flex:1;display:flex;align-items:center;gap:10px;font-size:12px;color:var(--muted);min-width:0}.seek input{flex:1;min-width:80px}.vol{margin-left:auto;display:flex;gap:8px;align-items:center;font-size:12px}
  .burger,.scrim{display:none}
  @media (max-width:800px){
    .shell{grid-template-columns:1fr;grid-template-rows:56px 1fr auto}
    header{grid-column:1;padding:0 12px;gap:10px}
    .burger{display:block;width:40px;height:40px;border-radius:10px;border:1px solid #2A3747;background:transparent;color:var(--silver);font-size:20px}
    header img{margin-right:auto}
    nav{position:fixed;z-index:20;top:0;bottom:0;left:0;width:260px;max-width:80vw;background:var(--panel);transform:translateX(-105%);transition:transform .18s;padding-top:24px}
    nav.open{transform:none}
    .scrim{display:block;position:fixed;z-index:19;inset:0;border:0;background:#000A}
    main{padding:14px}
    main h1{font-size:24px}
    footer{grid-column:1;flex-wrap:wrap;gap:8px 12px;padding:8px 12px}
    .np{min-width:0;flex:1 1 55%;overflow:hidden}.np b,.np span{overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
    .tx .btn{height:40px}
    .seek{order:5;flex:1 0 100%}
    .vol{display:none}
  }
</style>
