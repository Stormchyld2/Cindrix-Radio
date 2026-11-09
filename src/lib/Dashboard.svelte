<script lang="ts">
  import { onMount } from "svelte";
  import { api, type Counts, type SourceStatus, type Track } from "./api";
  import { player } from "./player.svelte";
  import { store, settingsActions } from "./settings.svelte";
  import { startTrack } from "./playlistStore.svelte";
  import { recent } from "./recent.svelte";
  import Icon from "./Icon.svelte";

  let { go }: { go: (tab: string, arg?: string) => void } = $props();

  let counts = $state<Counts>({ music: 0, podcast: 0, audiobook: 0, video: 0 });
  let sources = $state<SourceStatus[]>([]);
  let stations = $state(0);
  let message = $state<string | null>(null);

  onMount(async () => {
    const [c, s, st] = await Promise.all([api.libCounts(), api.libSources(), api.radioStations()]);
    counts = c; sources = s; stations = st.length;
  });

  const downloads = $derived(sources.find((x) => x.source.id === "downloads")?.items ?? 0);
  const err = $derived(message ?? player.s.error ?? null);
  const kindIcon = (t: Track) => (t.kind === "Radio" ? "radio" : t.kind === "Podcast" ? "podcast" : t.kind === "Audiobook" ? "book" : t.kind === "Video" ? "video" : "music");
  const kindLabel = (t: Track) => (t.kind === "Music" ? "Song" : t.kind === "Radio" ? "Radio Station" : t.kind);
  const pct = (pos: number, dur: number | null) => (dur ? Math.min(100, Math.round((pos / dur) * 100)) : 0);
  async function resume(t: Track) {
    message = null;
    try { await startTrack(t); } catch (e) { message = String(e instanceof Error ? e.message : e); }
  }
</script>

<section class="hero" aria-label="Cindrix Radio">
  <svg class="mt" viewBox="0 0 800 220" preserveAspectRatio="xMidYMax slice" aria-hidden="true">
    <defs>
      <linearGradient id="sky" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#1b2a44" /><stop offset=".55" stop-color="#6d5a7a" /><stop offset="1" stop-color="#d98b5f" /></linearGradient>
      <linearGradient id="fog" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#9aa9c4" stop-opacity=".0" /><stop offset="1" stop-color="#9aa9c4" stop-opacity=".45" /></linearGradient>
    </defs>
    <rect width="800" height="220" fill="url(#sky)" />
    <path d="M0 150 L70 95 L120 125 L190 70 L260 130 L330 85 L400 135 L470 75 L540 130 L620 90 L700 135 L800 100 V220 H0Z" fill="#2c3550" />
    <path d="M0 220 V170 L60 140 L130 170 L210 135 L300 175 L380 145 L470 180 L560 140 L650 175 L730 150 L800 170 V220Z" fill="#141c2c" />
    <rect y="120" width="800" height="100" fill="url(#fog)" />
    <path d="M0 220 V190 L25 160 L50 190 L80 150 L110 192 L140 165 L170 195 V220Z M630 220 V185 L660 150 L690 188 L720 155 L750 190 L780 160 L800 185 V220Z" fill="#0a1019" />
  </svg>
  <div class="over">
    <img src="/logo.png" alt="Cindrix Radio" />
    <p>MUSIC <i>•</i> RADIO <i>•</i> PODCASTS <i>•</i> AUDIOBOOKS <i>•</i> MORE</p>
  </div>
</section>

<div class="stats">
  <button class="stat" onclick={() => go("Library", "Music")}><span class="ic"><Icon name="db" size={34} /></span><div><b>Library</b><span>{counts.music.toLocaleString()} Tracks</span></div></button>
  <button class="stat" onclick={() => go("Radio")}><span class="ic"><Icon name="radio" size={34} /></span><div><b>Radio</b><span>{stations.toLocaleString()} Stations</span></div></button>
  <button class="stat" onclick={() => go("Library", "Music")}><span class="ic"><Icon name="download" size={34} /></span><div><b>Downloads</b><span>{downloads.toLocaleString()} Items</span></div></button>
</div>

<section class="panel">
  <h3><span class="ph"><Icon name="check" size={16} /></span>Continue Listening</h3>
  {#if recent.items.length}
    <div class="cards">
      {#each recent.items.slice(0, 4) as r (r.track.id)}
        <button class="card2" onclick={() => resume(r.track)}>
          <span class="art k-{r.track.kind}"><Icon name={kindIcon(r.track)} size={54} /></span>
          <span class="bar"><i style="width:{pct(r.pos, r.track.duration_secs)}%"></i></span>
          <b>{r.track.title}</b>
          <span class="sub">{r.track.kind === "Music" ? kindLabel(r.track) : kindLabel(r.track)}{r.track.subtitle ? ` • ${r.track.subtitle}` : ""}</span>
        </button>
      {/each}
    </div>
  {:else}
    <p class="note">Nothing yet. Play a song, station, podcast or audiobook and it shows up here to pick up later.</p>
  {/if}
  {#if err}<p class="err">{err}</p>{/if}
</section>

<section class="panel">
  <h3><span class="ph"><Icon name="sliders" size={16} /></span>Quick Tools</h3>
  <div class="tools">
    <button onclick={() => go("Settings", "Audio & Equalizer")}><Icon name="sliders" size={34} />Equalizer</button>
    <button onclick={() => go("Karaoke")}><Icon name="karaoke" size={34} />Karaoke</button>
    <button onclick={() => go("Radio")}><span class="redot"><Icon name="rec" size={34} /></span>Record</button>
    <button onclick={() => go("Settings", "LAN & Sharing")}><Icon name="share" size={34} />AirLink</button>
  </div>
</section>

<section class="panel">
  <h3><span class="ph"><Icon name="wifi" size={16} /></span>Connectivity</h3>
  <div class="modes" role="group" aria-label="Connection mode">
    <button class:on={!store.s?.grid_down} onclick={() => settingsActions.setGridDown(false)}><Icon name="wifi" size={26} />Online{#if !store.s?.grid_down}<span class="ck"><Icon name="check" size={20} /></span>{/if}</button>
    <button class:on={store.s?.grid_down} onclick={() => settingsActions.setGridDown(true)}><Icon name="wifioff" size={26} />Grid-Down{#if store.s?.grid_down}<span class="ck"><Icon name="check" size={20} /></span>{/if}</button>
  </div>
  <p class="note">{store.s?.grid_down ? "Local and downloaded content only. No network use." : "Streams, feeds and downloads available."}</p>
</section>

<style>
  .hero{position:relative;height:clamp(190px,34vw,300px);border-radius:18px;overflow:hidden;border:1px solid var(--line)}
  .mt{position:absolute;inset:0;width:100%;height:100%}
  .over{position:absolute;inset:0;display:flex;flex-direction:column;align-items:center;justify-content:center;gap:8px;background:radial-gradient(ellipse at center,#0006,#0000 70%)}
  .over img{height:min(58%,170px);mix-blend-mode:screen;filter:drop-shadow(0 4px 16px #000a)}
  .over p{margin:0;font-size:clamp(9px,1.5vw,13px);letter-spacing:.28em;color:#e6ebf3;text-shadow:0 1px 6px #000}.over i{font-style:normal;margin:0 .4em;color:#9fb0c7}
  .stats{display:grid;grid-template-columns:repeat(3,1fr);gap:12px;margin:14px 0}
  .stat{display:flex;align-items:center;gap:14px;padding:16px;border-radius:16px;border:1px solid var(--line);background:var(--panel);color:var(--silver);text-align:left}
  .stat .ic{color:var(--blue);flex:none}.stat div{display:flex;flex-direction:column;min-width:0}.stat b{font-size:17px;color:#F1F4F8}.stat div span{font-size:14px;color:var(--silver)}
  .panel{border:1px solid var(--line);border-radius:18px;background:var(--panel);padding:14px 16px;margin-bottom:12px}
  .panel h3{display:flex;align-items:center;gap:10px;font-size:20px;margin-bottom:12px}
  .ph{color:var(--blue);display:inline-flex;width:26px;height:26px;border:2px solid var(--blue);border-radius:13px;align-items:center;justify-content:center}
  .cards{display:grid;grid-template-columns:repeat(auto-fill,minmax(150px,1fr));gap:12px}
  .card2{display:flex;flex-direction:column;gap:4px;padding:0 0 10px;border-radius:14px;border:1px solid var(--line);background:#0D1620;color:var(--silver);text-align:left;overflow:hidden}
  .card2 b{padding:0 10px;font-size:15px;color:#F1F4F8;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}.sub{padding:0 10px;font-size:12px;color:var(--muted);overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
  .art{height:96px;display:flex;align-items:center;justify-content:center;color:#fff}
  .k-Music{background:linear-gradient(135deg,#0d2a5c,#1f6fd0)}.k-Podcast{background:linear-gradient(135deg,#2b1b66,#7a4fe0)}
  .k-Audiobook{background:linear-gradient(135deg,#4a3410,#c8902a)}.k-Radio{background:linear-gradient(135deg,#1b2c44,#d98b5f)}.k-Video{background:linear-gradient(135deg,#1d2b2b,#3d8f86)}
  .bar{display:block;height:4px;background:#243246}.bar i{display:block;height:100%;background:var(--blue)}
  .tools{display:grid;grid-template-columns:repeat(auto-fit,minmax(110px,1fr));gap:10px}
  .tools button{display:flex;flex-direction:column;align-items:center;gap:8px;padding:14px 6px;border-radius:14px;border:1px solid var(--line);background:linear-gradient(#17212d,#101822);color:var(--silver);font-size:15px}
  .redot{color:var(--red);display:inline-flex}
  .modes{display:grid;grid-template-columns:repeat(2,1fr);gap:10px}
  .modes button{position:relative;display:flex;align-items:center;justify-content:center;gap:10px;height:56px;border-radius:14px;border:1px solid #2A3747;background:#0D1620;color:var(--silver);font-size:16px}
  .modes button.on{border-color:var(--blue);color:#fff;background:#1FA8FF14}.ck{color:var(--blue);display:inline-flex}
  .note{font-size:13px;color:var(--muted);margin:10px 0 0}.err{color:var(--red);font-size:13px}
  @media (max-width:800px){
    .stats{grid-template-columns:1fr}
    .stat{padding:12px}
  }
</style>
