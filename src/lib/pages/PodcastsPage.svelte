<script lang="ts">
  import { store, settingsActions } from "../settings.svelte";
</script>
<div class="card">
  <h3>Podcasts</h3>
  <label class="row"><input type="checkbox" checked={store.s?.prefs.auto_refresh_podcasts ?? true} onchange={(e) => settingsActions.setPrefs({ auto_refresh_podcasts: e.currentTarget.checked })} />
    Check for new episodes when the app opens (Online mode only)</label>
  <p class="note">Episodes you download are saved in Downloads/Cindrix/Podcasts and work with no internet. You can also add a podcast's address or import an OPML file on the Podcasts tab.</p>
</div>
<style>
  .row{display:flex;gap:8px;align-items:center;font-size:14px;margin:6px 0 14px}
  .note{font-size:12px;color:var(--muted);line-height:1.6}
</style>
