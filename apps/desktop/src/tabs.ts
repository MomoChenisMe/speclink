// 專案分頁列的純函式面（design D8/D10）：分頁清單操作與 localStorage 持久化
// （路徑＋顯示名＋順序＋最後活躍）。分頁列即最近開啟清單，不寫入專案目錄。
import { changeStage, type ChangeItem, type DiscussionItem } from "@speclink/ui";

export interface ProjectTab {
  root: string;
  name: string;
  /** 待收尾數（spec-archive-drawer design D6）；null＝尚未取得（restore 快照前）。 */
  badge: number | null;
}

export interface PersistedTabs {
  tabs: Array<{ root: string; name: string }>;
  activeRoot: string | null;
}

export const MAX_TABS = 10;
const STORAGE_KEY = "speclink.projectTabs";

/** 成功開啟後記入分頁：既有者原地去重（更新顯示名）、新專案 append 尾端；
 * 超過上限時丟最舊（清單首位）。 */
export function upsertTab(tabs: ProjectTab[], entry: { root: string; name: string }): ProjectTab[] {
  if (tabs.some((t) => t.root === entry.root)) {
    return tabs.map((t) => (t.root === entry.root ? { ...t, name: entry.name } : t));
  }
  const next = [...tabs, { root: entry.root, name: entry.name, badge: null }];
  return next.length > MAX_TABS ? next.slice(next.length - MAX_TABS) : next;
}

export function removeTab(tabs: ProjectTab[], root: string): ProjectTab[] {
  return tabs.filter((t) => t.root !== root);
}

export function persistTabs(
  tabs: ProjectTab[],
  activeRoot: string | null,
  storage: Storage = localStorage,
): void {
  const payload: PersistedTabs = {
    tabs: tabs.map(({ root, name }) => ({ root, name })),
    activeRoot,
  };
  storage.setItem(STORAGE_KEY, JSON.stringify(payload));
}

/** 讀取持久化分頁；缺鍵或壞 JSON（手改 localStorage）一律還原為零分頁。 */
export function readPersistedTabs(storage: Storage = localStorage): PersistedTabs {
  try {
    const raw = storage.getItem(STORAGE_KEY);
    if (!raw) return { tabs: [], activeRoot: null };
    const parsed = JSON.parse(raw) as Partial<PersistedTabs>;
    const tabs = Array.isArray(parsed.tabs)
      ? parsed.tabs.filter((t) => typeof t?.root === "string" && typeof t?.name === "string")
      : [];
    return {
      tabs,
      activeRoot: typeof parsed.activeRoot === "string" ? parsed.activeRoot : null,
    };
  } catch {
    return { tabs: [], activeRoot: null };
  }
}

/** 徽章派生：待收尾數＝已就緒變更（changeStage ready，與看板欄位規則同源）＋
 * 已結論未轉出討論（concluded；promoted 已轉出、open 仍在推進，皆不計）。
 * 待收尾＝等使用者執行動詞的卡片（spec-archive-drawer design D6）。 */
export function pendingWrapUpCount(changes: ChangeItem[], discussions: DiscussionItem[]): number {
  const ready = changes.filter((c) => changeStage(c) === "ready").length;
  const concluded = discussions.filter((d) => d.status === "concluded").length;
  return ready + concluded;
}
