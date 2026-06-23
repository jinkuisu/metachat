<script lang="ts">
  import { ui } from "$lib/stores/uiStore";

  let rules = $state([
    { id: "r1", name: "新客户问候", trigger: "new_contact", keywords: [], template: "您好，欢迎咨询！请问有什么可以帮您的？", active: true },
    { id: "r2", name: "价格咨询", trigger: "keyword", keywords: ["价格", "多少钱", "报价"], template: "感谢您的咨询！我们的报价是...", active: true },
    { id: "r3", name: "发送资料", trigger: "keyword", keywords: ["资料", "介绍", "详情"], template: "已为您发送资料，请查收。", active: true },
    { id: "r4", name: "下班回复", trigger: "time_range", keywords: [], template: "您好，现在是非工作时间，我会在上班后第一时间回复您。", active: false },
  ]);

  let selectedGroup = $state("全部");
  let groups = ["全部", "产品介绍", "常用话术", "节日问候"];
</script>

<div class="glass-heavy radius-panel shadow-theme-glass-lg animate-fade-in"
     style="width: 640px; max-height: 80vh; display: flex; flex-direction: column;">
  <div class="flex items-center justify-between px-6 py-4" style="border-bottom: 1px solid var(--glass-border);">
    <div class="flex items-center gap-2">
      <span class="text-lg">⚡</span>
      <span class="font-semibold text-theme-primary">自动回复</span>
    </div>
    <button onclick={() => ui.setConfigView(null)}
            class="size-7 flex items-center justify-center radius-glass cursor-pointer text-theme-muted hover:bg-black/5"
            style="border: none; background: transparent;">✕</button>
  </div>

  <div class="flex-1 overflow-y-auto p-6">
    <!-- 分组筛选 -->
    <div class="flex gap-2 mb-4 flex-wrap">
      {#each groups as g}
        <button onclick={() => selectedGroup = g}
                class="px-3 py-1.5 radius-glass text-xs cursor-pointer"
                style="border: 1px solid var(--glass-border); background: {selectedGroup === g ? 'var(--primary)' : 'transparent'};
                       color: {selectedGroup === g ? 'white' : 'var(--text-primary)'}; font-family: inherit;">
          {g}
        </button>
      {/each}
      <button class="px-3 py-1.5 radius-glass text-xs cursor-pointer"
              style="border: 1px dashed var(--glass-border); background: transparent; color: var(--text-primary); font-family: inherit;">
        + 新建分组
      </button>
    </div>

    <!-- 规则列表 -->
    <div class="flex flex-col gap-2">
      {#each rules as rule}
        <div class="glass-light radius-card p-4" style="border: 1px solid var(--glass-border);">
          <div class="flex items-center justify-between mb-2">
            <div class="flex items-center gap-2">
              <span class="font-medium text-sm text-theme-primary">{rule.name}</span>
              <span class="text-[10px] px-2 py-0.5 rounded-full bg-theme-glass text-theme-muted"
                    style="border: 1px solid var(--glass-border);">
                {rule.trigger === "new_contact" ? "新联系人" : rule.trigger === "time_range" ? "定时" : "关键词"}
              </span>
            </div>
            <button onclick={() => rule.active = !rule.active}
                    class="w-9 h-5 rounded-full transition-colors"
                    style="background: {rule.active ? 'var(--primary)' : 'var(--glass-border)'}; border: none; cursor: pointer; position: relative;">
              <span class="absolute w-3.5 h-3.5 rounded-full bg-white top-0.5 transition-all"
                    style="left: {rule.active ? '4px' : '19px'};"></span>
            </button>
          </div>
          {#if rule.keywords.length > 0}
            <div class="flex gap-1 mb-2">
              {#each rule.keywords as kw}
                <span class="text-[10px] px-2 py-0.5 radius-glass" style="background: rgba(13,148,136,0.1); color: var(--primary);">{kw}</span>
              {/each}
            </div>
          {/if}
          <div class="text-xs text-theme-muted truncate">{rule.template}</div>
        </div>
      {/each}
    </div>
  </div>
</div>
