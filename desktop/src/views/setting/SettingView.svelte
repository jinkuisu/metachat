<script lang="ts">
  import { i18n } from "$lib/i18n";
  import { theme } from "$lib/stores/themeStore";
  import { ui } from "$lib/stores/uiStore";

  let proxyType = $state("none");
  let proxyHost = $state("");
  let proxyPort = $state("1080");
  let transEngine = $state("google");
</script>

<div class="glass-heavy radius-panel shadow-theme-glass-lg animate-fade-in"
     style="width: 640px; max-height: 80vh; display: flex; flex-direction: column;">
  <!-- header -->
  <div class="flex items-center justify-between px-6 py-4" style="border-bottom: 1px solid var(--glass-border);">
    <div class="flex items-center gap-2">
      <span class="text-lg">⚙️</span>
      <span class="font-semibold text-theme-primary">设置</span>
    </div>
    <button onclick={() => ui.setConfigView(null)}
            class="size-7 flex items-center justify-center radius-glass cursor-pointer text-theme-muted hover:bg-black/5"
            style="border: none; background: transparent;">✕</button>
  </div>

  <!-- body -->
  <div class="flex-1 overflow-y-auto p-6 space-y-6 text-sm">

    <!-- 主题 -->
    <div>
      <div class="font-medium text-theme-primary mb-3">主题</div>
      <div class="flex gap-3">
        <button onclick={() => theme.setTheme("light")}
                class="flex-1 p-3 radius-glass cursor-pointer transition-all"
                class:selected={theme.current === "light"}
                style="border: 1px solid var(--glass-border); background: var(--glass-light); color: var(--text-primary);
                       font-family: inherit; text-align: left;">
          <div class="text-base mb-1">☀️ Light</div>
          <div class="text-xs text-theme-muted">暖白渐变背景</div>
        </button>
        <button onclick={() => theme.setTheme("dark")}
                class="flex-1 p-3 radius-glass cursor-pointer transition-all"
                class:selected={theme.current === "dark"}
                style="border: 1px solid var(--glass-border); background: var(--glass-light); color: var(--text-primary);
                       font-family: inherit; text-align: left;">
          <div class="text-base mb-1">🌙 Dark</div>
          <div class="text-xs text-theme-muted">深色系氛围背景</div>
        </button>
      </div>
    </div>

    <!-- 语言 -->
    <div>
      <div class="font-medium text-theme-primary mb-3">界面语言</div>
      <div class="flex gap-3">
        <button onclick={() => i18n.setLocale("zh-CN")}
                class="flex-1 p-3 radius-glass cursor-pointer"
                class:selected={i18n.locale === "zh-CN"}
                style="border: 1px solid var(--glass-border); background: var(--glass-light); color: var(--text-primary);
                       font-family: inherit;">
          <div class="text-base">🇨🇳 中文</div>
        </button>
        <button onclick={() => i18n.setLocale("en-US")}
                class="flex-1 p-3 radius-glass cursor-pointer"
                class:selected={i18n.locale === "en-US"}
                style="border: 1px solid var(--glass-border); background: var(--glass-light); color: var(--text-primary);
                       font-family: inherit;">
          <div class="text-base">🇺🇸 English</div>
        </button>
      </div>
    </div>

    <!-- 代理 -->
    <div>
      <div class="font-medium text-theme-primary mb-3">默认代理</div>
      <div class="glass-light radius-card p-4" style="border: 1px solid var(--glass-border);">
        <div class="flex gap-2 mb-3">
          {#each ["none","http","https","socks5"] as t}
            <button onclick={() => proxyType = t}
                    class="px-3 py-1.5 radius-glass text-xs cursor-pointer"
                    class:selected={proxyType === t}
                    style="border: 1px solid var(--glass-border); background: {proxyType === t ? 'var(--primary)' : 'transparent'};
                           color: {proxyType === t ? 'white' : 'var(--text-primary)'}; font-family: inherit;">
              {t === "none" ? "无" : t.toUpperCase()}
            </button>
          {/each}
        </div>
        {#if proxyType !== "none"}
          <div class="flex gap-2">
            <input type="text" placeholder="主机" bind:value={proxyHost}
                   class="flex-1 px-3 py-1.5 radius-glass text-xs outline-none"
                   style="border: 1px solid var(--glass-border); font-family: inherit; background: rgba(255,255,255,0.5); color: var(--text-primary);">
            <input type="text" placeholder="端口" bind:value={proxyPort}
                   class="w-20 px-3 py-1.5 radius-glass text-xs outline-none"
                   style="border: 1px solid var(--glass-border); font-family: inherit; background: rgba(255,255,255,0.5); color: var(--text-primary);">
          </div>
        {/if}
      </div>
    </div>

    <!-- 翻译引擎 -->
    <div>
      <div class="font-medium text-theme-primary mb-3">默认翻译引擎</div>
      <div class="flex gap-2 flex-wrap">
        {#each ["google","deepl","openai","deepseek"] as eng}
          <button onclick={() => transEngine = eng}
                  class="px-3 py-1.5 radius-glass text-xs cursor-pointer"
                  class:selected={transEngine === eng}
                  style="border: 1px solid var(--glass-border); background: {transEngine === eng ? 'var(--primary)' : 'transparent'};
                         color: {transEngine === eng ? 'white' : 'var(--text-primary)'}; font-family: inherit; text-transform: capitalize;">
            {eng}
          </button>
        {/each}
      </div>
    </div>

    <!-- 关于 -->
    <div>
      <div class="font-medium text-theme-primary mb-3">关于</div>
      <div class="glass-light radius-card p-4 text-xs text-theme-muted" style="border: 1px solid var(--glass-border);">
        <div>MetaChat v0.1.0</div>
        <div>Tauri 2.0 + Svelte 5 + Rust</div>
      </div>
    </div>
  </div>
</div>

<style>
  .selected { border-color: var(--primary) !important; }
</style>
