<script lang="ts">
  import Equalizer from "./Equalizer.svelte";
  import Legal from "./Legal.svelte";
  import Sources from "./Sources.svelte";
  import { store, settingsActions } from "./settings.svelte";

  // Subtabs are added here as each is built; the full list is in docs/010.
  const tabs = ["General", "Audio & Equalizer", "Library & Sources", "Karaoke & Lyrics", "Legal"] as const;
  let tab = $state<(typeof tabs)[number]>("General");
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
    {:else if tab === "Audio & Equalizer"}
      <Equalizer />
    {:else if tab === "Library & Sources"}
      <Sources />
    {:else if tab === "Karaoke & Lyrics"}
      <div class="card">
        <h3>Lyrics</h3>
        <label class="row">
          <input type="checkbox" checked={store.s?.lyrics_viewer ?? false} onchange={(e) => settingsActions.setLyrics(e.currentTarget.checked)} />
          Show lyrics viewer (My lyrics, added by you)
        </label>
        <p class="note">
          Off by default. Cindrix Radio ships with no lyrics and looks none up online. When on, you can paste lyrics
          for a song; they are stored only on this device. You are responsible for any text you add.
        </p>
      </div>
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
</style>
