<script lang="ts">
  import eula from "../../docs/legal/EULA.md?raw";
  import tos from "../../docs/legal/TERMS-OF-SERVICE.md?raw";
  import privacy from "../../docs/legal/PRIVACY-POLICY.md?raw";
  import { store, TERMS_VERSION } from "./settings.svelte";

  const docs = [
    { id: "eula", label: "End User License Agreement", text: eula },
    { id: "tos", label: "Terms of Service", text: tos },
    { id: "privacy", label: "Privacy Policy", text: privacy },
  ];
  let open = $state("eula");
  const current = $derived(docs.find((d) => d.id === open)!);
</script>

<div class="legal">
  <div class="plain">
    <b>In plain words</b>
    <ul>
      <li>Cindrix Radio is a media player. It comes with no music, videos, podcasts or lyrics of its own.</li>
      <li>What you add, import, paste, record, stream or download is your responsibility. Use it only for your own personal listening and viewing, and only where you have the right to.</li>
      <li>Downloads are for offline use on your own devices, like the offline feature in other apps. Don't share or sell them.</li>
      <li>Lyrics you paste stay on your device. They are never uploaded or shared.</li>
      <li>There are no accounts, ads or tracking. Your library, lyrics and listening history stay on your device. The app only contacts the services you use, such as the radio directory, podcast feeds and the sites you download from.</li>
      <li>The app is provided as is, with no warranty.</li>
    </ul>
    <span>This summary is only a guide. The full documents below are what apply.</span>
  </div>
  <div class="tabs">
    {#each docs as d}
      <button class="btn" class:on={open === d.id} onclick={() => (open = d.id)}>{d.label}</button>
    {/each}
  </div>
  <p class="meta">
    Version {TERMS_VERSION}.
    {#if store.s?.terms_version}Accepted version {store.s.terms_version}.{:else}Not yet accepted.{/if}
  </p>
  <pre class="doc">{current.text}</pre>
</div>

<style>
  .plain{border:1px solid var(--blue);background:#1FA8FF0F;border-radius:12px;padding:14px 16px;display:flex;flex-direction:column;gap:6px;font-size:13px;line-height:1.5}
  .plain ul{margin:0;padding-left:20px;display:flex;flex-direction:column;gap:4px}
  .plain span{color:var(--muted);font-size:12px}
  .tabs{display:flex;gap:8px;flex-wrap:wrap}
  .btn.on{border-color:var(--blue);color:var(--lblue);background:#1FA8FF14}
  .meta{font-size:12px;color:var(--muted)}
  .doc{white-space:pre-wrap;font:13px/1.55 Inter,system-ui,sans-serif;background:var(--panel);border:1px solid var(--line);border-radius:12px;padding:16px;height:460px;overflow-y:auto;margin:0}
</style>
