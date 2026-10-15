import { beforeEach, describe, expect, it } from "vitest";
import { readThemePreference, writeThemePreference } from "../theme";

const entries = new Map<string, string>();
const storage = {
  getItem: (key: string) => entries.get(key) ?? null,
  setItem: (key: string, value: string) => entries.set(key, value),
  removeItem: (key: string) => entries.delete(key),
} as unknown as Storage;

beforeEach(() => entries.clear());

describe("Fork 外觀升級相容", () => {
  it.each(["light", "dark"] as const)("恢復舊 speclink.uiTheme=%s", (pref) => {
    entries.set("speclink.uiTheme", pref);
    expect(readThemePreference(storage)).toBe(pref);
  });

  it("官方新鍵優先，不讓舊鍵覆蓋新版選擇", () => {
    entries.set("speclink.uiTheme", "dark");
    entries.set("speclink.theme", "light");
    expect(readThemePreference(storage)).toBe("light");
  });

  it("跟隨系統清除舊明示偏好，重開不會恢復舊深色", () => {
    entries.set("speclink.uiTheme", "dark");
    writeThemePreference(null, storage);
    expect(readThemePreference(storage)).toBeNull();
    expect(entries.has("speclink.uiTheme")).toBe(false);
  });

  it("保存失敗回報錯誤，讓設定頁保留原選擇", () => {
    const broken = { ...storage, setItem: () => { throw new Error("quota"); } } as Storage;
    expect(() => writeThemePreference("dark", broken)).toThrow("quota");
  });
});
