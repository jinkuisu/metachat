import { zhCN } from "./zh-CN";
import { enUS } from "./en-US";
import type { PlatformInfo, AccountInfo } from "../types";

const locales: Record<string, Record<string, any>> = {
  "zh-CN": zhCN,
  "en-US": enUS,
};

const LOCALE_STORAGE_KEY = "metachat-locale";

class I18nStore {
  locale = $state("zh-CN");
  messages = $derived(locales[this.locale] ?? locales["zh-CN"]);

  constructor() {
    if (typeof localStorage !== "undefined") {
      const saved = localStorage.getItem(LOCALE_STORAGE_KEY);
      if (saved && locales[saved]) {
        this.locale = saved;
      }
    }
  }

  setLocale(locale: string) {
    if (locales[locale]) {
      this.locale = locale;
      if (typeof localStorage !== "undefined") {
        localStorage.setItem(LOCALE_STORAGE_KEY, locale);
      }
    }
  }

  t(key: string, params?: Record<string, string | number>): string {
    const keys = key.split(".");
    let value: any = this.messages;
    for (const k of keys) {
      if (value == null || typeof value !== "object") return key;
      value = value[k];
    }
    if (typeof value !== "string") return key;
    if (params) {
      return value.replace(/\{(\w+)\}/g, (_, k) => String(params[k] ?? `{${k}}`));
    }
    return value;
  }

  // 获取问候语（含时间判断）
  greeting(): string {
    const h = new Date().getHours();
    const key = h < 12 ? "morning" : h < 18 ? "afternoon" : "evening";
    return this.t(`greeting.${key}`);
  }
}

export const i18n = new I18nStore();
