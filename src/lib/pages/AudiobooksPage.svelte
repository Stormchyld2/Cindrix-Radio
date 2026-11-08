<script lang="ts">
  import { store, settingsActions } from "../settings.svelte";
  const p = $derived(store.s?.prefs);
</script>
{#if p}
<div class="card">
  <h3>Audiobooks</h3>
  <p class="note">Audiobooks resume where you stopped. The jump buttons use the times you set under Playback.</p>
  <div class="row">Jump back {p.skip_back} s · Jump forward {p.skip_fwd} s</div>
  <p class="note">Put books in Downloads/Cindrix/Audiobooks, or add a folder under Library &amp; Sources and tick Audiobooks. Edit a book's title, date and description with the ✎ button in the Library.</p>
</div>
{/if}
<style>
  .row{font-size:14px;margin:12px 0}
  .note{font-size:12px;color:var(--muted);line-height:1.6}
</style>
