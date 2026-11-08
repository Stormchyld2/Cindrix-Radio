<script lang="ts">
  import eula from "../../docs/legal/EULA.md?raw";
  import { settingsActions, TERMS_VERSION } from "./settings.svelte";
  let agreed = $state(false);
</script>

<div class="gate" role="dialog" aria-modal="true" aria-label="Terms of use">
  <div class="card box">
    <h2>Before you start</h2>
    <p class="lead">
      You are responsible for the content you add or play. Cindrix Radio does not condone copyright infringement,
      ships with no lyrics or licensed content, and gives no warranty. Read the full terms below.
      They are always available in Settings → Legal.
    </p>
    <pre class="doc">{eula}</pre>
    <label class="check"><input type="checkbox" bind:checked={agreed} /> I have read and accept the Terms (version {TERMS_VERSION}).</label>
    <button class="btn primary" disabled={!agreed} onclick={settingsActions.accept}>Accept and continue</button>
  </div>
</div>

<style>
  .gate{position:fixed;inset:0;background:#000000CC;display:flex;align-items:center;justify-content:center;z-index:10}
  .box{width:min(760px,92vw);display:flex;flex-direction:column;gap:12px;max-height:90vh}
  .lead{margin:0;font-size:13px;color:var(--silver)}
  .doc{white-space:pre-wrap;font:12px/1.5 Inter,system-ui,sans-serif;background:#0D1620;border:1px solid var(--line);border-radius:10px;padding:12px;overflow-y:auto;margin:0;flex:1;min-height:160px}
  .check{font-size:13px;display:flex;gap:8px;align-items:center}
  button:disabled{opacity:.4;cursor:not-allowed}
</style>
