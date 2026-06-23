export type ThemeName = "light" | "dark";
export const THEME_STORAGE_KEY = "metachat-theme";

/// 将主题应用到 HTML 元素
function applyTheme(name: ThemeName) {
  document.documentElement.setAttribute("data-theme", name);
}

class ThemeStore {
  current = $state<ThemeName>("light");

  constructor() {
    const saved = (typeof localStorage !== "undefined"
      ? localStorage.getItem(THEME_STORAGE_KEY)
      : null) as ThemeName | null;
    if (saved && ["light", "dark"].includes(saved)) {
      this.current = saved;
    }
    // 构造函数就设置 data-theme，不需要等 onMount
    if (typeof document !== "undefined") {
      applyTheme(this.current);
    }
  }

  setTheme(name: ThemeName) {
    this.current = name;
    if (typeof localStorage !== "undefined") {
      localStorage.setItem(THEME_STORAGE_KEY, name);
    }
    applyTheme(name);
  }

  toggle() {
    const next = this.current === "light" ? "dark" : "light";
    this.setTheme(next);
  }
}

export const theme = new ThemeStore();
