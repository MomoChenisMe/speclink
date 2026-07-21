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
import { locatorKey, type WorkspaceLocator } from "../session";
import { APP_MESSAGES } from "../i18n/messages";

function local(root: string): WorkspaceLocator {
  return { kind: "local", root };
}

function tab(root: string, name = root): ProjectTab {
  return { locator: local(root), name, badge: null };
}

function keys(tabs: Array<{ locator: WorkspaceLocator }>): string[] {
  return tabs.map((t) => locatorKey(t.locator));
}

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

describe("upsertTab（spec「成功開啟後記入分頁並去重上移」的清單面）", () => {
  it("appends a new project at the end", () => {
    const tabs = upsertTab([tab("A")], { locator: local("B"), name: "B" });
    expect(keys(tabs)).toEqual(["local:A", "local:B"]);
  });

  it("reopening an existing project keeps one tab in place (dedup)", () => {
    const tabs = upsertTab([tab("A"), tab("B")], { locator: local("A"), name: "A" });
    expect(keys(tabs)).toEqual(["local:A", "local:B"]);
    expect(tabs).toHaveLength(2);
  });

  it("caps at MAX_TABS by dropping the oldest tab", () => {
    let tabs: ProjectTab[] = [];
    for (let i = 0; i < MAX_TABS; i++)
      tabs = upsertTab(tabs, { locator: local(`p${i}`), name: `p${i}` });
    expect(tabs).toHaveLength(MAX_TABS);
    tabs = upsertTab(tabs, { locator: local("extra"), name: "extra" });
    expect(tabs).toHaveLength(MAX_TABS);
    expect(keys(tabs)).not.toContain("local:p0");
    expect(keys(tabs).at(-1)).toBe("local:extra");
  });

  it("updates the display name of an existing tab", () => {
    const tabs = upsertTab([tab("A", "old")], { locator: local("A"), name: "new" });
    expect(tabs[0].name).toBe("new");
  });
});

describe("removeTab", () => {
  it("removes the tab with the given locator key", () => {
    expect(keys(removeTab([tab("A"), tab("B")], "local:A"))).toEqual(["local:B"]);
  });
});

describe("分頁持久化 v2 與 v1 靜默遷移（spec「分頁持久化 v2 與 v1 靜默遷移」）", () => {
  it("empty storage restores to zero tabs", () => {
    const restored = readPersistedTabs(memStorage());
    expect(restored.tabs).toEqual([]);
    expect(restored.activeKey).toBeNull();
  });

  it("round-trips v2 (locator, order, names, activeKey) and stamps version 2", () => {
    const st = memStorage();
    const tabs: ProjectTab[] = [
      { locator: local("B"), name: "beta", badge: null },
      { locator: local("A"), name: "alpha", badge: null },
    ];
    persistTabs(tabs, "local:A", st);
    const raw = JSON.parse(st.getItem("speclink.projectTabs") ?? "{}");
    expect(raw.version).toBe(2);
    const restored = readPersistedTabs(st);
    expect(restored.tabs.map((t) => t.locator)).toEqual([local("B"), local("A")]);
    expect(restored.tabs.map((t) => t.name)).toEqual(["beta", "alpha"]);
    expect(restored.activeKey).toBe("local:A");
  });

  it("v2 round-trips remote checkoutRoot without putting it in activeKey", () => {
    const st = memStorage();
    const locator: WorkspaceLocator = {
      kind: "remote",
      connectionId: "c1",
      projectId: "demo",
      repoId: "backend",
      checkoutRoot: "/work/backend",
    };
    persistTabs([{ locator, name: "Demo/Backend", badge: null }], locatorKey(locator), st);
    const restored = readPersistedTabs(st);
    expect(restored.tabs).toEqual([{ locator, name: "Demo/Backend" }]);
    expect(restored.activeKey).toBe("remote:c1/demo/backend");
  });

  it("v1 payload (root＋name＋activeRoot) silently migrates to local locators and activeKey", () => {
    // 契約範例（spec Scenario「舊版使用者升級後分頁完整保留」）：兩個專案分頁＋activeRoot。
    const st = memStorage();
    st.setItem(
      "speclink.projectTabs",
      JSON.stringify({
        tabs: [
          { root: "A", name: "alpha" },
          { root: "B", name: "beta" },
        ],
        activeRoot: "B",
      }),
    );
    const restored = readPersistedTabs(st);
    expect(restored.tabs).toEqual([
      { locator: local("A"), name: "alpha" },
      { locator: local("B"), name: "beta" },
    ]);
    expect(restored.activeKey).toBe("local:B");
  });

  it("the next persist after a v1 read writes v2", () => {
    const st = memStorage();
    st.setItem(
      "speclink.projectTabs",
      JSON.stringify({ tabs: [{ root: "A", name: "alpha" }], activeRoot: "A" }),
    );
    const restored = readPersistedTabs(st);
    persistTabs(
      restored.tabs.map((t) => ({ ...t, badge: null })),
      restored.activeKey,
      st,
    );
    const raw = JSON.parse(st.getItem("speclink.projectTabs") ?? "{}");
    expect(raw.version).toBe(2);
    expect(raw.tabs).toEqual([{ locator: local("A"), name: "alpha" }]);
    expect(raw.activeKey).toBe("local:A");
  });

  it("v1 entries with a non-string root are dropped", () => {
    const st = memStorage();
    st.setItem(
      "speclink.projectTabs",
      JSON.stringify({
        tabs: [
          { root: 7, name: "bogus" },
          { root: "A", name: "alpha" },
        ],
        activeRoot: "A",
      }),
    );
    expect(readPersistedTabs(st).tabs).toEqual([{ locator: local("A"), name: "alpha" }]);
  });

  it("garbage JSON restores to zero tabs and no active key（spec「壞 JSON 歸零」）", () => {
    const st = memStorage();
    st.setItem("speclink.projectTabs", "{not json");
    const restored = readPersistedTabs(st);
    expect(restored.tabs).toEqual([]);
    expect(restored.activeKey).toBeNull();
  });

  it("unrecognized shapes restore to zero tabs", () => {
    const st = memStorage();
    st.setItem("speclink.projectTabs", JSON.stringify({ version: 99, tabs: "x" }));
    const restored = readPersistedTabs(st);
    expect(restored.tabs).toEqual([]);
    expect(restored.activeKey).toBeNull();
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
