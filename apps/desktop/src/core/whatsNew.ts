// 更新日誌彈窗的判定（desktop-app spec「更新日誌彈窗」）：純函式、不依賴 Tauri（比照
// core/updater.ts）。已看過版號存 app 本機持久化，不進任何專案檔案（比照 assetPrompt.ts）。
import type { ReleaseNotesEntry } from "../release-notes/release-notes";

export type WhatsNewDecision =
  | { kind: "show"; entries: ReleaseNotesEntry[] }
  | { kind: "record" }
  | { kind: "none" };

/** 規則依序：dev→none；頂端版號≠app 版號→none（版號不齊時不彈錯版內容）；無記錄→record
 * （首次安裝：不彈、記現版號）；已看過現版→none；否則 show 頂端起到已看過那筆之前的全部
 * （已看過版號不在清單內時只含頂端一筆；但已看過的比 app 版號還新＝降版，看過的不再彈）。 */
export function whatsNewDecision(input: {
  appVersion: string;
  entries: ReleaseNotesEntry[];
  lastSeen: string | null;
  dev: boolean;
}): WhatsNewDecision {
  const { appVersion, entries, lastSeen, dev } = input;
  if (dev) return { kind: "none" };
  if (entries.length === 0 || entries[0].version !== appVersion) return { kind: "none" };
  if (lastSeen === null) return { kind: "record" };
  if (lastSeen === appVersion) return { kind: "none" };
  const seenIndex = entries.findIndex((entry) => entry.version === lastSeen);
  if (seenIndex !== -1) return { kind: "show", entries: entries.slice(0, seenIndex) };
  return compareVersions(lastSeen, appVersion) > 0 ? { kind: "none" } : { kind: "show", entries: [entries[0]] };
}

/** X.Y.Z 逐段數值比較：正＝a 新、負＝b 新、0＝同版。 */
function compareVersions(a: string, b: string): number {
  const pa = a.split(".").map(Number);
  const pb = b.split(".").map(Number);
  for (let i = 0; i < 3; i += 1) {
    if (pa[i] !== pb[i]) return pa[i] - pb[i];
  }
  return 0;
}

const STORAGE_KEY = "speclink.lastSeenVersion";
const VERSION_SHAPE = /^\d+\.\d+\.\d+$/;

/** 讀已看過版號；不是 X.Y.Z 形狀的值（JSON、數字、空字串）一律視為無記錄。 */
export function readLastSeenVersion(storage: Storage = localStorage): string | null {
  try {
    const raw = storage.getItem(STORAGE_KEY);
    return raw !== null && VERSION_SHAPE.test(raw) ? raw : null;
  } catch {
    return null;
  }
}

/** 記下已看過版號；儲存不可用時靜默（下次啟動會再彈一次，比卡住 UI 好）。 */
export function writeLastSeenVersion(version: string, storage: Storage = localStorage): void {
  try {
    storage.setItem(STORAGE_KEY, version);
  } catch {
    // 讀取端同樣把壞儲存視為無記錄。
  }
}
