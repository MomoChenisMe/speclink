// 專案分頁列的純函式面（design D8/D10；workspace-session 決策 1/2）：分頁清單
// 操作與 localStorage 持久化。分頁身分＝WorkspaceLocator（經 locatorKey 比對，
// 不再以裸 root 字串）；持久化 v2（version＋locator＋activeKey），v1 靜默遷移。
// 分頁列只管目前開著的專案（關閉即自清單移除），不寫入專案目錄；曾開啟過的
// 「最近開啟」記憶另見 recents.ts（chooser-recent-workspaces D6）。
import { changeStage, type ChangeItem, type DiscussionItem } from "@speclink/ui";

import { locatorKey, type WorkspaceLocator } from "./session";

export interface ProjectTab {
  locator: WorkspaceLocator;
  name: string;
}

export interface PersistedTabs {
  tabs: Array<{ locator: WorkspaceLocator; name: string }>;
  /** 最後活躍分頁的 locator key（null＝無）。 */
  activeKey: string | null;
}

export const MAX_TABS = 10;
const STORAGE_KEY = "speclink.projectTabs";

/** 成功開啟後記入分頁：既有者（同 locator key）原地去重（更新顯示名）、
 * 新專案 append 尾端；超過上限時丟最舊（清單首位）。 */
export function upsertTab(
  tabs: ProjectTab[],
  entry: { locator: WorkspaceLocator; name: string },
): ProjectTab[] {
  const key = locatorKey(entry.locator);
  if (tabs.some((t) => locatorKey(t.locator) === key)) {
    return tabs.map((t) =>
      locatorKey(t.locator) === key
        ? { ...t, locator: entry.locator, name: entry.name }
        : t,
    );
  }
  const next = [...tabs, { locator: entry.locator, name: entry.name }];
  return next.length > MAX_TABS ? next.slice(next.length - MAX_TABS) : next;
}

export function removeTab(tabs: ProjectTab[], key: string): ProjectTab[] {
  return tabs.filter((t) => locatorKey(t.locator) !== key);
}

export function persistTabs(
  tabs: ProjectTab[],
  activeKey: string | null,
  storage: Storage = localStorage,
): void {
  const payload = {
    version: 2,
    tabs: tabs.map(({ locator, name }) => ({ locator, name })),
    activeKey,
  };
  storage.setItem(STORAGE_KEY, JSON.stringify(payload));
}

/** 持久化條目的 locator 形狀驗證（v2 讀取）：local 需 root 字串、remote 需
 * 三段識別字串（checkoutRoot 可選字串）；其餘形狀丟棄該條目。 */
export function isLocator(v: unknown): v is WorkspaceLocator {
  if (typeof v !== "object" || v === null) return false;
  const l = v as Record<string, unknown>;
  if (l.kind === "local") return typeof l.root === "string";
  if (l.kind === "remote") {
    return (
      typeof l.connectionId === "string" &&
      typeof l.projectId === "string" &&
      typeof l.repoId === "string" &&
      (l.checkoutRoot === undefined || typeof l.checkoutRoot === "string")
    );
  }
  return false;
}

/** 讀取持久化分頁（design 決策 2）：v2 依 version 解析；無 version 欄位且形如
 * v1（tabs 條目有 root 字串）則靜默遷移——root 逐條映射 local locator、
 * activeRoot 映射 local key，下次寫入即 v2。缺鍵、壞 JSON 或不識別形狀一律
 * 還原為零分頁；v1 條目 root 非字串者丟棄。 */
export function readPersistedTabs(storage: Storage = localStorage): PersistedTabs {
  try {
    const raw = storage.getItem(STORAGE_KEY);
    if (!raw) return { tabs: [], activeKey: null };
    const parsed = JSON.parse(raw) as {
      version?: unknown;
      tabs?: unknown;
      activeKey?: unknown;
      activeRoot?: unknown;
    };
    if (parsed.version === 2) {
      const entries = Array.isArray(parsed.tabs) ? parsed.tabs : [];
      const tabs = entries
        .filter(
          (t): t is { locator: WorkspaceLocator; name: string } =>
            isLocator(t?.locator) && typeof t?.name === "string",
        )
        .map((t) => ({ locator: t.locator, name: t.name }));
      return {
        tabs,
        activeKey: typeof parsed.activeKey === "string" ? parsed.activeKey : null,
      };
    }
    if (parsed.version === undefined && Array.isArray(parsed.tabs)) {
      const tabs = parsed.tabs
        .filter(
          (t): t is { root: string; name: string } =>
            typeof t?.root === "string" && typeof t?.name === "string",
        )
        .map((t) => ({ locator: { kind: "local", root: t.root } as const, name: t.name }));
      return {
        tabs,
        activeKey:
          typeof parsed.activeRoot === "string"
            ? locatorKey({ kind: "local", root: parsed.activeRoot })
            : null,
      };
    }
    return { tabs: [], activeKey: null };
  } catch {
    return { tabs: [], activeKey: null };
  }
}

