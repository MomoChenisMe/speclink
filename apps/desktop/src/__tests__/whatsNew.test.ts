// 更新日誌彈窗的純判定（desktop-app spec「更新日誌彈窗」Example「彈出判定」）與已看過記錄
// 的本機持久化：不依賴 Tauri，Storage 可注入（比照 assetPrompt.ts）。
import { describe, it, expect, beforeEach } from "vitest";

import {
  readLastSeenVersion,
  whatsNewDecision,
  writeLastSeenVersion,
} from "../core/whatsNew";
import type { ReleaseNotesEntry } from "../release-notes/release-notes";

const entry = (version: string): ReleaseNotesEntry => ({
  version,
  date: "2026-09-10",
  sections: [{ title: "新功能", items: [`${version} 的內容`] }],
});
const ENTRIES = [entry("0.3.1"), entry("0.3.0"), entry("0.2.0")];
const versions = (decision: ReturnType<typeof whatsNewDecision>) =>
  decision.kind === "show" ? decision.entries.map((e) => e.version) : decision.kind;

describe("whatsNewDecision", () => {
  it("dev 建置永不自動彈、也不寫記錄", () => {
    expect(
      whatsNewDecision({ appVersion: "0.3.0", entries: ENTRIES.slice(1), lastSeen: "0.2.0", dev: true }),
    ).toEqual({ kind: "none" });
  });

  it("JSON 頂端版號不等於 app 版號時不彈（建置鏈版號不齊）", () => {
    expect(
      whatsNewDecision({ appVersion: "0.3.0", entries: [entry("0.2.0")], lastSeen: "0.2.0", dev: false }),
    ).toEqual({ kind: "none" });
    expect(whatsNewDecision({ appVersion: "0.3.0", entries: [], lastSeen: null, dev: false })).toEqual({
      kind: "none",
    });
  });

  it("無已看過記錄（首次安裝）→ record：不彈、記現版號", () => {
    expect(
      whatsNewDecision({ appVersion: "0.3.0", entries: ENTRIES.slice(1), lastSeen: null, dev: false }),
    ).toEqual({ kind: "record" });
  });

  it("已看過記錄等於 app 版號 → none", () => {
    expect(
      whatsNewDecision({ appVersion: "0.3.0", entries: ENTRIES.slice(1), lastSeen: "0.3.0", dev: false }),
    ).toEqual({ kind: "none" });
  });

  it("由 0.2.0 更新到 0.3.0 → show 只含 0.3.0", () => {
    expect(
      versions(
        whatsNewDecision({ appVersion: "0.3.0", entries: ENTRIES.slice(1), lastSeen: "0.2.0", dev: false }),
      ),
    ).toEqual(["0.3.0"]);
  });

  it("跳版：已看過 0.2.0、清單 0.3.1／0.3.0／0.2.0 → show 依序含 0.3.1 與 0.3.0", () => {
    expect(
      versions(whatsNewDecision({ appVersion: "0.3.1", entries: ENTRIES, lastSeen: "0.2.0", dev: false })),
    ).toEqual(["0.3.1", "0.3.0"]);
  });

  it("降版：已看過記錄比 app 版號新且不在清單內 → none（不把看過的內容再彈一次）", () => {
    expect(
      whatsNewDecision({ appVersion: "0.3.1", entries: ENTRIES, lastSeen: "0.4.0", dev: false }),
    ).toEqual({ kind: "none" });
  });

  it("已看過記錄不在清單內 → show 只含頂端一筆", () => {
    expect(
      versions(whatsNewDecision({ appVersion: "0.3.1", entries: ENTRIES, lastSeen: "0.1.9", dev: false })),
    ).toEqual(["0.3.1"]);
  });
});

describe("已看過記錄的本機持久化", () => {
  const KEY = "speclink.lastSeenVersion";
  beforeEach(() => localStorage.clear());

  it("writeLastSeenVersion 寫入鍵 speclink.lastSeenVersion，readLastSeenVersion 讀回", () => {
    writeLastSeenVersion("0.3.0");
    expect(localStorage.getItem(KEY)).toBe("0.3.0");
    expect(readLastSeenVersion()).toBe("0.3.0");
  });

  it("無記錄回 null", () => {
    expect(readLastSeenVersion()).toBeNull();
  });

  it("壞值（非版號字串、JSON 物件、數字）一律視為無記錄", () => {
    for (const bad of ['{"version":"0.3.0"}', "123", '"0.3.0"', "", "abc"]) {
      localStorage.setItem(KEY, bad);
      expect(readLastSeenVersion(), `壞值 ${bad}`).toBeNull();
    }
  });

  it("Storage 可注入", () => {
    const store = new Map<string, string>();
    const storage = {
      getItem: (k: string) => store.get(k) ?? null,
      setItem: (k: string, v: string) => void store.set(k, v),
    } as unknown as Storage;
    writeLastSeenVersion("0.2.0", storage);
    expect(store.get(KEY)).toBe("0.2.0");
    expect(readLastSeenVersion(storage)).toBe("0.2.0");
    expect(localStorage.getItem(KEY)).toBeNull();
  });
});
