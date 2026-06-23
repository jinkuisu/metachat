export interface PlatformInfo {
  id: string;
  name: string;
  icon: string;
  color: string;
  accounts: AccountInfo[];
}

export interface AccountInfo {
  id: string;
  platformId: string;
  nickname: string;
  avatar?: string;
  status: AccountStatus;
  unread: number;
  totalFans: number;
  currentFans: number;
}

export type AccountStatus = "online" | "offline" | "error" | "connecting";

export type ViewMode = "canvas" | "browser";

export type NotifyMode = "beads" | "bar";

export type ToolPanel = "ai" | "reply" | "translate" | null;

export interface PoolStats {
  active: number;
  monitor: number;
  frozen: number;
  totalMemoryMb: number;
}
