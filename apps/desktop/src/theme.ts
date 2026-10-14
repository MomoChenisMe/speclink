export type ThemePreference = "system" | "light" | "dark";
export type ResolvedTheme = "light" | "dark";

const STORAGE_KEY = "speclink.uiTheme";

export function applyTheme(theme: ResolvedTheme): void {
  document.documentElement.dataset.theme = theme;
}

/** 僅主視窗擁有偏好；面板從快照取得有效值後呼叫 applyTheme。 */
export function createThemeController(media = window.matchMedia?.("(prefers-color-scheme: dark)")) {
  let preference: ThemePreference = "system";
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored === "light" || stored === "dark") preference = stored;
  } catch {
    // 儲存不可讀時採系統預設。
  }
  const resolve = (): ResolvedTheme =>
    preference === "system" ? (media?.matches ? "dark" : "light") : preference;
  let snapshot = { preference, effectiveTheme: resolve() };
  const listeners = new Set<() => void>();
  const update = () => {
    const effectiveTheme = resolve();
    applyTheme(effectiveTheme);
    if (snapshot.preference === preference && snapshot.effectiveTheme === effectiveTheme) return;
    snapshot = { preference, effectiveTheme };
    listeners.forEach((listener) => listener());
  };
  applyTheme(snapshot.effectiveTheme);

  return {
    getSnapshot: () => snapshot,
    subscribe(listener: () => void) {
      if (listeners.size === 0) media?.addEventListener("change", update);
      listeners.add(listener);
      update();
      return () => {
        listeners.delete(listener);
        if (listeners.size === 0) media?.removeEventListener("change", update);
      };
    },
    setPreference(value: ThemePreference) {
      if (value !== "system" && value !== "light" && value !== "dark") {
        throw new Error("Invalid appearance preference");
      }
      // 先保存，失敗不得把畫面切成沒有保存的選擇。
      if (value === "system") localStorage.removeItem(STORAGE_KEY);
      else localStorage.setItem(STORAGE_KEY, value);
      preference = value;
      update();
    },
  };
}

export type ThemeController = ReturnType<typeof createThemeController>;
