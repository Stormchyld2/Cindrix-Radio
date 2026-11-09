<script lang="ts">
  import Equalizer from "./Equalizer.svelte";
  import Legal from "./Legal.svelte";
  import Sources from "./Sources.svelte";
  import Appearance from "./pages/Appearance.svelte";
  import PlaybackPage from "./pages/PlaybackPage.svelte";
  import Devices from "./pages/Devices.svelte";
  import PodcastsPage from "./pages/PodcastsPage.svelte";
  import RadioPage from "./pages/RadioPage.svelte";
  import AudiobooksPage from "./pages/AudiobooksPage.svelte";
  import LanPage from "./pages/LanPage.svelte";
  import StoragePage from "./pages/StoragePage.svelte";
  import PrivacyPage from "./pages/PrivacyPage.svelte";
  import BackupPage from "./pages/BackupPage.svelte";
  import AdvancedPage from "./pages/AdvancedPage.svelte";
  import About from "./pages/About.svelte";
  import { store, settingsActions } from "./settings.svelte";
  import { ui } from "./ui.svelte";

  // Subtabs are added here as each is built; the full list is in docs/010.
  const tabs = ["General", "Appearance", "Playback", "Audio & Equalizer", "Devices & Bluetooth", "Library & Sources", "Podcasts", "Internet Radio", "Audiobooks",
    "LAN & Sharing", "Downloads & Storage", "Privacy & Security", "Backup & Restore", "Advanced & Diagnostics", "About", "Legal"] as const;
  let tab = $state<(typeof tabs)[number]>("General");
  // Home shortcuts can open a particular page
  $effect(() => { const t = ui.settingsTab; if (t && (tabs as readonly string[]).includes(t)) { tab = t as (typeof tabs)[number]; ui.settingsTab = null; } });
</script>

<div class="wrap">
  <div class="sub">
    {#each tabs as t}
      <button class="subbtn" class:on={tab === t} onclick={() => (tab = t)}>{t}</button>
    {/each}
  </div>
  <div class="pane">
    {#if tab === "General"}
      <div class="card">
        <h3>Connection mode</h3>
        <div class="seg big" role="group" aria-label="Connection mode">
          <button class:on={!store.s?.grid_down} onclick={() => settingsActions.setGridDown(false)}>Online</button>
          <button class:on={store.s?.grid_down} onclick={() => settingsActions.setGridDown(true)}>Grid-Down</button>
        </div>
        <p class="note">
          <b>Online:</b> radio streams, podcast feeds and downloads are available.<br />
          <b>Grid-Down:</b> the app uses no network at all. It hides anything that needs the internet and shows only
          what is on your drives and in your downloads. It also saves power. Switching never stops what is playing.
        </p>
      </div>
    {:else if tab === "Appearance"}
      <Appearance />
    {:else if tab === "Playback"}
      <PlaybackPage />
    {:else if tab === "Devices & Bluetooth"}
      <Devices />
    {:else if tab === "Podcasts"}
      <PodcastsPage />
    {:else if tab === "Internet Radio"}
      <RadioPage />
    {:else if tab === "Audiobooks"}
      <AudiobooksPage />
    {:else if tab === "LAN & Sharing"}
      <LanPage />
    {:else if tab === "Downloads & Storage"}
      <StoragePage />
    {:else if tab === "Privacy & Security"}
      <PrivacyPage />
    {:else if tab === "Backup & Restore"}
      <BackupPage />
    {:else if tab === "Advanced & Diagnostics"}
      <AdvancedPage />
    {:else if tab === "About"}
      <About />
    {:else if tab === "Audio & Equalizer"}
      <Equalizer />
    {:else if tab === "Library & Sources"}
      <Sources />
    {:else}
      <Legal />
    {/if}
  </div>
</div>

<style>
  .wrap{display:flex;gap:20px;margin-top:16px}
  .sub{width:200px;display:flex;flex-direction:column;gap:6px}
  .subbtn{height:38px;text-align:left;padding:0 12px;border-radius:10px;border:0;background:transparent;color:var(--silver);font-size:13px}
  .subbtn.on{background:#1FA8FF18;color:var(--lblue)}
  .pane{flex:1;min-width:0}
  .row{display:flex;gap:8px;align-items:center;font-size:14px;margin:10px 0}
  .note{font-size:12px;color:var(--muted);margin:10px 0 0;line-height:1.6}
  .seg{display:flex;border:1px solid #2A3747;border-radius:12px;overflow:hidden;margin-top:10px;width:320px}
  .seg button{flex:1;height:44px;border:0;background:transparent;color:var(--silver);font-size:15px}
  .seg button.on{background:#1FA8FF18;color:var(--lblue);font-weight:600}
  @media (max-width:800px){
    .wrap{flex-direction:column;gap:10px}
    .sub{width:auto;flex-direction:row;overflow-x:auto;gap:4px}
    .subbtn{flex:none;white-space:nowrap}
    .seg{width:100%}
  }
</style>
