<script lang="ts">
  import { ui } from "$lib/stores/uiStore";

  let roles = $state([
    { id: "a1", name: "外贸助手", category: "销售", desc: "专业的外贸销售助理，熟悉国际贸易流程", prompt: "你是一位经验丰富的外贸销售专家...", default: true },
    { id: "a2", name: "客服专员", category: "客服", desc: "耐心细致的客户服务代表", prompt: "你是一位专业的客服人员...", default: false },
    { id: "a3", name: "翻译员", category: "翻译", desc: "精通多国语言的专业翻译", prompt: "请将以下内容翻译为目标语言...", default: false },
    { id: "a4", name: "营销策划", category: "营销", desc: "创意营销策划专家", prompt: "你是一位营销策划专家...", default: false },
  ]);
</script>

<div class="glass-heavy radius-panel shadow-theme-glass-lg animate-fade-in"
     style="width: 640px; max-height: 80vh; display: flex; flex-direction: column;">
  <div class="flex items-center justify-between px-6 py-4" style="border-bottom: 1px solid var(--glass-border);">
    <div class="flex items-center gap-2">
      <span class="text-lg">🤖</span>
      <span class="font-semibold text-theme-primary">AI 提示词</span>
    </div>
    <button onclick={() => ui.setConfigView(null)}
            class="size-7 flex items-center justify-center radius-glass cursor-pointer text-theme-muted hover:bg-black/5"
            style="border: none; background: transparent;">✕</button>
  </div>

  <div class="flex-1 overflow-y-auto p-6">
    <div class="grid grid-cols-2 gap-3">
      {#each roles as role}
        <div class="glass-light radius-card p-4" style="border: 1px solid var(--glass-border);">
          <div class="flex items-center justify-between mb-2">
            <div class="flex items-center gap-2">
              <span class="font-medium text-sm text-theme-primary">{role.name}</span>
              {#if role.default}
                <span class="text-[10px] px-1.5 py-0.5 rounded-full" style="background: rgba(13,148,136,0.1); color: var(--primary);">默认</span>
              {/if}
            </div>
            <span class="text-[10px] px-2 py-0.5 rounded-full bg-theme-glass text-theme-muted" style="border: 1px solid var(--glass-border);">{role.category}</span>
          </div>
          <div class="text-xs text-theme-muted mb-3">{role.desc}</div>
          <div class="text-[10px] text-theme-muted-light truncate leading-relaxed">{role.prompt}</div>
        </div>
      {/each}
    </div>
  </div>
</div>
