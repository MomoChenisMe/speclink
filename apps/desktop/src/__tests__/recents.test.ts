// 最近開啟清單的純函式面（design D1／D3）：localStorage 獨立鍵持久化（locator＋
// 顯示名、最新在前、上限 20）、同 locator key 去重上移、移除、鍵缺席回 null、
// 壞資料歸零且逐條驗證、顯示期濾掉分頁列上已開著的項目。
import { describe, it, expect } from "vitest";

import {
  MAX_RECENTS,
  RECENTS_STORAGE_KEY,
  upsertRecent,
  removeRecent,
  persistRecents,
  readPersistedRecents,
  visibleRecents,
  type RecentEntry,
} from "../recents";
import { locatorKey, type WorkspaceLocator } from "../session";
import type { ProjectTab } from "../tabs";

function local(root: string): WorkspaceLocator {
  return { kind: "local", root };
}

function entry(root: string, name = root): RecentEntry {
  return { locator: local(root), name };
}

function tab(root: string, name = root): ProjectTab {
  return { locator: local(root), name };
}

function keys(entries: Array<{ locator: WorkspaceLocator }>): string[] {
  return entries.map((e) => locatorKey(e.locator));
}

function memStorage(): Storage {
  const map = new Map<string, string>();
  return {
    get length() {
      return map.size;
    },
    clear: () => map.clear(),
    getItem: (k: string) => map.get(k) ?? null,
    key: (i: number) => [...map.keys()][i] ?? null,
    removeItem: (k: string) => {
      map.delete(k);
    },
    setItem: (k: string, v: string) => {
      map.set(k, String(v));
    },
  } as Storage;
}

describe("upsertRecent（spec「去重上移與上限截尾」）", () => {
  it("a new workspace goes to the front", () => {
    const next = upsertRecent([entry("A")], entry("B"));
    expect(keys(next)).toEqual(["local:B", "local:A"]);
  });

  it("reopening A after A, B keeps one entry and moves it to the front", () => {
    let list: RecentEntry[] = [];
    list = upsertRecent(list, entry("A"));
    list = upsertRecent(list, entry("B"));
    list = upsertRecent(list, entry("A"));
    expect(keys(list)).toEqual(["local:A", "local:B"]);
  });

  it("updates the display name of an existing entry", () => {
    const next = upsertRecent([entry("A", "old")], entry("A", "new"));
    expect(next).toEqual([entry("A", "new")]);
  });

  it("caps at MAX_RECENTS (20) by dropping the oldest entry", () => {
    expect(MAX_RECENTS).toBe(20);
    let list: RecentEntry[] = [];
    for (let i = 1; i <= 21; i += 1) list = upsertRecent(list, entry(`W${i}`));
    expect(list).toHaveLength(20);
    expect(keys(list)[0]).toBe("local:W21");
    expect(keys(list)[19]).toBe("local:W2");
    expect(keys(list)).not.toContain("local:W1");
  });
});

describe("removeRecent", () => {
  it("removes only the entry with the given locator key", () => {
    const list = [entry("A"), entry("B"), entry("C")];
    expect(keys(removeRecent(list, "local:B"))).toEqual(["local:A", "local:C"]);
  });
});

describe("最近開啟持久化（design D1）", () => {
  it("an absent key reads as null so the caller can seed", () => {
    expect(readPersistedRecents(memStorage())).toBeNull();
  });

  it("round-trips local and remote entries in order under version 1", () => {
    const st = memStorage();
    const remote: RecentEntry = {
      locator: {
        kind: "remote",
        connectionId: "c1",
        projectId: "prj",
        repoId: "repo",
        checkoutRoot: "/work/repo",
      },
      name: "Demo/repo",
    };
    persistRecents([remote, entry("A", "a")], st);
    const raw = JSON.parse(st.getItem(RECENTS_STORAGE_KEY) ?? "{}");
    expect(raw.version).toBe(1);
    expect(raw.entries).toHaveLength(2);
    expect(readPersistedRecents(st)).toEqual([remote, entry("A", "a")]);
  });

  it("garbage JSON reads as an empty list（spec「壞資料歸零且不補種」）", () => {
    const st = memStorage();
    st.setItem(RECENTS_STORAGE_KEY, "{not json");
    expect(readPersistedRecents(st)).toEqual([]);
  });

  it("an unknown version reads as an empty list", () => {
    const st = memStorage();
    st.setItem(RECENTS_STORAGE_KEY, JSON.stringify({ version: 2, entries: [entry("A")] }));
    expect(readPersistedRecents(st)).toEqual([]);
  });

  it("a non-array entries field reads as an empty list", () => {
    const st = memStorage();
    st.setItem(RECENTS_STORAGE_KEY, JSON.stringify({ version: 1, entries: { a: 1 } }));
    expect(readPersistedRecents(st)).toEqual([]);
  });

  it("drops entries with an unrecognized locator or a non-string name, keeps the rest", () => {
    const st = memStorage();
    st.setItem(
      RECENTS_STORAGE_KEY,
      JSON.stringify({
        version: 1,
        entries: [
          { locator: { kind: "local", root: 42 }, name: "bad-root" },
          { locator: { kind: "remote", connectionId: "c1" }, name: "bad-remote" },
          { locator: local("A"), name: 7 },
          { locator: local("B"), name: "b" },
          "junk",
        ],
      }),
    );
    expect(readPersistedRecents(st)).toEqual([entry("B", "b")]);
  });
});

describe("visibleRecents（spec「記錄全在分頁列上時不顯示區段」的過濾面）", () => {
  it("hides entries whose locator key is on the tab bar and keeps order", () => {
    const list = [entry("C"), entry("B"), entry("A")];
    const tabs = [tab("A"), tab("C")];
    expect(keys(visibleRecents(list, tabs))).toEqual(["local:B"]);
  });

  it("returns an empty list when every entry is open", () => {
    const list = [entry("A"), entry("B")];
    const tabs = [tab("A"), tab("B")];
    expect(visibleRecents(list, tabs)).toEqual([]);
  });
});

describe("讀取時的收斂（review 第 1 輪 SUGGESTION）", () => {
  it("drops duplicate locators, keeping the first occurrence", () => {
    const st = memStorage();
    st.setItem(
      RECENTS_STORAGE_KEY,
      JSON.stringify({
        version: 1,
        entries: [entry("A", "first"), entry("B"), entry("A", "second")],
      }),
    );
    expect(readPersistedRecents(st)).toEqual([entry("A", "first"), entry("B")]);
  });

  it("caps a hand-edited over-long list at MAX_RECENTS", () => {
    const st = memStorage();
    const entries = Array.from({ length: 25 }, (_, i) => entry(`W${i}`));
    st.setItem(RECENTS_STORAGE_KEY, JSON.stringify({ version: 1, entries }));
    expect(readPersistedRecents(st)).toHaveLength(MAX_RECENTS);
    expect(keys(readPersistedRecents(st) ?? [])[0]).toBe("local:W0");
  });
});
