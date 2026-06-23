<script lang="ts">
  import { ui } from "$lib/stores/uiStore";
  import { i18n } from "$lib/i18n";
</script>

  <div class="glass radius-card shadow-theme-glass animate-fade-in"
     style="padding: 28px 36px; max-width: 400px;">
  <div class="text-2xl font-semibold text-theme-primary" style="margin-bottom: 4px;">
    {i18n.greeting()}，Alice
  </div>
  <div class="text-sm text-theme-muted" style="margin-bottom: 16px;">
    {i18n.t("greeting.unread", { count: ui.totalUnread })}
  </div>
  <div class="flex gap-4 text-xs text-theme-muted">
    <span class="flex items-center gap-1.5">
      <span class="w-2 h-2 rounded-full bg-[#22C55E] inline-block"></span>
      {i18n.t("greeting.online", { count: ui.totalOnline })}
    </span>
    <span>{i18n.t("greeting.contacts", { count: 342 })}</span>
  </div>
</div>
