// design D8：UI 語言偏好存 localStorage 單鍵、null 表跟隨系統；
// 系統語言以 zh 開頭判 zh-TW 否則 en。
import { describe, it, expect } from "vitest";

import {
  detectSystemLocale,
  readLocalePreference,
  writeLocalePreference,
  resolveUiLocale,
} from "../i18n/locale";

describe("detectSystemLocale（spec Example「系統語言判定」）", () => {
  // | 系統語言 | UI 語言 |
  it.each([
    ["zh-TW", "zh-TW"],
    ["zh-CN", "zh-TW"],
    ["en-US", "en"],
    ["ja-JP", "en"],
  ] as const)("%s → %s", (system, ui) => {
    expect(detectSystemLocale(system)).toBe(ui);
  });

  it("undefined 系統語言退回 en", () => {
    expect(detectSystemLocale(undefined)).toBe("en");
  });
});

describe("locale preference（localStorage 單鍵）", () => {
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

  it("round-trips an explicit preference", () => {
    const st = memStorage();
    writeLocalePreference("en", st);
    expect(readLocalePreference(st)).toBe("en");
    writeLocalePreference("zh-TW", st);
    expect(readLocalePreference(st)).toBe("zh-TW");
  });

  it("null preference removes the key (follow system)", () => {
    const st = memStorage();
    writeLocalePreference("en", st);
    writeLocalePreference(null, st);
    expect(readLocalePreference(st)).toBeNull();
    expect(st.length).toBe(0);
  });

  it("garbage stored values read as null instead of crashing", () => {
    const st = memStorage();
    st.setItem("speclink.uiLocale", "fr");
    expect(readLocalePreference(st)).toBeNull();
  });
});

describe("resolveUiLocale", () => {
  it("explicit preference wins over the system language", () => {
    expect(resolveUiLocale("en", "zh-TW")).toBe("en");
    expect(resolveUiLocale("zh-TW", "en-US")).toBe("zh-TW");
  });

  it("null preference follows the system language", () => {
    expect(resolveUiLocale(null, "zh-TW")).toBe("zh-TW");
    expect(resolveUiLocale(null, "en-US")).toBe("en");
  });
});
