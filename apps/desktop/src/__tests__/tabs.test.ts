// 專案分頁列的純函式面（design D8/D10）：localStorage 持久化（路徑＋顯示名＋
// 順序＋最後活躍）、上限 10、去重、關閉移除；徽章派生為待收尾數
//（spec-archive-drawer design D6：已就緒變更＋已結論未轉出討論）。
import { describe, it, expect } from "vitest";
import type { ChangeItem, DiscussionItem } from "@speclink/ui";

import {
  MAX_TABS,
  upsertTab,
  removeTab,
  persistTabs,
  readPersistedTabs,
  pendingWrapUpCount,
  type ProjectTab,
} from "../tabs";
import { APP_MESSAGES } from "../i18n/messages";

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

describe("pendingWrapUpCount（徽章＝待收尾數：已就緒變更＋已結論未轉出討論）", () => {
  function discussion(slug: string, status: string, promotedTo: string[] = []): DiscussionItem {
    return { slug, topic: slug, status, rounds: 1, created: "2026-01-02", promotedTo };
  }

  it("counts ready changes plus concluded discussions; in-progress/open/promoted excluded", () => {
    // 契約範例：2 個已就緒變更＋1 份已結論未轉出討論 → 徽章 3。
    const changes: ChangeItem[] = [
      { name: "proposed", status: "x", totalTasks: 28, completedTasks: 0 },
      { name: "started", status: "x", totalTasks: 10, completedTasks: 0, startedAt: "2026-07-06" },
      { name: "progressed", status: "x", totalTasks: 14, completedTasks: 2 },
      { name: "ready-a", status: "x", totalTasks: 5, completedTasks: 5 },
      { name: "ready-b", status: "x", totalTasks: 3, completedTasks: 3 },
    ];
    const discussions: DiscussionItem[] = [
      discussion("alpha-concluded", "concluded"),
      discussion("beta-open", "open"),
      discussion("gamma-promoted", "promoted", ["cut-a"]),
    ];
    expect(pendingWrapUpCount(changes, discussions)).toBe(3);
  });

  it("returns zero when nothing awaits the user", () => {
    // 全部收尾後歸零：只剩進行中變更與 open／promoted 討論。
    const changes: ChangeItem[] = [
      { name: "started", status: "x", totalTasks: 10, completedTasks: 2, startedAt: "2026-07-06" },
    ];
    const discussions: DiscussionItem[] = [
      discussion("beta-open", "open"),
      discussion("gamma-promoted", "promoted", ["cut-a"]),
    ];
    expect(pendingWrapUpCount(changes, discussions)).toBe(0);
  });
});

describe("分頁徽章 tooltip（待收尾語意）", () => {
  it("uses pending-wrap-up wording with the {n} placeholder in both locales", () => {
    expect(APP_MESSAGES["zh-TW"]["app.tabBadgeTooltip"]).toContain("待收尾");
    expect(APP_MESSAGES["zh-TW"]["app.tabBadgeTooltip"]).toContain("{n}");
    expect(APP_MESSAGES.en["app.tabBadgeTooltip"].toLowerCase()).toContain("wrap");
    expect(APP_MESSAGES.en["app.tabBadgeTooltip"]).toContain("{n}");
  });

  it("zh-TW and en app dictionaries expose the same key set", () => {
    expect(Object.keys(APP_MESSAGES["zh-TW"]).sort()).toEqual(Object.keys(APP_MESSAGES.en).sort());
  });
});
