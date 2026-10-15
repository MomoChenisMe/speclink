// 外觀偏好（desktop-settings-mode design D4）：localStorage 單鍵、null 表跟隨系統；
// 明示淺色／深色以根元素的 data-theme 屬性觸發 theme.css 的對應區塊。
import { describe, it, expect, beforeEach } from "vitest";

import {
  THEME_STORAGE_KEY,
  applyThemePreference,
  readThemePreference,
  writeThemePreference,
} from "../theme";

const KEY = "speclink.theme";
// 本套件的 jsdom 環境不掛 Web Storage：注入記憶體版 Storage（函式只用 get／set／remove）。
const entries = new Map<string, string>();
const storage = {
  getItem: (key: string) => entries.get(key) ?? null,
  setItem: (key: string, value: string) => void entries.set(key, String(value)),
  removeItem: (key: string) => void entries.delete(key),
  clear: () => entries.clear(),
} as unknown as Storage;

beforeEach(() => {
  storage.clear();
  document.documentElement.removeAttribute("data-theme");
});

describe("外觀偏好讀寫", () => {
  it.each([
    ["light", "light"],
    ["dark", "dark"],
  ] as const)("寫入 %s 後讀回 %s", (pref, expected) => {
    writeThemePreference(pref, storage);
    expect(storage.getItem(KEY)).toBe(pref);
    expect(readThemePreference(storage)).toBe(expected);
  });

  it("寫入 null（跟隨系統）即移除鍵，讀回 null", () => {
    writeThemePreference("dark", storage);
    writeThemePreference(null, storage);
    expect(storage.getItem(KEY)).toBeNull();
    expect(readThemePreference(storage)).toBeNull();
  });

  it.each(["", "Dark", "system", "1", "null"])("壞值 %j 視為跟隨系統", (bad) => {
    storage.setItem(KEY, bad);
    expect(readThemePreference(storage)).toBeNull();
  });

  it("儲存區拋錯時讀為跟隨系統，寫入回報錯誤供設定頁保留原值", () => {
    const broken = {
      getItem: () => {
        throw new Error("denied");
      },
      setItem: () => {
        throw new Error("quota");
      },
      removeItem: () => {
        throw new Error("denied");
      },
    } as unknown as Storage;
    expect(readThemePreference(broken)).toBeNull();
    expect(() => writeThemePreference("dark", broken)).toThrow("quota");
    expect(() => writeThemePreference(null, broken)).toThrow("denied");
  });

  it("鍵名對外匯出（index.html 的開機腳本讀同一個鍵）", () => {
    expect(THEME_STORAGE_KEY).toBe(KEY);
  });
});

// spec desktop-config Example「有效主題判定」：偏好經 applyThemePreference 落到根元素，
// 系統深色時看媒體查詢區塊的選擇器、明示深色看 data-theme 區塊的選擇器（兩個選擇器
// 在 theme.css 的位置由 theme.test.ts 釘住）。
describe("有效主題判定", () => {
  const SYSTEM_DARK_SELECTOR = ':root:not([data-theme="light"])';
  const EXPLICIT_DARK_SELECTOR = ':root[data-theme="dark"]';
  const PREF = { 跟隨系統: null, 淺色: "light", 深色: "dark" } as const;

  it.each([
    ["跟隨系統", "淺色", "淺色"],
    ["跟隨系統", "深色", "深色"],
    ["淺色", "深色", "淺色"],
    ["深色", "淺色", "深色"],
  ] as const)("偏好 %s、系統 %s → %s", (pref, system, effective) => {
    const root = document.documentElement;
    applyThemePreference(PREF[pref], root);
    const dark =
      (system === "深色" && root.matches(SYSTEM_DARK_SELECTOR)) || root.matches(EXPLICIT_DARK_SELECTOR);
    expect(dark ? "深色" : "淺色").toBe(effective);
  });
});

describe("applyThemePreference", () => {
  it("明示淺色／深色設 data-theme，跟隨系統移除屬性", () => {
    const root = document.documentElement;
    applyThemePreference("dark");
    expect(root.getAttribute("data-theme")).toBe("dark");
    applyThemePreference("light");
    expect(root.getAttribute("data-theme")).toBe("light");
    applyThemePreference(null);
    expect(root.hasAttribute("data-theme")).toBe(false);
  });

  it("可指定套用的根元素", () => {
    const el = document.createElement("div");
    applyThemePreference("dark", el);
    expect(el.getAttribute("data-theme")).toBe("dark");
    expect(document.documentElement.hasAttribute("data-theme")).toBe(false);
  });
});
