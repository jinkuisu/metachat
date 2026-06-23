<script lang="ts">
  import Canvas from "$lib/components/Canvas.svelte";
  import GreetingCard from "$lib/components/GreetingCard.svelte";
  import PlatformPills from "$lib/components/PlatformPills.svelte";
  import AccountCards from "$lib/components/AccountCards.svelte";
  import ToolIsland from "$lib/components/ToolIsland.svelte";
  import ToolPanel from "$lib/components/ToolPanel.svelte";
  import BrowserView from "$lib/components/BrowserView.svelte";
  import SettingView from "../setting/SettingView.svelte";
  import ReplyView from "../reply/ReplyView.svelte";
  import PromptsView from "../prompts/PromptsView.svelte";
  import { ui } from "$lib/stores/uiStore";
  import { theme } from "$lib/stores/themeStore";
  import { i18n } from "$lib/i18n";
</script>

<Canvas />

<div class="fixed top-4 left-4 flex gap-2 items-center z-50 flex-wrap" style="max-width: 600px;">
  <button onclick={() => ui.setViewMode("canvas")} class="dev-btn" style="background:{ui.viewMode==='canvas'?'var(--primary)':'var(--glass-light)'};color:{ui.viewMode==='canvas'?'white':'var(--text-primary)'};">Canvas</button>
  <button onclick={() => ui.setViewMode("browser")} class="dev-btn" style="background:{ui.viewMode==='browser'?'var(--primary)':'var(--glass-light)'};color:{ui.viewMode==='browser'?'white':'var(--text-primary)'};">Browser</button>
  <button onclick={() => theme.toggle()} class="dev-btn">{theme.current === "light" ? "🌙 Dark" : "☀️ Light"}</button>
  <button onclick={() => i18n.setLocale(i18n.locale === "zh-CN" ? "en-US" : "zh-CN")} class="dev-btn">🌐 {i18n.locale}</button>
  <button onclick={() => ui.setNotifyMode(ui.notifyMode === "beads" ? "bar" : "beads")} class="dev-btn">{ui.notifyMode === "beads" ? "📊 Bar" : "🔵 Beads"}</button>
</div>

<ToolIsland />

{#if ui.viewMode === "canvas"}
  {#if ui.configView === "settings"}
    <div class="fixed inset-0 flex items-center justify-center z-10"><SettingView /></div>
  {:else if ui.configView === "reply"}
    <div class="fixed inset-0 flex items-center justify-center z-10"><ReplyView /></div>
  {:else if ui.configView === "prompts"}
    <div class="fixed inset-0 flex items-center justify-center z-10"><PromptsView /></div>
  {:else}
    <div class="fixed inset-0 flex flex-col items-center justify-center z-10"><GreetingCard /></div>
    <AccountCards />
  {/if}
  <PlatformPills />
{/if}

{#if ui.viewMode === "browser"}
  {#key ui.activeAccountId}
    <BrowserView />
  {/key}
{/if}

<ToolPanel />

<style>
  .dev-btn { padding: 4px 8px; font-size: 11px; border-radius: var(--radius-glass); border: 1px solid var(--glass-border); cursor: pointer; background: var(--glass-light); color: var(--text-primary); font-family: inherit; white-space: nowrap; }
  .dev-btn:hover { background: var(--glass-bg); }
</style>

