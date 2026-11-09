<script lang="ts">
  import { onMount } from "svelte";
  import { player, actions } from "./lib/player.svelte";
  import Dashboard from "./lib/Dashboard.svelte";
  import SettingsView from "./lib/SettingsView.svelte";
  import Podcasts from "./lib/Podcasts.svelte";
  import Library from "./lib/Library.svelte";
  import Radio from "./lib/Radio.svelte";
  import Videos from "./lib/Videos.svelte";
  import HubView from "./lib/HubView.svelte";
  import Playlists from "./lib/Playlists.svelte";
  import Favorites from "./lib/Favorites.svelte";
  import VideoPlayer from "./lib/VideoPlayer.svelte";
  import TermsGate from "./lib/TermsGate.svelte";
  import NowPlaying from "./lib/NowPlaying.svelte";
  import Icon from "./lib/Icon.svelte";
  import { noteListening } from "./lib/recent.svelte";
  import { ui } from "./lib/ui.svelte";
  import { video, videoActions } from "./lib/video.svelte";
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

  // Side menu, in the order of the design. `tab` is the screen, `arg` picks a sub-view.
  type NavItem = { id: string; label: string; icon: string; tab: string; arg?: string; car?: boolean; online?: boolean };
  const nav: NavItem[] = [
    { id: "home", label: "Home", icon: "home", tab: "Dashboard" },
    { id: "music", label: "Music", icon: "music", tab: "Library", arg: "Music" },
    { id: "radio", label: "Radio", icon: "radio", tab: "Radio", online: true },
    { id: "podcasts", label: "Podcasts", icon: "podcast", tab: "Podcasts", online: true },
    { id: "audiobooks", label: "Audiobooks", icon: "book", tab: "Library", arg: "Audiobook" },
    { id: "videos", label: "Videos", icon: "video", tab: "Videos", online: true },
    { id: "playlists", label: "Playlists", icon: "list", tab: "Playlists" },
    { id: "favorites", label: "Favorites", icon: "star", tab: "Favorites" },
    { id: "tools", label: "Audio Tools", icon: "sliders", tab: "Settings", arg: "Audio & Equalizer" },
    { id: "hub", label: "Cindrix Hub", icon: "share", tab: "Hub" },
    { id: "airlink", label: "AirLink", icon: "share", tab: "Settings", arg: "LAN & Sharing" },
    { id: "car", label: "Car Mode", icon: "car", tab: "Dashboard", car: true },
  ];
  // Grid-Down hides what only works with internet; going back Online shows it again
  const visibleNav = $derived(nav.filter((n) => !(store.s?.grid_down && n.online)));
  let active = $state("Dashboard");
  let activeId = $state("home");
  let libKind = $state<"Music" | "Audiobook">("Music");
  let menu = $state(false);
  function navigate(tab: string, arg?: string, id?: string) {
    if (tab === "Library" && (arg === "Music" || arg === "Audiobook")) libKind = arg;
    if (tab === "Settings" && arg) ui.settingsTab = arg;
    active = tab;
    activeId = id ?? nav.find((n) => n.tab === tab && n.arg === arg)?.id ?? nav.find((n) => n.tab === tab)?.id ?? (tab === "Settings" ? "" : activeId);
    menu = false;
  }
  // Phone back button: close the top screen (video, full player, menu, then go Home) instead of leaving the app.
  const topLayer = $derived(video.open ? "video" : ui.np ? "np" : menu ? "menu" : active !== "Dashboard" ? "tab" : null);
  let armed = false;
  function arm() { try { history.pushState({ cx: 1 }, ""); armed = true; } catch { /* no history */ } }
  $effect(() => { if (topLayer && !armed) arm(); });
  function onBack() {
    armed = false;
    if (video.open) videoActions.close();
    else if (ui.np) { ui.np = false; ui.car = false; }
    else if (menu) menu = false;
    else if (active !== "Dashboard") navigate("Dashboard");
    else return; // already Home with nothing open: the next back press leaves the app
    arm();
  }
  function pickNav(n: NavItem) {
    if (n.car) { ui.car = true; ui.np = true; menu = false; return; }
    navigate(n.tab, n.arg, n.id);
  }
  $effect(() => { if (store.s?.grid_down && ["Radio", "Podcasts", "Videos"].includes(active)) navigate("Library", "Music"); });
  const back = $derived(store.s?.prefs.skip_back ?? 15);
  const heading = $derived(active === "Dashboard" ? "Home" : active === "Hub" ? "Cindrix Hub" : active === "Library" ? (libKind === "Audiobook" ? "Audiobooks" : "Music") : active);
  // keep the position and queue in step with the audio engine
  onMount(() => {
    const id = setInterval(async () => { await actions.init(); noteListening(player.s.current, player.s.position_secs, player.s.playing); }, 500);
    return () => clearInterval(id);
  });
</script>

{#if store.s && store.s.terms_version !== TERMS_VERSION}
  <TermsGate />
{/if}

<svelte:window onpopstate={onBack} />
<VideoPlayer />
<NowPlaying />

<div class="shell">
  <header>
    <button class="burger" aria-label="Menu" aria-expanded={menu} onclick={() => (menu = !menu)}><Icon name="queue" /></button>
    <img src="/logo.png" alt="Cindrix Radio" height="44" />
    <span class="sp"></span>
    <button class="mode" class:grid={store.s?.grid_down} onclick={() => settingsActions.setGridDown(!store.s?.grid_down)}
      title="Switch between Online and Grid-Down mode" aria-label="Connection mode: {store.s?.grid_down ? 'Grid-Down' : 'Online'}. Click to switch.">
      <Icon name={store.s?.grid_down ? "wifioff" : "wifi"} size={20} />{store.s?.grid_down ? "Grid-Down" : "Online"}
    </button>
    <button class="hb" aria-label="Output devices and Bluetooth" onclick={() => navigate("Settings", "Devices & Bluetooth", "")}><Icon name="device" size={26} /></button>
    <button class="hb" aria-label="Settings" onclick={() => navigate("Settings", "General", "")}><Icon name="gear" size={26} /></button>
    <button class="hb" aria-label="Help and about" onclick={() => navigate("Settings", "About", "")}><Icon name="help" size={26} /></button>
  </header>
  {#if menu}<button class="scrim" aria-label="Close menu" onclick={() => (menu = false)}></button>{/if}
  <nav class:open={menu}>
    {#each visibleNav as n}
      <button class="navbtn" class:on={activeId === n.id} onclick={() => pickNav(n)}><Icon name={n.icon} size={24} />{n.label}</button>
    {/each}
  </nav>

  <main>
    {#if active !== "Dashboard"}<h1>{heading}</h1>{/if}
    {#if active === "Settings"}
      <SettingsView />
    {:else if active === "Radio"}
      <Radio go={(t) => navigate(t)} />
    {:else if active === "Playlists"}
      <Playlists />
    {:else if active === "Favorites"}
      <Favorites />
    {:else if active === "Hub"}
      <HubView />
    {:else if active === "Library"}
      {#key libKind}<Library initial={libKind} />{/key}
    {:else if active === "Podcasts"}
      <Podcasts />
    {:else if active === "Videos"}
      <Videos />
    {:else}
      <Dashboard go={navigate} />
    {/if}
  </main>

  <footer>
    <button class="np" aria-label="Open the full player" onclick={() => (ui.np = true)}>
      <span class="thumb"><Icon name={player.s.current?.kind === "Radio" ? "radio" : player.s.current?.kind === "Podcast" ? "podcast" : player.s.current?.kind === "Audiobook" ? "book" : "music"} size={26} /></span>
      <span class="who">
        {#if player.s.current}<b>{player.s.current.title}</b><span>{player.s.now_playing || player.s.current.subtitle}</span>
          {#if seekable}<span class="tm">{mmss(dragging ?? player.s.position_secs)} / {mmss(dur)}</span>{/if}
        {:else}<span>Nothing playing</span>{/if}
      </span>
    </button>
    <div class="tx">
      <button class="skip" aria-label="Back {back} seconds" disabled={!seekable} onclick={() => actions.seek(Math.max(0, player.s.position_secs - back))}><svg width="26" height="26" viewBox="0 0 24 24" fill="currentColor"><path d="M6 5h2v14H6zM20 5v14L9 12z" /></svg></button>
      <button class="ring" aria-label={player.s.playing ? "Pause" : "Play"} onclick={actions.toggle}>
        {#if player.s.playing}<svg width="30" height="30" viewBox="0 0 24 24" fill="currentColor"><path d="M6 4h4v16H6zM14 4h4v16h-4z" /></svg>{:else}<svg width="30" height="30" viewBox="0 0 24 24" fill="currentColor"><path d="M7 4l13 8-13 8z" /></svg>{/if}
      </button>
      <button class="skip" aria-label="Next" onclick={actions.next}><svg width="26" height="26" viewBox="0 0 24 24" fill="currentColor"><path d="M16 5h2v14h-2zM4 5v14l11-7z" /></svg></button>
    </div>
    {#if seekable}
      <input class="seekline" type="range" min="0" max={dur} step="1" value={dragging ?? player.s.position_secs} aria-label="Seek"
        oninput={(e) => (dragging = +e.currentTarget.value)}
        onchange={async (e) => { const v = +e.currentTarget.value; await actions.seek(v); dragging = null; }} />
    {/if}
    <label class="vol"><Icon name="vol" size={22} />
      <input type="range" min="0" max="1" step="0.01" value={player.s.volume} aria-label="Volume" oninput={(e) => actions.volume(+e.currentTarget.value)} />
    </label>
    <div class="tools">
      <button class="hb" aria-label="Equalizer" title="Equalizer" onclick={() => navigate("Settings", "Audio & Equalizer")}><Icon name="sliders" size={24} /></button>
      <button class="hb" aria-label="Car mode" title="Car mode" onclick={() => { ui.car = true; ui.np = true; }}><Icon name="car" size={24} /></button>
      <button class="hb" aria-label="Queue, sleep timer and full player" title="Queue and sleep timer" onclick={() => (ui.np = true)}><Icon name="queue" size={24} /></button>
    </div>
  </footer>
</div>

<style>
  .shell{display:grid;grid-template-columns:232px 1fr;grid-template-rows:68px 1fr 92px;height:100%}
  header{grid-column:1/3;display:flex;align-items:center;gap:10px;padding:0 20px;border-bottom:1px solid var(--line);background:linear-gradient(#0f1823,var(--bg))}
  header img{mix-blend-mode:screen}.sp{flex:1;min-width:0}
  .mode{height:38px;padding:0 16px;border-radius:19px;border:1px solid #2A3747;background:#0A0E13;color:#fff;font-size:14px;display:flex;align-items:center;gap:8px}
  .mode :global(svg){color:var(--green)}
  .mode.grid{border-color:var(--blue);color:var(--lblue);background:#1FA8FF14}.mode.grid :global(svg){color:var(--blue)}
  .hb{width:44px;height:44px;border-radius:12px;border:0;background:transparent;color:var(--silver);display:inline-flex;align-items:center;justify-content:center}
  nav{padding:14px 10px;display:flex;flex-direction:column;gap:4px;border-right:1px solid var(--line);overflow-y:auto}
  .navbtn{height:46px;display:flex;align-items:center;gap:14px;text-align:left;padding:0 14px;border-radius:12px;border:1px solid transparent;background:transparent;color:var(--silver);font-size:15px}
  .navbtn.on{background:linear-gradient(#1b3a55,#12283c);border-color:var(--blue);color:#fff;box-shadow:0 0 14px #1fa8ff33}
  .navbtn.on :global(svg){color:var(--blue)}
  main{padding:20px;overflow:auto;overflow-x:hidden;min-width:0}
  footer{grid-column:1/3;display:flex;align-items:center;gap:22px;padding:0 20px;border-top:1px solid var(--blue);background:linear-gradient(#0f1823,#0a0e13);position:relative}
  .np{display:flex;align-items:center;gap:12px;text-align:left;font-size:14px;min-width:0;width:300px;flex:none;border:0;background:transparent;color:inherit;padding:0;font-family:inherit}
  .thumb{width:56px;height:56px;border-radius:10px;flex:none;display:flex;align-items:center;justify-content:center;background:linear-gradient(135deg,#3b2a66,#d98b5f);color:#fff}
  .who{display:flex;flex-direction:column;min-width:0}.who b,.who span{overflow:hidden;text-overflow:ellipsis;white-space:nowrap}.who span{color:var(--muted);font-size:13px}.who .tm{font-size:12px}
  .tx{display:flex;align-items:center;gap:22px;margin:0 auto}
  .skip{width:44px;height:44px;border:0;background:transparent;color:#fff;display:inline-flex;align-items:center;justify-content:center}.skip:disabled{opacity:.35}
  .ring{width:64px;height:64px;border-radius:32px;border:3px solid var(--blue);background:#0A0E13;color:#fff;display:inline-flex;align-items:center;justify-content:center;box-shadow:0 0 16px #1fa8ff55}
  .seekline{position:absolute;left:0;right:0;top:-9px;width:100%;margin:0;height:18px;opacity:.9}
  .vol{display:flex;gap:8px;align-items:center;color:var(--silver)}.vol input{width:140px}
  .tools{display:flex;gap:2px;align-items:center}
  .burger,.scrim{display:none}
  @media (max-width:800px){
    .shell{grid-template-columns:minmax(0,1fr);grid-template-rows:56px minmax(0,1fr) auto;width:100%;max-width:100vw;overflow:hidden}
    .shell>*{min-width:0}
    header{overflow:hidden}
    header{grid-column:1;padding:0 10px;gap:4px}
    header img{height:34px}
    .burger{display:inline-flex;width:44px;height:44px;border-radius:12px;border:0;background:transparent;color:var(--silver);align-items:center;justify-content:center}
    .mode{padding:0 10px;font-size:12px;height:34px}
    .hb{width:38px}
    nav{position:fixed;z-index:20;top:0;bottom:0;left:0;width:270px;max-width:82vw;background:var(--panel);transform:translateX(-105%);transition:transform .18s;padding-top:24px}
    nav.open{transform:none}
    .scrim{display:block;position:fixed;z-index:19;inset:0;border:0;background:#000A}
    main{padding:12px}
    main h1{font-size:24px}
    footer{grid-column:1;flex-wrap:wrap;gap:6px 10px;padding:10px 12px}
    .np{width:auto;flex:1 1 55%}.thumb{width:44px;height:44px}
    .tx{gap:10px;margin:0}
    .ring{width:52px;height:52px}
    .vol{display:none}
    .tools{margin-left:auto}
    .seekline{top:-9px}
  }
  @media (max-width:430px){
    .mode{padding:0 8px;font-size:11px;gap:4px;height:32px}
    .mode :global(svg){width:16px;height:16px}
    .hb{width:32px;height:38px}
    header{gap:0;padding:0 6px}
    header img{height:28px}
  }
</style>
