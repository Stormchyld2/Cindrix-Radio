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
  .tabs{display:flex;gap:8px;flex-wrap:wrap}
  .btn.on{border-color:var(--blue);color:var(--lblue);background:#1FA8FF14}
  .meta{font-size:12px;color:var(--muted)}
  .doc{white-space:pre-wrap;font:13px/1.55 Inter,system-ui,sans-serif;background:var(--panel);border:1px solid var(--line);border-radius:12px;padding:16px;height:460px;overflow-y:auto;margin:0}
</style>
