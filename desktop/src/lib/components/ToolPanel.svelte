<script lang="ts">
  import { toolState } from "../stores/toolStore";
  import { i18n } from "../i18n";

  const tabIcon: Record<string, string> = { ai: "🤖", reply: "⚡", translate: "🌐" };
  const languages = [
    { code: "auto", label: "Auto Detect" },
    { code: "zh-CN", label: "中文" },
    { code: "en", label: "English" },
    { code: "ja", label: "日本語" },
    { code: "ko", label: "한국어" },
  ];

  function onBackdrop(e: MouseEvent) {
    if (e.target === e.currentTarget) toolState.closeTool();
  }
  function onInput(e: KeyboardEvent) {
    if (e.key === "Enter" && !e.shiftKey) { e.preventDefault(); toolState.sendAiMessage(); }
  }
  function copyReply(id: string, content: string) {
    navigator.clipboard?.writeText(content).catch(() => {});
  }
  function formatTime(ts: number) {
    const d = new Date(ts);
    return d.getHours().toString().padStart(2, "0") + ":" + d.getMinutes().toString().padStart(2, "0");
  }
</script>

{#if toolState.activeTool}
  <div class="fixed inset-0" style="z-index: 150; background: rgba(0,0,0,0.02);"
       onclick={onBackdrop} role="presentation"></div>

  <div class="slide-panel glass-heavy animate-slide-in" style="width: 380px; z-index: 160;">
    <div class="panel-header">
      <div class="flex items-center gap-2">
        <span class="text-lg">{tabIcon[toolState.activeTool] ?? ""}</span>
        <span class="font-semibold text-sm text-theme-primary">{i18n.t("tool." + toolState.activeTool)}</span>
      </div>
      <button onclick={() => toolState.closeTool()}
              class="size-7 flex items-center justify-center cursor-pointer radius-glass text-theme-muted hover:bg-black/5"
              style="border: none; background: transparent;">✕</button>
    </div>

    {#if toolState.activeTool === "ai"}
      <div class="flex-1 flex flex-col overflow-hidden">
        <div class="flex-1 overflow-y-auto p-4 flex flex-col gap-3">
          {#each toolState.aiMessages as msg}
            <div class="flex {msg.role === "user" ? "justify-end" : "justify-start"}">
              <div class="msg-bubble {msg.role === "user" ? "user-msg" : "ai-msg"}">
                <div class="text-sm">{msg.content}</div>
                <div class="msg-time">{formatTime(msg.time)}</div>
              </div>
            </div>
          {/each}
        </div>
        <div class="p-3 border-t" style="border-color: var(--glass-border);">
          <div class="flex gap-2">
            <input type="text" placeholder={i18n.t("tool.input")}
                   bind:value={toolState.aiInput} onkeydown={onInput}
                   class="flex-1 radius-glass px-3 py-2 text-sm outline-none input-field">
            <button onclick={() => toolState.sendAiMessage()} class="send-btn">发送</button>
          </div>
        </div>
      </div>

    {:else if toolState.activeTool === "reply"}
      <div class="flex-1 overflow-y-auto p-4">
        {#each toolState.replyGroups as group}
          <div class="mb-4">
            <div class="text-xs font-semibold text-theme-muted mb-2 px-1">{group.name}</div>
            <div class="flex flex-col gap-1.5">
              {#each group.items as item}
                <button onclick={() => copyReply(item.id, item.content)} class="reply-chip">
                  {item.content}
                </button>
              {/each}
            </div>
          </div>
        {/each}
      </div>

    {:else if toolState.activeTool === "translate"}
      <div class="flex-1 overflow-y-auto p-4 flex flex-col gap-3">
        <div class="flex items-center gap-2">
          <select bind:value={toolState.translateSourceLang} class="lang-select">
            {#each languages as lang}
              <option value={lang.code}>{lang.label}</option>
            {/each}
          </select>
          <span class="text-theme-muted">→</span>
          <select bind:value={toolState.translateTargetLang} class="lang-select">
            {#each languages as lang}
              {#if lang.code !== "auto"}
                <option value={lang.code}>{lang.label}</option>
              {/if}
            {/each}
          </select>
        </div>
        <textarea placeholder="输入要翻译的文本..."
                  bind:value={toolState.translateInput}
                  class="w-full h-24 resize-none radius-glass p-3 text-sm outline-none input-field"></textarea>
        <button onclick={() => toolState.doTranslate()} class="translate-btn">翻译</button>

        {#if toolState.translateResult}
          <div class="glass radius-card p-3">
            <div class="text-sm text-theme-primary">{toolState.translateResult}</div>
          </div>
        {/if}

        {#if toolState.translateHistory.length > 0}
          <div class="text-xs font-semibold text-theme-muted mt-2">翻译历史</div>
          {#each toolState.translateHistory as h}
            <div class="glass-light radius-card p-2 text-xs" style="border: 1px solid var(--glass-border);">
              <div class="text-theme-muted mb-1">{h.from} → {h.to}</div>
              <div class="text-theme-primary truncate">{h.text}</div>
              <div class="text-theme-muted mt-1 truncate">{h.result}</div>
            </div>
          {/each}
        {/if}
      </div>
    {/if}
  </div>
{/if}

<style>
  .slide-panel {
    position: fixed; top: 0; right: 0; height: 100%;
    border-radius: var(--radius-panel) 0 0 var(--radius-panel);
    box-shadow: -4px 0 24px rgba(0,0,0,0.08);
    display: flex; flex-direction: column;
  }
  .panel-header {
    display: flex; align-items: center; justify-content: space-between;
    padding: 14px 18px; border-bottom: 1px solid var(--glass-border);
  }
  .msg-bubble {
    max-width: 85%; padding: 8px 12px;
    color: var(--text-primary);
  }
  .user-msg {
    background: var(--primary); color: white;
    border-radius: 16px 16px 4px 16px;
  }
  .ai-msg {
    background: var(--glass-bg);
    border-radius: 16px 16px 16px 4px;
  }
  .msg-time {
    font-size: 10px; margin-top: 4px; opacity: 0.6; text-align: right;
  }
  .input-field {
    border: 1px solid var(--glass-border); font-family: inherit;
    background: rgba(255,255,255,0.5); color: var(--text-primary);
  }
  .input-field:focus {
    border-color: color-mix(in srgb, var(--primary) 30%, transparent);
  }
  .send-btn {
    padding: 8px 14px; border-radius: 12px; border: none;
    background: var(--primary); color: white;
    font-size: 13px; font-weight: 500; cursor: pointer;
  }
  .send-btn:hover { opacity: 0.9; }
  .reply-chip {
    text-align: left; width: 100%; padding: 8px 12px;
    font-size: 13px; border-radius: 12px; cursor: pointer;
    border: 1px solid var(--glass-border); background: var(--glass-light);
    color: var(--text-primary); font-family: inherit; transition: background 0.15s;
  }
  .reply-chip:hover { background: var(--glass-bg); }
  .lang-select {
    flex: 1; font-size: 12px; padding: 4px 8px; border-radius: 8px;
    border: 1px solid var(--glass-border); font-family: inherit;
    background: var(--glass-light); color: var(--text-primary);
    outline: none; cursor: pointer;
  }
  .translate-btn {
    width: 100%; padding: 8px; border-radius: 12px; border: none;
    background: var(--primary); color: white;
    font-size: 13px; font-weight: 500; cursor: pointer;
  }
  .translate-btn:hover { opacity: 0.9; }
</style>
