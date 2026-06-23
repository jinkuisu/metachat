import type { ViewMode, NotifyMode, ToolPanel, PlatformInfo, PoolStats } from "../types";

class UiState {
  viewMode = $state<ViewMode>("canvas");
  notifyMode = $state<NotifyMode>("beads");
  activeTool = $state<ToolPanel>(null);
  selectedPlatform = $state<string | null>(null);
  activeAccountId = $state<string | null>(null);
  poolStats = $state<PoolStats>({ active: 0, monitor: 0, frozen: 0, totalMemoryMb: 0 });
  isBackHovered = $state(false);

  platforms = $state<PlatformInfo[]>([
    {
      id: "whatsapp", name: "WhatsApp", icon: "WA", color: "#25D366",
      accounts: [
        { id: "wa1", platformId: "whatsapp", nickname: "Alice", status: "online", unread: 12, totalFans: 342, currentFans: 128 },
        { id: "wa2", platformId: "whatsapp", nickname: "Bob", status: "online", unread: 3, totalFans: 156, currentFans: 89 },
        { id: "wa3", platformId: "whatsapp", nickname: "Charlie", status: "online", unread: 0, totalFans: 89, currentFans: 45 },
        { id: "wa4", platformId: "whatsapp", nickname: "David", status: "offline", unread: 0, totalFans: 0, currentFans: 0 },
      ],
    },
    {
      id: "telegram", name: "Telegram", icon: "TG", color: "#0088cc",
      accounts: [
        { id: "tg1", platformId: "telegram", nickname: "Bot 1", status: "online", unread: 7, totalFans: 203, currentFans: 156 },
        { id: "tg2", platformId: "telegram", nickname: "Bot 2", status: "online", unread: 1, totalFans: 89, currentFans: 67 },
      ],
    },
    {
      id: "facebook", name: "Facebook", icon: "FB", color: "#1877F2",
      accounts: [
        { id: "fb1", platformId: "facebook", nickname: "Sales", status: "online", unread: 3, totalFans: 567, currentFans: 234 },
        { id: "fb2", platformId: "facebook", nickname: "Support", status: "offline", unread: 0, totalFans: 0, currentFans: 0 },
      ],
    },
    {
      id: "instagram", name: "Instagram", icon: "IG", color: "#E4405F",
      accounts: [
        { id: "ig1", platformId: "instagram", nickname: "Promo", status: "online", unread: 0, totalFans: 890, currentFans: 456 },
      ],
    },
    {
      id: "x", name: "X", icon: "X", color: "#1DA1F2",
      accounts: [],
    },
    {
      id: "line", name: "LINE", icon: "LN", color: "#00B900",
      accounts: [],
    },
  ]);

  totalUnread = $derived(
    this.platforms.reduce((sum, p) => sum + p.accounts.reduce((s, a) => s + a.unread, 0), 0)
  );

  totalOnline = $derived(
    this.platforms.reduce((sum, p) => sum + p.accounts.filter(a => a.status === "online").length, 0)
  );

  getAccountsForPlatform(platformId: string) {
    return this.platforms.find(p => p.id === platformId)?.accounts ?? [];
  }

  setViewMode(mode: ViewMode) { this.viewMode = mode; }
  setNotifyMode(mode: NotifyMode) { this.notifyMode = mode; }
  toggleTool(panel: ToolPanel) { this.activeTool = this.activeTool === panel ? null : panel; }
  selectPlatform(id: string | null) { this.selectedPlatform = id; }
  setActiveAccount(id: string | null) { this.activeAccountId = id; }
  closeTool() { this.activeTool = null; }
  configView = $state<string | null>(null);

  setConfigView(view: string | null) {
    if (this.configView === view) { this.configView = null; return; }
    this.configView = view;
    this.selectedPlatform = null;
  }
}

export const ui = new UiState();
