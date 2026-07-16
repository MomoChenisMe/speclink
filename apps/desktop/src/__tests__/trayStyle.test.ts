// 系統匣樣式偏好（design D4）：localStorage 單鍵，缺鍵或非法值一律視為
// native-menu——舊安裝升級後行為不變（spec「系統匣樣式偏好」向後相容場景）。
import { describe, it, expect } from "vitest";

import { readTrayStylePreference, writeTrayStylePreference } from "../trayStyle";

function memStorage(): Storage {
  const map = new Map<string, string>();
  return {
    getItem: (k: string) => map.get(k) ?? null,
    setItem: (k: string, v: string) => void map.set(k, v),
    removeItem: (k: string) => void map.delete(k),
    clear: () => map.clear(),
    key: () => null,
    get length() {
      return map.size;
    },
  } as Storage;
}

describe("tray style preference（localStorage 單鍵）", () => {
  it("缺鍵視為 native-menu（舊安裝向後相容）", () => {
    expect(readTrayStylePreference(memStorage())).toBe("native-menu");
  });

  it("非法值（手改 localStorage）一律視為 native-menu", () => {
    const s = memStorage();
    s.setItem("speclink.trayStyle", "florp");
    expect(readTrayStylePreference(s)).toBe("native-menu");
  });

  it("寫入 panel 後可讀回（切換持久化）", () => {
    const s = memStorage();
    writeTrayStylePreference("panel", s);
    expect(readTrayStylePreference(s)).toBe("panel");
    writeTrayStylePreference("native-menu", s);
    expect(readTrayStylePreference(s)).toBe("native-menu");
  });
});
