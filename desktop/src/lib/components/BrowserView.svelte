<script lang="ts">
  import { toolState } from "../stores/toolStore";
  import NotifySystem from "$lib/components/NotifySystem.svelte";
  import { onMount } from "svelte";
  import { ui } from "$lib/stores/uiStore";

  let showContent = $state(false);

  onMount(() => {
    requestAnimationFrame(() => { showContent = true; });
  });

  function handleBack() {
    showContent = false;
    setTimeout(() => {
      ui.setViewMode("canvas");
      ui.selectPlatform(null);
    }, 250);
  }
</script>

<div class="browser-container animate-zoom-in"
     style="position: fixed; inset: 0; z-index: 20; display: flex; flex-direction: column;">

  <div class="browser-toolbar" style="height: 0; position: relative; z-index: 30; pointer-events: none;">
    <button onclick={handleBack}
            class="glass-light radius-glass shadow-theme-glass-sm back-btn" title="返回">←</button>

    <div class="glass-light radius-pill shadow-theme-glass-sm flex items-center"
         style="position: fixed; top: 16px; right: 16px; padding: 4px; pointer-events: auto;">
      {#each [{id:"ai"},{id:"reply"},{id:"translate"}] as tool}
        <button
          onclick={() => toolState.toggleTool(tool.id)}
          class="size-8 flex items-center justify-center cursor-pointer radius-glass transition-all duration-150"
          class:active={toolState.activeTool === tool.id}
          style="border: none; background: transparent;
                 {toolState.activeTool === tool.id ? 'background: color-mix(in srgb, var(--primary) 12%, transparent);' : ''}">
          <span class="text-base">{tool.id === "ai" ? "🤖" : tool.id === "reply" ? "⚡" : "🌐"}</span>
        </button>
      {/each}
    </div>
  </div>

  <div id="cloak-viewport" class="flex-1" style="border: none;"></div>
  <NotifySystem />
</div>

<style>
  .back-btn {
    position: fixed; top: 16px; left: 16px; width: 36px; height: 36px;
    display: flex; align-items: center; justify-content: center;
    border: none; cursor: pointer; font-size: 18px; z-index: 35;
    pointer-events: auto; opacity: 0.3; transition: opacity 0.2s;
  }
  .back-btn:hover { opacity: 0.9; }
  .active { background: color-mix(in srgb, var(--primary) 12%, transparent); }
</style>
