// 專案分頁列的純函式面（design D8/D10）：localStorage 持久化（路徑＋顯示名＋
// 順序＋最後活躍）、上限 10、去重、關閉移除；徽章派生與看板欄位規則一致。
import { describe, it, expect } from "vitest";
import type { ChangeItem } from "@speclink/ui";

import {
  MAX_TABS,
  upsertTab,
  removeTab,
  persistTabs,
  readPersistedTabs,
  inProgressCount,
  type ProjectTab,
} from "../tabs";

function tab(root: string, name = root): ProjectTab {
  return { root, name, badge: null };
}

describe("upsertTab（spec「成功開啟後記入分頁並去重上移」的清單面）", () => {
  it("appends a new project at the end", () => {
    const tabs = upsertTab([tab("A")], { root: "B", name: "B" });
    expect(tabs.map((t) => t.root)).toEqual(["A", "B"]);
  });

  it("reopening an existing project keeps one tab in place (dedup)", () => {
    const tabs = upsertTab([tab("A"), tab("B")], { root: "A", name: "A" });
    expect(tabs.map((t) => t.root)).toEqual(["A", "B"]);
    expect(tabs).toHaveLength(2);
  });

  it("caps at MAX_TABS by dropping the oldest tab", () => {
    let tabs: ProjectTab[] = [];
    for (let i = 0; i < MAX_TABS; i++) tabs = upsertTab(tabs, { root: `p${i}`, name: `p${i}` });
    expect(tabs).toHaveLength(MAX_TABS);
    tabs = upsertTab(tabs, { root: "extra", name: "extra" });
    expect(tabs).toHaveLength(MAX_TABS);
    expect(tabs.some((t) => t.root === "p0")).toBe(false);
    expect(tabs.at(-1)?.root).toBe("extra");
  });

  it("updates the display name of an existing tab", () => {
    const tabs = upsertTab([tab("A", "old")], { root: "A", name: "new" });
    expect(tabs[0].name).toBe("new");
  });
});

describe("removeTab", () => {
  it("removes the tab with the given root", () => {
    expect(removeTab([tab("A"), tab("B")], "A").map((t) => t.root)).toEqual(["B"]);
  });
});

describe("persistTabs / readPersistedTabs（跨啟動還原）", () => {
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

  it("round-trips tabs (order), names and the active root", () => {
    const st = memStorage();
    persistTabs([tab("B", "beta"), tab("A", "alpha")], "A", st);
    const restored = readPersistedTabs(st);
    expect(restored.tabs.map((t) => t.root)).toEqual(["B", "A"]);
    expect(restored.tabs[0].name).toBe("beta");
    expect(restored.activeRoot).toBe("A");
  });

  it("empty storage restores to zero tabs", () => {
    const restored = readPersistedTabs(memStorage());
    expect(restored.tabs).toEqual([]);
    expect(restored.activeRoot).toBeNull();
  });

  it("garbage stored values restore to zero tabs instead of crashing", () => {
    const st = memStorage();
    st.setItem("speclink.projectTabs", "{not json");
    expect(readPersistedTabs(st).tabs).toEqual([]);
  });
});

describe("inProgressCount（徽章＝進行中變更數，與看板欄位派生一致）", () => {
  it("counts started or progressed changes, excluding ready ones", () => {
    const changes: ChangeItem[] = [
      { name: "proposed", status: "x", totalTasks: 28, completedTasks: 0 },
      { name: "started", status: "x", totalTasks: 10, completedTasks: 0, startedAt: "2026-07-06" },
      { name: "progressed", status: "x", totalTasks: 14, completedTasks: 2 },
      { name: "ready", status: "x", totalTasks: 5, completedTasks: 5 },
    ];
    expect(inProgressCount(changes)).toBe(2);
  });
});
