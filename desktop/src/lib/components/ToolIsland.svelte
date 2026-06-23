<script lang="ts">
  import { toolState } from "../stores/toolStore";
  import { i18n } from "../i18n";

  const tools: { id: "ai" | "reply" | "translate"; icon: string }[] = [
    { id: "ai", icon: "🤖" },
    { id: "reply", icon: "⚡" },
    { id: "translate", icon: "🌐" },
  ];
</script>

<div class="fixed" style="top: 16px; right: 16px; z-index: 100;">
  <div class="glass-light radius-pill shadow-theme-glass-sm flex items-center" style="padding: 4px;">
    {#each tools as t}
      <button
        onclick={() => toolState.toggleTool(t.id)}
        class="size-8 flex items-center justify-center cursor-pointer transition-all duration-150"
        class:active={toolState.activeTool === t.id}
        style="border: none; background: transparent; border-radius: 14px;
               {toolState.activeTool === t.id ? 'background: color-mix(in srgb, var(--primary) 12%, transparent);' : ''}"
        title={i18n.t("tool." + t.id)}
      >
        <span class="text-base">{t.icon}</span>
      </button>
    {/each}
  </div>
</div>
<style>
  .active { background: color-mix(in srgb, var(--primary) 12%, transparent); }
</style>
