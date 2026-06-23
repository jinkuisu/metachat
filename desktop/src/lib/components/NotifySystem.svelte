<script lang="ts">
  import { ui } from "$lib/stores/uiStore";

  function switchAccount(accountId: string) {
    ui.setActiveAccount(accountId);
    ui.setViewMode("browser");
  }
</script>

<!-- Ember Beads — 左侧边缘光点 -->
{#if ui.notifyMode === "beads" && ui.viewMode === "browser"}
  <div class="fixed top-1/2 -translate-y-1/2 flex flex-col items-center gap-2" style="left: 8px; z-index: 80;">
    {#each ui.platforms as platform}
      {#each platform.accounts as account}
        <button
          onclick={() => switchAccount(account.id)}
          class="bead"
          class:has-unread={account.unread > 0}
          class:active={account.id === ui.activeAccountId}
          style="--pcolor: {platform.color}; opacity: {account.status === "offline" ? 0.3 : 1}"
          title="{platform.name} · {account.nickname} ({account.unread})"
        ></button>
      {/each}
    {/each}
  </div>
{/if}

<!-- Status Bar — 底部薄条 -->
{#if ui.notifyMode === "bar" && ui.viewMode === "browser"}
  <div class="fixed bottom-0 left-0 right-0 z-80 status-bar">
    <div class="flex items-center gap-2 px-3 h-full text-xs text-theme-primary">
      {#each ui.platforms as platform}
        {#each platform.accounts as account}
          <button
            onclick={() => switchAccount(account.id)}
            class="flex items-center gap-1.5 px-2 py-1 radius-glass hover:bg-black/5 cursor-pointer whitespace-nowrap"
            class:font-semibold={account.id === ui.activeAccountId}
            style="border: none; background: transparent; font-family: inherit;">
            <span class="w-1.5 h-1.5 rounded-full"
                  style="background: {account.status === "online" ? (account.unread > 0 ? "#EF4444" : "#22C55E") : "#D4D4D4"};">
            </span>
            <span class="font-bold" style="color: {platform.color};">{platform.icon}</span>
            <span>{account.nickname}</span>
            {#if account.unread > 0}
              <span class="text-red-500 font-semibold">({account.unread})</span>
            {/if}
          </button>
        {/each}
      {/each}
      <div class="ml-auto flex items-center gap-1.5 text-theme-muted">
        <span class="w-1.5 h-1.5 rounded-full bg-[#22C55E]"></span>
        {ui.totalOnline}
      </div>
    </div>
  </div>
{/if}

<style>
  .bead {
    width: 6px; height: 6px; border-radius: 50%;
    background: rgba(0,0,0,0.12);
    border: none; padding: 0;
    cursor: pointer;
    transition: all 0.3s cubic-bezier(0.16, 1, 0.3, 1);
  }
  .bead.has-unread {
    background: var(--pcolor, #0D9488);
    box-shadow: 0 0 6px var(--pcolor, #0D9488);
    animation: bead-pulse 2s ease infinite;
  }
  .bead.active {
    width: 10px; height: 10px;
    box-shadow: 0 0 8px var(--pcolor, #0D9488);
  }
  @keyframes bead-pulse {
    0%, 100% { opacity: 1; transform: scale(1); }
    50% { opacity: 0.5; transform: scale(1.4); }
  }
  .status-bar {
    height: 32px;
    background: rgba(255,255,255,0.75);
    backdrop-filter: blur(24px);
    -webkit-backdrop-filter: blur(24px);
    border-top: 1px solid var(--glass-border);
    border-radius: 16px 16px 0 0;
  }
</style>
