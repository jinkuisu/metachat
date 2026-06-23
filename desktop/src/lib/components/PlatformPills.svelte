<script lang="ts">
  import { ui } from "../stores/uiStore";

  function togglePlatform(id: string) {
    ui.selectPlatform(id === ui.selectedPlatform ? null : id);
  }
</script>

<div class="fixed" style="bottom: 32px; left: 50%; transform: translateX(-50%); z-index: 50;">
  <div class="glass-light radius-pill shadow-theme-glass-sm flex items-center gap-1"
       style="padding: 6px 8px;">
    {#each ui.platforms as platform, i}
      <button onclick={() => togglePlatform(platform.id)}
              class="pill-btn"
              class:selected={ui.selectedPlatform === platform.id}
              style="--pcolor: {platform.color};"
              title={platform.name}>
        <span class="platform-icon" style="color: {platform.color};">{platform.icon}</span>
        <span class="text-sm font-medium text-theme-primary" style="white-space: nowrap;">
          {platform.name}
        </span>
        {#if platform.accounts.some(a => a.unread > 0)}
          <span class="w-1.5 h-1.5 rounded-full bg-red-500 flex-shrink-0"></span>
        {/if}
      </button>
      {#if i < ui.platforms.length - 1}
        <div class="w-px h-4" style="background: var(--glass-border);"></div>
      {/if}
    {/each}
    <div class="w-px h-5 mx-2" style="background: var(--glass-border);"></div>

    <button onclick={() => ui.setConfigView("settings")}
            class="pill-btn" class:selected={ui.configView === "settings"}
            style="--pcolor: #78716C;" title="设置">
      <span class="text-sm">⚙️</span>
      <span class="text-sm font-medium text-theme-primary" style="white-space: nowrap;">设置</span>
    </button>
    <button onclick={() => ui.setConfigView("reply")}
            class="pill-btn" class:selected={ui.configView === "reply"}
            style="--pcolor: #78716C;" title="自动回复">
      <span class="text-sm">⚡</span>
      <span class="text-sm font-medium text-theme-primary" style="white-space: nowrap;">回复</span>
    </button>
    <button onclick={() => ui.setConfigView("prompts")}
            class="pill-btn" class:selected={ui.configView === "prompts"}
            style="--pcolor: #78716C;" title="AI 提示词">
      <span class="text-sm">🤖</span>
      <span class="text-sm font-medium text-theme-primary" style="white-space: nowrap;">提示词</span>
    </button>

    <div class="w-px h-4 mx-1" style="background: var(--glass-border);"></div>
    <button class="pill-btn add" title="添加账号">
      <span class="platform-icon" style="color: var(--text-muted-light); font-weight: 300; font-size: 16px;">+</span>
    </button>
  </div>
</div>

<style>
  .pill-btn {
    display: flex; align-items: center; gap: 6px;
    padding: 6px 12px; border-radius: 20px;
    border: 1px solid transparent;
    cursor: pointer; transition: all 0.2s ease;
    background: transparent;
    font-family: inherit; font-size: 13px;
  }
  .pill-btn:hover {
    background: rgba(255,255,255,0.5);
    transform: translateY(-1px);
  }
  .pill-btn.selected {
    background: color-mix(in srgb, var(--pcolor, #0D9488) 12%, transparent);
    border-color: color-mix(in srgb, var(--pcolor, #0D9488) 25%, transparent);
  }
  .pill-btn.add:hover { background: rgba(0,0,0,0.04); }
  .platform-icon {
    font-size: 12px; font-weight: 700;
    width: 22px; height: 22px;
    display: flex; align-items: center; justify-content: center;
    border-radius: 6px;
    background: rgba(0,0,0,0.04);
  }
</style>
