<script lang="ts">
  import { store, settingsActions } from "../settings.svelte";
  import { api } from "../api";
  import { isAndroid } from "../platform";
  let msg = $state<string | null>(null);
  async function openBt() { msg = null; try { await api.openBluetooth(); } catch (e) { msg = String(e instanceof Error ? e.message : e); } }
</script>
<div class="card">
  <h3>Devices &amp; Bluetooth</h3>
  <p class="note">
    {#if isAndroid}Sound goes to whatever your phone is connected to: speaker, headphones, a Bluetooth speaker or your car. Pair and choose devices in your phone's Bluetooth settings. Lock-screen, headset and car buttons (play, pause, next, previous) control Cindrix Radio, and the song title shows on your car display.
    {:else}Sound goes to your Windows default output. Pair Bluetooth speakers and headphones in Windows Settings, then choose them as the output device there.{/if}
  </p>
  <label class="row"><input type="checkbox" checked={store.s?.pause_on_disconnect ?? true} onchange={(e) => settingsActions.setPauseOnDisconnect(e.currentTarget.checked)} />
    Pause when headphones or a Bluetooth device disconnect</label>
  {#if isAndroid}<button class="btn primary" onclick={openBt}>Open Bluetooth settings</button>{/if}
  {#if msg}<p class="note">{msg}</p>{/if}
</div>
<style>
  .note{font-size:13px;color:var(--muted);line-height:1.6}
  .row{display:flex;gap:8px;align-items:center;font-size:14px;margin:14px 0}
</style>
