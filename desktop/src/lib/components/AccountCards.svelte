<script lang="ts">
  import { ui } from "$lib/stores/uiStore";
  import { i18n } from "$lib/i18n";
import { invoke } from "@tauri-apps/api/core";

  async function enterBrowser(platformId: string, accountId: string) {
    try { await invoke("open_session_browser", { platform: platformId, accountId }); } catch (e) { alert("启动失败: " + e); return; }
    ui.setActiveAccount(accountId);
    ui.setViewMode("browser");
  }
</script>

{#if ui.selectedPlatform}
  {#each ui.platforms as platform}
    {#if platform.id === ui.selectedPlatform}
      <div class="fixed inset-0 flex items-center justify-center" style="z-index: 40; pointer-events: none;">
        <div class="flex flex-col items-center gap-3" style="pointer-events: auto; min-width: 280px;">
          <div class="text-sm font-medium text-theme-muted" style="margin-bottom: 2px;">
            {i18n.t("platform." + platform.id)}
          </div>

          {#each platform.accounts as account}
            <button
              onclick={() => enterBrowser(platform.id, account.id)}
              class="account-card glass radius-card shadow-theme-glass"
              style="width: 280px; padding: 14px 18px; display: flex; align-items: center; gap: 14px; cursor: pointer; text-align: left;"
            >
              <div class="rounded-full flex items-center justify-center text-white font-bold text-sm flex-shrink-0"
                   style="width: 40px; height: 40px; background: {platform.color};">
                {platform.icon}
              </div>
              <div class="flex-1 min-w-0">
                <div class="font-medium text-theme-primary text-sm">{account.nickname}</div>
                <div class="text-xs text-theme-muted">
                  {i18n.t("account.fans", { current: account.currentFans, total: account.totalFans })}
                </div>
              </div>
              <div class="flex flex-col items-end gap-0.5 flex-shrink-0">
                <span class="w-2 h-2 rounded-full"
                      style="background: {account.status === "online" ? "#22C55E" : account.status === "error" ? "#EF4444" : "#D4D4D4"};">
                </span>
                {#if account.unread > 0}
                  <span class="text-xs font-semibold text-red-500">{account.unread}</span>
                {/if}
              </div>
            </button>
          {/each}

          <button class="add-card glass-light radius-card flex items-center justify-center gap-2 text-theme-muted text-sm cursor-pointer"
                  style="width: 280px; padding: 10px; border: 1px dashed var(--glass-border);"
                  title={i18n.t("account.addHint")}>
            <span class="text-lg font-light">+</span>
            {i18n.t("account.add")}
          </button>
        </div>
      </div>
    {/if}
  {/each}
{/if}

<style>
  .account-card {
    transition: all 0.2s ease;
  }
  .account-card:hover {
    transform: translateY(-2px);
    box-shadow: var(--shadow-glass-lg);
  }
  .add-card:hover {
    background: var(--glass-bg);
  }
</style>
