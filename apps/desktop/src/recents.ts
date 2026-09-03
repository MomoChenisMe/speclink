// 最近開啟清單的純函式面（design D1／D3／D5）：與分頁列分離的「曾開啟過」記憶。
// localStorage 獨立鍵持久化（version 1、locator＋顯示名），最新在前、同 locator key
// 去重上移、上限 20；關閉分頁不動記錄。鍵缺席回 null 供啟動自分頁補種，壞資料歸零。
import { locatorKey, type WorkspaceLocator } from "./session";
import { isLocator, type ProjectTab } from "./tabs";

export interface RecentEntry {
  locator: WorkspaceLocator;
  name: string;
}

export const MAX_RECENTS = 20;
export const RECENTS_STORAGE_KEY = "speclink.recentWorkspaces";

/** 成功開啟後記入：同 locator key 者移除後放到最前（顯示名取新值），超過上限截尾。 */
export function upsertRecent(entries: RecentEntry[], entry: RecentEntry): RecentEntry[] {
  const key = locatorKey(entry.locator);
  const rest = entries.filter((e) => locatorKey(e.locator) !== key);
  return [{ locator: entry.locator, name: entry.name }, ...rest].slice(0, MAX_RECENTS);
}

export function removeRecent(entries: RecentEntry[], key: string): RecentEntry[] {
  return entries.filter((e) => locatorKey(e.locator) !== key);
}

export function persistRecents(entries: RecentEntry[], storage: Storage = localStorage): void {
  storage.setItem(RECENTS_STORAGE_KEY, JSON.stringify({ version: 1, entries }));
}

/** 鍵缺席回 null（啟動時據此自分頁補種）；壞 JSON、version 非 1 或 entries 非陣列
 * 回空陣列；條目逐筆驗證 locator 形狀與 name 字串，不合者只丟該條目。 */
export function readPersistedRecents(storage: Storage = localStorage): RecentEntry[] | null {
  try {
    const raw = storage.getItem(RECENTS_STORAGE_KEY);
    if (raw === null) return null;
    const parsed = JSON.parse(raw) as { version?: unknown; entries?: unknown } | null;
    if (parsed?.version !== 1 || !Array.isArray(parsed.entries)) return [];
    // 手改或舊版殘留的 payload 可能含重複 locator 或超長清單——讀進來就收斂，
    // 否則 chooser 會撞上重複的 React key，且要等下次寫入才套上限。
    const seen = new Set<string>();
    return parsed.entries
      .filter((e): e is RecentEntry => {
        if (typeof e !== "object" || e === null) return false;
        const candidate = e as { locator?: unknown; name?: unknown };
        return isLocator(candidate.locator) && typeof candidate.name === "string";
      })
      .filter((e) => {
        const key = locatorKey(e.locator);
        if (seen.has(key)) return false;
        seen.add(key);
        return true;
      })
      .slice(0, MAX_RECENTS)
      .map((e) => ({ locator: e.locator, name: e.name }));
  } catch {
    return [];
  }
}

/** 顯示期過濾（design D3）：濾掉分頁列上已開著的 workspace，順序不變。 */
export function visibleRecents(entries: RecentEntry[], tabs: ProjectTab[]): RecentEntry[] {
  const open = new Set(tabs.map((t) => locatorKey(t.locator)));
  return entries.filter((e) => !open.has(locatorKey(e.locator)));
}
