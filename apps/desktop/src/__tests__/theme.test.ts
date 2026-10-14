import { afterEach, describe, expect, it, vi } from "vitest";
import { applyTheme, createThemeController, type ThemePreference } from "../theme";

afterEach(() => {
  localStorage.removeItem("speclink.uiTheme");
  delete document.documentElement.dataset.theme;
  vi.restoreAllMocks();
});

function mediaQuery(dark = false) {
  const listeners = new Set<() => void>();
  const media = {
    matches: dark,
    addEventListener: vi.fn((_event: string, listener: () => void) => listeners.add(listener)),
    removeEventListener: vi.fn((_event: string, listener: () => void) => listeners.delete(listener)),
  };
  return { media: media as unknown as MediaQueryList, change(value: boolean) {
    media.matches = value;
    listeners.forEach((listener) => listener());
  } };
}

describe("裝置級外觀偏好", () => {
  it.each([
    [null, false, "light"], [null, true, "dark"],
    ["light", false, "light"], ["light", true, "light"],
    ["dark", false, "dark"], ["dark", true, "dark"],
    ["invalid", true, "dark"],
  ])("儲存 %s／系統深色 %s → %s", (value, dark, effectiveTheme) => {
    if (value !== null) localStorage.setItem("speclink.uiTheme", value);
    const { media } = mediaQuery(dark);
    const theme = createThemeController(media);
    expect(theme.getSnapshot()).toEqual({ preference: value === "light" || value === "dark" ? value : "system", effectiveTheme });
    expect(document.documentElement.dataset.theme).toBe(effectiveTheme);
    expect(localStorage.getItem("speclink.uiTheme")).toBe(value);
  });

  it("系統變更只影響 system，取消訂閱會清理監聽", () => {
    const { media, change } = mediaQuery();
    const theme = createThemeController(media);
    const listener = vi.fn();
    const stop = theme.subscribe(listener);
    change(true);
    expect(theme.getSnapshot().effectiveTheme).toBe("dark");
    theme.setPreference("light");
    change(false);
    change(true);
    expect(theme.getSnapshot().effectiveTheme).toBe("light");
    theme.setPreference("system");
    expect(theme.getSnapshot().effectiveTheme).toBe("dark");
    expect(localStorage.getItem("speclink.uiTheme")).toBeNull();
    stop();
    expect(media.removeEventListener).toHaveBeenCalledTimes(1);
    const calls = listener.mock.calls.length;
    change(false);
    expect(listener).toHaveBeenCalledTimes(calls);
    const stopAgain = theme.subscribe(listener);
    expect(theme.getSnapshot().effectiveTheme).toBe("light");
    stopAgain();
  });

  it("保存成功後重建恢復偏好，未改 UI 語言", () => {
    localStorage.setItem("speclink.uiLocale", "zh-TW");
    createThemeController(mediaQuery().media).setPreference("dark");
    expect(createThemeController(mediaQuery().media).getSnapshot().preference).toBe("dark");
    expect(localStorage.getItem("speclink.uiLocale")).toBe("zh-TW");
  });

  it("讀取失敗時跟隨系統；寫入失敗保留原配色與選擇", () => {
    const proto = Object.getPrototypeOf(localStorage);
    vi.spyOn(proto, "getItem").mockImplementation(() => { throw new Error("read denied"); });
    const theme = createThemeController(mediaQuery().media);
    expect(theme.getSnapshot()).toEqual({ preference: "system", effectiveTheme: "light" });
    vi.spyOn(proto, "setItem").mockImplementation(() => { throw new Error("write denied"); });
    expect(() => theme.setPreference("dark")).toThrow("write denied");
    expect(theme.getSnapshot()).toEqual({ preference: "system", effectiveTheme: "light" });
    expect(document.documentElement.dataset.theme).toBe("light");
  });

  it("非法外部值拒絕寫入；套用面板配色不寫偏好", () => {
    const theme = createThemeController(mediaQuery().media);
    expect(() => theme.setPreference("unexpected" as ThemePreference)).toThrow();
    applyTheme("dark");
    expect(document.documentElement.dataset.theme).toBe("dark");
    expect(localStorage.getItem("speclink.uiTheme")).toBeNull();
  });
});
